mod core;
mod data;
mod message;
#[cfg(test)]
mod tests;

pub use core::LogCore;
pub use data::LogLevel;
use data::LogMessage;

use std::fmt::Display;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

/// The logging actor that provides a thread-safe interface for logging operations.
///
/// This enum represents either a real logging actor or a mock implementation
/// for testing purposes. It provides a unified interface for logging operations
/// regardless of the underlying implementation.
///
/// # Examples
/// ```
/// let (log, _) = LogCore::build(fs, LogLevel::Info, 7, log_dir).await?.spawn();
/// log.info("Application started");
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender.
#[derive(Debug, Clone)]
pub enum Log {
    /// A real logging actor that writes to files and stderr
    Actual(Sender<message::Message>),
    /// A mock implementation for testing that does nothing
    #[allow(dead_code)]
    Mock,
}

impl From<core::LogCore> for Log {
    fn from(value: core::LogCore) -> Self {
        value.spawn().0
    }
}

#[allow(dead_code)]
impl Log {
    fn log(&self, message: String, level: LogLevel) {
        let sender = match self {
            Log::Mock => return,
            Log::Actual(sender) => sender.clone(),
        };
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
        let Self::Actual(sender) = self else {
            return tokio::spawn(async {});
        };
        tokio::spawn(async move {
            sender
                .send(message::Message::Flush)
                .await
                .expect("Flushing a logger twice");
        })
    }

    /// Collects the garbage from the logs directory. Garbage logs are the ones
    /// older than the [`max_age`] set during the logger [`build`].
    pub async fn collect_garbage(&self) {
        let Self::Actual(sender) = self else {
            return;
        };
        sender
            .send(message::Message::CollectGarbage)
            .await
            .expect("Attempt to use logger after a flush")
    }
}
