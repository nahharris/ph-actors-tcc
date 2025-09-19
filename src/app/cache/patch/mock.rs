use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ArcStr;
use crate::app::cache::patch::MockData;

/// Mock implementation of the PatchCache actor for testing purposes.
///
/// This struct stores patch cache data in memory,
/// allowing tests to run without creating actual cache files.
#[derive(Debug, Clone)]
pub struct Mock {
    data: Arc<Mutex<MockData>>,
}

impl Mock {
    /// Creates a new mock instance with the provided patch cache data.
    ///
    /// # Arguments
    /// * `data` - Initial patch cache data
    pub fn new(data: MockData) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }

    /// Fetches a patch by mailing list and message ID.
    /// Mock implementation retrieves the patch from stored data.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    /// * `message_id` - The message ID of the patch
    ///
    /// # Returns
    /// Ok(String) with the patch content if found
    pub async fn get(&self, list: ArcStr, message_id: ArcStr) -> anyhow::Result<String> {
        let data = self.data.lock().await;
        let key = format!("{}:{}", list, message_id);
        data.patches
            .get(&key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Patch not found in mock data"))
    }

    /// Invalidates a specific patch.
    /// Mock implementation removes the patch from stored data.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    /// * `message_id` - The message ID of the patch
    ///
    /// # Returns
    /// Ok(()) always
    pub async fn invalidate(&self, list: ArcStr, message_id: ArcStr) -> anyhow::Result<()> {
        let mut data = self.data.lock().await;
        let key = format!("{}:{}", list, message_id);
        data.patches.remove(&key);
        Ok(())
    }

    /// Checks if a patch is available in cache.
    /// Mock implementation checks the stored data.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    /// * `message_id` - The message ID of the patch
    ///
    /// # Returns
    /// true if the patch is available, false otherwise
    pub async fn is_available(&self, list: ArcStr, message_id: ArcStr) -> bool {
        let data = self.data.lock().await;
        let key = format!("{}:{}", list, message_id);
        data.patches.contains_key(&key)
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
