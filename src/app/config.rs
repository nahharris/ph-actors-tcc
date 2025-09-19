use data::Data;
pub use data::{PathOpt, Renderer, RendererOpt, USizeOpt};
use message::Message;

use crate::{ArcPath, env::Env, fs::Fs, log::LogLevel};
use anyhow::Context;

mod core;
mod data;
mod mock;
mod message;
#[cfg(test)]
mod tests;

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
pub enum Config {
    /// A real configuration actor that reads from and writes to a file
    Actual(tokio::sync::mpsc::Sender<Message>),
    /// A mock implementation for testing that stores data in memory
    Mock(mock::Mock),
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
        let (config, _) = core::Core::new(env, fs, path).spawn();
        config
    }

    /// Creates a new mock configuration instance for testing.
    ///
    /// # Arguments
    /// * `data` - Optional initial configuration data. If None, default values will be used.
    ///
    /// # Returns
    /// A new mock configuration instance that stores data in memory.
    pub fn mock(data: Data) -> Self {
        Self::Mock(mock::Mock::new(data))
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
                    .context("Loading config with Config actor")
                    .expect("Config actor is dead");
                rx.await
                    .context("Awaiting response for config load with Config actor")
                    .expect("Config actor is dead")
            }
            Self::Mock(mock) => {
                mock.load().await
            }
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
                    .context("Saving config with Config actor")
                    .expect("Config actor is dead");
                rx.await
                    .context("Awaiting response for config save with Config actor")
                    .expect("Config actor is dead")
            }
            Self::Mock(mock) => {
                mock.save().await
            }
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
                    .context("Getting path with Config actor")
                    .expect("Config actor is dead");
                rx.await
                    .context("Awaiting response for path with Config actor")
                    .expect("Config actor is dead")
            }
            Self::Mock(mock) => {
                mock.path(opt).await
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
                    .context("Setting path with Config actor")
                    .expect("Config actor is dead");
            }
            Self::Mock(mock) => {
                mock.set_path(opt, path).await
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
                    .context("Getting log level with Config actor")
                    .expect("Config actor died");
                rx.await
                    .context("Awaiting response for log level with Config actor")
                    .expect("Config actor died")
            }
            Self::Mock(mock) => {
                mock.log_level().await
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
            Self::Mock(mock) => {
                mock.set_log_level(level).await
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
                    .context("Getting numeric value with Config actor")
                    .expect("Config actor died");
                rx.await
                    .context("Awaiting response for numeric value with Config actor")
                    .expect("Config actor died")
            }
            Self::Mock(mock) => {
                mock.usize(opt).await
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
            Self::Mock(mock) => {
                mock.set_usize(opt, value).await
            }
        }
    }

    /// Gets a renderer configuration value.
    ///
    /// # Arguments
    /// * `opt` - The renderer option to retrieve
    ///
    /// # Returns
    /// The requested renderer value.
    pub async fn renderer(&self, opt: RendererOpt) -> Renderer {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::GetRenderer { opt, tx })
                    .await
                    .context("Getting renderer value with Config actor")
                    .expect("Config actor died");
                rx.await
                    .context("Awaiting response for renderer value with Config actor")
                    .expect("Config actor died")
            }
            Self::Mock(mock) => {
                mock.renderer(opt).await
            }
        }
    }

    /// Sets a renderer configuration value.
    ///
    /// # Arguments
    /// * `opt` - The renderer option to set
    /// * `renderer` - The new renderer value
    pub async fn set_renderer(&self, opt: RendererOpt, renderer: Renderer) {
        match self {
            Self::Actual(sender) => {
                let _ = sender.send(Message::SetRenderer { opt, renderer }).await;
            }
            Self::Mock(mock) => {
                mock.set_renderer(opt, renderer).await
            }
        }
    }
}
