use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    config::{Config, ConfigCore, Data, PathOpt, USizeOpt},
    env::Env,
    fs::Fs,
    log::LogLevel,
    ArcPath,
};

#[tokio::test]
async fn test_config_core_default_values() {
    let env = Env::mock();
    let fs = Fs::mock();
    let path = ArcPath::from("/tmp/config.toml");
    let config = ConfigCore::new(env, fs, path);

    assert_eq!(config.data.log_level(), LogLevel::Warning);
    assert_eq!(config.data.path(PathOpt::LogDir).to_str().unwrap(), "/tmp");
    assert_eq!(config.data.usize(USizeOpt::MaxAge), 30);
}

#[tokio::test]
async fn test_config_data_serialization() {
    let data = Data::default();
    let toml = toml::to_string_pretty(&data).unwrap();
    let deserialized: Data = toml::from_str(&toml).unwrap();

    assert_eq!(data.log_level(), deserialized.log_level());
    assert_eq!(data.path(PathOpt::LogDir), deserialized.path(PathOpt::LogDir));
    assert_eq!(data.usize(USizeOpt::MaxAge), deserialized.usize(USizeOpt::MaxAge));
}

#[tokio::test]
async fn test_config_actor_operations() {
    let env = Env::mock();
    let fs = Fs::mock();
    let path = ArcPath::from("/tmp/config.toml");
    let (config, _) = ConfigCore::new(env, fs, path).spawn();

    // Test setting and getting values
    config.set_log_level(LogLevel::Info).await;
    assert_eq!(config.log_level().await, LogLevel::Info);

    let new_path = ArcPath::from("/var/log");
    config.set_path(PathOpt::LogDir, new_path.clone()).await;
    assert_eq!(config.path(PathOpt::LogDir).await, new_path);

    config.set_usize(USizeOpt::MaxAge, 60).await;
    assert_eq!(config.usize(USizeOpt::MaxAge).await, 60);
}

#[tokio::test]
async fn test_config_mock() {
    let config = Config::mock(None);

    // Test default values
    assert_eq!(config.log_level().await, LogLevel::Warning);
    assert_eq!(config.path(PathOpt::LogDir).await.to_str().unwrap(), "/tmp");
    assert_eq!(config.usize(USizeOpt::MaxAge).await, 30);

    // Test setting and getting values
    config.set_log_level(LogLevel::Error).await;
    assert_eq!(config.log_level().await, LogLevel::Error);

    let new_path = ArcPath::from("/var/log");
    config.set_path(PathOpt::LogDir, new_path.clone()).await;
    assert_eq!(config.path(PathOpt::LogDir).await, new_path);

    config.set_usize(USizeOpt::MaxAge, 90).await;
    assert_eq!(config.usize(USizeOpt::MaxAge).await, 90);
}

#[tokio::test]
async fn test_config_load_save() {
    let env = Env::mock();
    let fs = Fs::mock();
    let path = ArcPath::from("/tmp/config.toml");
    let config = Config::spawn(env, fs, path);

    // Test load/save operations
    assert!(config.load().await.is_ok());
    assert!(config.save().await.is_ok());
}

#[tokio::test]
async fn test_config_with_custom_data() {
    let custom_data = Data {
        log_dir: ArcPath::from("/custom/log"),
        log_level: LogLevel::Info,
        max_age: 45,
    };
    let config = Config::mock(Some(custom_data));

    assert_eq!(config.log_level().await, LogLevel::Info);
    assert_eq!(config.path(PathOpt::LogDir).await.to_str().unwrap(), "/custom/log");
    assert_eq!(config.usize(USizeOpt::MaxAge).await, 45);
} 