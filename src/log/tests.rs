use super::*;
use crate::FsError;
use crate::{ArcPath, app::config::mock::MockConfig, fs::mock::MockFs};
use std::collections::LinkedList;
use std::time::Duration;
use tokio::fs::File;

#[tokio::test]
async fn test_log_message_creation() {
    // Test that we can create log messages
    let msg = LogMessage {
        level: LogLevel::Info,
        scope: "test",
        message: "test message".to_string(),
    };

    assert_eq!(msg.level, LogLevel::Info);
    assert_eq!(msg.scope, "test");
    assert_eq!(msg.message, "test message");
}

#[tokio::test]
async fn test_log_level_ordering() {
    // Test that log levels are ordered correctly
    assert!(LogLevel::Info < LogLevel::Warning);
    assert!(LogLevel::Warning < LogLevel::Error);
    assert!(LogLevel::Info < LogLevel::Error);
}

#[tokio::test]
async fn test_log_level_display() {
    // Test that log levels display correctly
    assert_eq!(LogLevel::Info.to_string(), "INFO");
    assert_eq!(LogLevel::Warning.to_string(), "WARN");
    assert_eq!(LogLevel::Error.to_string(), "ERROR");
}

#[tokio::test]
async fn test_log_flush_with_messages_above_level() {
    let temp_dir = tempfile::tempdir().expect("Creating temp directory to succeed");
    let log_dir = ArcPath::from(temp_dir.path());

    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();

    // Set up config expectations
    mock_config
        .expect_log_level()
        .returning(|| Ok(LogLevel::Warning));
    mock_config
        .expect_usize()
        .with(mockall::predicate::eq(crate::app::config::USizeOpt::MaxAge))
        .returning(|_| Ok(0));
    mock_config
        .expect_path()
        .with(mockall::predicate::eq(crate::app::config::PathOpt::LogDir))
        .returning(move |_| Ok(log_dir.clone()));

    // Set up fs expectations for initialization
    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs.expect_write_file().times(2).returning(|_| {
        let file = tempfile::tempfile().expect("Creating temp file to succeed");
        Ok(File::from_std(file))
    });

    let log = Log::spawn(mock_fs, mock_config).await.expect("Spawning log to succeed");

    // Log messages at different levels
    log.info("test", "Info message".to_string());
    log.warn("test", "Warning message".to_string());
    log.error("test", "Error message".to_string());

    // Give time for async logging to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Flush should complete successfully
    let result = log.flush().await;
    assert!(result.is_ok());

    // Note: We can't easily capture stderr in tests, but we verify flush completes
    // The actual stderr output would contain Warning and Error messages (above Info level)
}

#[tokio::test]
async fn test_log_flush_with_no_messages() {
    let temp_dir = tempfile::tempdir().expect("Creating temp directory to succeed");
    let log_dir = ArcPath::from(temp_dir.path());

    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();

    mock_config.expect_log_level().returning(|| Ok(LogLevel::Error));
    mock_config
        .expect_usize()
        .with(mockall::predicate::eq(crate::app::config::USizeOpt::MaxAge))
        .returning(|_| Ok(0));
    mock_config
        .expect_path()
        .with(mockall::predicate::eq(crate::app::config::PathOpt::LogDir))
        .returning(move |_| Ok(log_dir.clone()));

    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs.expect_write_file().times(2).returning(|_| {
        let file = tempfile::tempfile().expect("Creating temp file to succeed");
        Ok(File::from_std(file))
    });

    let log = Log::spawn(mock_fs, mock_config).await.expect("Spawning log to succeed");

    // Log a message below the print level (Info < Error)
    log.info("test", "Info message".to_string());

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Flush should complete successfully even with no messages to print
    let result = log.flush().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_log_collect_garbage_with_max_age_zero() {
    let temp_dir = tempfile::tempdir().expect("Creating temp directory to succeed");
    let log_dir = ArcPath::from(temp_dir.path());

    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();

    mock_config.expect_log_level().returning(|| Ok(LogLevel::Info)  );
    mock_config
        .expect_usize()
        .with(mockall::predicate::eq(crate::app::config::USizeOpt::MaxAge))
        .returning(|_| Ok(0)); // max_age = 0 means no cleanup

    mock_config
        .expect_path()
        .with(mockall::predicate::eq(crate::app::config::PathOpt::LogDir))
        .returning(move |_| Ok(log_dir.clone()));

    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs.expect_write_file().times(2).returning(|_| {
        let file = tempfile::tempfile().expect("Creating temp file to succeed");
        Ok(File::from_std(file))
    });

    let log = Log::spawn(mock_fs, mock_config).await.expect("Spawning log to succeed");

    // With max_age=0, collect_garbage should return immediately without reading directory
    log.collect_garbage().await;

    // Wait a bit to ensure the message is processed
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Cleanup
    let _ = log.flush().await;
}

#[tokio::test]
async fn test_log_collect_garbage_with_old_files() {
    let temp_dir = tempfile::tempdir().expect("Creating temp directory to succeed");
    let log_dir = ArcPath::from(temp_dir.path());

    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();

    mock_config.expect_log_level().returning(|| Ok(LogLevel::Info));
    mock_config
        .expect_usize()
        .with(mockall::predicate::eq(crate::app::config::USizeOpt::MaxAge))
        .returning(|_| Ok(7)); // max_age = 7 days

    let log_dir_clone = log_dir.clone();
    mock_config
        .expect_path()
        .with(mockall::predicate::eq(crate::app::config::PathOpt::LogDir))
        .returning(move |_| Ok(log_dir_clone.clone()));

    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs.expect_write_file().times(2).returning(|_| {
        let file = tempfile::tempfile().expect("Creating temp file to succeed");
        Ok(File::from_std(file))
    });

    // Create an old log file that should be deleted
    let old_log_file = log_dir.join("patch-hub_2020-01-01-00-00-00.log");
    std::fs::File::create(&old_log_file).expect("Creating old log file to succeed");

    // Create a recent log file that should be kept
    let recent_log_file = log_dir.join("patch-hub_2025-01-01-00-00-00.log");
    std::fs::File::create(&recent_log_file).expect("Creating recent log file to succeed");

    // Create a non-matching file that should be ignored
    let other_file = log_dir.join("other.log");
    std::fs::File::create(&other_file).expect("Creating other file to succeed");

    // Mock read_dir to return the log files
    let mut logs = LinkedList::new();
    logs.push_back(ArcPath::from(&old_log_file));
    logs.push_back(ArcPath::from(&recent_log_file));
    logs.push_back(ArcPath::from(&other_file));

    mock_fs
        .expect_read_dir()
        .returning(move |_| Ok(logs.clone()));

    // Mock remove_file - should only be called for old log file
    mock_fs
        .expect_remove_file()
        .withf(move |path| path.to_string_lossy().contains("2020"))
        .returning(|_| Ok(()));

    // Mock metadata for files (old file has old creation time)
    // Note: We can't easily mock metadata in the current setup, so we'll test the logic flow

    let log = Log::spawn(mock_fs, mock_config).await.expect("Spawning log to succeed");

    log.collect_garbage().await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Cleanup
    let _ = log.flush().await;
    std::fs::remove_file(&old_log_file).ok();
    std::fs::remove_file(&recent_log_file).ok();
    std::fs::remove_file(&other_file).ok();
}

#[tokio::test]
async fn test_log_collect_garbage_read_dir_error() {
    let temp_dir = tempfile::tempdir().expect("Creating temp directory to succeed");
    let log_dir = ArcPath::from(temp_dir.path());

    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();

    mock_config.expect_log_level().returning(|| Ok(LogLevel::Info));
    mock_config
        .expect_usize()
        .with(mockall::predicate::eq(crate::app::config::USizeOpt::MaxAge))
        .returning(|_| Ok(7));

    mock_config
        .expect_path()
        .with(mockall::predicate::eq(crate::app::config::PathOpt::LogDir))
        .returning(move |_| Ok(log_dir.clone()));

    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs
        .expect_write_file()
        .times(3) // Initial 2 + 1 for error log
        .returning(|_| {
            let file = tempfile::tempfile().expect("Creating temp file to succeed");
            Ok(File::from_std(file))
        });

    // Mock read_dir to return an error
    mock_fs.expect_read_dir().returning(|_| {
        Err(FsError::OperationFailed {
            path: None,
            operation: "read directory".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied"),
            retryable: false,
        })
    });

    let log = Log::spawn(mock_fs, mock_config).await.expect("Spawning log to succeed");

    // collect_garbage should handle the error gracefully and log it
    log.collect_garbage().await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Cleanup
    let _ = log.flush().await;
}
