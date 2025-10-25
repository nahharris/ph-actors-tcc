use crate::ArcPath;

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
                    let res = self.data.path(opt);
                    let _ = tx.send(res);
                }
                GetLogLevel { tx } => {
                    let res = self.data.log_level();
                    let _ = tx.send(res);
                }
                GetUSize { opt, tx } => {
                    let res = self.data.usize(opt);
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
                    let res = self.data.renderer(opt);
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
    async fn handle_load(&mut self) -> anyhow::Result<()> {
        let mut file = self.fs.read_file(self.path.clone()).await?;
        let mut contents = String::new();
        use tokio::io::AsyncReadExt;
        file.read_to_string(&mut contents).await?;
        let data = toml::from_str(&contents)?;
        self.data = data;
        Ok(())
    }

    /// Saves the current configuration to the file.
    ///
    /// # Returns
    /// `Ok(())` if the configuration was saved successfully.
    async fn handle_save(&self) -> anyhow::Result<()> {
        let contents = toml::to_string(&self.data)?;
        let mut file = self.fs.write_file(self.path.clone()).await?;
        use tokio::io::AsyncWriteExt;
        file.write_all(contents.as_bytes()).await?;
        Ok(())
    }
}
