use super::data::{LogLevel, LogMessage};
use super::message::Message;
use crate::{error::LogError, ArcPath};
#[cfg(not(test))]
use crate::{app::config::Config, fs::Fs};
#[cfg(test)]
use crate::{app::config::mock::MockConfig as Config, fs::mock::MockFs as Fs};

const SCOPE: &str = "log";

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
/// ```ignore
/// let core = Core::new(fs, config).await?;
/// let log = Log::spawn(core);
/// log.info("Application started");
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads through the actor pattern.
/// All logging operations are handled sequentially to ensure consistency.
#[derive(Debug)]
pub struct Core {
    /// Filesystem interface for file operations
    fs: Fs,
    /// Configuration interface for settings
    config: Config,
    /// Directory where log files are stored
    log_dir: ArcPath,
    /// Path to the current timestamped log file
    log_path: ArcPath,
    /// Handle to the current log file
    log_file: tokio::fs::File,
    /// Handle to the "latest" log file
    latest_log_file: tokio::fs::File,
    /// Buffer of messages to be printed to stderr
    logs_to_print: Vec<LogMessage>,
    /// Minimum level of messages to be printed to stderr
    print_level: LogLevel,
    /// Maximum age of log files in days before they are deleted
    max_age: usize,
}

impl Core {
    pub async fn new(fs: Fs, config: Config) -> Result<Self, LogError> {
        // Load configuration values
        let log_level = config.log_level().await.map_err(|e| match e {
            crate::error::ConfigError::Fatal(fatal) => LogError::Fatal(fatal),
            _ => LogError::FileOperationFailed {
                path: "configuration".to_string(),
                operation: "get log level".to_string(),
                source: std::io::Error::from(std::io::ErrorKind::Other),
            },
        })?;
        let max_age = config.usize(crate::app::config::USizeOpt::MaxAge).await.map_err(|e| match e {
            crate::error::ConfigError::Fatal(fatal) => LogError::Fatal(fatal),
            _ => LogError::FileOperationFailed {
                path: "configuration".to_string(),
                operation: "get max age".to_string(),
                source: std::io::Error::from(std::io::ErrorKind::Other),
            },
        })?;
        let log_dir = config.path(crate::app::config::PathOpt::LogDir).await.map_err(|e| match e {
            crate::error::ConfigError::Fatal(fatal) => LogError::Fatal(fatal),
            _ => LogError::FileOperationFailed {
                path: "configuration".to_string(),
                operation: "get log directory".to_string(),
                source: std::io::Error::from(std::io::ErrorKind::Other),
            },
        })?;

        let log_path = ArcPath::from(&log_dir.join(format!(
            "patch-hub_{}.log",
            chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S")
        )));
        let latest_log_path = ArcPath::from(&log_dir.join("latest.log"));

        // Create log directory and files
        fs.mkdir(log_dir.clone()).await.map_err(|e| match e {
            crate::error::FsError::Fatal(fatal) => LogError::Fatal(fatal),
            crate::error::FsError::OperationFailed { path, operation, source, .. } => {
                LogError::FileOperationFailed {
                    path: path.unwrap_or_else(|| log_dir.to_string_lossy().to_string()),
                    operation,
                    source,
                }
            }
        })?;

        let log_file = fs.write_file(log_path.clone()).await.map_err(|e| match e {
            crate::error::FsError::Fatal(fatal) => LogError::Fatal(fatal),
            crate::error::FsError::OperationFailed { path, operation, source, .. } => {
                LogError::FileOperationFailed {
                    path: path.unwrap_or_else(|| log_path.to_string_lossy().to_string()),
                    operation,
                    source,
                }
            }
        })?;

        let latest_log_file = fs.write_file(latest_log_path.clone()).await.map_err(|e| match e {
            crate::error::FsError::Fatal(fatal) => LogError::Fatal(fatal),
            crate::error::FsError::OperationFailed { path, operation, source, .. } => {
                LogError::FileOperationFailed {
                    path: path.unwrap_or_else(|| latest_log_path.to_string_lossy().to_string()),
                    operation,
                    source,
                }
            }
        })?;

        Ok(Self {
            fs,
            config,
            log_dir,
            log_path,
            log_file,
            latest_log_file,
            logs_to_print: Vec::new(),
            print_level: log_level,
            max_age,
        })
    }

    pub async fn init(mut self, mut rx: tokio::sync::mpsc::Receiver<Message>) {
        while let Some(msg) = rx.recv().await {
            use Message::*;
            match msg {
                Log(msg) => {
                    self.handle_log(msg).await;
                }
                Flush { tx } => {
                    self.handle_flush(tx);
                    rx.close();
                    break;
                }
                CollectGarbage => {
                    self.handle_collect_garbage().await;
                }
            }
        }
    }

    async fn handle_log(&mut self, message: LogMessage) {
        use tokio::io::AsyncWriteExt;
        let log_path_str = self.log_path.to_string_lossy().to_string();
        
        // Write to current log file - if this fails, log an error but continue
        if let Err(e) = self.log_file.write_all(format!("{}\n", &message).as_bytes()).await {
            eprintln!("Failed to write to log file {}: {}", log_path_str, e);
            // Continue anyway - logging should not break the application
        }
        
        if let Err(e) = self.log_file.flush().await {
            eprintln!("Failed to flush log file {}: {}", log_path_str, e);
        }

        // Write to latest log file - if this fails, log an error but continue
        if let Err(e) = self.latest_log_file.write_all(format!("{}\n", &message).as_bytes()).await {
            eprintln!("Failed to write to latest log file: {}", e);
        }
        
        if let Err(e) = self.latest_log_file.flush().await {
            eprintln!("Failed to flush latest log file: {}", e);
        }

        if message.level >= self.print_level {
            self.logs_to_print.push(message);
        }
    }

    fn handle_flush(self, tx: tokio::sync::oneshot::Sender<Result<(), LogError>>) {
        for message in &self.logs_to_print {
            eprintln!("{message}");
        }
        if !self.logs_to_print.is_empty() {
            eprintln!("Check the full log file: {}", self.log_path.display());
        }
        let _ = tx.send(Ok(()));
    }

    async fn handle_collect_garbage(&mut self) {
        if self.max_age == 0 {
            return;
        }
        let now = std::time::SystemTime::now();
        let Ok(logs) = self.fs.read_dir(self.log_dir.clone()).await else {
            self.handle_log(LogMessage {
                level: LogLevel::Error,
                scope: SCOPE,
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
                self.handle_log(LogMessage {
                    scope: SCOPE,
                    message: format!("Failed to remove the log file: {}", log.to_string_lossy()),
                    level: LogLevel::Warning,
                })
                .await;
            }
        }
    }
}
