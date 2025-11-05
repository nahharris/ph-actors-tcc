pub use data::{PathOpt, Renderer, RendererOpt, USizeOpt};

use tokio::sync::mpsc::Sender;

use crate::{error::ConfigError, error::FatalActorError, ArcPath, log::LogLevel};
#[cfg(not(test))]
use crate::{env::Env, fs::Fs};
#[cfg(test)]
use crate::{env::mock::MockEnv as Env, fs::mock::MockFs as Fs};

use message::Message;

mod core;
mod data;
mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

/// Actor name for error reporting.
pub const ACTOR_NAME: &'static str = "Config";

/// The configuration actor that provides a thread-safe interface for configuration operations.
///
/// This enum represents either a real configuration actor or a mock implementation
/// for testing purposes. It provides a unified interface for configuration operations
/// regardless of the underlying implementation.
///
/// # Examples
/// ```ignore
/// let config = Config::spawn(env, fs, config_path);
/// config.load().await?;
/// let log_dir = config.path(PathOpt::LogDir).await;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender.
#[derive(Debug, Clone)]
pub struct Config {
    tx: Sender<Message>,
}

impl Config {
    /// Creates a new configuration instance and spawns its actor.
    ///
    /// # Arguments
    /// * `env` - The environment actor for system operations
    /// * `fs` - The filesystem actor for file operations
    /// * `path` - The path to the configuration file
    ///
    /// # Returns
    /// A new configuration instance with a spawned actor.
    pub fn spawn(env: Env, fs: Fs, path: ArcPath) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(env, fs, path);
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }

    /// Loads the configuration from the file.
    ///
    /// For the mock implementation, this is a no-op that always succeeds.
    ///
    /// # Returns
    /// `Ok(())` for mock implementation.
    pub async fn load(&self) -> Result<(), ConfigError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Load { tx })
            .await
            .map_err(|_e| {
                ConfigError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "load config".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                ConfigError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "load config".to_string(),
                    source: e,
                })
            })?
    }

    /// Saves the current configuration to the file.
    ///
    pub async fn save(&self) -> Result<(), ConfigError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Save { tx })
            .await
            .map_err(|_e| {
                ConfigError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "save config".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                ConfigError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "save config".to_string(),
                    source: e,
                })
            })?
    }

    /// Gets a path-based configuration value.
    ///
    /// # Arguments
    /// * `opt` - The path option to retrieve
    ///
    /// # Returns
    /// The requested path value.
    pub async fn path(&self, opt: PathOpt) -> Result<ArcPath, ConfigError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::GetPath { opt, tx })
            .await
            .map_err(|_e| {
                ConfigError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "get path".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                ConfigError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "get path".to_string(),
                    source: e,
                })
            })?
    }

    /// Sets a path-based configuration value.
    ///
    /// # Arguments
    /// * `opt` - The path option to set
    /// * `path` - The new path value
    pub async fn set_path(&self, opt: PathOpt, path: ArcPath) -> Result<(), ConfigError> {
        self.tx
            .send(Message::SetPath { opt, path })
            .await
            .map_err(|_e| {
                ConfigError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "set path".to_string(),
                })
            })
    }

    /// Gets the current log level.
    ///
    /// # Returns
    /// The current log level.
    pub async fn log_level(&self) -> Result<LogLevel, ConfigError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::GetLogLevel { tx })
            .await
            .map_err(|_e| {
                ConfigError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "get log level".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                ConfigError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "get log level".to_string(),
                    source: e,
                })
            })?
    }

    /// Sets the log level.
    ///
    /// # Arguments
    /// * `level` - The new log level value
    pub async fn set_log_level(&self, level: LogLevel) -> Result<(), ConfigError> {
        self.tx
            .send(Message::SetLogLevel { level })
            .await
            .map_err(|_e| {
                ConfigError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "set log level".to_string(),
                })
            })
    }

    /// Gets a numeric configuration value.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to retrieve
    ///
    /// # Returns
    /// The requested numeric value.
    pub async fn usize(&self, opt: USizeOpt) -> Result<usize, ConfigError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::GetUSize { opt, tx })
            .await
            .map_err(|_e| {
                ConfigError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "get usize".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                ConfigError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "get usize".to_string(),
                    source: e,
                })
            })?
    }

    /// Sets a numeric configuration value.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to set
    /// * `value` - The new numeric value
    pub async fn set_usize(&self, opt: USizeOpt, value: usize) -> Result<(), ConfigError> {
        self.tx
            .send(Message::SetUSize { opt, size: value })
            .await
            .map_err(|_e| {
                ConfigError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "set usize".to_string(),
                })
            })
    }

    /// Gets a renderer configuration value.
    ///
    /// # Arguments
    /// * `opt` - The renderer option to retrieve
    ///
    /// # Returns
    /// The requested renderer value.
    pub async fn renderer(&self, opt: RendererOpt) -> Result<Renderer, ConfigError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::GetRenderer { opt, tx })
            .await
            .map_err(|_e| {
                ConfigError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "get renderer".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                ConfigError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "get renderer".to_string(),
                    source: e,
                })
            })?
    }

    /// Sets a renderer configuration value.
    ///
    /// # Arguments
    /// * `opt` - The renderer option to set
    /// * `renderer` - The new renderer value
    pub async fn set_renderer(&self, opt: RendererOpt, renderer: Renderer) -> Result<(), ConfigError> {
        self.tx
            .send(Message::SetRenderer { opt, renderer })
            .await
            .map_err(|_e| {
                ConfigError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::config::ACTOR_NAME,
                    operation: "set renderer".to_string(),
                })
            })
    }
}
