use anyhow::Result;
use tokio::sync::oneshot;

use super::data::Command;
use crate::terminal::UiEvent;

/// Messages for communicating with the App actor
#[derive(Debug)]
pub enum Message {
    /// Execute a CLI command (for resolve mode)
    ExecuteCommand {
        command: Command,
        tx: oneshot::Sender<Result<()>>,
    },
    /// Handle a key event from the terminal (for spawn mode)
    KeyEvent { event: UiEvent },
    /// Shutdown the application gracefully
    Shutdown { tx: oneshot::Sender<Result<()>> },
}
