//! API module providing high-level interfaces for external services.
//!
//! This module contains actors that intermediate calls to the networking actor,
//! providing domain-specific APIs for different services. Each submodule represents
//! a different API or service group.

pub mod lore;

/// Re-exports for convenience
pub use lore::LoreApi;
