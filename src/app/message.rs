use anyhow::Result;
use tokio::sync::oneshot;

use super::data::Command;

/// Messages for communicating with the App actor
#[derive(Debug)]
pub enum Message {
    /// Execute a CLI command
    ExecuteCommand {
        command: Command,
        tx: oneshot::Sender<Result<()>>,
    },
    /// Run the TUI (Terminal User Interface) mode
    RunTui { tx: oneshot::Sender<Result<()>> },
    /// Shutdown the application gracefully
    Shutdown { tx: oneshot::Sender<Result<()>> },
}
