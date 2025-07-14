use crate::ArcStr;
use chrono::{DateTime, Utc};

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

/// Represents a paginated response of available mailing lists from the Lore Kernel Archive.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LoreAvailableLists {
    /// The current start index for pagination
    pub start_index: usize,
    /// The next page start index, or None if there is no next page
    pub next_page_index: Option<usize>,
    /// The total number of available items (if known)
    pub total_items: Option<usize>,
    /// The list of available mailing lists
    pub items: Vec<LoreMailingList>,
}
