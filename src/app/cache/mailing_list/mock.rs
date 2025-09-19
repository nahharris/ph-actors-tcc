use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::lore::LoreMailingList;
use crate::app::cache::mailing_list::MockData;

/// Mock implementation of the MailingListCache actor for testing purposes.
///
/// This struct stores mailing list cache data in memory,
/// allowing tests to run without creating actual cache files.
#[derive(Debug, Clone)]
pub struct Mock {
    data: Arc<Mutex<MockData>>,
}

impl Mock {
    /// Creates a new mock instance with the provided mailing list cache data.
    ///
    /// # Arguments
    /// * `data` - Initial mailing list cache data
    pub fn new(data: MockData) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }

    /// Fetches a single mailing list by index.
    /// Mock implementation retrieves the item from stored data.
    ///
    /// # Arguments
    /// * `index` - The index of the mailing list to retrieve
    ///
    /// # Returns
    /// Ok(Some(LoreMailingList)) if found, Ok(None) if not found
    pub async fn get(&self, index: usize) -> anyhow::Result<Option<LoreMailingList>> {
        let data = self.data.lock().await;
        Ok(data.mailing_lists.get(index).cloned())
    }

    /// Fetches a slice of mailing lists by range.
    /// Mock implementation retrieves the slice from stored data.
    ///
    /// # Arguments
    /// * `range` - The range of mailing lists to retrieve
    ///
    /// # Returns
    /// Ok(Vec<LoreMailingList>) with the requested items
    pub async fn get_slice(
        &self,
        range: std::ops::Range<usize>,
    ) -> anyhow::Result<Vec<LoreMailingList>> {
        let data = self.data.lock().await;
        Ok(data.mailing_lists[range].to_vec())
    }

    /// Refreshes the cache by fetching from the API.
    /// Mock implementation is a no-op that always succeeds.
    ///
    /// # Returns
    /// Ok(()) always
    pub async fn refresh(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Invalidates the current cache.
    /// Mock implementation clears the stored data.
    ///
    /// # Returns
    /// Ok(()) always
    pub async fn invalidate(&self) -> anyhow::Result<()> {
        let mut data = self.data.lock().await;
        data.mailing_lists.clear();
        Ok(())
    }

    /// Checks if the requested range is available in cache.
    /// Mock implementation checks the stored data.
    ///
    /// # Arguments
    /// * `range` - The range to check availability for
    ///
    /// # Returns
    /// true if the range is available, false otherwise
    pub async fn is_available(&self, range: std::ops::Range<usize>) -> bool {
        let data = self.data.lock().await;
        range.end <= data.mailing_lists.len()
    }

    /// Returns the number of cached mailing lists.
    /// Mock implementation returns the length from stored data.
    ///
    /// # Returns
    /// The number of cached mailing lists
    pub async fn len(&self) -> usize {
        let data = self.data.lock().await;
        data.mailing_lists.len()
    }

    /// Persists the cache to the filesystem.
    /// Mock implementation is a no-op that always succeeds.
    ///
    /// # Returns
    /// Ok(()) always
    pub async fn persist(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Loads the cache from the filesystem.
    /// Mock implementation is a no-op that always succeeds.
    ///
    /// # Returns
    /// Ok(()) always
    pub async fn load(&self) -> anyhow::Result<()> {
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
