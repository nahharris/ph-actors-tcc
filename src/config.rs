use std::{path::Path, str::FromStr, sync::Arc};

use anyhow::Context;
use serde::{Deserialize, Serialize, ser::SerializeStruct};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{ArcPath, env::Env, fs::Fs, log::LogLevel};

pub struct ConfigCore {
    data: Data,
    path: ArcPath,
    env: Env,
    fs: Fs,
}

impl ConfigCore {
    /// Creates a new instance of [`ConfigCore`]
    ///
    /// # Arguments
    /// * `env` - The environment actor
    /// * `fs` - The filesystem actor
    /// * `path` - The path to the configuration file
    pub fn new(env: Env, fs: Fs, path: ArcPath) -> Self {
        Self {
            data: Data::default(),
            path,
            env,
            fs,
        }
    }

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

    /// Loads the configuration from the file
    ///
    /// If it fails, it means that either: the config file does not exist, or the file is not a valid TOML file.
    async fn load(&mut self) -> anyhow::Result<()> {
        let config = self.fs.open_file(Arc::clone(&self.path)).await?;
        let mut buf = String::new();

        config.write().await.read_to_string(&mut buf).await?;

        let data: Data = toml::de::from_str(&buf)?;
        self.data = data;
        Ok(())
    }

    /// Saves the configuration to the file
    async fn save(&self) -> anyhow::Result<()> {
        let config = self.fs.open_file(Arc::clone(&self.path)).await?;
        let buf = toml::ser::to_string_pretty(&self.data)?;

        config.write().await.write_all(buf.as_bytes()).await?;
        config.write().await.flush().await?;
        config.write().await.sync_all().await?;

        Ok(())
    }
}

/// The data structure for the configuration
#[derive(Debug)]
struct Data {
    log_dir: ArcPath,
    log_level: LogLevel,
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

/// The options for the configuration
#[derive(Debug, Clone, Copy)]
pub enum PathOpts {
    LogDir,
}

/// The options for the configuration
#[derive(Debug, Clone, Copy)]
pub enum USizeOpts {
    MaxAge,
}

/// The message type for the configuration actor
#[derive(Debug)]
pub enum Message {
    /// Load the configuration
    Load(tokio::sync::oneshot::Sender<anyhow::Result<()>>),
    /// Save the configuration
    Save(tokio::sync::oneshot::Sender<anyhow::Result<()>>),
    GetPath(PathOpts, tokio::sync::oneshot::Sender<ArcPath>),
    GetLogLevel(tokio::sync::oneshot::Sender<LogLevel>),
    GetUSize(USizeOpts, tokio::sync::oneshot::Sender<usize>),
    SetPath(PathOpts, ArcPath),
    SetLogLevel(LogLevel),
    SetUSize(USizeOpts, usize),
}

/// The configuration actor
#[derive(Debug, Clone)]
pub enum Config {
    /// The actual configuration actor
    Actual(tokio::sync::mpsc::Sender<Message>),
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
