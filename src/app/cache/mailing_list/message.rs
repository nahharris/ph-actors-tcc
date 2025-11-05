use crate::{error::CacheError, api::lore::LoreMailingList};
use tokio::sync::oneshot;

/// Messages for the Mailing List Actor.
#[derive(Debug)]
pub enum Message {
    /// Get a single mailing list by index
    Get {
        index: usize,
        tx: oneshot::Sender<Result<Option<LoreMailingList>, CacheError>>,
    },
    /// Get a slice of mailing lists by range
    GetSlice {
        range: std::ops::Range<usize>,
        tx: oneshot::Sender<Result<Vec<LoreMailingList>, CacheError>>,
    },
    /// Refresh the cache by fetching from API
    Refresh {
        tx: oneshot::Sender<Result<(), CacheError>>,
    },
    /// Invalidate the cache
    Invalidate {
        tx: oneshot::Sender<Result<(), CacheError>>,
    },
    /// Check if the requested range is available in cache
    IsAvailable {
        range: std::ops::Range<usize>,
        tx: oneshot::Sender<Result<bool, CacheError>>,
    },
    /// Get the number of cached mailing lists
    Len { tx: oneshot::Sender<Result<usize, CacheError>> },
    /// Persist the cache to filesystem
    Persist {
        tx: oneshot::Sender<Result<(), CacheError>>,
    },
    /// Load the cache from filesystem
    Load {
        tx: oneshot::Sender<Result<(), CacheError>>,
    },
}
