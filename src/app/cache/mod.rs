//! Cache module for the Patch Hub application.
//!
//! This module provides caching actors for different types of data:
//! - **Mailing List Actor**: Caches mailing lists with alphabetical sorting
//! - **Feed Actor**: Caches patch metadata per mailing list
//! - **Patch Actor**: Caches individual patch content

pub mod feed;
pub mod mailing_list;
pub mod patch;

// Re-export the main cache actors
pub use feed::FeedCache;
pub use mailing_list::MailingListCache;
pub use patch::PatchCache;

// Re-export mock data types for testing
pub use feed::MockData as FeedMockData;
pub use mailing_list::MockData as MailingListMockData;
pub use patch::MockData as PatchMockData;
