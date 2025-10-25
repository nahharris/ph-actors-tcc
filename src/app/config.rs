pub use data::{PathOpt, Renderer, RendererOpt, USizeOpt};

use anyhow::Context;
use tokio::sync::mpsc::Sender;

use crate::{ArcPath, log::LogLevel};
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
    pub async fn load(&self) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Load { tx })
            .await
            .context("Loading config with Config actor")
            .expect("Config actor is dead");
        rx.await
            .context("Awaiting response for config load with Config actor")
            .expect("Config actor is dead")
    }

    /// Saves the current configuration to the file.
    ///
    pub async fn save(&self) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Save { tx })
            .await
            .context("Saving config with Config actor")
            .expect("Config actor is dead");
        rx.await
            .context("Awaiting response for config save with Config actor")
            .expect("Config actor is dead")
    }

    /// Gets a path-based configuration value.
    ///
    /// # Arguments
    /// * `opt` - The path option to retrieve
    ///
    /// # Returns
    /// The requested path value.
    pub async fn path(&self, opt: PathOpt) -> ArcPath {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::GetPath { opt, tx })
            .await
            .context("Getting path with Config actor")
            .expect("Config actor is dead");
        rx.await
            .context("Awaiting response for path with Config actor")
            .expect("Config actor is dead")
    }

    /// Sets a path-based configuration value.
    ///
    /// # Arguments
    /// * `opt` - The path option to set
    /// * `path` - The new path value
    pub async fn set_path(&self, opt: PathOpt, path: ArcPath) {
        self.tx
            .send(Message::SetPath { opt, path })
            .await
            .context("Setting path with Config actor")
            .expect("Config actor is dead");
    }

    /// Gets the current log level.
    ///
    /// # Returns
    /// The current log level.
    pub async fn log_level(&self) -> LogLevel {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::GetLogLevel { tx })
            .await
            .context("Getting log level with Config actor")
            .expect("Config actor died");
        rx.await
            .context("Awaiting response for log level with Config actor")
            .expect("Config actor died")
    }

    /// Sets the log level.
    ///
    /// # Arguments
    /// * `level` - The new log level value
    pub async fn set_log_level(&self, level: LogLevel) {
        let _ = self.tx.send(Message::SetLogLevel { level }).await;
    }

    /// Gets a numeric configuration value.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to retrieve
    ///
    /// # Returns
    /// The requested numeric value.
    pub async fn usize(&self, opt: USizeOpt) -> usize {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::GetUSize { opt, tx })
            .await
            .context("Getting numeric value with Config actor")
            .expect("Config actor died");
        rx.await
            .context("Awaiting response for numeric value with Config actor")
            .expect("Config actor died")
    }

    /// Sets a numeric configuration value.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to set
    /// * `value` - The new numeric value
    pub async fn set_usize(&self, opt: USizeOpt, value: usize) {
        let _ = self.tx.send(Message::SetUSize { opt, size: value }).await;
    }

    /// Gets a renderer configuration value.
    ///
    /// # Arguments
    /// * `opt` - The renderer option to retrieve
    ///
    /// # Returns
    /// The requested renderer value.
    pub async fn renderer(&self, opt: RendererOpt) -> Renderer {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::GetRenderer { opt, tx })
            .await
            .context("Getting renderer value with Config actor")
            .expect("Config actor died");
        rx.await
            .context("Awaiting response for renderer value with Config actor")
            .expect("Config actor died")
    }

    /// Sets a renderer configuration value.
    ///
    /// # Arguments
    /// * `opt` - The renderer option to set
    /// * `renderer` - The new renderer value
    pub async fn set_renderer(&self, opt: RendererOpt, renderer: Renderer) {
        let _ = self.tx.send(Message::SetRenderer { opt, renderer }).await;
    }
}
