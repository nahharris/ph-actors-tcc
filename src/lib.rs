//! Library entry point for the ph crate.
//! This file re-exports key types for use in examples and other crates.

pub mod api;
pub mod app;
pub mod env;
pub mod fs;
pub mod log;
pub mod net;
pub mod render;
pub mod shell;
pub mod terminal;
pub mod utils;

#[macro_use]
pub mod macros;

pub use utils::*;

/// Default buffer size used for various operations in the application.
/// This constant defines the size of buffers used for reading and writing operations.
pub const BUFFER_SIZE: usize = 128;
