use super::data::FeedData;
use super::message::Message;
use crate::api::lore::LorePatchMetadata;
use crate::{ArcPath, ArcStr, error::CacheError};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[cfg(not(test))]
use crate::{api::lore::LoreApi, app::config::Config, fs::Fs, log::Log};
#[cfg(test)]
use crate::{
    api::lore::mock::MockLoreApi as LoreApi, app::config::mock::MockConfig as Config,
    fs::mock::MockFs as Fs, log::mock::MockLog as Log,
};

const SCOPE: &str = "app.cache.feed";

/// Core implementation for the Feed Actor.
pub struct Core {
    /// Lore API actor for fetching patch metadata
    lore: LoreApi,
    /// Filesystem actor for persistence
    fs: Fs,
    /// Config actor for configuration
    config: Config,
    /// Log actor for logging
    log: Log,
    /// Internal state
    data: FeedData,
}

impl Core {
    /// Creates a new Core instance.
    pub async fn new(lore: LoreApi, fs: Fs, config: Config, log: Log) -> Result<Self, CacheError> {
        let cache_dir = config
            .path(crate::app::config::PathOpt::CachePath)
            .await
            .map_err(|e| match e {
                crate::error::ConfigError::Fatal(fatal) => CacheError::Fatal(fatal),
                crate::error::ConfigError::ParseFailed { .. }
                | crate::error::ConfigError::InvalidValue { .. }
                | crate::error::ConfigError::FileOperationFailed { .. } => {
                    CacheError::FileOperationFailed {
                        path: "config".to_string(),
                        operation: "get cache path".to_string(),
                        source: std::io::Error::new(std::io::ErrorKind::NotFound, format!("{}", e)),
                    }
                }
            })?;
        let feed_cache_dir = ArcPath::from(&cache_dir.join("feed"));
        let data = FeedData::new(feed_cache_dir);

        Ok(Self {
            lore,
            fs,
            config,
            log,
            data,
        })
    }

    /// Spawns the actor and returns the public interface and join handle.
    pub async fn init(mut self, mut rx: mpsc::Receiver<Message>) {
        while let Some(message) = rx.recv().await {
            match message {
                Message::Get { list, index, tx } => {
                    let result = self.handle_get(&list, index).await;
                    let _ = tx.send(result);
                }
                Message::GetSlice { list, range, tx } => {
                    let result = self.handle_get_slice(&list, range).await;
                    let _ = tx.send(result);
                }
                Message::Refresh { list, tx } => {
                    let result = self.refresh_cache(&list).await;
                    let _ = tx.send(result);
                }
                Message::Invalidate { list, tx } => {
                    let result = self.handle_invalidate(&list).await;
                    let _ = tx.send(result);
                }
                Message::IsAvailable { list, range, tx } => {
                    let result = Ok(self.handle_is_available(&list, range));
                    let _ = tx.send(result);
                }
                Message::Len { list, tx } => {
                    let result = Ok(self.data.len(&list));
                    let _ = tx.send(result);
                }
                Message::Persist { list, tx } => {
                    let result = self.persist_cache(&list).await;
                    let _ = tx.send(result);
                }
                Message::Load { list, tx } => {
                    let result = self.load_cache(&list).await;
                    let _ = tx.send(result);
                }
                Message::IsLoaded { list, tx } => {
                    let result = Ok(self.data.feeds.contains_key(&list.to_string()));
                    let _ = tx.send(result);
                }
            }
        }
    }

    /// Handles getting a single patch metadata item by index.
    async fn handle_get(
        &mut self,
        list: &str,
        index: usize,
    ) -> Result<Option<LorePatchMetadata>, CacheError> {
        // Check if we have the item in cache
        if let Some(item) = self.data.feeds.get(list).and_then(|v| v.get(index)) {
            return Ok(Some(item.clone()));
        }

        // Item not in cache, try to fetch it
        self.fetch_until_index(list, index).await?;

        // Return the item if we now have it
        Ok(self
            .data
            .feeds
            .get(list)
            .and_then(|v| v.get(index))
            .cloned())
    }

    /// Handles getting a slice of patch metadata items by range.
    async fn handle_get_slice(
        &mut self,
        list: &str,
        range: std::ops::Range<usize>,
    ) -> Result<Vec<LorePatchMetadata>, CacheError> {
        // Check if we have the entire range in cache
        if self.data.contains_range(list, range.clone()) {
            let feed = self.data.feeds.get(list).unwrap();
            return Ok(feed[range.start..range.end].to_vec());
        }

        // Range not fully available, try to fetch it
        self.fetch_until_index(list, range.end).await?;

        // Return what we have
        let feed = self.data.feeds.get(list);
        match feed {
            Some(feed) => {
                if range.start >= feed.len() {
                    return Ok(Vec::new());
                }
                let end = range.end.min(feed.len());
                Ok(feed[range.start..end].to_vec())
            }
            None => Ok(Vec::new()),
        }
    }

    /// Handles invalidating the cache for a specific mailing list.
    async fn handle_invalidate(&mut self, list: &str) -> Result<(), CacheError> {
        self.data.feeds.remove(list);
        self.data.last_updated.remove(list);
        self.persist_cache(list).await
    }

    /// Handles checking if a range is available.
    fn handle_is_available(&self, list: &str, range: std::ops::Range<usize>) -> bool {
        self.data.contains_range(list, range)
    }

    /// Gets the newest cached item for a mailing list.
    fn get_newest_cached_item(&self, list: &str) -> Option<&LorePatchMetadata> {
        self.data.feeds.get(list)?.first()
    }

    /// Fetches pages until we have enough data to reach the specified index.
    async fn fetch_until_index(
        &mut self,
        list: &str,
        target_index: usize,
    ) -> Result<(), CacheError> {
        let mut min_index = self.data.len(list);

        // If we already have enough data, no need to fetch
        if min_index > target_index {
            return Ok(());
        }

        self.log.info(
            SCOPE,
            format!(
                "Fetching pages for list '{}' until index {} (current: {})",
                list, target_index, min_index
            ),
        );

        // Fetch pages until we have enough data or reach the end
        loop {
            let page = self
                .lore
                .get_patch_feed_page(ArcStr::from(list), min_index)
                .await
                .map_err(|e| match e {
                    crate::error::LoreApiError::Fatal(fatal) => CacheError::Fatal(fatal),
                    crate::error::LoreApiError::RequestFailed {
                        endpoint,
                        message,
                        retryable: _,
                    } => CacheError::FileOperationFailed {
                        path: endpoint,
                        operation: "fetch patch feed".to_string(),
                        source: std::io::Error::new(std::io::ErrorKind::Other, message),
                    },
                    crate::error::LoreApiError::ParseFailed {
                        format,
                        operation,
                        details,
                    } => CacheError::SerializationFailed {
                        message: format!(
                            "Failed to parse {} for {}: {}",
                            format, operation, details
                        ),
                        source: None,
                    },
                })?;

            match page {
                Some(page) => {
                    let items_len = page.items.len();
                    if items_len == 0 {
                        // No more items available
                        break;
                    }

                    // Add new items to the cache (prepend since they're newer)
                    let feed = self
                        .data
                        .feeds
                        .entry(list.to_string())
                        .or_insert_with(Vec::new);
                    feed.extend(page.items);

                    // Update last_updated with the newest item's timestamp
                    if let Some(newest_item) = feed.first() {
                        let last_update = newest_item.last_update;
                        self.data
                            .update_last_updated(list.to_string(), Some(last_update));
                    }

                    // Update min_index for next page
                    min_index = page.next_page_index.unwrap_or(min_index + items_len);

                    // If no next page or we have enough data, we're done
                    if page.next_page_index.is_none() || min_index > target_index {
                        break;
                    }
                }
                None => {
                    // No more pages available
                    break;
                }
            }
        }

        // Persist the updated cache
        self.persist_cache(list).await?;

        self.log.info(
            SCOPE,
            format!(
                "Fetched pages for list '{}', now have {} items",
                list,
                self.data.len(list)
            ),
        );

        Ok(())
    }

    /// Refreshes the cache for a specific mailing list with smart pagination.
    async fn refresh_cache(&mut self, list: &str) -> Result<(), CacheError> {
        self.log
            .info(SCOPE, format!("Refreshing feed cache for list: {}", list));

        // Check if cache is empty
        if self
            .data
            .feeds
            .get(list)
            .map(|v| v.is_empty())
            .unwrap_or(true)
        {
            // Cache is empty, fetch just one page
            self.log.info(
                SCOPE,
                format!("Cache empty for list '{}', fetching first page", list),
            );
            self.fetch_until_index(list, 0).await?;
            return Ok(());
        }

        // Cache is not empty, fetch pages until we find an item we already have
        let newest_cached_message_id = self
            .get_newest_cached_item(list)
            .expect("Cache should not be empty at this point")
            .message_id
            .clone();

        self.log.info(
            SCOPE,
            format!(
                "Cache not empty for list '{}', fetching until we find item: {}",
                list, newest_cached_message_id
            ),
        );

        let mut min_index = 0;
        let mut new_items_count = 0;

        // Fetch pages until we find a page containing our newest cached item
        loop {
            let page = self
                .lore
                .get_patch_feed_page(ArcStr::from(list), min_index)
                .await
                .map_err(|e| match e {
                    crate::error::LoreApiError::Fatal(fatal) => CacheError::Fatal(fatal),
                    crate::error::LoreApiError::RequestFailed {
                        endpoint,
                        message,
                        retryable: _,
                    } => CacheError::FileOperationFailed {
                        path: endpoint,
                        operation: "fetch patch feed".to_string(),
                        source: std::io::Error::new(std::io::ErrorKind::Other, message),
                    },
                    crate::error::LoreApiError::ParseFailed {
                        format,
                        operation,
                        details,
                    } => CacheError::SerializationFailed {
                        message: format!(
                            "Failed to parse {} for {}: {}",
                            format, operation, details
                        ),
                        source: None,
                    },
                })?;

            match page {
                Some(page) => {
                    let items_len = page.items.len();
                    if items_len == 0 {
                        // No more items available
                        break;
                    }

                    // Check if this page contains our newest cached item
                    let contains_cached_item = page
                        .items
                        .iter()
                        .any(|item| item.message_id == newest_cached_message_id);

                    if contains_cached_item {
                        // Found our newest cached item, we're done
                        self.log.info(
                            SCOPE,
                            format!(
                                "Found newest cached item in page for list '{}', stopping refresh",
                                list
                            ),
                        );
                        break;
                    }

                    // This page contains only new items, add them to the cache
                    let feed = self.data.feeds.get_mut(list).unwrap();
                    feed.extend(page.items);
                    new_items_count += items_len;

                    // Update last_updated with the newest item's timestamp
                    if let Some(newest_item) = feed.first() {
                        let last_update = newest_item.last_update;
                        self.data
                            .update_last_updated(list.to_string(), Some(last_update));
                    }

                    // Update min_index for next page
                    min_index = page.next_page_index.unwrap_or(min_index + items_len);

                    // If no next page, we're done
                    if page.next_page_index.is_none() {
                        break;
                    }
                }
                None => {
                    // No more pages available
                    break;
                }
            }
        }

        // Persist the updated cache
        self.persist_cache(list).await?;

        self.log.info(
            SCOPE,
            format!(
                "Refreshed cache for list '{}', added {} new items, total: {}",
                list,
                new_items_count,
                self.data.len(list)
            ),
        );

        Ok(())
    }

    /// Persists the cache for a specific mailing list to the filesystem.
    async fn persist_cache(&self, list: &str) -> Result<(), CacheError> {
        let cache_path = self.data.get_cache_path(list);

        // Create parent directory if it doesn't exist
        if let Some(parent) = cache_path.parent() {
            self.fs.mkdir(ArcPath::from(parent)).await?;
        }

        // Create cache data for this list only
        let cache_data = super::data::CacheData {
            feeds: {
                let mut feeds = HashMap::new();
                if let Some(items) = self.data.feeds.get(list) {
                    feeds.insert(list.to_string(), items.clone());
                }
                feeds
            },
            last_updated: {
                let mut last_updated = HashMap::new();
                if let Some(updated) = self.data.last_updated.get(list) {
                    last_updated.insert(list.to_string(), *updated);
                }
                last_updated
            },
        };

        let content =
            toml::to_string_pretty(&cache_data).map_err(|e| CacheError::SerializationFailed {
                message: format!("Failed to serialize cache data: {}", e),
                source: Some(Box::new(e)),
            })?;

        // Write the file
        let mut file = self.fs.write_file(cache_path.clone()).await?;

        use tokio::io::AsyncWriteExt;
        file.write_all(content.as_bytes())
            .await
            .map_err(|e| CacheError::FileOperationFailed {
                path: cache_path.to_string_lossy().to_string(),
                operation: "write cache file".to_string(),
                source: e,
            })?;

        Ok(())
    }

    /// Loads the cache for a specific mailing list from the filesystem.
    async fn load_cache(&mut self, list: &str) -> Result<(), CacheError> {
        let cache_path = self.data.get_cache_path(list);

        // Check if file exists by trying to read it
        let file = match self.fs.read_file(cache_path.clone()).await {
            Ok(file) => file,
            Err(e) => {
                match e {
                    crate::error::FsError::Fatal(fatal) => return Err(CacheError::Fatal(fatal)),
                    crate::error::FsError::OperationFailed { .. } => {
                        // File doesn't exist, that's ok
                        return Ok(());
                    }
                }
            }
        };

        // Read the content
        use tokio::io::AsyncReadExt;
        let mut content = String::new();
        let mut file = file;
        file.read_to_string(&mut content)
            .await
            .map_err(|e| CacheError::FileOperationFailed {
                path: cache_path.to_string_lossy().to_string(),
                operation: "read cache file content".to_string(),
                source: e,
            })?;

        let cache_data: super::data::CacheData =
            toml::from_str(&content).map_err(|e| CacheError::SerializationFailed {
                message: format!("Failed to deserialize cache data: {}", e),
                source: Some(Box::new(e)),
            })?;

        // Merge with existing data
        self.data.feeds.extend(cache_data.feeds);
        self.data.last_updated.extend(cache_data.last_updated);

        self.log.info(
            SCOPE,
            format!("Loaded {} items for list: {}", self.data.len(list), list),
        );
        Ok(())
    }
}
