use crate::ArcPath;
use crate::api::lore::LoreFeedItem;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data structure for persisting the feed cache to disk.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CacheData {
    /// Cached patch metadata per mailing list
    pub feeds: HashMap<String, Vec<LoreFeedItem>>,
    /// Last updated time per mailing list for cache validation
    pub last_updated: HashMap<String, Option<DateTime<Utc>>>,
}

/// Internal state for the Feed Actor.
pub struct FeedData {
    /// Cached patch metadata per mailing list
    pub feeds: HashMap<String, Vec<LoreFeedItem>>,
    /// Last updated time per mailing list for cache validation
    pub last_updated: HashMap<String, Option<DateTime<Utc>>>,
    /// Directory for cache files
    pub cache_dir: ArcPath,
}

impl FeedData {
    /// Creates a new FeedData instance.
    pub fn new(cache_dir: ArcPath) -> Self {
        Self {
            feeds: HashMap::new(),
            last_updated: HashMap::new(),
            cache_dir,
        }
    }

    /// Gets the cache file path for a specific mailing list.
    pub fn get_cache_path(&self, list: &str) -> ArcPath {
        ArcPath::from(&self.cache_dir.join(format!("{}.toml", list)))
    }

    /// Updates the last_updated time for a mailing list.
    pub fn update_last_updated(&mut self, list: String, last_updated: Option<DateTime<Utc>>) {
        self.last_updated.insert(list, last_updated);
    }

    /// Checks if the cache is valid for a mailing list by comparing the last_updated time.
    pub fn is_cache_valid(&self, list: &str, api_last_updated: Option<DateTime<Utc>>) -> bool {
        let cached_last_updated = self.last_updated.get(list).copied().flatten();
        match (cached_last_updated, api_last_updated) {
            (Some(cached), Some(api)) => cached >= api,
            (Some(_), None) => true,  // We have data, API doesn't
            (None, Some(_)) => false, // We don't have data, API does
            (None, None) => true,     // Neither has data, consider valid
        }
    }

    /// Converts to CacheData for persistence.
    pub fn to_cache_data(&self) -> CacheData {
        CacheData {
            feeds: self.feeds.clone(),
            last_updated: self.last_updated.clone(),
        }
    }

    /// Updates from CacheData after loading from disk.
    pub fn from_cache_data(&mut self, data: CacheData) {
        self.feeds = data.feeds;
        self.last_updated = data.last_updated;
    }

    /// Gets the number of cached items for a mailing list.
    pub fn len(&self, list: &str) -> usize {
        self.feeds.get(list).map(|v| v.len()).unwrap_or(0)
    }

    /// Checks if a range is available for a mailing list.
    pub fn contains_range(&self, list: &str, range: std::ops::Range<usize>) -> bool {
        self.feeds
            .get(list)
            .map(|v| v.len() >= range.end)
            .unwrap_or(false)
    }
}
