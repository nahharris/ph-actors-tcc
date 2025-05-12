use std::{fmt::Display, str::FromStr};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, sync::mpsc::Sender, task::JoinHandle};

use crate::{ArcFile, ArcPath, fs::Fs};

/// Describes the log level of a message.
///
/// This enum is used to determine the severity of a log message so the logger
/// can handle it according to the configured verbosity level.
///
/// # Ordering
/// The levels are ordered by severity: `Info` < `Warning` < `Error`
///
/// # Examples
/// ```
/// let level = LogLevel::Info;
/// assert!(level < LogLevel::Warning);
/// assert!(level < LogLevel::Error);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum LogLevel {
    #[default]
    /// The lowest level, dedicated to regular information that is not critical.
    /// Used for general operational messages and debugging information.
    Info,
    /// Mid level, used to indicate when something went wrong but it's not
    /// critical. Used for recoverable errors or potential issues.
    Warning,
    /// The highest level, used to indicate critical errors that require attention
    /// but are not severe enough to crash the program.
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl FromStr for LogLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warning),
            "error" => Ok(LogLevel::Error),
            _ => Err(anyhow::anyhow!("Invalid log level: {}", s)),
        }
    }
}

/// Describes a message to be logged.
///
/// Contains both the message content and its associated log level.
/// This struct is used internally by the logger to manage log entries.
///
/// # Examples
/// ```
/// let msg = LogMessage {
///     level: LogLevel::Info,
///     message: "Application started".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogMessage {
    /// The severity level of the message
    level: LogLevel,
    /// The actual message content
    message: String,
}

impl Display for LogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.level, self.message)
    }
}

/// The core of the logging system that manages logging to both stderr and log files.
///
/// This struct provides thread-safe logging capabilities through an actor pattern.
/// It handles writing messages to both a timestamped log file and a "latest" log file,
/// while also maintaining a buffer of messages to be printed to stderr when requested.
///
/// # Features
/// - Concurrent logging through an actor pattern
/// - Dual logging to files (timestamped and latest)
/// - Configurable log levels
/// - Automatic log file rotation and cleanup
/// - Buffered stderr output
///
/// # Examples
/// ```
/// let (log, _) = LogCore::build(fs, LogLevel::Info, 7, log_dir).await?.spawn();
/// log.info("Application started");
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads through the actor pattern.
/// All logging operations are handled sequentially to ensure consistency.
#[derive(Debug)]
pub struct LogCore {
    /// Filesystem interface for file operations
    fs: Fs,
    /// Directory where log files are stored
    log_dir: ArcPath,
    /// Path to the current timestamped log file
    log_path: ArcPath,
    /// Handle to the current log file
    log_file: ArcFile,
    /// Handle to the "latest" log file
    latest_log_file: ArcFile,
    /// Buffer of messages to be printed to stderr
    logs_to_print: Vec<LogMessage>,
    /// Minimum level of messages to be printed to stderr
    print_level: LogLevel,
    /// Maximum age of log files in days before they are deleted
    max_age: usize,
}

impl LogCore {
    /// Creates a new logger instance with the specified configuration.
    ///
    /// # Arguments
    /// * `fs` - Filesystem interface for file operations
    /// * `level` - Minimum log level for messages to be printed to stderr
    /// * `max_age` - Maximum age of log files in days before they are deleted
    /// * `log_dir` - Directory where log files will be stored
    ///
    /// # Returns
    /// A new instance of `LogCore` ready to be spawned as an actor.
    ///
    /// # Errors
    /// Returns an error if either the timestamped log file or the latest log file
    /// cannot be created.
    ///
    /// # Examples
    /// ```
    /// let log = LogCore::build(fs, LogLevel::Info, 7, log_dir).await?;
    /// ```
    pub async fn build(
        fs: Fs,
        level: LogLevel,
        max_age: usize,
        log_dir: ArcPath,
    ) -> anyhow::Result<Self> {
        let log_path = ArcPath::from(&log_dir.join(format!(
            "patch-hub_{}.log",
            chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S")
        )));
        let latest_log_path = ArcPath::from(&log_dir.join("latest.log"));

        let log_file = fs
            .open_file(log_path.clone())
            .await
            .context("Failed to create log file")?;
        let latest_log_file = fs
            .open_file(latest_log_path)
            .await
            .context("Failed to create latest log file")?;

        Ok(Self {
            fs,
            log_dir,
            log_path,
            log_file,
            latest_log_file,
            logs_to_print: Vec::new(),
            print_level: level,
            max_age,
        })
    }

    /// Transforms the logger core instance into an actor.
    ///
    /// This method spawns a new task that will handle logging operations
    /// asynchronously through a message channel. Commands are processed
    /// sequentially to ensure consistency.
    ///
    /// # Returns
    /// A tuple containing:
    /// - A [`Log`] instance that can be used to send messages to the actor
    /// - A join handle for the spawned task
    ///
    /// # Examples
    /// ```
    /// let (log, handle) = log_core.spawn();
    /// ```
    pub fn spawn(mut self) -> (Log, JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let handle = tokio::spawn(async move {
            while let Some(command) = rx.recv().await {
                match command {
                    Message::Log(msg) => {
                        self.log(msg).await;
                    }
                    Message::Flush => {
                        self.flush();
                        rx.close();
                        break;
                    }
                    Message::CollectGarbage => {
                        self.collect_garbage().await;
                    }
                }
            }
        });

        (Log::Actual(tx), handle)
    }

    /// Writes a log message to both log files and potentially to the stderr buffer.
    ///
    /// The message is written to both the timestamped log file and the latest log file.
    /// If the message's level is equal to or higher than the configured print level,
    /// it is also added to the stderr buffer.
    ///
    /// # Arguments
    /// * `message` - The log message to write
    ///
    /// # Panics
    /// This function will panic if it fails to write to either log file.
    async fn log(&mut self, message: LogMessage) {
        let mut lock = self.log_file.write().await;
        lock.write_all(format!("{}\n", &message).as_bytes())
            .await
            .expect("Failed to write to the current log file");

        lock.flush()
            .await
            .expect("Failed to flush the current log file");

        let mut lock = self.latest_log_file.write().await;
        lock.write_all(format!("{}\n", &message).as_bytes())
            .await
            .expect("Failed to write to the latest log file");

        lock.flush()
            .await
            .expect("Failed to flush the latest log file");

        if message.level >= self.print_level {
            self.logs_to_print.push(message);
        }
    }

    /// Writes buffered log messages to stderr and destroys the logger.
    ///
    /// This method prints all buffered messages to stderr and then destroys
    /// the logger instance. It should be called when the application is shutting down.
    ///
    /// # Note
    /// The logger is destroyed after this method is called. Any subsequent
    /// logging attempts will fail.
    fn flush(self) {
        for message in &self.logs_to_print {
            eprintln!("{}", message);
        }

        if !self.logs_to_print.is_empty() {
            eprintln!("Check the full log file: {}", self.log_path.display());
        }
    }

    /// Runs the garbage collector to delete old log files.
    ///
    /// This method scans the log directory and deletes any log files that are
    /// older than the configured maximum age. If max_age is 0, no files are deleted.
    ///
    /// # Errors
    /// If the log directory cannot be read, an error message is logged but
    /// the function continues execution.
    async fn collect_garbage(&mut self) {
        if self.max_age == 0 {
            return;
        }

        let now = std::time::SystemTime::now();

        let Ok(logs) = self.fs.read_dir(self.log_dir.clone()).await else {
            self.log(LogMessage {
                level: LogLevel::Error,
                message: "Failed to read the logs directory during garbage collection".into(),
            })
            .await;
            return;
        };

        for log in logs {
            let Some(filename) = log.file_name() else {
                continue;
            };

            if !filename.to_string_lossy().ends_with(".log")
                || !filename.to_string_lossy().starts_with("patch-hub_")
            {
                continue;
            }

            let Ok(Ok(created_date)) = log.metadata().map(|meta| meta.created()) else {
                continue;
            };
            let Ok(age) = now.duration_since(created_date) else {
                continue;
            };
            let age = age.as_secs() / 60 / 60 / 24;

            if age as usize > self.max_age && self.fs.remove_file(log.clone()).await.is_err() {
                self.log(LogMessage {
                    message: format!("Failed to remove the log file: {}", log.to_string_lossy()),
                    level: LogLevel::Warning,
                })
                .await;
            }
        }
    }
}

/// Messages that can be sent to a [`LogCore`] actor.
///
/// This enum defines the different types of operations that can be performed
/// through the logging actor system.
#[derive(Debug)]
pub enum Message {
    /// Logs a message with the specified level and content
    Log(LogMessage),
    /// Flushes the logger by writing buffered messages to stderr and destroying the instance
    Flush,
    /// Runs the log garbage collector to delete old log files
    CollectGarbage,
}

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
    Actual(Sender<Message>),
    /// A mock implementation for testing that does nothing
    #[allow(dead_code)]
    Mock,
}

impl From<LogCore> for Log {
    fn from(value: LogCore) -> Self {
        value.spawn().0
    }
}

#[allow(dead_code)]
impl Log {
    /// Helper to simplify the logging process. This method sends a
    /// [`LogMessage`] to the logger. Will send the message in a new task so it
    /// won't block the caller
    ///
    /// # Panics
    /// If the logger was flushed
    fn log(&self, message: String, level: LogLevel) {
        let sender = match self {
            Log::Mock => return,
            Log::Actual(sender) => sender.clone(),
        };

        tokio::spawn(async move {
            sender
                .send(Message::Log(LogMessage {
                    level,
                    message: message.to_string(),
                }))
                .await
                .expect("Attemp to use logger after a flush");
        });
    }

    /// Log a message with the `INFO` level
    ///
    /// # Panics
    /// If the logger was flushed
    pub fn info<M: Display>(&self, message: M) {
        self.log(message.to_string(), LogLevel::Info);
    }

    /// Log a message with the `WARNING` level
    ///
    /// # Panics
    /// If the logger was flushed
    pub fn warn<M: Display>(&self, message: M) {
        self.log(message.to_string(), LogLevel::Warning);
    }

    /// Log a message with the `ERROR` level
    pub fn error<M: Display>(&self, message: M) {
        self.log(message.to_string(), LogLevel::Error);
    }

    /// Log an info message if the result is an error
    /// and return the result as is
    ///
    /// # Panics
    /// If the logger was flushed
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

    /// Log an warning message if the result is an error
    /// and return the result as is
    ///
    /// # Panics
    /// If the logger was flushed
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
    ///
    /// # Panics
    /// If the logger was flushed
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
    ///
    /// # Panics
    /// If called twice
    ///
    /// [`stderr`]: std::io::stderr
    pub fn flush(self) -> JoinHandle<()> {
        let Self::Actual(sender) = self else {
            return tokio::spawn(async {});
        };

        tokio::spawn(async move {
            sender
                .send(Message::Flush)
                .await
                .expect("Flushing a logger twice");
        })
    }

    /// Collects the garbage from the logs directory. Garbage logs are the ones
    /// older than the [`max_age`] set during the logger [`build`].
    ///
    /// # Panics
    /// If called after a flush
    ///
    /// [`build`]: Log::build
    /// [`max_age`]: Log::max_age
    pub async fn collect_garbage(&self) {
        let Self::Actual(sender) = self else {
            return;
        };

        sender
            .send(Message::CollectGarbage)
            .await
            .expect("Attemp to use logger after a flush")
    }
}
