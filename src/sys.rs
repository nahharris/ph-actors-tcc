use std::{collections::HashMap, env::VarError, ffi::OsString, fmt::Display, io, sync::Arc};

use anyhow::Context;
use tokio::{
    fs::OpenOptions,
    sync::{Mutex, RwLock, mpsc::Sender},
    task::JoinHandle,
};

use crate::{ArcFile, ArcOsString, ArcPathBuf, ArcString};

/// The core of the Sys actor, responsible for handling OS operations, mainly
/// FS and env interactions
#[derive(Debug, Default)]
pub struct SysCore {
    /// The cache of open files
    files: HashMap<ArcPathBuf, ArcFile>,
}

impl SysCore {
    /// Creates a new Sys core
    pub fn new() -> Self {
        Default::default()
    }

    /// Transforms an instance of [`SysCore`] into an actor ready to receive
    /// messages
    pub fn spawn(mut self) -> (Sys, JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(crate::BUFFER_SIZE);
        let handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                use Message::*;
                match msg {
                    SetEnv { key, value } => unsafe {
                        std::env::set_var(key.as_os_str(), value.as_os_str());
                    },
                    UnsetEnv { key } => unsafe {
                        std::env::remove_var(key.as_os_str());
                    },
                    GetEnv { tx, key } => {
                        let _ = tx.send(std::env::var(key.as_os_str()).map(Arc::new));
                    }
                    OpenFile { tx, path } => {
                        let f = match self.files.get(path.as_ref()) {
                            Some(f) => Arc::clone(f),
                            None => match OpenOptions::new().read(true).open(path.as_ref()).await {
                                Ok(f) => {
                                    let f = Arc::new(RwLock::new(f));
                                    self.files.insert(path, Arc::clone(&f));
                                    f
                                }
                                Err(e) => {
                                    let _ = tx.send(Err(e));
                                    continue;
                                }
                            },
                        };
                        let _ = tx.send(Ok(f));
                    }
                    CloseFile { path } => {
                        self.files.remove(path.as_ref());
                    }
                }
            }
        });

        (Sys::Actual(tx), handle)
    }
}

/// Messages that can be sent to a [`SysCore`]
#[derive(Debug)]
pub enum Message {
    SetEnv {
        key: ArcOsString,
        value: OsString,
    },
    UnsetEnv {
        key: ArcOsString,
    },
    GetEnv {
        tx: tokio::sync::oneshot::Sender<Result<ArcString, VarError>>,
        key: ArcOsString,
    },
    OpenFile {
        tx: tokio::sync::oneshot::Sender<Result<ArcFile, tokio::io::Error>>,
        path: ArcPathBuf,
    },
    CloseFile {
        path: ArcPathBuf,
    },
}

#[derive(Debug, Clone, Default)]
pub struct SysMock {
    env: HashMap<ArcOsString, OsString>,
    files: HashMap<ArcPathBuf, ArcFile>,
}

#[derive(Debug, Clone)]
pub enum Sys {
    Actual(Sender<Message>),
    Mock(Arc<Mutex<SysMock>>),
}

impl From<SysCore> for Sys {
    fn from(core: SysCore) -> Self {
        let (sys, _) = core.spawn();
        sys
    }
}

use Sys::*;
#[allow(dead_code)]
impl Sys {
    pub fn mock() -> Self {
        Mock(Arc::new(Mutex::new(SysMock::default())))
    }

    pub async fn set_env<V>(&self, key: ArcOsString, value: V)
    where
        V: Display,
    {
        let value = format!("{}", value).into();
        match self {
            Actual(sender) => sender
                .send(Message::SetEnv { key, value })
                .await
                .context("Setting environment variable with Sys")
                .expect("sys actor died"),

            Mock(lock) => {
                let mut lock = lock.lock().await;
                lock.env.insert(key, value);
            }
        }
    }

    pub async fn unset_env(&self, key: ArcOsString) {
        match self {
            Actual(sender) => sender
                .send(Message::UnsetEnv { key })
                .await
                .context("Unsetting environment variable with Sys")
                .expect("sys actor died"),
            Mock(lock) => {
                let mut lock = lock.lock().await;
                lock.env.remove(&key);
            }
        }
    }

    pub async fn get_env(&self, key: ArcOsString) -> Result<ArcString, VarError> {
        match self {
            Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::GetEnv { tx, key })
                    .await
                    .context("Getting environment variable with Sys")
                    .expect("sys actor died");
                rx.await
                    .context("Awaiting response for environment variable get with Sys")
                    .expect("sys actor died")
            }
            Mock(lock) => {
                let lock = lock.lock().await;
                lock.env
                    .get(&key)
                    .map(|s| s.to_string_lossy().to_string().into())
                    .ok_or(VarError::NotPresent)
            }
        }
    }

    pub async fn open_file(&self, path: ArcPathBuf) -> Result<ArcFile, io::Error> {
        match self {
            Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::OpenFile { tx, path })
                    .await
                    .context("Opening file with Sys")
                    .expect("sys actor died");
                rx.await
                    .context("Awaiting response for file open with Sys")
                    .expect("sys actor died")
            }
            Mock(lock) => {
                let lock = lock.lock().await;
                lock.files
                    .get(&path)
                    .map(Arc::clone)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))
            }
        }
    }

    pub async fn close_file(&self, path: ArcPathBuf) {
        match self {
            Actual(sender) => sender
                .send(Message::CloseFile { path })
                .await
                .context("Closing file with Sys")
                .expect("sys actor died"),
            Mock(mutex) => {
                let mut lock = mutex.lock().await;
                lock.files.remove(&path);
            }
        }
    }
}
