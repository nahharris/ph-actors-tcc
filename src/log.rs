mod core;
mod data;
mod message;
mod mock;
#[cfg(test)]
mod tests;

pub use core::Core;
pub use data::LogLevel;
use data::LogMessage;

#[cfg(not(test))]
use crate::fs::Fs;
#[cfg(test)]
use crate::fs::mock::MockFs as Fs;

use std::fmt::Display;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

/// The logging actor that provides a thread-safe interface for logging operations.
///
/// This struct provides a unified interface for logging operations
/// using message passing to a background actor.
///
/// # Examples
/// ```ignore
/// let log = Log::spawn(fs, LogLevel::Info, 7, log_dir).await?;
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
    /// * `level` - The minimum log level to print to stderr
    /// * `max_age` - Maximum age of log files in days before deletion
    /// * `log_dir` - Directory where log files are stored
    ///
    /// # Returns
    /// A new logging instance with a spawned actor.
    pub async fn spawn(
        fs: Fs,
        level: LogLevel,
        max_age: usize,
        log_dir: crate::ArcPath,
    ) -> anyhow::Result<Self> {
        let (tx, rx) = tokio::sync::mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::build(fs, level, max_age, log_dir).await?;
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Ok(Self { tx })
    }

    fn log(&self, scope: &'static str, message: String, level: LogLevel) {
        let sender = self.tx.clone();
        tokio::spawn(async move {
            sender
                .send(message::Message::Log(LogMessage {
                    level,
                    scope,
                    message: message.to_string(),
                }))
                .await
                .expect("Attempt to use logger after a flush");
        });
    }

    /// Log a message with the `INFO` level
    pub fn info<M: Display>(&self, scope: &'static str, message: M) {
        self.log(scope, message.to_string(), LogLevel::Info);
    }

    /// Log a message with the `WARNING` level
    pub fn warn<M: Display>(&self, scope: &'static str, message: M) {
        self.log(scope, message.to_string(), LogLevel::Warning);
    }

    /// Log a message with the `ERROR` level
    pub fn error<M: Display>(&self, scope: &'static str, message: M) {
        self.log(scope, message.to_string(), LogLevel::Error);
    }

    /// Flushes the logger by printing its messages to [`stderr`] and closing
    /// the log file. After this method is called, the logger is destroyed and
    /// any attempt to use it will panic.
    pub fn flush(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.tx
                .send(message::Message::Flush)
                .await
                .expect("Flushing a logger twice");
        })
    }

    /// Collects the garbage from the logs directory. Garbage logs are the ones
    /// older than the [`max_age`] set during the logger [`build`].
    pub async fn collect_garbage(&self) {
        self.tx
            .send(message::Message::CollectGarbage)
            .await
            .expect("Attempt to use logger after a flush")
    }

    /// Gets all logged messages from the mock implementation.
    /// This method is only available for mock instances and is useful for testing.
    ///
    /// # Returns
    /// A vector of all logged messages, or None if this is not a mock instance.
    ///
    /// # Note
    /// This method is deprecated in the struct-based approach. Use MockLog directly for testing.
    #[deprecated(note = "Use MockLog directly for testing instead")]
    pub async fn get_messages(&self) -> Option<Vec<LogMessage>> {
        None
    }
}
