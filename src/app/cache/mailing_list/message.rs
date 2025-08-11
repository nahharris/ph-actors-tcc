use crate::api::lore::LoreMailingList;
use tokio::sync::oneshot;

/// Messages for the Mailing List Actor.
#[derive(Debug)]
pub enum Message {
    /// Get a single mailing list by index
    Get {
        index: usize,
        tx: oneshot::Sender<anyhow::Result<Option<LoreMailingList>>>,
    },
    /// Get a slice of mailing lists by range
    GetSlice {
        range: std::ops::Range<usize>,
        tx: oneshot::Sender<anyhow::Result<Vec<LoreMailingList>>>,
    },
    /// Refresh the cache by fetching from API
    Refresh {
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Invalidate the cache
    Invalidate {
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Check if the requested range is available in cache
    IsAvailable {
        range: std::ops::Range<usize>,
        tx: oneshot::Sender<bool>,
    },
    /// Get the number of cached mailing lists
    Len { tx: oneshot::Sender<usize> },
    /// Persist the cache to filesystem
    Persist {
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Load the cache from filesystem
    Load {
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
}
