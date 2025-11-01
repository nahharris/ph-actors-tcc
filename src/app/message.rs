use anyhow::Result;
use tokio::sync::oneshot;

/// Messages for communicating with the App actor
#[derive(Debug)]
pub enum Message {
    /// Shutdown the application gracefully
    Shutdown { tx: oneshot::Sender<Result<()>> },
}
