use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::utils::ArcSlice;
use crate::{ArcStr, api::lore::{LorePage, LorePatchMetadata, LoreMailingList}};

/// Mock implementation of the Lore API for testing purposes.
///
/// This struct contains predefined responses for various Lore API operations,
/// allowing tests to run without making actual HTTP requests.
#[derive(Debug, Clone)]
pub struct Mock {
    responses: Arc<Mutex<HashMap<String, ArcStr>>>,
}

impl Mock {
    /// Creates a new mock instance with the provided responses.
    ///
    /// # Arguments
    /// * `responses` - Initial response cache mapping operation keys to responses
    pub fn new(responses: HashMap<String, ArcStr>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
        }
    }

    /// Creates a new empty mock instance.
    pub fn empty() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Fetches a patch feed from a specific mailing list with pagination.
    ///
    /// # Arguments
    /// * `target_list` - The mailing list name (e.g., "amd-gfx", "linux-kernel")
    /// * `min_index` - The offset for pagination (0-based)
    ///
    /// # Returns
    /// The parsed patch feed page, or an error if not found in mock responses.
    pub async fn get_patch_feed_page(
        &self,
        target_list: ArcStr,
        min_index: usize,
    ) -> anyhow::Result<Option<LorePage<LorePatchMetadata>>> {
        let responses = self.responses.lock().await;
        let key = format!("patch_feed_page_{target_list}_{min_index}");
        let xml = responses.get(&key).cloned().ok_or_else(|| {
            anyhow::anyhow!("Patch feed page not found in mock responses: {}", key)
        })?;
        
        // Parse the XML string into LorePage<LorePatchMetadata>
        let page: LorePage<LorePatchMetadata> =
            crate::api::lore::parse::parse_patch_feed_xml(&xml, min_index)?;
        Ok(Some(page))
    }

    /// Fetches a single page of available mailing lists with pagination.
    ///
    /// # Arguments
    /// * `min_index` - The offset for pagination (0-based)
    ///
    /// # Returns
    /// The parsed available lists page, or an error if not found in mock responses.
    pub async fn get_available_lists_page(
        &self,
        min_index: usize,
    ) -> anyhow::Result<Option<LorePage<LoreMailingList>>> {
        let responses = self.responses.lock().await;
        let key = format!("available_lists_page_{min_index}");
        let html = responses.get(&key).cloned().ok_or_else(|| {
            anyhow::anyhow!("Available lists page not found in mock responses: {}", key)
        })?;
        
        let page: LorePage<LoreMailingList> =
            crate::api::lore::parse::parse_available_lists_html(&html, min_index)?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "No available lists page found in mock responses: {}",
                        key
                    )
                })?;
        Ok(Some(page))
    }

    /// Fetches all available mailing lists, aggregating all paginated results.
    ///
    /// # Returns
    /// An `ArcSlice<LoreMailingList>` containing all available mailing lists.
    pub async fn get_available_lists(&self) -> anyhow::Result<ArcSlice<LoreMailingList>> {
        let responses = self.responses.lock().await;
        let mut all_lists = Vec::new();
        let mut min_index = 0;
        
        loop {
            let key = format!("available_lists_page_{min_index}");
            if let Some(html) = responses.get(&key) {
                let page: LorePage<LoreMailingList> =
                    crate::api::lore::parse::parse_available_lists_html(html, min_index)?
                        .ok_or_else(|| {
                        anyhow::anyhow!(
                            "No available lists page found in mock responses: {}",
                            key
                        )
                    })?;
                all_lists.extend(page.items.iter().cloned());
                if page.next_page_index.is_none() {
                    break;
                }
                min_index = page.next_page_index.unwrap();
            } else {
                break;
            }
        }
        
        Ok(ArcSlice::from(all_lists))
    }

    /// Fetches the HTML content of a specific patch.
    ///
    /// # Arguments
    /// * `target_list` - The mailing list name (e.g., "amd-gfx", "linux-kernel")
    /// * `message_id` - The unique message ID of the patch
    ///
    /// # Returns
    /// The HTML content of the patch, or an error if not found in mock responses.
    pub async fn get_patch_html(
        &self,
        target_list: ArcStr,
        message_id: ArcStr,
    ) -> anyhow::Result<ArcStr> {
        let responses = self.responses.lock().await;
        let key = format!("patch_html_{target_list}_{message_id}");
        responses.get(&key).cloned().ok_or_else(|| {
            anyhow::anyhow!("Patch HTML not found in mock responses: {}", key)
        })
    }

    /// Fetches a raw patch in plain text format.
    ///
    /// # Arguments
    /// * `target_list` - The mailing list name
    /// * `message_id` - The unique message ID of the patch
    ///
    /// # Returns
    /// The raw patch content as plain text, or an error if not found in mock responses.
    pub async fn get_raw_patch(
        &self,
        target_list: ArcStr,
        message_id: ArcStr,
    ) -> anyhow::Result<ArcStr> {
        let responses = self.responses.lock().await;
        let key = format!("raw_patch_{target_list}_{message_id}");
        responses.get(&key).cloned().ok_or_else(|| {
            anyhow::anyhow!("Raw patch not found in mock responses: {}", key)
        })
    }

    /// Fetches patch metadata in JSON format.
    ///
    /// # Arguments
    /// * `target_list` - The mailing list name
    /// * `message_id` - The unique message ID of the patch
    ///
    /// # Returns
    /// The patch metadata as JSON, or an error if not found in mock responses.
    pub async fn get_patch_metadata(
        &self,
        target_list: ArcStr,
        message_id: ArcStr,
    ) -> anyhow::Result<ArcStr> {
        let responses = self.responses.lock().await;
        let key = format!("patch_metadata_{target_list}_{message_id}");
        responses.get(&key).cloned().ok_or_else(|| {
            anyhow::anyhow!("Patch metadata not found in mock responses: {}", key)
        })
    }
}
