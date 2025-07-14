use anyhow::Context;
use std::collections::HashMap;
use tokio::task::JoinHandle;

use super::data::{LoreMailingList, LorePage, LorePatchMetadata};
use super::parse;
use crate::ArcSlice;
use crate::{ArcStr, api::lore::message::LoreApiMessage, net::Net};

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
            domain: ArcStr::from("https://lore.kernel.org"),
        }
    }

    /// Creates a new Lore API core instance with a custom domain.
    ///
    /// # Arguments
    /// * `net` - The networking actor for making HTTP requests
    /// * `domain` - The base domain for API requests
    ///
    /// # Returns
    /// A new instance of `Core` configured with the specified domain.
    pub fn with_domain(net: Net, domain: ArcStr) -> Self {
        Self { net, domain }
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
    pub fn spawn(self) -> (crate::api::lore::LoreApi, JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let handle = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    LoreApiMessage::GetPatchFeedPage {
                        target_list,
                        min_index,
                        tx,
                    } => {
                        let response = self
                            .handle_get_patch_feed_page(&target_list, min_index)
                            .await
                            .with_context(|| {
                                format!("GET patch feed failed for list: {}", target_list)
                            });
                        let _ = tx.send(response);
                    }
                    LoreApiMessage::GetAvailableLists { tx } => {
                        let response = self
                            .handle_get_available_lists()
                            .await
                            .with_context(|| "GET available lists failed");
                        let _ = tx.send(response);
                    }
                    LoreApiMessage::GetAvailableListsPage { min_index, tx } => {
                        let response = self
                            .handle_get_available_lists_page(min_index)
                            .await
                            .with_context(|| {
                                format!("GET available lists failed for index: {}", min_index)
                            });
                        let _ = tx.send(response);
                    }
                    LoreApiMessage::GetPatchHtml {
                        target_list,
                        message_id,
                        tx,
                    } => {
                        let response = self
                            .handle_get_patch_html(&target_list, &message_id)
                            .await
                            .with_context(|| {
                                format!(
                                    "GET patch HTML failed for list: {}, message: {}",
                                    target_list, message_id
                                )
                            });
                        let _ = tx.send(response);
                    }
                    LoreApiMessage::GetRawPatch {
                        target_list,
                        message_id,
                        tx,
                    } => {
                        let response = self
                            .handle_get_raw_patch(&target_list, &message_id)
                            .await
                            .with_context(|| {
                                format!(
                                    "GET raw patch failed for list: {}, message: {}",
                                    target_list, message_id
                                )
                            });
                        let _ = tx.send(response);
                    }
                    LoreApiMessage::GetPatchMetadata {
                        target_list,
                        message_id,
                        tx,
                    } => {
                        let response = self
                            .handle_get_patch_metadata(&target_list, &message_id)
                            .await
                            .with_context(|| {
                                format!(
                                    "GET patch metadata failed for list: {}, message: {}",
                                    target_list, message_id
                                )
                            });
                        let _ = tx.send(response);
                    }
                }
            }
        });

        (crate::api::lore::LoreApi::Actual(tx), handle)
    }

    /// Handles GET patch feed requests
    async fn handle_get_patch_feed_page(
        &self,
        target_list: &str,
        min_index: usize,
    ) -> anyhow::Result<Option<LorePage<LorePatchMetadata>>> {
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
        if <ArcStr as AsRef<str>>::as_ref(&response) == "</feed>" {
            return Ok(None);
        }

        let page = parse::parse_patch_feed_xml(&response, min_index)?;

        Ok(Some(page))
    }

    /// Handles GET available lists requests
    async fn handle_get_available_lists(&self) -> anyhow::Result<ArcSlice<LoreMailingList>> {
        let mut all_items = Vec::new();
        let mut min_index = 0;
        loop {
            let page = self.handle_get_available_lists_page(min_index).await?;
            all_items.extend(page.items);

            if let Some(next) = page.next_page_index
                && page.next_page_index != page.total_items
            {
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
    ) -> anyhow::Result<LorePage<LoreMailingList>> {
        let url = ArcStr::from(&format!("{}/?&o={}", self.domain, min_index));

        let mut headers = HashMap::new();
        headers.insert(
            ArcStr::from("Accept"),
            ArcStr::from("text/html,application/xhtml+xml,application/xml"),
        );

        let html = self.net.get(url, Some(headers)).await?;
        parse::parse_available_lists_html(&html, min_index)
            .context("Failed to parse available lists page")
    }

    /// Handles GET patch HTML requests
    async fn handle_get_patch_html(
        &self,
        target_list: &str,
        message_id: &str,
    ) -> anyhow::Result<ArcStr> {
        let url = format!("{}/{}/{}/", self.domain, target_list, message_id);

        let mut headers = HashMap::new();
        headers.insert(
            ArcStr::from("Accept"),
            ArcStr::from("text/html,application/xhtml+xml,application/xml"),
        );

        self.net.get(ArcStr::from(&url), Some(headers)).await
    }

    /// Handles GET raw patch requests
    async fn handle_get_raw_patch(
        &self,
        target_list: &str,
        message_id: &str,
    ) -> anyhow::Result<ArcStr> {
        let url = format!("{}/{}/{}/raw", self.domain, target_list, message_id);

        let mut headers = HashMap::new();
        headers.insert(ArcStr::from("Accept"), ArcStr::from("text/plain"));

        self.net.get(ArcStr::from(&url), Some(headers)).await
    }

    /// Handles GET patch metadata requests
    async fn handle_get_patch_metadata(
        &self,
        target_list: &str,
        message_id: &str,
    ) -> anyhow::Result<ArcStr> {
        let url = format!("{}/{}/{}/json", self.domain, target_list, message_id);

        let mut headers = HashMap::new();
        headers.insert(ArcStr::from("Accept"), ArcStr::from("application/json"));

        self.net.get(ArcStr::from(&url), Some(headers)).await
    }
}
