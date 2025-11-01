use anyhow::Result;
use tokio::sync::oneshot;

use crate::terminal::UiEvent;

/// Messages for communicating with the App actor
#[derive(Debug)]
pub enum Message {
    /// Handle a key event from the terminal
    KeyEvent { event: UiEvent },
    /// Shutdown the application gracefully
    Shutdown { tx: oneshot::Sender<Result<()>> },
}
