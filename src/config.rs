use std::{path::Path, str::FromStr, sync::Arc};

use anyhow::Context;
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{ArcPath, env::Env, fs::Fs, log::LogLevel};

/// The core of the configuration system that manages application settings.
///
/// This struct provides thread-safe access to configuration data through an actor pattern.
/// It handles loading and saving configuration from/to a TOML file, and provides
/// methods to access and modify various configuration options.
///
/// # Features
/// - Thread-safe configuration access through actor pattern
/// - TOML-based configuration storage
/// - Automatic configuration file creation
/// - Type-safe configuration options
///
/// # Examples
/// ```
/// let config = ConfigCore::new(env, fs, config_path);
/// let (config, _) = config.spawn();
/// config.load().await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads through the actor pattern.
/// All configuration operations are handled sequentially to ensure consistency.
#[derive(Debug)]
pub struct ConfigCore {
    /// The current configuration data
    data: Data,
    /// Path to the configuration file
    path: ArcPath,
    /// Environment interface for system operations
    env: Env,
    /// Filesystem interface for file operations
    fs: Fs,
}

impl ConfigCore {
    /// Creates a new configuration instance.
    ///
    /// # Arguments
    /// * `env` - The environment actor for system operations
    /// * `fs` - The filesystem actor for file operations
    /// * `path` - The path to the configuration file
    ///
    /// # Returns
    /// A new instance of `ConfigCore` with default configuration values.
    pub fn new(env: Env, fs: Fs, path: ArcPath) -> Self {
        Self {
            data: Data::default(),
            path,
            env,
            fs,
        }
    }

    /// Transforms the configuration core instance into an actor.
    ///
    /// This method spawns a new task that will handle configuration operations
    /// asynchronously through a message channel. All operations are processed
    /// sequentially to ensure consistency.
    ///
    /// # Returns
    /// A tuple containing:
    /// - A [`Config`] instance that can be used to send messages to the actor
    /// - A join handle for the spawned task
    ///
    /// # Panics
    /// This function will panic if the underlying task fails to spawn.
    pub fn spawn(mut self) -> (Config, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    Message::Load(tx) => {
                        let res = self.load().await;
                        let _ = tx.send(res);
                    }
                    Message::Save(tx) => {
                        let res = self.save().await;
                        let _ = tx.send(res);
                    }
                    Message::GetPath(opt, tx) => {
                        let res = match opt {
                            PathOpts::LogDir => Arc::clone(&self.data.log_dir),
                        };
                        let _ = tx.send(res);
                    }
                    Message::GetLogLevel(tx) => {
                        let res = self.data.log_level;
                        let _ = tx.send(res);
                    }
                    Message::GetUSize(opt, tx) => {
                        let res = match opt {
                            USizeOpts::MaxAge => self.data.max_age,
                        };
                        let _ = tx.send(res);
                    }
                    Message::SetPath(opt, path) => match opt {
                        PathOpts::LogDir => self.data.log_dir = path,
                    },
                    Message::SetLogLevel(level) => {
                        self.data.log_level = level;
                    }
                    Message::SetUSize(opt, size) => match opt {
                        USizeOpts::MaxAge => self.data.max_age = size,
                    },
                }
            }
        });
        (Config::Actual(tx), handle)
    }

    /// Loads the configuration from the file.
    ///
    /// This method reads the configuration file and deserializes its contents
    /// into the internal data structure. If the file doesn't exist or is invalid,
    /// an error is returned.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The configuration file doesn't exist
    /// - The file is not a valid TOML file
    /// - The file cannot be read
    /// - The configuration data is invalid
    async fn load(&mut self) -> anyhow::Result<()> {
        let config = self.fs.open_file(Arc::clone(&self.path)).await?;
        let mut buf = String::new();

        config.write().await.read_to_string(&mut buf).await?;

        let data: Data = toml::de::from_str(&buf)?;
        self.data = data;
        Ok(())
    }

    /// Saves the current configuration to the file.
    ///
    /// This method serializes the internal configuration data to TOML format
    /// and writes it to the configuration file. The file is created if it
    /// doesn't exist.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The configuration file cannot be created or opened
    /// - The configuration data cannot be serialized
    /// - The file cannot be written
    async fn save(&self) -> anyhow::Result<()> {
        let config = self.fs.open_file(Arc::clone(&self.path)).await?;
        let buf = toml::ser::to_string_pretty(&self.data)?;

        config.write().await.write_all(buf.as_bytes()).await?;
        config.write().await.flush().await?;
        config.write().await.sync_all().await?;

        Ok(())
    }
}

/// The data structure that holds all configuration values.
///
/// This struct contains all the configurable options for the application.
/// It implements serialization and deserialization for TOML format.
#[derive(Debug)]
struct Data {
    /// Directory where log files are stored
    log_dir: ArcPath,
    /// Minimum level of messages to be logged
    log_level: LogLevel,
    /// Maximum age of log files in days before they are deleted
    max_age: usize,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            log_dir: Arc::from(Path::new("/tmp")),
            log_level: LogLevel::Warning,
            max_age: 30,
        }
    }
}

impl Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Data", 1)?;
        state.serialize_field("log_dir", self.log_dir.to_str().unwrap())?;
        state.serialize_field("log_level", &self.log_level)?;
        state.serialize_field("max_age", &self.max_age)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Data {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut data: Data = Default::default();

        let map = toml::Value::deserialize(deserializer)?;
        if let Some(log_dir) = map.get("log_dir") {
            if let Some(log_dir) = log_dir.as_str() {
                data.log_dir = Arc::from(Path::new(log_dir));
            }
        }
        if let Some(log_level) = map.get("log_level") {
            if let Some(log_level) = log_level.as_str() {
                data.log_level = LogLevel::from_str(log_level).map_err(serde::de::Error::custom)?;
            }
        }
        if let Some(max_age) = map.get("max_age") {
            if let Some(max_age) = max_age.as_integer() {
                data.max_age = max_age as usize;
            }
        }
        Ok(data)
    }
}

/// Options for path-based configuration values.
///
/// This enum defines the different path-based configuration options
/// that can be accessed and modified.
#[derive(Debug, Clone, Copy)]
pub enum PathOpts {
    /// Directory where log files are stored
    LogDir,
}

/// Options for numeric configuration values.
///
/// This enum defines the different numeric configuration options
/// that can be accessed and modified.
#[derive(Debug, Clone, Copy)]
pub enum USizeOpts {
    /// Maximum age of log files in days before they are deleted
    MaxAge,
}

/// Messages that can be sent to a [`ConfigCore`] actor.
///
/// This enum defines the different types of operations that can be performed
/// through the configuration actor system.
#[derive(Debug)]
pub enum Message {
    /// Loads the configuration from the file
    Load(tokio::sync::oneshot::Sender<anyhow::Result<()>>),
    /// Saves the current configuration to the file
    Save(tokio::sync::oneshot::Sender<anyhow::Result<()>>),
    /// Gets a path-based configuration value
    GetPath(PathOpts, tokio::sync::oneshot::Sender<ArcPath>),
    /// Gets the current log level
    GetLogLevel(tokio::sync::oneshot::Sender<LogLevel>),
    /// Gets a numeric configuration value
    GetUSize(USizeOpts, tokio::sync::oneshot::Sender<usize>),
    /// Sets a path-based configuration value
    SetPath(PathOpts, ArcPath),
    /// Sets the log level
    SetLogLevel(LogLevel),
    /// Sets a numeric configuration value
    SetUSize(USizeOpts, usize),
}

/// The configuration actor that provides a thread-safe interface for configuration operations.
///
/// This enum represents either a real configuration actor or a mock implementation
/// for testing purposes. It provides a unified interface for configuration operations
/// regardless of the underlying implementation.
///
/// # Examples
/// ```
/// let (config, _) = ConfigCore::new(env, fs, config_path).spawn();
/// config.load().await?;
/// let log_dir = config.path(PathOpts::LogDir).await;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender.
#[derive(Debug, Clone)]
pub enum Config {
    /// A real configuration actor that reads from and writes to a file
    Actual(tokio::sync::mpsc::Sender<Message>),
    /// A mock implementation for testing that does nothing
    Mock,
}

impl Config {
    /// Loads the configuration
    pub async fn load(&self) -> anyhow::Result<()> {
        if let Self::Actual(sender) = self {
            let (tx, rx) = tokio::sync::oneshot::channel();
            sender
                .send(Message::Load(tx))
                .await
                .expect("Config actor is dead");
            return rx.await.expect("Config actor is dead");
        }

        Ok(())
    }

    /// Saves the configuration
    pub async fn save(&self) -> anyhow::Result<()> {
        if let Self::Actual(sender) = self {
            let (tx, rx) = tokio::sync::oneshot::channel();
            sender
                .send(Message::Save(tx))
                .await
                .expect("Config actor is dead");
            return rx.await.expect("Config actor is dead");
        }

        Ok(())
    }

    /// Gets a config of type path
    pub async fn path(&self, opt: PathOpts) -> ArcPath {
        if let Self::Actual(sender) = self {
            let (tx, rx) = tokio::sync::oneshot::channel();
            sender
                .send(Message::GetPath(opt, tx))
                .await
                .expect("Config actor is dead");
            return rx.await.expect("Config actor is dead");
        }

        unimplemented!()
    }

    /// Sets a config of type path
    pub async fn set_path(&self, opt: PathOpts, path: ArcPath) {
        if let Self::Actual(sender) = self {
            sender
                .send(Message::SetPath(opt, path))
                .await
                .expect("Config actor is dead");
        }
    }

    pub async fn log_level(&self) -> LogLevel {
        if let Self::Actual(sender) = self {
            let (tx, rx) = tokio::sync::oneshot::channel();
            sender
                .send(Message::GetLogLevel(tx))
                .await
                .expect("Config actor died");
            return rx.await.expect("Config actor died");
        }

        unimplemented!()
    }

    pub async fn set_log_level(&self, level: LogLevel) {
        if let Self::Actual(sender) = self {
            let _ = sender.send(Message::SetLogLevel(level)).await;
        }
    }

    pub async fn usize(&self, opt: USizeOpts) -> usize {
        if let Self::Actual(sender) = self {
            let (tx, rx) = tokio::sync::oneshot::channel();
            sender
                .send(Message::GetUSize(opt, tx))
                .await
                .expect("Config actor died");
            return rx.await.expect("Config actor died");
        }

        unimplemented!()
    }

    pub async fn set_usize(&self, opt: USizeOpts, value: usize) {
        if let Self::Actual(sender) = self {
            let _ = sender.send(Message::SetUSize(opt, value)).await;
        }
    }
}
