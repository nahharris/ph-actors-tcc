use super::*;
use crate::{ArcPath, env::mock::MockEnv, fs::mock::MockFs};
use std::io;
use tokio::fs::File;

#[tokio::test]
async fn test_config_load_valid_toml() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = ArcPath::from(&temp_dir.path().join("config.toml"));

    let mock_env = MockEnv::new();
    let mut mock_fs = MockFs::new();

    // Create valid TOML content
    let toml_content = r#"
log_level = "Info"
cache_path = "/tmp/cache"
log_dir = "/tmp/logs"
max_age = 30
timeout = 60
patch_renderer = "Bat"
"#;

    // Mock read_file to return valid TOML
    let toml_content_clone = toml_content.to_string();
    mock_fs
        .expect_read_file()
        .with(mockall::predicate::eq(config_path.clone()))
        .returning(move |_| {
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(temp_file.path(), &toml_content_clone).unwrap();
            Ok(File::from_std(temp_file.into_file()))
        });

    let config = Config::spawn(mock_env, mock_fs, config_path.clone());

    // Load should succeed
    let result = config.load().await;
    assert!(result.is_ok());

    // Verify values were loaded
    assert_eq!(config.log_level().await, LogLevel::Info);
    assert_eq!(
        config.path(PathOpt::CachePath).await.to_str().unwrap(),
        "/tmp/cache"
    );
    assert_eq!(config.usize(USizeOpt::MaxAge).await, 30);
    assert_eq!(config.usize(USizeOpt::Timeout).await, 60);
    assert_eq!(
        config.renderer(RendererOpt::PatchRenderer).await,
        Renderer::Bat
    );
}

#[tokio::test]
async fn test_config_load_invalid_toml() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = ArcPath::from(&temp_dir.path().join("config.toml"));

    let mock_env = MockEnv::new();
    let mut mock_fs = MockFs::new();

    // Create invalid TOML content
    let invalid_toml = "{ invalid toml syntax }".to_string();

    mock_fs
        .expect_read_file()
        .with(mockall::predicate::eq(config_path.clone()))
        .returning(move |_| {
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(temp_file.path(), &invalid_toml).unwrap();
            Ok(File::from_std(temp_file.into_file()))
        });

    let config = Config::spawn(mock_env, mock_fs, config_path);

    // Load should fail
    let result = config.load().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_load_file_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = ArcPath::from(&temp_dir.path().join("nonexistent.toml"));

    let mock_env = MockEnv::new();
    let mut mock_fs = MockFs::new();

    mock_fs
        .expect_read_file()
        .with(mockall::predicate::eq(config_path.clone()))
        .returning(|_| Err(io::Error::new(io::ErrorKind::NotFound, "File not found")));

    let config = Config::spawn(mock_env, mock_fs, config_path);

    // Load should fail
    let result = config.load().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_save() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = ArcPath::from(&temp_dir.path().join("config.toml"));

    let mock_env = MockEnv::new();
    let mut mock_fs = MockFs::new();

    // Mock write_file for save
    mock_fs
        .expect_write_file()
        .with(mockall::predicate::eq(config_path.clone()))
        .returning(|_| {
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            Ok(File::from_std(temp_file.into_file()))
        });

    let config = Config::spawn(mock_env, mock_fs, config_path);

    // Modify some values
    config.set_log_level(LogLevel::Error).await;
    config.set_usize(USizeOpt::Timeout, 120).await;

    // Save should succeed (we verify by checking no error occurs)
    let result = config.save().await;
    assert!(result.is_ok());

    // Verify the changes persisted
    assert_eq!(config.log_level().await, LogLevel::Error);
    assert_eq!(config.usize(USizeOpt::Timeout).await, 120);
}

#[tokio::test]
async fn test_config_get_set_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = ArcPath::from(&temp_dir.path().join("config.toml"));

    let mock_env = MockEnv::new();
    let mock_fs = MockFs::new();

    let config = Config::spawn(mock_env, mock_fs, config_path);

    // Test LogDir
    let new_log_dir = ArcPath::from("/custom/logs");
    config.set_path(PathOpt::LogDir, new_log_dir.clone()).await;
    assert_eq!(config.path(PathOpt::LogDir).await, new_log_dir);

    // Test CachePath
    let new_cache_path = ArcPath::from("/custom/cache");
    config
        .set_path(PathOpt::CachePath, new_cache_path.clone())
        .await;
    assert_eq!(config.path(PathOpt::CachePath).await, new_cache_path);
}

#[tokio::test]
async fn test_config_get_set_log_level() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = ArcPath::from(&temp_dir.path().join("config.toml"));

    let mock_env = MockEnv::new();
    let mock_fs = MockFs::new();

    let config = Config::spawn(mock_env, mock_fs, config_path);

    // Test default
    assert_eq!(config.log_level().await, LogLevel::Warning);

    // Test setting all levels
    config.set_log_level(LogLevel::Info).await;
    assert_eq!(config.log_level().await, LogLevel::Info);

    config.set_log_level(LogLevel::Warning).await;
    assert_eq!(config.log_level().await, LogLevel::Warning);

    config.set_log_level(LogLevel::Error).await;
    assert_eq!(config.log_level().await, LogLevel::Error);
}

#[tokio::test]
async fn test_config_get_set_usize() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = ArcPath::from(&temp_dir.path().join("config.toml"));

    let mock_env = MockEnv::new();
    let mock_fs = MockFs::new();

    let config = Config::spawn(mock_env, mock_fs, config_path);

    // Test MaxAge
    assert_eq!(config.usize(USizeOpt::MaxAge).await, 0); // default
    config.set_usize(USizeOpt::MaxAge, 30).await;
    assert_eq!(config.usize(USizeOpt::MaxAge).await, 30);

    // Test Timeout
    assert_eq!(config.usize(USizeOpt::Timeout).await, 30); // default
    config.set_usize(USizeOpt::Timeout, 120).await;
    assert_eq!(config.usize(USizeOpt::Timeout).await, 120);
}

#[tokio::test]
async fn test_config_get_set_renderer() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = ArcPath::from(&temp_dir.path().join("config.toml"));

    let mock_env = MockEnv::new();
    let mock_fs = MockFs::new();

    let config = Config::spawn(mock_env, mock_fs, config_path);

    // Test default
    assert_eq!(
        config.renderer(RendererOpt::PatchRenderer).await,
        Renderer::None
    );

    // Test setting all renderers
    config
        .set_renderer(RendererOpt::PatchRenderer, Renderer::Bat)
        .await;
    assert_eq!(
        config.renderer(RendererOpt::PatchRenderer).await,
        Renderer::Bat
    );

    config
        .set_renderer(RendererOpt::PatchRenderer, Renderer::Delta)
        .await;
    assert_eq!(
        config.renderer(RendererOpt::PatchRenderer).await,
        Renderer::Delta
    );

    config
        .set_renderer(RendererOpt::PatchRenderer, Renderer::None)
        .await;
    assert_eq!(
        config.renderer(RendererOpt::PatchRenderer).await,
        Renderer::None
    );
}
