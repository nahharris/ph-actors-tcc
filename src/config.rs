use std::{path::Path, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize, ser::SerializeStruct};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::Mutex};

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
                    Message::Load { tx } => {
                        let res = self.load().await;
                        let _ = tx.send(res);
                    }
                    Message::Save { tx } => {
                        let res = self.save().await;
                        let _ = tx.send(res);
                    }
                    Message::GetPath { opt, tx } => {
                        let res = self.data.path(opt);
                        let _ = tx.send(res);
                    }
                    Message::GetLogLevel { tx } => {
                        let res = self.data.log_level();
                        let _ = tx.send(res);
                    }
                    Message::GetUSize { opt, tx } => {
                        let res = self.data.usize(opt);
                        let _ = tx.send(res);
                    }
                    Message::SetPath { opt, path } => {
                        self.data.set_path(opt, path);
                    }
                    Message::SetLogLevel { level } => {
                        self.data.set_log_level(level);
                    }
                    Message::SetUSize { opt, size } => {
                        self.data.set_usize(opt, size);
                    }
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
#[derive(Debug, Clone)]
pub struct Data {
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

impl Data {
    /// Gets a path-based configuration value.
    ///
    /// # Arguments
    /// * `opt` - The path option to retrieve
    ///
    /// # Returns
    /// The requested path value.
    pub fn path(&self, opt: PathOpt) -> ArcPath {
        match opt {
            PathOpt::LogDir => Arc::clone(&self.log_dir),
        }
    }

    /// Sets a path-based configuration value.
    ///
    /// # Arguments
    /// * `opt` - The path option to set
    /// * `path` - The new path value
    pub fn set_path(&mut self, opt: PathOpt, path: ArcPath) {
        match opt {
            PathOpt::LogDir => self.log_dir = path,
        }
    }

    /// Gets the current log level.
    ///
    /// # Returns
    /// The current log level.
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    /// Sets the log level.
    ///
    /// # Arguments
    /// * `level` - The new log level value
    pub fn set_log_level(&mut self, level: LogLevel) {
        self.log_level = level;
    }

    /// Gets a numeric configuration value.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to retrieve
    ///
    /// # Returns
    /// The requested numeric value.
    pub fn usize(&self, opt: USizeOpt) -> usize {
        match opt {
            USizeOpt::MaxAge => self.max_age,
        }
    }

    /// Sets a numeric configuration value.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to set
    /// * `value` - The new numeric value
    pub fn set_usize(&mut self, opt: USizeOpt, value: usize) {
        match opt {
            USizeOpt::MaxAge => self.max_age = value,
        }
    }
}

/// Options for path-based configuration values.
///
/// This enum defines the different path-based configuration options
/// that can be accessed and modified.
#[derive(Debug, Clone, Copy)]
pub enum PathOpt {
    /// Directory where log files are stored
    LogDir,
}

/// Options for numeric configuration values.
///
/// This enum defines the different numeric configuration options
/// that can be accessed and modified.
#[derive(Debug, Clone, Copy)]
pub enum USizeOpt {
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
    Load {
        /// Channel to send the result back to the caller
        tx: tokio::sync::oneshot::Sender<anyhow::Result<()>>,
    },
    /// Saves the current configuration to the file
    Save {
        /// Channel to send the result back to the caller
        tx: tokio::sync::oneshot::Sender<anyhow::Result<()>>,
    },
    /// Gets a path-based configuration value
    GetPath {
        /// The path option to retrieve
        opt: PathOpt,
        /// Channel to send the result back to the caller
        tx: tokio::sync::oneshot::Sender<ArcPath>,
    },
    /// Gets the current log level
    GetLogLevel {
        /// Channel to send the result back to the caller
        tx: tokio::sync::oneshot::Sender<LogLevel>,
    },
    /// Gets a numeric configuration value
    GetUSize {
        /// The numeric option to retrieve
        opt: USizeOpt,
        /// Channel to send the result back to the caller
        tx: tokio::sync::oneshot::Sender<usize>,
    },
    /// Sets a path-based configuration value
    SetPath {
        /// The path option to set
        opt: PathOpt,
        /// The new path value
        path: ArcPath,
    },
    /// Sets the log level
    SetLogLevel {
        /// The new log level value
        level: LogLevel,
    },
    /// Sets a numeric configuration value
    SetUSize {
        /// The numeric option to set
        opt: USizeOpt,
        /// The new numeric value
        size: usize,
    },
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
    Mock(Arc<Mutex<Data>>),
}

impl Config {
    /// Creates a new mock configuration instance for testing.
    ///
    /// # Arguments
    /// * `data` - Optional initial configuration data. If None, default values will be used.
    ///
    /// # Returns
    /// A new mock configuration instance that stores data in memory.
    pub fn mock(data: Option<Data>) -> Self {
        Self::Mock(Arc::new(Mutex::new(data.unwrap_or_default())))
    }

    /// Loads the configuration from the file.
    ///
    /// For the mock implementation, this is a no-op that always succeeds.
    ///
    /// # Returns
    /// `Ok(())` for mock implementation.
    pub async fn load(&self) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Load { tx })
                    .await
                    .expect("Config actor is dead");
                rx.await.expect("Config actor is dead")
            }
            Self::Mock(_) => Ok(()),
        }
    }

    /// Saves the current configuration to the file.
    ///
    /// For the mock implementation, this is a no-op that always succeeds.
    ///
    /// # Returns
    /// `Ok(())` for mock implementation.
    pub async fn save(&self) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Save { tx })
                    .await
                    .expect("Config actor is dead");
                rx.await.expect("Config actor is dead")
            }
            Self::Mock(_) => Ok(()),
        }
    }

    /// Gets a path-based configuration value.
    ///
    /// # Arguments
    /// * `opt` - The path option to retrieve
    ///
    /// # Returns
    /// The requested path value.
    pub async fn path(&self, opt: PathOpt) -> ArcPath {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::GetPath { opt, tx })
                    .await
                    .expect("Config actor is dead");
                rx.await.expect("Config actor is dead")
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                data.path(opt)
            }
        }
    }

    /// Sets a path-based configuration value.
    ///
    /// # Arguments
    /// * `opt` - The path option to set
    /// * `path` - The new path value
    pub async fn set_path(&self, opt: PathOpt, path: ArcPath) {
        match self {
            Self::Actual(sender) => {
                sender
                    .send(Message::SetPath { opt, path })
                    .await
                    .expect("Config actor is dead");
            }
            Self::Mock(data) => {
                let mut data = data.lock().await;
                data.set_path(opt, path);
            }
        }
    }

    /// Gets the current log level.
    ///
    /// # Returns
    /// The current log level.
    pub async fn log_level(&self) -> LogLevel {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::GetLogLevel { tx })
                    .await
                    .expect("Config actor died");
                rx.await.expect("Config actor died")
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                data.log_level()
            }
        }
    }

    /// Sets the log level.
    ///
    /// # Arguments
    /// * `level` - The new log level value
    pub async fn set_log_level(&self, level: LogLevel) {
        match self {
            Self::Actual(sender) => {
                let _ = sender.send(Message::SetLogLevel { level }).await;
            }
            Self::Mock(data) => {
                let mut data = data.lock().await;
                data.set_log_level(level);
            }
        }
    }

    /// Gets a numeric configuration value.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to retrieve
    ///
    /// # Returns
    /// The requested numeric value.
    pub async fn usize(&self, opt: USizeOpt) -> usize {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::GetUSize { opt, tx })
                    .await
                    .expect("Config actor died");
                rx.await.expect("Config actor died")
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                data.usize(opt)
            }
        }
    }

    /// Sets a numeric configuration value.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to set
    /// * `value` - The new numeric value
    pub async fn set_usize(&self, opt: USizeOpt, value: usize) {
        match self {
            Self::Actual(sender) => {
                let _ = sender.send(Message::SetUSize { opt, size: value }).await;
            }
            Self::Mock(data) => {
                let mut data = data.lock().await;
                data.set_usize(opt, value);
            }
        }
    }
}
