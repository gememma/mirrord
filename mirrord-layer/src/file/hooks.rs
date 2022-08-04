use std::{ffi::CStr, io::SeekFrom, os::unix::io::RawFd, path::PathBuf, ptr, slice};

use frida_gum::interceptor::Interceptor;
use libc::{self, c_char, c_int, c_void, off_t, size_t, ssize_t, AT_FDCWD, FILE};
use mirrord_macro::hook_fn;
use mirrord_protocol::ReadFileResponse;
use tracing::{error, trace};

use super::{
    ops::{fdopen, fopen, openat},
    OpenOptionsInternalExt, IGNORE_FILES, OPEN_FILES,
};
use crate::{
    error::LayerError,
    file::ops::{lseek, open, read, write},
    replace,
};

/// Hook for `libc::open`.
///
/// **Bypassed** by `raw_path`s that match `IGNORE_FILES` regex.
#[hook_fn]
pub(super) unsafe extern "C" fn open_detour(raw_path: *const c_char, open_flags: c_int) -> RawFd {
    trace!("open_detour -> open_flags {:#?}", open_flags);

    let path = match CStr::from_ptr(raw_path)
        .to_str()
        .map_err(LayerError::from)
        .map(PathBuf::from)
    {
        Ok(path) => path,
        Err(fail) => return fail.into(),
    };

    // Calls with non absolute paths are sent to libc::open.
    if IGNORE_FILES.is_match(path.to_str().unwrap_or_default()) || !path.is_absolute() {
        FN_OPEN(raw_path, open_flags)
    } else {
        let open_options = OpenOptionsInternalExt::from_flags(open_flags);
        let open_result = open(path, open_options);

        let (Ok(result) | Err(result)) = open_result.map_err(From::from);
        result
    }
}

/// Hook for `libc::fopen`.
///
/// **Bypassed** by `raw_path`s that match `IGNORE_FILES` regex.
#[hook_fn]
pub(super) unsafe extern "C" fn fopen_detour(
    raw_path: *const c_char,
    raw_mode: *const c_char,
) -> *mut FILE {
    trace!("fopen_detour ->");

    let path = match CStr::from_ptr(raw_path)
        .to_str()
        .map_err(LayerError::from)
        .map(PathBuf::from)
    {
        Ok(path) => path,
        Err(fail) => return fail.into(),
    };

    let mode = match CStr::from_ptr(raw_mode)
        .to_str()
        .map(String::from)
        .map_err(LayerError::from)
    {
        Ok(mode) => mode,
        Err(fail) => return fail.into(),
    };

    if IGNORE_FILES.is_match(path.to_str().unwrap()) || !path.is_absolute() {
        FN_FOPEN(raw_path, raw_mode)
    } else {
        let open_options = OpenOptionsInternalExt::from_mode(mode);
        let fopen_result = fopen(path, open_options);

        let (Ok(result) | Err(result)) = fopen_result.map_err(From::from);
        result
    }
}

/// Hook for `libc::fdopen`.
///
/// Converts a `RawFd` into `*mut FILE` only for files that are already being managed by
/// mirrord-layer.
#[hook_fn]
pub(super) unsafe extern "C" fn fdopen_detour(fd: RawFd, raw_mode: *const c_char) -> *mut FILE {
    trace!("fdopen_detour -> fd {:#?}", fd);

    let mode = match CStr::from_ptr(raw_mode)
        .to_str()
        .map(String::from)
        .map_err(LayerError::from)
    {
        Ok(mode) => mode,
        Err(fail) => return fail.into(),
    };

    let open_files = OPEN_FILES.lock().unwrap();
    let open_file = open_files.get_key_value(&fd);

    if let Some((local_fd, remote_fd)) = open_file {
        let open_options = OpenOptionsInternalExt::from_mode(mode);
        let fdopen_result = fdopen(local_fd, *remote_fd, open_options);

        let (Ok(result) | Err(result)) = fdopen_result.map_err(From::from);
        result
    } else {
        FN_FDOPEN(fd, raw_mode)
    }
}

/// Equivalent to `open_detour`, **except** when `raw_path` specifies a relative path.
///
/// If `fd == AT_FDCWD`, the current working directory is used, and the behavior is the same as
/// `open_detour`.
/// `fd` for a file descriptor with the `O_DIRECTORY` flag.
#[hook_fn]
pub(super) unsafe extern "C" fn openat_detour(
    fd: RawFd,
    raw_path: *const c_char,
    open_flags: c_int,
) -> RawFd {
    trace!(
        "openat_detour -> fd {:#?} | open_flags {:#?}",
        fd,
        open_flags
    );

    let path = match CStr::from_ptr(raw_path)
        .to_str()
        .map_err(LayerError::from)
        .map(PathBuf::from)
    {
        Ok(path) => path,
        Err(fail) => return fail.into(),
    };

    // `openat` behaves the same as `open` when the path is absolute.
    // when called with AT_FDCWD, the call is propagated to `open`.

    if path.is_absolute() || fd == AT_FDCWD {
        open_detour(raw_path, open_flags)
    } else {
        // Relative path requires special handling, we must identify the relative part (relative to
        // what).
        let remote_fd = OPEN_FILES.lock().unwrap().get(&fd).cloned();

        // Are we managing the relative part?
        if let Some(remote_fd) = remote_fd {
            let openat_result = openat(path, open_flags, remote_fd);

            let (Ok(result) | Err(result)) = openat_result.map_err(From::from);
            result
        } else {
            // Nope, it's relative outside of our hands.

            FN_OPENAT(fd, raw_path, open_flags)
        }
    }
}

/// Hook for `libc::read`.
///
/// Reads `count` bytes into `out_buffer`, only for `fd`s that are being managed by mirrord-layer.
#[hook_fn]
pub(crate) unsafe extern "C" fn read_detour(
    fd: RawFd,
    out_buffer: *mut c_void,
    count: size_t,
) -> ssize_t {
    trace!("read_detour -> fd {:#?} | count {:#?}", fd, count);

    // We're only interested in files that are paired with mirrord-agent.
    let remote_fd = OPEN_FILES.lock().unwrap().get(&fd).cloned();

    if let Some(remote_fd) = remote_fd {
        let read_result = read(remote_fd, count).map(|read_file| {
            let ReadFileResponse { bytes, read_amount } = read_file;

            // There is no distinction between reading 0 bytes or if we hit EOF, but we only copy to
            // buffer if we have something to copy.
            if read_amount > 0 {
                let read_ptr = bytes.as_ptr();
                let out_buffer = out_buffer.cast();
                ptr::copy(read_ptr, out_buffer, read_amount);
            }

            // WARN: Must be careful when it comes to `EOF`, incorrect handling may appear as the
            // `read` call being repeated.
            read_amount.try_into().unwrap()
        });

        let (Ok(result) | Err(result)) = read_result.map_err(From::from);
        result
    } else {
        FN_READ(fd, out_buffer, count)
    }
}

/// Hook for `libc::fread`.
///
/// Reads `element_size * number_of_elements` bytes into `out_buffer`, only for `*mut FILE`s that
/// are being managed by mirrord-layer.
#[hook_fn]
pub(crate) unsafe extern "C" fn fread_detour(
    out_buffer: *mut c_void,
    element_size: size_t,
    number_of_elements: size_t,
    file_stream: *mut FILE,
) -> size_t {
    trace!(
        "fread_detour -> element_size {:#?} | number_of_elements {:#?}",
        element_size,
        number_of_elements
    );

    // Extract the fd from stream and check if it's managed by us, or should be bypassed.
    let fd = fileno_detour(file_stream);

    // We're only interested in files that are handled by `mirrord-agent`.
    let remote_fd = OPEN_FILES.lock().unwrap().get(&fd).cloned();
    if let Some(remote_fd) = remote_fd {
        let read_result = read(remote_fd, element_size * number_of_elements).map(|read_file| {
            let ReadFileResponse { bytes, read_amount } = read_file;

            // There is no distinction between reading 0 bytes or if we hit EOF, but we only
            // copy to buffer if we have something to copy.
            if read_amount > 0 {
                let read_ptr = bytes.as_ptr();
                let out_buffer = out_buffer.cast();
                ptr::copy(read_ptr, out_buffer, read_amount);
            }

            // TODO: The function fread() does not distinguish between end-of-file and error,
            // and callers must use feof(3) and ferror(3) to determine which occurred.
            read_amount
        });

        let (Ok(result) | Err(result)) = read_result.map_err(From::from);
        result
    } else {
        FN_FREAD(out_buffer, element_size, number_of_elements, file_stream)
    }
}

/// Hook for `libc::fileno`.
///
/// Converts a `*mut FILE` stream into an fd.
#[hook_fn]
pub(crate) unsafe extern "C" fn fileno_detour(file_stream: *mut FILE) -> c_int {
    trace!("fileno_detour ->");

    let local_fd = *(file_stream as *const _);

    if OPEN_FILES.lock().unwrap().contains_key(&local_fd) {
        local_fd
    } else {
        FN_FILENO(file_stream)
    }
}

/// Hook for `libc::lseek`.
///
/// **Bypassed** by `fd`s that are not managed by us (not found in `OPEN_FILES`).
#[hook_fn]
pub(crate) unsafe extern "C" fn lseek_detour(fd: RawFd, offset: off_t, whence: c_int) -> off_t {
    trace!(
        "lseek_detour -> fd {:#?} | offset {:#?} | whence {:#?}",
        fd,
        offset,
        whence
    );

    let remote_fd = OPEN_FILES.lock().unwrap().get(&fd).cloned();

    if let Some(remote_fd) = remote_fd {
        let seek_from = match whence {
            libc::SEEK_SET => SeekFrom::Start(offset as u64),
            libc::SEEK_CUR => SeekFrom::Current(offset),
            libc::SEEK_END => SeekFrom::End(offset),
            invalid => {
                error!(
                    "lseek_detour -> potential invalid value {:#?} for whence {:#?}",
                    invalid, whence
                );
                return -1;
            }
        };

        let lseek_result = lseek(remote_fd, seek_from).map(|offset| offset.try_into().unwrap());

        let (Ok(result) | Err(result)) = lseek_result.map_err(From::from);
        result
    } else {
        FN_LSEEK(fd, offset, whence)
    }
}

/// Hook for `libc::write`.
///
/// **Bypassed** by `fd`s that are not managed by us (not found in `OPEN_FILES`).
#[hook_fn]
pub(crate) unsafe extern "C" fn write_detour(
    fd: RawFd,
    buffer: *const c_void,
    count: size_t,
) -> ssize_t {
    trace!("write_detour -> fd {:#?} | count {:#?}", fd, count);

    let remote_fd = OPEN_FILES.lock().unwrap().get(&fd).cloned();

    if let Some(remote_fd) = remote_fd {
        if buffer.is_null() {
            return -1;
        }

        // WARN: Be veeery careful here, you cannot construct the `Vec` directly, as the
        // buffer allocation is handled on the C side.
        let outside_buffer = slice::from_raw_parts(buffer as *const u8, count);
        let write_bytes = outside_buffer.to_vec();

        let write_result = write(remote_fd, write_bytes);

        let (Ok(result) | Err(result)) = write_result.map_err(From::from);
        result
    } else {
        FN_WRITE(fd, buffer, count)
    }
}

/// Convenience function to setup file hooks (`x_detour`) with `frida_gum`.
pub(crate) unsafe fn enable_file_hooks(interceptor: &mut Interceptor) {
    let _ = replace!(interceptor, "open", open_detour, FnOpen, FN_OPEN);
    let _ = replace!(interceptor, "openat", openat_detour, FnOpenat, FN_OPENAT);
    let _ = replace!(interceptor, "fopen", fopen_detour, FnFopen, FN_FOPEN);
    let _ = replace!(interceptor, "fdopen", fdopen_detour, FnFdopen, FN_FDOPEN);
    let _ = replace!(interceptor, "read", read_detour, FnRead, FN_READ);
    let _ = replace!(interceptor, "fread", fread_detour, FnFread, FN_FREAD);
    let _ = replace!(interceptor, "fileno", fileno_detour, FnFileno, FN_FILENO);
    let _ = replace!(interceptor, "lseek", lseek_detour, FnLseek, FN_LSEEK);
    let _ = replace!(interceptor, "write", write_detour, FnWrite, FN_WRITE);
}