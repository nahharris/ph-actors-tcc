use std::collections::HashMap;

use crate::{
    ArcPath,
    app::config::{Config, PathOpt, USizeOpt, data::Data},
    env::Env,
    fs::Fs,
    log::LogLevel,
};
use anyhow::Result;

#[tokio::test]
async fn test_mock_config_creation() {
    let config = Config::mock(Data::default());
    assert!(matches!(config, Config::Mock(_)));
}

#[tokio::test]
async fn test_actual_config_creation() {
    let env = Env::mock();
    let fs = Fs::mock();
    let path = ArcPath::from("test_config.json");
    let config = Config::spawn(env, fs, path);
    assert!(matches!(config, Config::Actual(_)));
}

#[tokio::test]
async fn test_mock_path_operations() {
    let config = Config::mock(Data::default());

    // Test setting and getting path
    let new_path = ArcPath::from("/custom/path");
    config.set_path(PathOpt::LogDir, new_path.clone()).await;
    let retrieved_path = config.path(PathOpt::LogDir).await;
    assert_eq!(retrieved_path, new_path);
}

#[tokio::test]
async fn test_mock_log_level_operations() {
    let config = Config::mock(Data::default());

    // Test default log level
    let default_level = config.log_level().await;
    assert_eq!(default_level, LogLevel::Warning);

    // Test setting and getting log level
    config.set_log_level(LogLevel::Warning).await;
    let new_level = config.log_level().await;
    assert_eq!(new_level, LogLevel::Warning);
}

#[tokio::test]
async fn test_mock_usize_operations() {
    let config = Config::mock(Data::default());

    // Test setting and getting usize value
    let value = 1024;
    config.set_usize(USizeOpt::MaxAge, value).await;
    let retrieved_value = config.usize(USizeOpt::MaxAge).await;
    assert_eq!(retrieved_value, value);
}

#[tokio::test]
async fn test_actual_config_load_save() -> Result<()> {
    let env = Env::mock();
    let fs = Fs::mock();
    let path = ArcPath::from("test_config.toml");
    let config = Config::spawn(env, fs.clone(), path.clone());

    // Write a valid config TOML to the file before loading
    let valid_toml = include_str!("../../../samples/config.toml");
    let mut file = fs.open_file(path.clone()).await?;
    use tokio::io::AsyncWriteExt;
    file.write_all(valid_toml.as_bytes()).await?;

    config.load().await?;
    config.save().await?;
    Ok(())
}

#[tokio::test]
async fn test_mock_config_load_save() -> Result<()> {
    let config = Config::mock(Data::default());

    // Test load and save operations
    // These should be no-ops that always succeed for mock
    config.load().await?;
    config.save().await?;
    Ok(())
}

#[tokio::test]
async fn test_actual_config_path_operations() {
    let env = Env::mock();
    let fs = Fs::mock();
    let path = ArcPath::from("test_config.json");
    let config = Config::spawn(env, fs, path);

    // Test path operations
    let new_path = ArcPath::from("/custom/path");
    config.set_path(PathOpt::LogDir, new_path.clone()).await;
    let retrieved_path = config.path(PathOpt::LogDir).await;
    assert_eq!(retrieved_path, new_path);
}

#[tokio::test]
async fn test_actual_config_log_level_operations() {
    let env = Env::mock();
    let fs = Fs::mock();
    let path = ArcPath::from("test_config.json");
    let config = Config::spawn(env, fs, path);

    // Test log level operations
    config.set_log_level(LogLevel::Warning).await;
    let new_level = config.log_level().await;
    assert_eq!(new_level, LogLevel::Warning);
}

#[tokio::test]
async fn test_actual_config_usize_operations() {
    let env = Env::mock();
    let fs = Fs::mock();
    let path = ArcPath::from("test_config.json");
    let config = Config::spawn(env, fs, path);

    // Test usize operations
    let new_value = 1024;
    config.set_usize(USizeOpt::MaxAge, new_value).await;
    let retrieved_value = config.usize(USizeOpt::MaxAge).await;
    assert_eq!(retrieved_value, new_value);
}

#[tokio::test]
async fn test_multiple_path_options() {
    let config = Config::mock(Data::default());

    // Test different path options
    let paths = vec![(PathOpt::LogDir, "/logs")];

    for (opt, path_str) in paths {
        let path = ArcPath::from(path_str);
        config.set_path(opt, path.clone()).await;
        let retrieved = config.path(opt).await;
        assert_eq!(retrieved, path);
    }
}

#[tokio::test]
async fn test_multiple_usize_options() {
    let config = Config::mock(Data::default());

    // Test different usize options
    let values = vec![(USizeOpt::MaxAge, 1024)];

    for (opt, value) in values {
        config.set_usize(opt, value).await;
        let retrieved = config.usize(opt).await;
        assert_eq!(retrieved, value);
    }
}

#[tokio::test]
async fn test_actual_config_save() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("config.toml");
    let config_path = ArcPath::from(&config_path);

    // Use the real filesystem actor
    let fs = Fs::spawn();
    let env = Env::mock();
    let config = Config::spawn(env, fs.clone(), config_path.clone());

    // Set some values
    config
        .set_path(PathOpt::LogDir, ArcPath::from("/custom/logs"))
        .await;
    config.set_log_level(LogLevel::Info).await;
    config.set_usize(USizeOpt::MaxAge, 30).await;

    // Save the config
    config.save().await?;

    // Read and verify the saved file
    let contents = tokio::fs::read_to_string(&config_path).await?;
    let saved_data: Data = toml::from_str(&contents)?;

    assert_eq!(
        saved_data.path(PathOpt::LogDir).to_str().unwrap(),
        "/custom/logs"
    );
    assert_eq!(saved_data.log_level(), LogLevel::Info);
    assert_eq!(saved_data.usize(USizeOpt::MaxAge), 30);

    // Cleanup
    fs.remove_file(config_path.clone()).await.ok();
    temp_dir.close()?;

    Ok(())
}
