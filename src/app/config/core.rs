use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{ArcPath, env::Env, fs::Fs};

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

    /// Spawns the configuration actor and returns a handle to it.
    ///
    /// # Returns
    /// A tuple containing the configuration actor and its task handle.
    pub fn spawn(mut self) -> (super::Config, tokio::task::JoinHandle<()>) {
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
        (super::Config::Actual(tx), handle)
    }

    /// Loads the configuration from the file.
    ///
    /// # Returns
    /// `Ok(())` if the configuration was loaded successfully.
    async fn load(&mut self) -> anyhow::Result<()> {
        let mut file = self.fs.open_file(self.path.clone()).await?;
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
    async fn save(&self) -> anyhow::Result<()> {
        let contents = toml::to_string(&self.data)?;
        let mut file = self.fs.open_file(self.path.clone()).await?;
        use tokio::io::AsyncWriteExt;
        file.write_all(contents.as_bytes()).await?;
        Ok(())
    }
}
