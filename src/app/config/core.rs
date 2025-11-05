use crate::{error::ConfigError, ArcPath};

#[cfg(not(test))]
use crate::{env::Env, fs::Fs};
#[cfg(test)]
use crate::{env::mock::MockEnv as Env, fs::mock::MockFs as Fs};

use super::{data::Data, message::Message};

/// The core configuration actor that handles file I/O and data management.
///
/// This struct is responsible for:
/// - Reading and writing configuration data to/from files
/// - Managing the configuration data in memory
/// - Providing a thread-safe interface through message passing
///
/// # Thread Safety
/// This type is designed to be safely shared between threads through its message-based interface.
pub struct Core {
    /// The environment actor for system operations
    env: Env,
    /// The filesystem actor for file operations
    fs: Fs,
    /// The path to the configuration file
    path: ArcPath,
    /// The current configuration data
    data: Data,
}

impl Core {
    /// Creates a new configuration core instance.
    ///
    /// # Arguments
    /// * `env` - The environment actor for system operations
    /// * `fs` - The filesystem actor for file operations
    /// * `path` - The path to the configuration file
    ///
    /// # Returns
    /// A new configuration core instance.
    pub fn new(env: Env, fs: Fs, path: ArcPath) -> Self {
        Self {
            env,
            fs,
            path,
            data: Data::default(),
        }
    }

    /// Initializes the configuration actor message receiver.
    ///
    /// This method processes messages from the receiver in a loop, handling each message
    /// using pattern matching.
    ///
    /// # Arguments
    /// * `rx` - A receiver for messages to process
    pub async fn init(mut self, mut rx: tokio::sync::mpsc::Receiver<Message>) {
        while let Some(msg) = rx.recv().await {
            use Message::*;
            match msg {
                Load { tx } => {
                    let res = self.handle_load().await;
                    let _ = tx.send(res);
                }
                Save { tx } => {
                    let res = self.handle_save().await;
                    let _ = tx.send(res);
                }
                GetPath { opt, tx } => {
                    let res = Ok(self.data.path(opt));
                    let _ = tx.send(res);
                }
                GetLogLevel { tx } => {
                    let res = Ok(self.data.log_level());
                    let _ = tx.send(res);
                }
                GetUSize { opt, tx } => {
                    let res = Ok(self.data.usize(opt));
                    let _ = tx.send(res);
                }
                SetPath { opt, path } => {
                    self.data.set_path(opt, path);
                }
                SetLogLevel { level } => {
                    self.data.set_log_level(level);
                }
                SetUSize { opt, size } => {
                    self.data.set_usize(opt, size);
                }
                GetRenderer { opt, tx } => {
                    let res = Ok(self.data.renderer(opt));
                    let _ = tx.send(res);
                }
                SetRenderer { opt, renderer } => {
                    self.data.set_renderer(opt, renderer);
                }
            }
        }
    }

    /// Loads the configuration from the file.
    ///
    /// # Returns
    /// `Ok(())` if the configuration was loaded successfully.
    async fn handle_load(&mut self) -> Result<(), ConfigError> {
        let path_str = self.path.to_string_lossy().to_string();
        let mut file = self.fs.read_file(self.path.clone()).await.map_err(|e| match e {
            crate::error::FsError::Fatal(fatal) => ConfigError::Fatal(fatal),
            crate::error::FsError::OperationFailed { path, operation, source, .. } => {
                ConfigError::FileOperationFailed {
                    path: path.unwrap_or_else(|| path_str.clone()),
                    operation,
                    source,
                }
            }
        })?;
        let mut contents = String::new();
        use tokio::io::AsyncReadExt;
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| {
                ConfigError::FileOperationFailed {
                    path: path_str.clone(),
                    operation: "read config file".to_string(),
                    source: e,
                }
            })?;
        
        // Convert FsError to ConfigError::FileOperationFailed
        let data = toml::from_str(&contents).map_err(|e| {
            ConfigError::ParseFailed {
                key: None,
                source: e,
                message: format!("Failed to parse TOML configuration file: {}", path_str),
            }
        })?;
        self.data = data;
        Ok(())
    }

    /// Saves the current configuration to the file.
    ///
    /// # Returns
    /// `Ok(())` if the configuration was saved successfully.
    async fn handle_save(&self) -> Result<(), ConfigError> {
        let path_str = self.path.to_string_lossy().to_string();
        let contents = toml::to_string_pretty(&self.data).map_err(|e| {
            // Serialization error - create a parse error with a message
            let error_msg = e.to_string();
            ConfigError::InvalidValue {
                key: None,
                message: format!("Failed to serialize configuration to TOML: {}. Error: {}", path_str, error_msg),
            }
        })?;
        let mut file = self.fs.write_file(self.path.clone()).await.map_err(|e| match e {
            crate::error::FsError::Fatal(fatal) => ConfigError::Fatal(fatal),
            crate::error::FsError::OperationFailed { path, operation, source, .. } => {
                ConfigError::FileOperationFailed {
                    path: path.unwrap_or_else(|| path_str.clone()),
                    operation,
                    source,
                }
            }
        })?;
        use tokio::io::AsyncWriteExt;
        file.write_all(contents.as_bytes())
            .await
            .map_err(|e| {
                ConfigError::FileOperationFailed {
                    path: path_str,
                    operation: "write config file".to_string(),
                    source: e,
                }
            })?;
        Ok(())
    }
}
