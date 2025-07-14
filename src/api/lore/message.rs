use tokio::sync::oneshot::Sender;

use super::data::{LoreAvailableLists, LoreMailingList};
use crate::{ArcStr, ArcSlice};

/// Messages that can be sent to a [`LoreApiCore`] actor.
///
/// This enum defines the different types of Lore API operations that can be performed
/// through the Lore API actor system.
#[derive(Debug)]
pub enum LoreApiMessage {
    /// Fetches a patch feed from a specific mailing list with pagination
    GetPatchFeed {
        /// The mailing list name (e.g., "amd-gfx", "linux-kernel")
        target_list: ArcStr,
        /// The offset for pagination (0-based)
        min_index: usize,
        /// Response channel for the operation result
        tx: Sender<anyhow::Result<ArcStr>>,
    },
    GetAvailableLists {
        tx: Sender<anyhow::Result<ArcSlice<LoreMailingList>>>,
    },
    /// Fetches available mailing lists with pagination
    GetAvailableListsPage {
        /// The offset for pagination (0-based)
        min_index: usize,
        /// Response channel for the operation result
        tx: Sender<anyhow::Result<LoreAvailableLists>>,
    },
    /// Fetches the HTML content of a specific patch
    GetPatchHtml {
        /// The mailing list name (e.g., "amd-gfx", "linux-kernel")
        target_list: ArcStr,
        /// The unique message ID of the patch
        message_id: ArcStr,
        /// Response channel for the operation result
        tx: Sender<anyhow::Result<ArcStr>>,
    },
    /// Fetches a raw patch in plain text format
    GetRawPatch {
        /// The mailing list name
        target_list: ArcStr,
        /// The unique message ID of the patch
        message_id: ArcStr,
        /// Response channel for the operation result
        tx: Sender<anyhow::Result<ArcStr>>,
    },
    /// Fetches patch metadata in JSON format
    GetPatchMetadata {
        /// The mailing list name
        target_list: ArcStr,    
        /// The unique message ID of the patch
        message_id: ArcStr,
        /// Response channel for the operation result
        tx: Sender<anyhow::Result<ArcStr>>,
    },
}

/// Response types for Lore API operations.
///
/// This enum defines the different types of responses that can be returned
/// from Lore API operations. Currently, all operations return `ArcStr`,
/// but this structure allows for future expansion.
#[derive(Debug, Clone)]
pub enum LoreApiResponse {
    /// A successful response containing the requested data
    Success(ArcStr),
    /// An error response with details about what went wrong
    Error(String),
}

impl LoreApiResponse {
    /// Creates a success response with the given data.
    ///
    /// # Arguments
    /// * `data` - The response data
    ///
    /// # Returns
    /// A success response containing the data.
    pub fn success(data: ArcStr) -> Self {
        Self::Success(data)
    }

    /// Creates an error response with the given error message.
    ///
    /// # Arguments
    /// * `error` - The error message
    ///
    /// # Returns
    /// An error response containing the error message.
    pub fn error(error: String) -> Self {
        Self::Error(error)
    }

    /// Converts the response to a Result.
    ///
    /// # Returns
    /// Ok(ArcStr) for success responses, Err(anyhow::Error) for error responses.
    pub fn into_result(self) -> anyhow::Result<ArcStr> {
        match self {
            LoreApiResponse::Success(data) => Ok(data),
            LoreApiResponse::Error(error) => Err(anyhow::anyhow!(error)),
        }
    }
}
