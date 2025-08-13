use crate::{ArcStr, SequenceNumber};
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
pub struct LoreFeedItem {
    /// The author name for the patch
    pub author: ArcStr,
    /// The email of the author of the patch
    pub email: ArcStr,
    /// The datetime of the last update of the patch
    pub last_update: DateTime<Utc>,
    /// The title of the patch
    pub title: ArcStr,
    /// The version of the patch
    pub version: usize,
    /// The sequence number of the patch
    pub sequence: Option<SequenceNumber>,
    /// The link to the patch
    pub link: ArcStr,
    /// The mailing list which the patch belongs to
    pub list: ArcStr,
    /// The message ID of the patch
    pub message_id: ArcStr,
}

/// Represents a patch that is parsed from a mbox file
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LorePatch {
    /// The title of the patch
    pub title: ArcStr,
    /// The version of the patch
    pub version: usize,
    /// The sequence number of the patch is a patch series
    pub sequence: SequenceNumber,
    /// The sender of the patch
    pub from: ArcStr,
    /// The recipients of the patch
    pub to: Vec<ArcStr>,
    /// The carbon copy recipients of the patch
    pub cc: Vec<ArcStr>,
    /// The blind carbon copy recipients of the patch
    pub bcc: Vec<ArcStr>,
    /// The date of the patch
    pub date: DateTime<Utc>,
    /// The message of the patch that is a follow up to the title and explains the contribution
    pub message: ArcStr,
    /// The actual content of the patch: the diff that represents the contribution
    pub diff: ArcStr,
}

impl std::fmt::Display for LorePatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[PATCH")?;
        if self.version > 1 {
            write!(f, " v{}", self.version)?;
        }
        write!(f, " {}", self.sequence)?;
        write!(f, "] {}", self.title)?;
        writeln!(f)?;
        writeln!(f, "From: {}", self.from)?;
        writeln!(
            f,
            "To: {}",
            self.to
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        if !self.cc.is_empty() {
            writeln!(
                f,
                "Cc: {}",
                self.cc
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        if !self.bcc.is_empty() {
            writeln!(
                f,
                "Bcc: {}",
                self.bcc
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        writeln!(f, "Date: {}", self.date.format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(f)?;
        write!(f, "{}", self.message)?;
        writeln!(f, "\n\n\n")?;
        write!(f, "{}", self.diff)?;
        Ok(())
    }
}
