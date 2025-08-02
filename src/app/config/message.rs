use tokio::sync::oneshot;

use crate::{ArcPath, log::LogLevel};

use super::data::{PathOpt, Renderer, RendererOpt, USizeOpt};

/// Messages that can be sent to the configuration actor.
///
/// This enum represents all possible operations that can be performed on the configuration.
/// Each variant contains the necessary data to perform its operation and, for operations
/// that return a value, a channel to send the result back to the caller.
#[derive(Debug)]
pub enum Message {
    /// Load configuration from file
    Load {
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Save configuration to file
    Save {
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<anyhow::Result<()>>,
    },
    /// Get a path-based configuration value
    GetPath {
        /// The path option to retrieve
        opt: PathOpt,
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<ArcPath>,
    },
    /// Get the current log level
    GetLogLevel {
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<LogLevel>,
    },
    /// Get a numeric configuration value
    GetUSize {
        /// The numeric option to retrieve
        opt: USizeOpt,
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<usize>,
    },
    /// Set a path-based configuration value
    SetPath {
        /// The path option to set
        opt: PathOpt,
        /// The new path value
        path: ArcPath,
    },
    /// Set the log level
    SetLogLevel {
        /// The new log level value
        level: LogLevel,
    },
    /// Set a numeric configuration value
    SetUSize {
        /// The numeric option to set
        opt: USizeOpt,
        /// The new numeric value
        size: usize,
    },
    /// Get a renderer configuration value
    GetRenderer {
        /// The renderer option to retrieve
        opt: RendererOpt,
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<Renderer>,
    },
    /// Set a renderer configuration value
    SetRenderer {
        /// The renderer option to set
        opt: RendererOpt,
        /// The new renderer value
        renderer: Renderer,
    },
}
