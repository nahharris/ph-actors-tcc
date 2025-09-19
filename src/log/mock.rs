use std::{collections::VecDeque, fmt::Display, sync::Arc};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::log::{LogLevel, LogMessage};

/// Mock implementation of the Log actor for testing purposes.
///
/// This struct stores log messages in memory, allowing tests to run
/// without writing to actual files or stderr.
#[derive(Debug, Clone)]
pub struct Mock {
    messages: Arc<Mutex<VecDeque<LogMessage>>>,
}

impl Mock {
    /// Creates a new mock instance with an empty message store.
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Logs a message with the specified level and scope.
    ///
    /// # Arguments
    /// * `scope` - The scope/module name for the log message
    /// * `message` - The message content
    /// * `level` - The log level
    pub fn log(&self, scope: &'static str, message: String, level: LogLevel) {
        let messages = self.messages.clone();
        tokio::spawn(async move {
            let mut lock = messages.lock().await;
            lock.push_back(LogMessage {
                level,
                scope,
                message: message.to_string(),
            });
        });
    }

    /// Log a message with the `INFO` level.
    ///
    /// # Arguments
    /// * `scope` - The scope/module name for the log message
    /// * `message` - The message content
    pub fn info<M: Display>(&self, scope: &'static str, message: M) {
        self.log(scope, message.to_string(), LogLevel::Info);
    }

    /// Log a message with the `WARNING` level.
    ///
    /// # Arguments
    /// * `scope` - The scope/module name for the log message
    /// * `message` - The message content
    pub fn warn<M: Display>(&self, scope: &'static str, message: M) {
        self.log(scope, message.to_string(), LogLevel::Warning);
    }

    /// Log a message with the `ERROR` level.
    ///
    /// # Arguments
    /// * `scope` - The scope/module name for the log message
    /// * `message` - The message content
    pub fn error<M: Display>(&self, scope: &'static str, message: M) {
        self.log(scope, message.to_string(), LogLevel::Error);
    }

    /// Log an info message if the result is an error
    /// and return the result as is.
    ///
    /// # Arguments
    /// * `scope` - The scope/module name for the log message
    /// * `result` - The result to check for errors
    ///
    /// # Returns
    /// The original result, logging the error if present.
    pub fn info_on_error<T, E: Display>(
        &self,
        scope: &'static str,
        result: Result<T, E>,
    ) -> Result<T, E> {
        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                self.log(scope, err.to_string(), LogLevel::Info);
                Err(err)
            }
        }
    }

    /// Log a warning message if the result is an error
    /// and return the result as is.
    ///
    /// # Arguments
    /// * `scope` - The scope/module name for the log message
    /// * `result` - The result to check for errors
    ///
    /// # Returns
    /// The original result, logging the error if present.
    pub fn warn_on_error<T, E: Display>(
        &self,
        scope: &'static str,
        result: Result<T, E>,
    ) -> Result<T, E> {
        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                self.log(scope, err.to_string(), LogLevel::Warning);
                Err(err)
            }
        }
    }

    /// Log an error message if the result is an error
    /// and return the result as is.
    ///
    /// # Arguments
    /// * `scope` - The scope/module name for the log message
    /// * `result` - The result to check for errors
    ///
    /// # Returns
    /// The original result, logging the error if present.
    pub fn error_on_error<T, E: Display>(
        &self,
        scope: &'static str,
        result: Result<T, E>,
    ) -> Result<T, E> {
        match result {
            Ok(value) => Ok(value),
            Err(err) => {
                self.log(scope, err.to_string(), LogLevel::Error);
                Err(err)
            }
        }
    }

    /// Flushes the mock logger by printing its messages to stderr.
    ///
    /// # Returns
    /// A JoinHandle for the flush operation.
    pub fn flush(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            let lock = self.messages.lock().await;
            for message in lock.iter() {
                eprintln!("{message}");
            }
        })
    }

    /// Collects garbage from the logs directory.
    /// Mock implementation does nothing for garbage collection.
    pub async fn collect_garbage(&self) {
        // Mock implementation does nothing for garbage collection
    }

    /// Gets all logged messages from the mock implementation.
    ///
    /// # Returns
    /// A vector of all logged messages.
    pub async fn get_messages(&self) -> Vec<LogMessage> {
        let lock = self.messages.lock().await;
        lock.iter().cloned().collect()
    }
}
