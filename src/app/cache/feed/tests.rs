use super::*;
use crate::{
    ArcPath, ArcStr, FsError, api::lore::mock::MockLoreApi, app::config::mock::MockConfig,
    fs::mock::MockFs, log::mock::MockLog,
};
use tokio::fs::File;

async fn create_test_feed_cache() -> (FeedCache, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let cache_dir = ArcPath::from(&temp_dir.path().join("cache"));

    let mock_lore = MockLoreApi::new();
    let mock_fs = MockFs::new();
    let mut mock_config = MockConfig::new();
    let mut mock_log = MockLog::new();

    // Set up config expectations
    mock_config
        .expect_path()
        .with(mockall::predicate::eq(
            crate::app::config::PathOpt::CachePath,
        ))
        .returning(move |_| Ok(cache_dir.clone()));

    // Set up log expectations (for various log calls)
    mock_log.expect_info().returning(|_, _| ());
    mock_log.expect_warn().returning(|_, _| ());

    let cache = FeedCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning feed cache to succeed");

    (cache, temp_dir)
}

#[tokio::test]
async fn test_feed_cache_get_from_empty_cache() {
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

    // Mock API to return None (no more pages)
    mock_lore
        .expect_get_patch_feed_page()
        .returning(|_, _| Ok(None));

    // Mock mkdir and write_file for persistence
    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs.expect_write_file().returning(|_| {
        let file = tempfile::tempfile().unwrap();
        Ok(File::from_std(file))
    });

    let cache = FeedCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning feed cache to succeed");

    let list = ArcStr::from("test-list");

    // Get from empty cache should return None
    let result = cache.get(list, 0).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

#[tokio::test]
async fn test_feed_cache_len_empty() {
    let (cache, _temp_dir) = create_test_feed_cache().await;

    let list = ArcStr::from("test-list");

    // Length of empty cache should be 0
    let len = cache.len(list).await;
    assert_eq!(len.expect("Getting feed length to succeed"), 0);
}

#[tokio::test]
async fn test_feed_cache_is_loaded_empty() {
    let (cache, _temp_dir) = create_test_feed_cache().await;

    let list = ArcStr::from("test-list");

    // Initially, cache should not be loaded
    let is_loaded = cache.is_loaded(list).await;
    assert!(matches!(is_loaded, Ok(false)));
}

#[tokio::test]
async fn test_feed_cache_is_available_empty() {
    let (cache, _temp_dir) = create_test_feed_cache().await;

    let list = ArcStr::from("test-list");

    // Range should not be available in empty cache
    let is_available = cache.is_available(list, 0..10).await;
    assert!(matches!(is_available, Ok(false)));
}

#[tokio::test]
async fn test_feed_cache_invalidate() {
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

    // Mock mkdir and write_file for persistence during invalidate
    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs.expect_write_file().returning(|_| {
        let file = tempfile::tempfile().unwrap();
        Ok(File::from_std(file))
    });

    let cache = FeedCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning feed cache to succeed");

    let list = ArcStr::from("test-list");

    // Invalidate should complete without error
    let result = cache.invalidate(list).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_feed_cache_persist_with_data() {
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

    // Mock mkdir for cache directory creation
    mock_fs.expect_mkdir().returning(|_| Ok(()));

    // Mock write_file for persistence
    mock_fs.expect_write_file().returning(|_| {
        let file = tempfile::tempfile().unwrap();
        Ok(File::from_std(file))
    });

    let cache = FeedCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning feed cache to succeed");

    let list = ArcStr::from("test-list");

    // Persist should complete without error
    let result = cache.persist(list).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_feed_cache_load_nonexistent_file() {
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

    // Mock read_file to return file not found
    mock_fs.expect_read_file().returning(|_| {
        Err(FsError::OperationFailed {
            path: None,
            operation: "read file".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
            retryable: false,
        })
    });

    let cache = FeedCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning feed cache to succeed");

    let list = ArcStr::from("test-list");

    // Loading non-existent file should succeed (returns Ok(()))
    let result = cache.load(list).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_feed_cache_get_slice_empty() {
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

    // Mock API to return None (no more pages)
    mock_lore
        .expect_get_patch_feed_page()
        .returning(|_, _| Ok(None));

    // Mock mkdir and write_file for persistence
    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs.expect_write_file().returning(|_| {
        let file = tempfile::tempfile().unwrap();
        Ok(File::from_std(file))
    });

    let cache = FeedCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning feed cache to succeed");

    let list = ArcStr::from("test-list");

    // Get slice from empty cache should return empty vec
    let result = cache.get_slice(list, 0..10).await;
    assert!(result.is_ok());
    assert_eq!(
        result.expect("Getting feed slice to succeed"),
        Vec::<crate::api::lore::LorePatchMetadata>::new()
    );
}

#[tokio::test]
async fn test_feed_cache_refresh() {
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

    // Mock API to return None (no pages available)
    mock_lore
        .expect_get_patch_feed_page()
        .returning(|_, _| Ok(None));

    // Mock persistence calls
    mock_fs.expect_mkdir().returning(|_| Ok(()));
    mock_fs.expect_write_file().returning(|_| {
        let file = tempfile::tempfile().unwrap();
        Ok(File::from_std(file))
    });

    let cache = FeedCache::spawn(mock_lore, mock_fs, mock_config, mock_log)
        .await
        .expect("Spawning feed cache to succeed");

    let list = ArcStr::from("test-list");

    // Refresh should complete successfully
    let result = cache.refresh(list).await;
    assert!(result.is_ok());
}
