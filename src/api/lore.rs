use tokio::sync::{mpsc, oneshot};

use crate::{error::LoreApiError, error::FatalActorError, ArcStr};
use crate::utils::ArcSlice;

#[cfg(not(test))]
use crate::net::Net;
#[cfg(test)]
use crate::net::mock::MockNet as Net;

mod core;
pub mod data;
mod message;
#[cfg(test)]
pub mod mock;
pub mod parse;
#[cfg(test)]
mod tests;

/// Actor name for error reporting.
pub const ACTOR_NAME: &'static str = "LoreApi";

// Re-export public types for external use
pub use data::{LoreMailingList, LorePage, LorePatchMetadata};
pub use message::LoreApiMessage;

/// The Lore API actor that provides a high-level interface for interacting with the Lore Kernel API.
///
/// This actor intermediates calls to the networking actor, providing domain-specific methods
/// for fetching patch feeds, available mailing lists, and individual patch HTML content
/// from the Lore Kernel Archive.
///
/// # Examples
/// ```ignore
/// let lore_api = LoreApi::spawn(net);
/// let patch_feed = lore_api.get_patch_feed("amd-gfx", 0).await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender or mock reference.
#[derive(Debug, Clone)]
pub struct LoreApi {
    /// A real Lore API actor that performs HTTP requests through the networking actor
    tx: mpsc::Sender<LoreApiMessage>,
}

impl LoreApi {
    pub fn spawn(net: Net) -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(net);
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }

    /// Fetches a patch feed from a specific mailing list with pagination.
    ///
    /// This method retrieves a paginated list of patches from the specified mailing list,
    /// filtering for patches and RFCs while excluding replies.
    ///
    /// # Arguments
    /// * `target_list` - The mailing list name (e.g., "amd-gfx", "linux-kernel")
    /// * `min_index` - The offset for pagination (0-based)
    ///
    /// # Returns
    /// The XML feed content as a string, or an error if the request fails.
    ///
    /// # Example
    /// ```ignore
    /// let feed = lore_api.get_patch_feed("amd-gfx", 0).await?;
    /// ```
    pub async fn get_patch_feed_page(
        &self,
        target_list: ArcStr,
        min_index: usize,
    ) -> Result<Option<LorePage<LorePatchMetadata>>, LoreApiError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(LoreApiMessage::GetPatchFeedPage {
                target_list,
                min_index,
                tx,
            })
            .await
            .map_err(|_e| {
                LoreApiError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::api::lore::ACTOR_NAME,
                    operation: "get patch feed page".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                LoreApiError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::api::lore::ACTOR_NAME,
                    operation: "get patch feed page".to_string(),
                    source: e,
                })
            })?
    }

    /// Fetches a single page of available mailing lists with pagination.
    ///
    /// This method retrieves a paginated list of all available mailing lists
    /// archived on the Lore Kernel Archive.
    ///
    /// # Arguments
    /// * `min_index` - The offset for pagination (0-based)
    ///
    /// # Returns
    /// A `LoreAvailableLists` struct containing pagination info and a list of items.
    pub async fn get_available_lists_page(
        &self,
        min_index: usize,
    ) -> Result<Option<LorePage<LoreMailingList>>, LoreApiError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(LoreApiMessage::GetAvailableListsPage { min_index, tx })
            .await
            .map_err(|_e| {
                LoreApiError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::api::lore::ACTOR_NAME,
                    operation: "get available lists page".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                LoreApiError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::api::lore::ACTOR_NAME,
                    operation: "get available lists page".to_string(),
                    source: e,
                })
            })?
    }

    /// Fetches all available mailing lists, aggregating all paginated results.
    ///
    /// This method retrieves all available mailing lists archived on the Lore Kernel Archive,
    /// following pagination until all items are collected.
    ///
    /// # Returns
    /// An `ArcSlice<LoreMailingList>` containing all available mailing lists.
    pub async fn get_available_lists(&self) -> Result<ArcSlice<LoreMailingList>, LoreApiError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(LoreApiMessage::GetAvailableLists { tx })
            .await
            .map_err(|_e| {
                LoreApiError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::api::lore::ACTOR_NAME,
                    operation: "get available lists".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                LoreApiError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::api::lore::ACTOR_NAME,
                    operation: "get available lists".to_string(),
                    source: e,
                })
            })?
    }

    /// Fetches the HTML content of a specific patch.
    ///
    /// This method retrieves the full HTML content of a specific patch
    /// identified by its message ID within a mailing list.
    ///
    /// # Arguments
    /// * `target_list` - The mailing list name (e.g., "amd-gfx", "linux-kernel")
    /// * `message_id` - The unique message ID of the patch
    ///
    /// # Returns
    /// The HTML content of the patch, or an error if the request fails.
    ///
    /// # Example
    /// ```ignore
    /// let patch_html = lore_api.get_patch_html("amd-gfx", "20231201.123456.1-1@amd.com").await?;
    /// ```
    pub async fn get_patch_html(
        &self,
        target_list: ArcStr,
        message_id: ArcStr,
    ) -> Result<ArcStr, LoreApiError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(LoreApiMessage::GetPatchHtml {
                target_list,
                message_id,
                tx,
            })
            .await
            .map_err(|_e| {
                LoreApiError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::api::lore::ACTOR_NAME,
                    operation: "get patch HTML".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                LoreApiError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::api::lore::ACTOR_NAME,
                    operation: "get patch HTML".to_string(),
                    source: e,
                })
            })?
    }

    /// Fetches a raw patch in plain text format.
    ///
    /// This method retrieves the raw patch content in plain text format,
    /// which is useful for applying patches or extracting metadata.
    ///
    /// # Arguments
    /// * `target_list` - The mailing list name
    /// * `message_id` - The unique message ID of the patch
    ///
    /// # Returns
    /// The raw patch content as plain text, or an error if the request fails.
    ///
    /// # Example
    /// ```ignore
    /// let raw_patch = lore_api.get_raw_patch("amd-gfx", "20231201.123456.1-1@amd.com").await?;
    /// ```
    pub async fn get_raw_patch(
        &self,
        target_list: ArcStr,
        message_id: ArcStr,
    ) -> Result<ArcStr, LoreApiError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(LoreApiMessage::GetRawPatch {
                target_list,
                message_id,
                tx,
            })
            .await
            .map_err(|_e| {
                LoreApiError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::api::lore::ACTOR_NAME,
                    operation: "get raw patch".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                LoreApiError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::api::lore::ACTOR_NAME,
                    operation: "get raw patch".to_string(),
                    source: e,
                })
            })?
    }

    /// Fetches patch metadata in JSON format.
    ///
    /// This method retrieves structured metadata about a patch in JSON format,
    /// which is useful for programmatic access to patch information.
    ///
    /// # Arguments
    /// * `target_list` - The mailing list name
    /// * `message_id` - The unique message ID of the patch
    ///
    /// # Returns
    /// The patch metadata as JSON, or an error if the request fails.
    ///
    /// # Example
    /// ```ignore
    /// let metadata = lore_api.get_patch_metadata("amd-gfx", "20231201.123456.1-1@amd.com").await?;
    /// ```
    pub async fn get_patch_metadata(
        &self,
        target_list: ArcStr,
        message_id: ArcStr,
    ) -> Result<ArcStr, LoreApiError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(LoreApiMessage::GetPatchMetadata {
                target_list,
                message_id,
                tx,
            })
            .await
            .map_err(|_e| {
                LoreApiError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::api::lore::ACTOR_NAME,
                    operation: "get patch metadata".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                LoreApiError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::api::lore::ACTOR_NAME,
                    operation: "get patch metadata".to_string(),
                    source: e,
                })
            })?
    }
}
