use tokio::sync::oneshot::Sender;

use super::data::{LoreMailingList, LorePage, LorePatchMetadata};
use crate::{ArcSlice, ArcStr};

/// Messages that can be sent to a [`LoreApiCore`] actor.
///
/// This enum defines the different types of Lore API operations that can be performed
/// through the Lore API actor system.
#[derive(Debug)]
pub enum LoreApiMessage {
    /// Fetches a patch feed from a specific mailing list with pagination
    GetPatchFeedPage {
        /// The mailing list name (e.g., "amd-gfx", "linux-kernel")
        target_list: ArcStr,
        /// The offset for pagination (0-based)
        min_index: usize,
        /// Response channel for the operation result
        tx: Sender<anyhow::Result<Option<LorePage<LorePatchMetadata>>>>,
    },
    GetAvailableLists {
        tx: Sender<anyhow::Result<ArcSlice<LoreMailingList>>>,
    },
    /// Fetches available mailing lists with pagination
    GetAvailableListsPage {
        /// The offset for pagination (0-based)
        min_index: usize,
        /// Response channel for the operation result
        tx: Sender<anyhow::Result<Option<LorePage<LoreMailingList>>>>,
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
