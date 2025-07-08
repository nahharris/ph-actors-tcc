use anyhow::Context;
use tokio::{io::AsyncWriteExt, task::JoinHandle};

use super::data::{LogLevel, LogMessage};
use super::message::Message;
use crate::{ArcFile, ArcPath, fs::Fs};

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

        fs.mkdir(log_dir.clone())
            .await
            .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;

        let log_file = fs
            .open_file(log_path.clone())
            .await
            .with_context(|| format!("Failed to create log file: {}", log_path.display()))?;

        let latest_log_path_clone = latest_log_path.clone();
        let latest_log_file = fs.open_file(latest_log_path).await.with_context(|| {
            format!(
                "Failed to create latest log file: {}",
                latest_log_path_clone.display()
            )
        })?;

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

    pub fn spawn(mut self) -> (super::Log, JoinHandle<()>) {
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
        (super::Log::Actual(tx), handle)
    }

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

    fn flush(self) {
        for message in &self.logs_to_print {
            eprintln!("{}", message);
        }
        if !self.logs_to_print.is_empty() {
            eprintln!("Check the full log file: {}", self.log_path.display());
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::Fs;
    use crate::ArcPath;
    use crate::log::data::{LogLevel, LogMessage};
    use std::collections::HashMap;

    fn mock_fs() -> Fs {
        Fs::mock(HashMap::new())
    }

    fn temp_log_dir() -> ArcPath {
        ArcPath::from("/tmp/test-logs")
    }

    #[tokio::test]
    #[ignore]
    async fn test_build_and_spawn() {
        let fs = mock_fs();
        let log_dir = temp_log_dir();
        let log_core = LogCore::build(fs, LogLevel::Info, 1, log_dir).await;
        assert!(log_core.is_ok());
        let log_core = log_core.unwrap();
        let (_log, handle) = log_core.spawn();
        handle.abort(); // Clean up
    }

    #[tokio::test]
    #[ignore]
    async fn test_log_and_flush() {
        let fs = mock_fs();
        let log_dir = temp_log_dir();
        let mut log_core = LogCore::build(fs, LogLevel::Info, 1, log_dir).await.unwrap();
        let msg = LogMessage { level: LogLevel::Info, message: "test".to_string() };
        log_core.log(msg.clone()).await;
        assert_eq!(log_core.logs_to_print.len(), 1);
        log_core.flush();
    }

    #[tokio::test]
    #[ignore]
    async fn test_log_level_filtering() {
        let fs = mock_fs();
        let log_dir = temp_log_dir();
        let mut log_core = LogCore::build(fs, LogLevel::Warning, 1, log_dir).await.unwrap();
        let msg = LogMessage { level: LogLevel::Info, message: "info".to_string() };
        log_core.log(msg.clone()).await;
        assert!(log_core.logs_to_print.is_empty());
        let msg2 = LogMessage { level: LogLevel::Warning, message: "warn".to_string() };
        log_core.log(msg2.clone()).await;
        assert_eq!(log_core.logs_to_print.len(), 1);
    }

    #[tokio::test]
    #[ignore]
    async fn test_collect_garbage_noop_when_max_age_zero() {
        let fs = mock_fs();
        let log_dir = temp_log_dir();
        let mut log_core = LogCore::build(fs, LogLevel::Info, 0, log_dir).await.unwrap();
        log_core.collect_garbage().await;
        // Should not panic or do anything
    }
}
