use std::collections::HashMap;

use super::data::{LoreMailingList, LorePage, LorePatchMetadata};
use super::parse;
use crate::{ArcSlice, error::LoreApiError};
use crate::{ArcStr, api::lore::message::LoreApiMessage};

#[cfg(not(test))]
use crate::net::Net;
#[cfg(test)]
use crate::net::mock::MockNet as Net;

const DOMAIN: &str = "https://lore.kernel.org";

/// The core of the Lore API system that handles Lore-specific HTTP requests.
///
/// This struct provides thread-safe access to Lore API operations through an actor pattern.
/// It wraps the networking actor and provides domain-specific methods for interacting
/// with the Lore Kernel Archive.
///
/// # Features
/// - Thread-safe Lore API operations through actor pattern
/// - Domain-specific URL construction and request handling
/// - Integration with networking system
/// - Proper error handling and context
///
/// # Examples
/// ```ignore
/// let core = Core::new(net);
/// let (lore_api, _) = core.spawn();
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads through the actor pattern.
/// All operations are handled sequentially to ensure consistency.
#[derive(Debug)]
pub struct Core {
    /// The networking actor for making HTTP requests
    net: Net,
    /// The base domain for Lore API requests
    domain: ArcStr,
}

impl Core {
    /// Creates a new Lore API core instance.
    ///
    /// # Arguments
    /// * `net` - The networking actor for making HTTP requests
    ///
    /// # Returns
    /// A new instance of `Core` configured for the Lore Kernel Archive.
    pub fn new(net: Net) -> Self {
        Self {
            net,
            domain: ArcStr::from(DOMAIN),
        }
    }

    /// Transforms the Lore API core instance into an actor.
    ///
    /// This method spawns a new task that will handle Lore API operations
    /// asynchronously through a message channel. All operations are processed
    /// sequentially to ensure consistency.
    ///
    /// # Returns
    /// A tuple containing:
    /// - The `LoreApi` interface
    /// - A join handle for the spawned task
    ///
    /// # Panics
    /// This function will panic if the underlying task fails to spawn.
    pub async fn init(self, mut rx: tokio::sync::mpsc::Receiver<LoreApiMessage>) {
        while let Some(message) = rx.recv().await {
            match message {
                LoreApiMessage::GetPatchFeedPage {
                    target_list,
                    min_index,
                    tx,
                } => {
                    let response = self
                        .handle_get_patch_feed_page(&target_list, min_index)
                        .await;
                    let _ = tx.send(response);
                }
                LoreApiMessage::GetAvailableLists { tx } => {
                    let response = self.handle_get_available_lists().await;
                    let _ = tx.send(response);
                }
                LoreApiMessage::GetAvailableListsPage { min_index, tx } => {
                    let response = self.handle_get_available_lists_page(min_index).await;
                    let _ = tx.send(response);
                }
                LoreApiMessage::GetPatchHtml {
                    target_list,
                    message_id,
                    tx,
                } => {
                    let response = self.handle_get_patch_html(&target_list, &message_id).await;
                    let _ = tx.send(response);
                }
                LoreApiMessage::GetRawPatch {
                    target_list,
                    message_id,
                    tx,
                } => {
                    let response = self.handle_get_raw_patch(&target_list, &message_id).await;
                    let _ = tx.send(response);
                }
                LoreApiMessage::GetPatchMetadata {
                    target_list,
                    message_id,
                    tx,
                } => {
                    let response = self
                        .handle_get_patch_metadata(&target_list, &message_id)
                        .await;
                    let _ = tx.send(response);
                }
            }
        }
    }

    /// Handles GET patch feed requests
    async fn handle_get_patch_feed_page(
        &self,
        target_list: &str,
        min_index: usize,
    ) -> Result<Option<LorePage<LorePatchMetadata>>, LoreApiError> {
        let url = format!(
            "{}/{}/?x=A&q=((s:patch+OR+s:rfc)+AND+NOT+s:re:)&o={}",
            self.domain, target_list, min_index
        );

        let mut headers = HashMap::new();
        headers.insert(
            ArcStr::from("Accept"),
            ArcStr::from("text/html,application/xhtml+xml,application/xml"),
        );

        let response = self.net.get(ArcStr::from(&url), Some(headers)).await?;
        // Check for end of feed indicator
        if <ArcStr as AsRef<str>>::as_ref(&response) == "</feed>"
            || response.contains("[No results found]")
        {
            return Ok(None);
        }

        let page = parse::parse_patch_feed_xml(&response, min_index)?;

        Ok(Some(page))
    }

    /// Handles GET available lists requests
    async fn handle_get_available_lists(&self) -> Result<ArcSlice<LoreMailingList>, LoreApiError> {
        let mut all_items = Vec::new();
        let mut min_index = 0;
        loop {
            let page = self.handle_get_available_lists_page(min_index).await?;
            let Some(page) = page else {
                break;
            };

            all_items.extend(page.items);

            if let Some(next) = page.next_page_index {
                min_index = next;
            } else {
                break;
            }
        }
        Ok(ArcSlice::from(&all_items[..]))
    }

    /// Handles GET available lists requests
    async fn handle_get_available_lists_page(
        &self,
        min_index: usize,
    ) -> Result<Option<LorePage<LoreMailingList>>, LoreApiError> {
        let url = ArcStr::from(&format!("{}/?&o={}", self.domain, min_index));

        let mut headers = HashMap::new();
        headers.insert(
            ArcStr::from("Accept"),
            ArcStr::from("text/html,application/xhtml+xml,application/xml"),
        );

        let html = self.net.get(url, Some(headers)).await?;
        parse::parse_available_lists_html(&html, min_index)
    }

    /// Handles GET patch HTML requests
    async fn handle_get_patch_html(
        &self,
        target_list: &str,
        message_id: &str,
    ) -> Result<ArcStr, LoreApiError> {
        let url = format!("{}/{}/{}/", self.domain, target_list, message_id);

        let mut headers = HashMap::new();
        headers.insert(
            ArcStr::from("Accept"),
            ArcStr::from("text/html,application/xhtml+xml,application/xml"),
        );

        Ok(self.net.get(ArcStr::from(&url), Some(headers)).await?)
    }

    /// Handles GET raw patch requests
    async fn handle_get_raw_patch(
        &self,
        target_list: &str,
        message_id: &str,
    ) -> Result<ArcStr, LoreApiError> {
        let url = format!("{}/{}/{}/raw", self.domain, target_list, message_id);

        let mut headers = HashMap::new();
        headers.insert(ArcStr::from("Accept"), ArcStr::from("text/plain"));

        Ok(self.net.get(ArcStr::from(&url), Some(headers)).await?)
    }

    /// Handles GET patch metadata requests
    async fn handle_get_patch_metadata(
        &self,
        target_list: &str,
        message_id: &str,
    ) -> Result<ArcStr, LoreApiError> {
        let url = format!("{}/{}/{}/json", self.domain, target_list, message_id);

        let mut headers = HashMap::new();
        headers.insert(ArcStr::from("Accept"), ArcStr::from("application/json"));

        Ok(self.net.get(ArcStr::from(&url), Some(headers)).await?)
    }
}
