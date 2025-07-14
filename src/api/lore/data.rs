use crate::ArcStr;
use chrono::{DateTime, Utc};

/// Represents a paginated response of available mailing lists from the Lore Kernel Archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LorePage<T> {
    /// The current start index for pagination
    pub start_index: usize,
    /// The next page start index, or None if there is no next page
    pub next_page_index: Option<usize>,
    /// The total number of available items (if known)
    pub total_items: Option<usize>,
    /// The list of available mailing lists
    pub items: Vec<T>,
}

/// Represents a single available mailing list item from the Lore Kernel Archive.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LoreMailingList {
    /// The name of the mailing list (e.g., "linux-arch")
    pub name: ArcStr,
    /// The description of the mailing list
    pub description: ArcStr,
    /// The last update date and time (UTC, e.g., 2025-07-14 13:47)
    pub last_update: DateTime<Utc>,
}

/// Represents a patch that is obtained from the feed for a given patch list
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LorePatchMetadata {
    /// The author name for the patch
    pub author: ArcStr,
    /// The email of the author of the patch
    pub email: ArcStr,
    /// The datetime of the last update of the patch
    pub datetime: DateTime<Utc>,
    /// The title of the patch
    pub title: ArcStr,
    /// The link to the patch
    pub link: ArcStr,
}
