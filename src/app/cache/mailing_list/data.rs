use crate::ArcPath;
use crate::api::lore::LoreMailingList;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Data structure for persisting the mailing list cache to disk.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CacheData {
    /// Cached mailing lists sorted alphabetically
    pub lists: Vec<LoreMailingList>,
    /// Last updated time from the 0-th item for cache validation
    pub last_updated: Option<DateTime<Utc>>,
}

/// Internal state for the Mailing List Actor.
pub struct MailingListData {
    /// Cached mailing lists sorted alphabetically
    pub lists: Vec<LoreMailingList>,
    /// Last updated time from the 0-th item for cache validation
    pub last_updated: Option<DateTime<Utc>>,
    /// Path to the cache file
    pub cache_path: ArcPath,
}

impl MailingListData {
    /// Creates a new MailingListData instance.
    pub fn new(cache_path: ArcPath) -> Self {
        Self {
            lists: Vec::new(),
            last_updated: None,
            cache_path,
        }
    }

    /// Updates the last_updated time from the 0-th item.
    pub fn update_last_updated(&mut self) {
        self.last_updated = self.lists.first().map(|list| list.last_update);
    }

    /// Converts to CacheData for persistence.
    pub fn to_cache_data(&self) -> CacheData {
        CacheData {
            lists: self.lists.clone(),
            last_updated: self.last_updated,
        }
    }

    /// Updates from CacheData after loading from disk.
    pub fn from_cache_data(&mut self, data: CacheData) {
        self.lists = data.lists;
        self.last_updated = data.last_updated;
    }
}
