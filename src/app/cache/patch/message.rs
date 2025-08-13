use crate::ArcStr;
use crate::api::lore::data::LorePatch;
use tokio::sync::oneshot;

/// Messages for the Patch Actor.
#[derive(Debug)]
pub enum Message {
    /// Get a patch by mailing list and message ID
    Get {
        list: ArcStr,
        message_id: ArcStr,
        tx: oneshot::Sender<anyhow::Result<LorePatch>>,
    },
    /// Invalidate a specific patch
    Invalidate {
        list: ArcStr,
        message_id: ArcStr,
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Check if a patch is available in cache
    IsAvailable {
        list: ArcStr,
        message_id: ArcStr,
        tx: oneshot::Sender<bool>,
    },
}
