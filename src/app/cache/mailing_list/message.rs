use crate::api::lore::LoreMailingList;
use tokio::sync::oneshot;

/// Messages for the MailingListCache actor.
#[derive(Debug)]
pub enum Message {
    /// Get a single mailing list by index (demand-driven)
    Get {
        index: usize,
        tx: oneshot::Sender<anyhow::Result<Option<LoreMailingList>>>,
    },
    /// Get a slice of mailing lists by range (demand-driven)
    GetSlice {
        range: std::ops::Range<usize>,
        tx: oneshot::Sender<anyhow::Result<Vec<LoreMailingList>>>,
    },
    /// Invalidate the current cache
    InvalidateCache,
    /// Persist the cache to the filesystem
    PersistCache {
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Load the cache from the filesystem
    LoadCache {
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Get the number of cached mailing lists
    Len { tx: oneshot::Sender<usize> },
    /// Check if the cache is still valid (by last_update of 0th item)
    IsCacheValid {
        tx: oneshot::Sender<anyhow::Result<bool>>,
    },
}
