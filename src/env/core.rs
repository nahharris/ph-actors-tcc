use std::ffi::OsString;

use tokio::sync::mpsc;

use crate::{ArcOsStr, ArcStr, error::EnvError};

use super::message::Message;

/// The core of the Env actor, responsible for handling environment variable operations.
///
/// This struct provides thread-safe access to environment variables through an actor pattern.
/// It wraps the standard library's environment variable functions and provides a safe interface
/// for concurrent access.
#[derive(Debug, Default)]
pub struct Core {}

impl Core {
    /// Creates a new Env core instance.
    ///
    /// # Returns
    /// A new instance of `Core` with default values.
    pub fn new() -> Self {
        Default::default()
    }

    /// Initializes the environment actor message receiver.
    ///
    /// This method processes messages from the receiver in a loop, handling each message
    /// using pattern matching.
    ///
    /// # Arguments
    /// * `sender` - A receiver for messages to process
    pub async fn init(self, mut sender: mpsc::Receiver<Message>) {
        while let Some(msg) = sender.recv().await {
            use Message::*;
            match msg {
                Set { key, value } => self.set_env(key, value),
                Unset { key } => self.unset_env(key),
                Get { tx, key } => self.get_env(tx, key),
            }
        }
    }

    fn set_env(&self, key: ArcOsStr, value: OsString) {
        unsafe {
            std::env::set_var(key, value);
        }
    }

    fn unset_env(&self, key: ArcOsStr) {
        unsafe {
            std::env::remove_var(key);
        }
    }

    fn get_env(&self, tx: tokio::sync::oneshot::Sender<Result<ArcStr, EnvError>>, key: ArcOsStr) {
        let _ = tx.send(
            std::env::var(key.as_ref())
                .map(|s| ArcStr::from(&s))
                .map_err(|_e| EnvError::NotFound {
                    name: key.to_string_lossy().to_string(),
                }),
        );
    }
}
