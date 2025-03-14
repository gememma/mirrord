extern crate alloc;
use core::ops::Deref;
use std::{net::IpAddr, sync::LazyLock};

use bincode::{Decode, Encode};
use semver::VersionReq;

use crate::RemoteResult;

/// Minimal mirrord-protocol version that allows [`GetAddrInfoRequestV2`].
pub static ADDRINFO_V2_VERSION: LazyLock<VersionReq> =
    LazyLock::new(|| ">=1.15.0".parse().expect("Bad Identifier"));

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct LookupRecord {
    pub name: String,
    pub ip: IpAddr,
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct DnsLookup(pub Vec<LookupRecord>);

impl Deref for DnsLookup {
    type Target = Vec<LookupRecord>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for DnsLookup {
    type Item = LookupRecord;

    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct GetAddrInfoResponse(pub RemoteResult<DnsLookup>);

impl Deref for GetAddrInfoResponse {
    type Target = RemoteResult<DnsLookup>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Triggered by the `mirrord-layer` hook of `getaddrinfo_detour`.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct GetAddrInfoRequest {
    pub node: String,
}

/// For when the new request is not supported, and we have to fall back to the old version.
impl From<GetAddrInfoRequestV2> for GetAddrInfoRequest {
    fn from(value: GetAddrInfoRequestV2) -> Self {
        Self { node: value.node }
    }
}

#[derive(
    serde::Serialize, serde::Deserialize, Encode, Decode, Debug, PartialEq, Eq, Copy, Clone,
)]
pub enum AddressFamily {
    Ipv4Only,
    Ipv6Only,
    Both,
    Any,
    /// If we add a variant and a new client sends an old agent the new variant, the agent will see
    /// this variant.
    #[serde(other, skip_serializing)]
    UnknownAddressFamilyFromNewerClient,
}

#[derive(thiserror::Error, Debug)]
pub enum AddressFamilyError {
    #[error(
        "The agent received a GetAddrInfoRequestV2 with an address family that is not yet known \
        to this version of the agent."
    )]
    UnsupportedFamily,
}

#[derive(serde::Serialize, serde::Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum SockType {
    Stream,
    Dgram,
    Any,
    /// If we add a variant and a new client sends an old agent the new variant, the agent will see
    /// this variant.
    #[serde(other, skip_serializing)]
    UnknownSockTypeFromNewerClient,
}

/// Newer, advanced version of [`GetAddrInfoRequest`]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct GetAddrInfoRequestV2 {
    pub node: String,
    /// Currently not respected by the agent, there for future use.
    pub service_port: u16,
    pub family: AddressFamily,
    pub socktype: SockType,
    /// Including these fields so we can use them in the future without introducing a new request
    /// type. But note that the constants are different on macOS and Linux so they should be
    /// converted to the linux values first (on the client, because the agent does not know the
    /// client is macOS).
    pub flags: i32,
    pub protocol: i32,
}

impl From<GetAddrInfoRequest> for GetAddrInfoRequestV2 {
    fn from(value: GetAddrInfoRequest) -> Self {
        Self {
            node: value.node,
            service_port: 0,
            flags: 0,
            family: AddressFamily::Ipv4Only,
            socktype: SockType::Any,
            protocol: 0,
        }
    }
}
