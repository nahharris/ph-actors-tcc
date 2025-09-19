use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ArcStr;
use crate::api::lore::LorePatchMetadata;
use crate::app::cache::feed::MockData;

/// Mock implementation of the FeedCache actor for testing purposes.
///
/// This struct stores feed cache data in memory,
/// allowing tests to run without creating actual cache files.
#[derive(Debug, Clone)]
pub struct Mock {
    data: Arc<Mutex<MockData>>,
}

impl Mock {
    /// Creates a new mock instance with the provided feed cache data.
    ///
    /// # Arguments
    /// * `data` - Initial feed cache data
    pub fn new(data: MockData) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }

    /// Fetches a single patch metadata item by index for a given mailing list.
    /// Mock implementation retrieves the item from stored data.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    /// * `index` - The index of the item to retrieve
    ///
    /// # Returns
    /// Ok(Some(LorePatchMetadata)) if found, Ok(None) if not found
    pub async fn get(
        &self,
        list: ArcStr,
        index: usize,
    ) -> anyhow::Result<Option<LorePatchMetadata>> {
        let data = self.data.lock().await;
        Ok(data.feeds.get(&list).and_then(|v| v.get(index)).cloned())
    }

    /// Fetches a slice of patch metadata items by range for a given mailing list.
    /// Mock implementation retrieves the slice from stored data.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    /// * `range` - The range of items to retrieve
    ///
    /// # Returns
    /// Ok(Vec<LorePatchMetadata>) with the requested items
    pub async fn get_slice(
        &self,
        list: ArcStr,
        range: std::ops::Range<usize>,
    ) -> anyhow::Result<Vec<LorePatchMetadata>> {
        let data = self.data.lock().await;
        Ok(data
            .feeds
            .get(&list)
            .map(|v| v[range].to_vec())
            .unwrap_or_default())
    }

    /// Refreshes the cache for a specific mailing list.
    /// Mock implementation is a no-op that always succeeds.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    ///
    /// # Returns
    /// Ok(()) always
    pub async fn refresh(&self, _list: ArcStr) -> anyhow::Result<()> {
        Ok(())
    }

    /// Invalidates the cache for a specific mailing list.
    /// Mock implementation removes the list from stored data.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    ///
    /// # Returns
    /// Ok(()) always
    pub async fn invalidate(&self, list: ArcStr) -> anyhow::Result<()> {
        let mut data = self.data.lock().await;
        data.feeds.remove(&list);
        Ok(())
    }

    /// Checks if the requested range is available in cache for a mailing list.
    /// Mock implementation checks the stored data.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    /// * `range` - The range to check availability for
    ///
    /// # Returns
    /// true if the range is available, false otherwise
    pub async fn is_available(&self, list: ArcStr, range: std::ops::Range<usize>) -> bool {
        let data = self.data.lock().await;
        data.feeds
            .get(&list)
            .map(|v| v.len() >= range.end)
            .unwrap_or(false)
    }

    /// Returns the number of cached items for a given mailing list.
    /// Mock implementation returns the length from stored data.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    ///
    /// # Returns
    /// The number of cached items
    pub async fn len(&self, list: ArcStr) -> usize {
        let data = self.data.lock().await;
        data.feeds.get(&list).map(|v| v.len()).unwrap_or(0)
    }

    /// Checks if the cache has been loaded from disk for a given mailing list.
    /// Mock implementation checks if the list exists in stored data.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    ///
    /// # Returns
    /// true if the cache is loaded, false otherwise
    pub async fn is_loaded(&self, list: ArcStr) -> bool {
        let data = self.data.lock().await;
        data.feeds.contains_key(&list)
    }

    /// Ensures the cache is loaded for a given mailing list.
    /// Mock implementation is a no-op that always succeeds.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    ///
    /// # Returns
    /// Ok(()) always
    pub async fn ensure_loaded(&self, _list: ArcStr) -> anyhow::Result<()> {
        Ok(())
    }

    /// Persists the cache for a specific mailing list to the filesystem.
    /// Mock implementation is a no-op that always succeeds.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    ///
    /// # Returns
    /// Ok(()) always
    pub async fn persist(&self, _list: ArcStr) -> anyhow::Result<()> {
        Ok(())
    }

    /// Loads the cache for a specific mailing list from the filesystem.
    /// Mock implementation is a no-op that always succeeds.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    ///
    /// # Returns
    /// Ok(()) always
    pub async fn load(&self, _list: ArcStr) -> anyhow::Result<()> {
        Ok(())
    }

    /// Gets the mock data for inspection in tests.
    ///
    /// # Returns
    /// A copy of the current mock data
    pub async fn get_data(&self) -> MockData {
        let data = self.data.lock().await;
        data.clone()
    }
}
