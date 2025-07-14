mod core;
mod data;
mod message;
#[cfg(test)]
mod tests;

pub use core::LogCore;
pub use data::LogLevel;
use data::LogMessage;

use std::collections::VecDeque;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

/// The logging actor that provides a thread-safe interface for logging operations.
///
/// This enum represents either a real logging actor or a mock implementation
/// for testing purposes. It provides a unified interface for logging operations
/// regardless of the underlying implementation.
///
/// # Examples
/// ```ignore
/// let log = Log::spawn(fs, LogLevel::Info, 7, log_dir).await?;
/// log.info("Application started");
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender or mock reference.
#[derive(Debug, Clone)]
pub enum Log {
    /// A real logging actor that writes to files and stderr
    Actual(Sender<message::Message>),
    /// A mock implementation for testing that stores messages in memory
    Mock(Arc<Mutex<VecDeque<LogMessage>>>),
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
        fs: crate::fs::Fs,
        level: LogLevel,
        max_age: usize,
        log_dir: crate::ArcPath,
    ) -> anyhow::Result<Self> {
        let (log, _) = LogCore::build(fs, level, max_age, log_dir).await?.spawn();
        Ok(log)
    }

    /// Creates a new mock logging instance for testing.
    ///
    /// # Returns
    /// A new mock logging instance that stores messages in memory.
    pub fn mock() -> Self {
        Self::Mock(Arc::new(Mutex::new(VecDeque::new())))
    }

    fn log(&self, message: String, level: LogLevel) {
        match self {
            Log::Actual(sender) => {
                let sender = sender.clone();
                tokio::spawn(async move {
                    sender
                        .send(message::Message::Log(LogMessage {
                            level,
                            message: message.to_string(),
                        }))
                        .await
                        .expect("Attempt to use logger after a flush");
                });
            }
            Log::Mock(messages) => {
                let messages = messages.clone();
                tokio::spawn(async move {
                    let mut lock = messages.lock().await;
                    lock.push_back(LogMessage {
                        level,
                        message: message.to_string(),
                    });
                });
            }
        }
    }

    /// Log a message with the `INFO` level
    pub fn info<M: Display>(&self, message: M) {
        self.log(message.to_string(), LogLevel::Info);
    }

    /// Log a message with the `WARNING` level
    pub fn warn<M: Display>(&self, message: M) {
        self.log(message.to_string(), LogLevel::Warning);
    }

    /// Log a message with the `ERROR` level
    pub fn error<M: Display>(&self, message: M) {
        self.log(message.to_string(), LogLevel::Error);
    }

    /// Log an info message if the result is an error
    /// and return the result as is
    #[allow(dead_code)]
    pub fn info_on_error<T, E: Display>(&self, result: Result<T, E>) -> Result<T, E> {
        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                self.log(err.to_string(), LogLevel::Info);
                Err(err)
            }
        }
    }

    /// Log a warning message if the result is an error
    /// and return the result as is
    #[allow(dead_code)]
    pub fn warn_on_error<T, E: Display>(&self, result: Result<T, E>) -> Result<T, E> {
        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                self.log(err.to_string(), LogLevel::Warning);
                Err(err)
            }
        }
    }

    /// Log an error message if the result is an error
    /// and return the result as is
    pub fn error_on_error<T, E: Display>(&self, result: Result<T, E>) -> Result<T, E> {
        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                self.log(err.to_string(), LogLevel::Error);
                Err(err)
            }
        }
    }

    /// Flushes the logger by printing its messages to [`stderr`] and closing
    /// the log file. After this method is called, the logger is destroyed and
    /// any attempt to use it will panic.
    pub fn flush(self) -> JoinHandle<()> {
        match self {
            Self::Actual(sender) => tokio::spawn(async move {
                sender
                    .send(message::Message::Flush)
                    .await
                    .expect("Flushing a logger twice");
            }),
            Self::Mock(messages) => tokio::spawn(async move {
                let lock = messages.lock().await;
                for message in lock.iter() {
                    eprintln!("{}", message);
                }
            }),
        }
    }

    /// Collects the garbage from the logs directory. Garbage logs are the ones
    /// older than the [`max_age`] set during the logger [`build`].
    pub async fn collect_garbage(&self) {
        match self {
            Self::Actual(sender) => sender
                .send(message::Message::CollectGarbage)
                .await
                .expect("Attempt to use logger after a flush"),
            Self::Mock(_) => {
                // Mock implementation does nothing for garbage collection
            }
        }
    }

    /// Gets all logged messages from the mock implementation.
    /// This method is only available for mock instances and is useful for testing.
    ///
    /// # Returns
    /// A vector of all logged messages, or None if this is not a mock instance.
    pub async fn get_messages(&self) -> Option<Vec<LogMessage>> {
        match self {
            Self::Mock(messages) => {
                let lock = messages.lock().await;
                Some(lock.iter().cloned().collect())
            }
            Self::Actual(_) => None,
        }
    }
}
