use std::{
    collections::{HashMap, LinkedList},
    env::VarError,
    ffi::OsString,
    fmt::Display,
    io,
    sync::Arc,
};

use anyhow::Context;
use tokio::{
    fs::OpenOptions,
    sync::{Mutex, RwLock, mpsc::Sender},
    task::JoinHandle,
};

use crate::{ArcFile, ArcOsString, ArcPathBuf, ArcString};

/// The core of the Sys actor, responsible for handling OS operations, mainly
/// FS and env interactions
///
/// Interations with environment variables are done through the standard library,
/// but FS operations are done through tokio and are async. We also cache open
/// files to avoid opening the same file multiple times. Open files are shared
/// using `Arc` to avoid cloning the file descriptor.
///
/// This struct is not meant to be used directly, but rather through [`Sys`]
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
                    ReadDir { tx, path } => match tokio::fs::read_dir(path.as_ref()).await {
                        Ok(mut rd) => {
                            let mut entries = LinkedList::new();
                            let res = loop {
                                match rd.next_entry().await {
                                    Ok(Some(entry)) => entries.push_back(Arc::new(entry.path())),
                                    Ok(None) => break Ok(entries),
                                    Err(e) => break Err(e),
                                }
                            };

                            let _ = tx.send(res);
                        }
                        Err(e) => {
                            let _ = tx.send(Err(e));
                            continue;
                        }
                    },
                    RemoveFile { tx, path } => {
                        let res = tokio::fs::remove_file(path.as_ref()).await;
                        let _ = tx.send(res);
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
    RemoveFile {
        tx: tokio::sync::oneshot::Sender<Result<(), tokio::io::Error>>,
        path: ArcPathBuf,
    },
    ReadDir {
        tx: tokio::sync::oneshot::Sender<Result<LinkedList<ArcPathBuf>, io::Error>>,
        path: ArcPathBuf,
    },
}

/// A mock implementation of the Sys actor, used for testing, this implementation
/// won't actually interact with the OS, but rather store the state in memory.
///
/// Environment variables are stored in a hashmap, and files are stored in
/// another one as well. Files should be opened before being used, since it won't
/// attempt to interact with the filesystem.
#[derive(Debug, Clone, Default)]
pub struct SysMock {
    env: HashMap<ArcOsString, OsString>,
    files: HashMap<ArcPathBuf, ArcFile>,
    dirs: HashMap<ArcPathBuf, LinkedList<ArcPathBuf>>,
}

/// The sys actor is responsible for handling OS operations, mainly FS and env
/// This is a transmitter to a running actor, and can be used to send messages
///
/// Cloning this actor is cheap, since it's just a transmitter
///
/// Instantiate a new sys actor with [`SysCore::spawn`]
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
    /// Creates a new mock instance of the Sys actor for testing
    ///
    /// # Important
    /// Mocks cannot open new files, so you need to open your files beforehands
    /// and pass them to the mock
    pub fn mock(files: HashMap<ArcPathBuf, ArcFile>) -> Self {
        let mock = SysMock {
            files,
            ..SysMock::default()
        };

        Mock(Arc::new(Mutex::new(mock)))
    }

    /// Sets an environment variable
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

    /// Unsets an environment variable
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

    /// Gets an environment variable
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

    /// Opens a file
    ///
    /// File opening is cached, so opening a file multiple times will return the
    /// same file descriptor using `Arc` to avoid cloning.
    ///
    /// # Errors
    ///
    /// If the file cannot be opened, an error is returned, also if a mock is
    /// being using and the file was not previously opened and passed to [`Sys::mock`]
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

    /// Removes a file from the cache, notice that this won't close the file
    /// immediately. Every `Arc` must be dropped before the file is actually
    /// closed.
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

    /// Removes a file from the filesystem
    pub async fn remove_file(&self, path: ArcPathBuf) -> Result<(), io::Error> {
        match self {
            Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::RemoveFile { tx, path })
                    .await
                    .context("Removing file with Sys")
                    .expect("sys actor died");
                rx.await
                    .context("Awaiting response for file removal with Sys")
                    .expect("sys actor died")
            }
            Mock(lock) => {
                let mut lock = lock.lock().await;
                lock.files
                    .remove(&path)
                    .map(|_| ())
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))
            }
        }
    }

    pub async fn read_dir(&self, path: ArcPathBuf) -> Result<LinkedList<ArcPathBuf>, io::Error> {
        match self {
            Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::ReadDir { tx, path })
                    .await
                    .context("Reading directory with Sys")
                    .expect("sys actor died");
                rx.await
                    .context("Awaiting response for directory read with Sys")
                    .expect("sys actor died")
            }
            Mock(lock) => {
                let lock = lock.lock().await;
                let entries = lock.dirs.get(path.as_ref()).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "directory not found")
                })?;

                Ok(entries.clone())
            }
        }
    }
}
