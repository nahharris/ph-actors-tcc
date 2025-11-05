use tokio::sync::oneshot;

use crate::error::AppError;

/// Messages for communicating with the App actor
#[derive(Debug)]
pub enum Message {
    /// Shutdown the application gracefully
    Shutdown { tx: oneshot::Sender<Result<(), AppError>> },
}
