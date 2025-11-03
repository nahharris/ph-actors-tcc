use super::*;
use crate::{
    ArcPath, FsError, api::lore::mock::MockLoreApi, app::config::mock::MockConfig,
    fs::mock::MockFs, log::mock::MockLog,
};
use tokio::fs::File;

#[tokio::test]
async fn test_mailing_list_cache_len_empty() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = ArcPath::from(&temp_dir.path().join("cache"));

    let mock_lore = MockLoreApi::new();
    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();
    let mut mock_log = MockLog::new();

    mock_config
        .expect_path()
        .with(mockall::predicate::eq(
            crate::app::config::PathOpt::CachePath,
        ))
        .returning(move |_| Ok(cache_dir.clone()));

    mock_log.expect_info().returning(|_, _| ());
    mock_log.expect_error().returning(|_, _| ());

    // Mock read_file to return file not found (cache doesn't exist)
    mock_fs.expect_read_file().returning(|_| {
        Err(FsError::OperationFailed {
            path: None,
            operation: "read file".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
            retryable: false,
        })
    });

    let cache = MailingListCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning mailing list cache to succeed");

    // Length of empty cache should be 0
    let len = cache.len().await;
    assert_eq!(len.expect("Getting mailing list length to succeed"), 0);
}

#[tokio::test]
async fn test_mailing_list_cache_get_from_empty() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = ArcPath::from(&temp_dir.path().join("cache"));

    let mock_lore = MockLoreApi::new();
    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();
    let mut mock_log = MockLog::new();

    mock_config
        .expect_path()
        .with(mockall::predicate::eq(
            crate::app::config::PathOpt::CachePath,
        ))
        .returning(move |_| Ok(cache_dir.clone()));

    mock_log.expect_info().returning(|_, _| ());
    mock_log.expect_error().returning(|_, _| ());

    mock_fs.expect_read_file().returning(|_| {
        Err(FsError::OperationFailed {
            path: None,
            operation: "read file".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
            retryable: false,
        })
    });

    let cache = MailingListCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning mailing list cache to succeed");

    // Get from empty cache should return None
    let result = cache.get(0).await;
    assert!(result.is_ok());
    assert_eq!(result.expect("Getting mailing list to succeed"), None);
}

#[tokio::test]
async fn test_mailing_list_cache_is_available_empty() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = ArcPath::from(&temp_dir.path().join("cache"));

    let mock_lore = MockLoreApi::new();
    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();
    let mut mock_log = MockLog::new();

    mock_config
        .expect_path()
        .with(mockall::predicate::eq(
            crate::app::config::PathOpt::CachePath,
        ))
        .returning(move |_| Ok(cache_dir.clone()));

    mock_log.expect_info().returning(|_, _| ());
    mock_log.expect_error().returning(|_, _| ());

    mock_fs.expect_read_file().returning(|_| {
        Err(FsError::OperationFailed {
            path: None,
            operation: "read file".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
            retryable: false,
        })
    });

    let cache = MailingListCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning mailing list cache to succeed");

    // Range should not be available in empty cache
    let is_available = cache.is_available(0..10).await;
    assert!(matches!(is_available, Ok(false)));
}

#[tokio::test]
async fn test_mailing_list_cache_get_slice_empty() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = ArcPath::from(&temp_dir.path().join("cache"));

    let mock_lore = MockLoreApi::new();
    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();
    let mut mock_log = MockLog::new();

    mock_config
        .expect_path()
        .with(mockall::predicate::eq(
            crate::app::config::PathOpt::CachePath,
        ))
        .returning(move |_| Ok(cache_dir.clone()));

    mock_log.expect_info().returning(|_, _| ());
    mock_log.expect_error().returning(|_, _| ());

    mock_fs.expect_read_file().returning(|_| {
        Err(FsError::OperationFailed {
            path: None,
            operation: "read file".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
            retryable: false,
        })
    });

    let cache = MailingListCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning mailing list cache to succeed");

    // Get slice from empty cache should return empty vec
    let result = cache.get_slice(0..10).await;
    assert!(result.is_ok());
    assert_eq!(
        result.expect("Getting mailing list slice to succeed"),
        Vec::<crate::api::lore::LoreMailingList>::new()
    );
}

#[tokio::test]
async fn test_mailing_list_cache_invalidate() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = ArcPath::from(&temp_dir.path().join("cache"));

    let mock_lore = MockLoreApi::new();
    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();
    let mut mock_log = MockLog::new();

    mock_config
        .expect_path()
        .with(mockall::predicate::eq(
            crate::app::config::PathOpt::CachePath,
        ))
        .returning(move |_| Ok(cache_dir.clone()));

    mock_log.expect_info().returning(|_, _| ());
    mock_log.expect_error().returning(|_, _| ());

    mock_fs.expect_read_file().returning(|_| {
        Err(FsError::OperationFailed {
            path: None,
            operation: "read file".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
            retryable: false,
        })
    });

    // Mock mkdir and write_file for persistence during invalidate
    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs.expect_write_file().returning(|_| {
        let file = tempfile::tempfile().unwrap();
        Ok(File::from_std(file))
    });

    let cache = MailingListCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning mailing list cache to succeed");

    // Invalidate should complete without error
    let result = cache.invalidate().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mailing_list_cache_persist() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = ArcPath::from(&temp_dir.path().join("cache"));

    let mock_lore = MockLoreApi::new();
    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();
    let mut mock_log = MockLog::new();

    mock_config
        .expect_path()
        .with(mockall::predicate::eq(
            crate::app::config::PathOpt::CachePath,
        ))
        .returning(move |_| Ok(cache_dir.clone()));

    mock_log.expect_info().returning(|_, _| ());
    mock_log.expect_error().returning(|_, _| ());

    mock_fs.expect_read_file().returning(|_| {
        Err(FsError::OperationFailed {
            path: None,
            operation: "read file".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
            retryable: false,
        })
    });

    // Mock mkdir and write_file for persistence
    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs.expect_write_file().returning(|_| {
        let file = tempfile::tempfile().unwrap();
        Ok(File::from_std(file))
    });

    let cache = MailingListCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning mailing list cache to succeed");

    // Persist should complete without error
    let result = cache.persist().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mailing_list_cache_load_nonexistent_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = ArcPath::from(&temp_dir.path().join("cache"));

    let mock_lore = MockLoreApi::new();
    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();
    let mut mock_log = MockLog::new();

    mock_config
        .expect_path()
        .with(mockall::predicate::eq(
            crate::app::config::PathOpt::CachePath,
        ))
        .returning(move |_| Ok(cache_dir.clone()));

    mock_log.expect_info().returning(|_, _| ());
    mock_log.expect_error().returning(|_, _| ());

    // Mock read_file to return file not found
    mock_fs
        .expect_read_file()
        .times(2) // Called once during spawn (load_cache), once during explicit load
        .returning(|_| {
            Err(FsError::OperationFailed {
                path: None,
                operation: "read file".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
                retryable: false,
            })
        });
    let cache = MailingListCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning mailing list cache to succeed");

    // Loading non-existent file should succeed (returns Ok(()))
    let result = cache.load().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mailing_list_cache_refresh() {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = ArcPath::from(&temp_dir.path().join("cache"));

    let mut mock_lore = MockLoreApi::new();
    let mut mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();
    let mut mock_log = MockLog::new();

    mock_config
        .expect_path()
        .with(mockall::predicate::eq(
            crate::app::config::PathOpt::CachePath,
        ))
        .returning(move |_| Ok(cache_dir.clone()));

    mock_log.expect_info().returning(|_, _| ());
    mock_log.expect_error().returning(|_, _| ());

    mock_fs.expect_read_file().returning(|_| {
        Err(FsError::OperationFailed {
            path: None,
            operation: "read file".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
            retryable: false,
        })
    });

    // Mock API to return empty result (refresh uses pagination)
    mock_lore
        .expect_get_available_lists_page()
        .returning(|_| Ok(None));

    // Mock persistence calls
    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs.expect_write_file().returning(|_| {
        let file = tempfile::tempfile().unwrap();
        Ok(File::from_std(file))
    });

    let cache = MailingListCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning mailing list cache to succeed");

    // Refresh should complete successfully
    let result = cache.refresh().await;
    assert!(result.is_ok());
}
