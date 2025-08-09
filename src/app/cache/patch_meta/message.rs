use crate::ArcStr;
use crate::api::lore::LorePatchMetadata;
use tokio::sync::oneshot;

/// Messages for the PatchMetaCache actor.
#[derive(Debug)]
pub enum Message {
    /// Get a single patch metadata item by index for a given mailing list (demand-driven)
    Get {
        list: ArcStr,
        index: usize,
        tx: oneshot::Sender<anyhow::Result<Option<LorePatchMetadata>>>,
    },
    /// Get a slice of patch metadata items by range for a given mailing list (demand-driven)
    GetSlice {
        list: ArcStr,
        range: std::ops::Range<usize>,
        tx: oneshot::Sender<anyhow::Result<Vec<LorePatchMetadata>>>,
    },
    /// Invalidate the current cache
    InvalidateCache { list: ArcStr },
    /// Persist the cache to the filesystem
    PersistCache {
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Load the cache from the filesystem
    LoadCache {
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Get the number of cached patch metadata items for a given mailing list
    Len {
        list: ArcStr,
        tx: oneshot::Sender<usize>,
    },
    /// Check if the cache is still valid for a given mailing list (by last_update of 0th item)
    IsCacheValid {
        list: ArcStr,
        tx: oneshot::Sender<anyhow::Result<bool>>,
    },
    /// Check if the cache contains the given range for a mailing list (without fetching)
    ContainsRange {
        list: ArcStr,
        range: std::ops::Range<usize>,
        tx: oneshot::Sender<bool>,
    },
}
