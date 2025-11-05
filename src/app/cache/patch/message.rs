use crate::{error::CacheError, ArcStr};
use tokio::sync::oneshot;

/// Messages for the Patch Actor.
#[derive(Debug)]
pub enum Message {
    /// Get a patch by mailing list and message ID
    Get {
        list: ArcStr,
        message_id: ArcStr,
        tx: oneshot::Sender<Result<String, CacheError>>,
    },
    /// Invalidate a specific patch
    Invalidate {
        list: ArcStr,
        message_id: ArcStr,
        tx: oneshot::Sender<Result<(), CacheError>>,
    },
    /// Check if a patch is available in cache
    IsAvailable {
        list: ArcStr,
        message_id: ArcStr,
        tx: oneshot::Sender<Result<bool, CacheError>>,
    },
}
