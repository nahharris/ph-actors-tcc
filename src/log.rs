mod core;
mod data;
mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

pub use core::Core;
pub use data::LogLevel;
use data::LogMessage;

/// Actor name for error reporting.
pub const ACTOR_NAME: &'static str = "Log";

#[cfg(not(test))]
use crate::{app::config::Config, fs::Fs};
#[cfg(test)]
use crate::{app::config::mock::MockConfig as Config, fs::mock::MockFs as Fs};

use tokio::sync::mpsc::{self, Sender};

use crate::{error::FatalActorError, error::LogError};

/// The logging actor that provides a thread-safe interface for logging operations.
///
/// This struct provides a unified interface for logging operations
/// using message passing to a background actor.
///
/// # Examples
/// ```ignore
/// let log = Log::spawn();
/// log.info("Application started");
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender.
#[derive(Debug, Clone)]
pub struct Log {
    tx: Sender<message::Message>,
}

impl Log {
    /// Creates a new logging instance and spawns its actor.
    ///
    /// # Arguments
    /// * `fs` - The filesystem actor for file operations
    /// * `config` - The configuration actor for settings
    ///
    /// # Returns
    /// A new logging instance with a spawned actor.
    pub async fn spawn(fs: Fs, config: Config) -> Result<Self, LogError> {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new(fs, config).await?;
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Ok(Self { tx })
    }

    fn log(&self, scope: &'static str, message: String, level: LogLevel) {
        let sender = self.tx.clone();
        tokio::spawn(async move {
            if sender
                .send(message::Message::Log(LogMessage {
                    level,
                    scope,
                    message: message.to_string(),
                }))
                .await
                .is_err()
            {
                // Logger has been flushed, silently ignore
                // This is expected behavior when the logger is being shut down
            }
        });
    }

    /// Log a message with the `INFO` level
    pub fn info(&self, scope: &'static str, message: String) {
        self.log(scope, message, LogLevel::Info);
    }

    /// Log a message with the `WARNING` level
    pub fn warn(&self, scope: &'static str, message: String) {
        self.log(scope, message, LogLevel::Warning);
    }

    /// Log a message with the `ERROR` level
    pub fn error(&self, scope: &'static str, message: String) {
        self.log(scope, message, LogLevel::Error);
    }

    /// Flushes the logger by printing its messages to [`stderr`] and closing
    /// the log file. After this method is called, the logger is destroyed and
    /// any attempt to use it will silently fail.
    pub async fn flush(self) -> Result<(), LogError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::Flush { tx })
            .await
            .map_err(|_e| {
                LogError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::log::ACTOR_NAME,
                    operation: "flush log".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            LogError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::log::ACTOR_NAME,
                operation: "flush log".to_string(),
                source: e,
            })
        })?
    }

    /// Collects the garbage from the logs directory. Garbage logs are the ones
    /// older than the [`max_age`] set during the logger [`build`].
    pub async fn collect_garbage(&self) -> Result<(), LogError> {
        self.tx
            .send(message::Message::CollectGarbage)
            .await
            .map_err(|_e| {
                LogError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::log::ACTOR_NAME,
                    operation: "collect garbage".to_string(),
                })
            })
    }
}
