use std::future::Future;

use futures::FutureExt;
use http_body_util::BodyExt;
use hyper::body::{Body, Frame};

/// Utility extension trait for [`Body`].
///
/// Contains methods that allow for reading [`Frame`]s in batches.
pub trait BatchedBody: Body {
    /// Reads all [`Frame`]s that are available without blocking.
    fn ready_frames(&mut self) -> Result<Frames<Self::Data>, Self::Error>;

    /// Waits for the next [`Frame`] then reads all [`Frame`]s that are available without blocking.
    fn next_frames(&mut self) -> impl Future<Output = Result<Frames<Self::Data>, Self::Error>>;
}

impl<B> BatchedBody for B
where
    B: Body + Unpin,
{
    fn ready_frames(&mut self) -> Result<Frames<Self::Data>, Self::Error> {
        let mut frames = Frames {
            frames: vec![],
            is_last: false,
        };
        extend_with_ready(self, &mut frames)?;
        Ok(frames)
    }

    async fn next_frames(&mut self) -> Result<Frames<Self::Data>, Self::Error> {
        let mut frames = Frames {
            frames: vec![],
            is_last: false,
        };

        match self.frame().await {
            None => {
                frames.is_last = true;
                return Ok(frames);
            }
            Some(result) => {
                frames.frames.push(result?);
            }
        }

        extend_with_ready(self, &mut frames)?;

        Ok(frames)
    }
}

/// Extends the given [`Frames`] instance with [`Frame`]s that are available without blocking.
fn extend_with_ready<B: Body + Unpin>(
    body: &mut B,
    frames: &mut Frames<B::Data>,
) -> Result<(), B::Error> {
    loop {
        match body.frame().now_or_never() {
            None => {
                frames.is_last = false;
                break;
            }
            Some(None) => {
                frames.is_last = true;
                break;
            }
            Some(Some(result)) => {
                frames.frames.push(result?);
                frames.is_last = false;
            }
        }
    }

    Ok(())
}

/// A batch of body [`Frame`]s.
///
/// `D` parameter determines [`Body::Data`] type.
pub struct Frames<D> {
    /// A batch of consecutive [`Frames`].
    pub frames: Vec<Frame<D>>,
    /// Whether the [`Body`] has finished and this is the last batch.
    pub is_last: bool,
}
