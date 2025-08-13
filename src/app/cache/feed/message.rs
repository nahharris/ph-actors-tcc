use crate::ArcStr;
use crate::api::lore::LoreFeedItem;
use tokio::sync::oneshot;

/// Messages for the Feed Actor.
#[derive(Debug)]
pub enum Message {
    /// Get a single patch metadata item by index for a given mailing list
    Get {
        list: ArcStr,
        index: usize,
        tx: oneshot::Sender<anyhow::Result<Option<LoreFeedItem>>>,
    },
    /// Get a slice of patch metadata items by range for a given mailing list
    GetSlice {
        list: ArcStr,
        range: std::ops::Range<usize>,
        tx: oneshot::Sender<anyhow::Result<Vec<LoreFeedItem>>>,
    },
    /// Refresh the cache for a specific mailing list
    Refresh {
        list: ArcStr,
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Invalidate the cache for a specific mailing list
    Invalidate {
        list: ArcStr,
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Check if the requested range is available in cache for a mailing list
    IsAvailable {
        list: ArcStr,
        range: std::ops::Range<usize>,
        tx: oneshot::Sender<bool>,
    },
    /// Get the number of cached items for a mailing list
    Len {
        list: ArcStr,
        tx: oneshot::Sender<usize>,
    },
    /// Persist the cache to filesystem
    Persist {
        list: ArcStr,
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Load the cache from filesystem
    Load {
        list: ArcStr,
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Check if the cache has been loaded from disk for a mailing list
    IsLoaded {
        list: ArcStr,
        tx: oneshot::Sender<bool>,
    },
}
