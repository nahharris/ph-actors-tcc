use std::{
    collections::{HashMap, LinkedList},
    io,
    sync::Arc,
};

use anyhow::Context;
use tokio::sync::mpsc::Sender;

use crate::{ArcFile, ArcPath};

mod core;
mod message;
#[cfg(test)]
mod tests;

/// The filesystem actor that provides a thread-safe interface for filesystem operations.
///
/// This enum represents either a real filesystem actor or a mock implementation
/// for testing purposes. It provides a unified interface for filesystem operations
/// regardless of the underlying implementation.
///
/// # Examples
/// ```
/// let fs = Fs::spawn();
/// let path = Arc::from(Path::new("example.txt"));
/// let file = fs.open_file(path).await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender or mock reference.
#[derive(Debug, Clone)]
pub enum Fs {
    /// A real filesystem actor that interacts with the system
    Actual(Sender<message::Message>),
    /// A mock implementation for testing
    Mock(Arc<tokio::sync::Mutex<HashMap<ArcPath, ArcFile>>>),
}

impl Fs {
    /// Creates a new filesystem instance and spawns its actor.
    ///
    /// # Returns
    /// A new filesystem instance with a spawned actor.
    pub fn spawn() -> Self {
        let (fs, _) = core::Core::new().spawn();
        fs
    }

    /// Creates a new mock filesystem instance for testing.
    ///
    /// # Arguments
    /// * `files` - Optional initial file cache. If None, an empty cache will be used.
    ///
    /// # Returns
    /// A new mock filesystem instance that stores files in memory.
    pub fn mock(files: HashMap<ArcPath, ArcFile>) -> Self {
        Self::Mock(Arc::new(tokio::sync::Mutex::new(files)))
    }

    /// Opens a file
    ///
    /// File opening is cached, so opening a file multiple times will return the
    /// same file descriptor using `Arc` to avoid cloning.
    ///
    /// # Errors
    ///
    /// If the file cannot be opened, an error is returned, also if a mock is
    /// being using and the file was not previously opened and passed to [`Fs::mock`]
    pub async fn open_file(&self, path: ArcPath) -> Result<ArcFile, io::Error> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(message::Message::OpenFile { tx, path })
                    .await
                    .context("Opening file with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for file open with Fs")
                    .expect("fs actor died")
            }
            Self::Mock(lock) => {
                let lock = lock.lock().await;
                lock.get(&path)
                    .map(ArcFile::clone)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))
            }
        }
    }

    /// Removes a file from the cache, notice that this won't close the file
    /// immediately. Every `Arc` must be dropped before the file is actually
    /// closed.
    pub async fn close_file(&self, path: ArcPath) {
        match self {
            Self::Actual(sender) => sender
                .send(message::Message::CloseFile { path })
                .await
                .context("Closing file with Fs")
                .expect("fs actor died"),
            Self::Mock(mutex) => {
                let mut lock = mutex.lock().await;
                lock.remove(&path);
            }
        }
    }

    /// Removes a file from the filesystem
    pub async fn remove_file(&self, path: ArcPath) -> Result<(), io::Error> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(message::Message::RemoveFile { tx, path })
                    .await
                    .context("Removing file with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for file removal with Fs")
                    .expect("fs actor died")
            }
            Self::Mock(lock) => {
                let mut lock = lock.lock().await;
                lock.remove(&path)
                    .map(|_| ())
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))
            }
        }
    }

    /// Reads a directory
    pub async fn read_dir(&self, path: ArcPath) -> Result<LinkedList<ArcPath>, io::Error> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(message::Message::ReadDir { tx, path })
                    .await
                    .context("Reading directory with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for directory read with Fs")
                    .expect("fs actor died")
            }
            Self::Mock(_) => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "read_dir not supported in mock",
            )),
        }
    }

    /// Creates a directory if it doesn't exist
    pub async fn mkdir(&self, path: ArcPath) -> Result<(), io::Error> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(message::Message::MkDir { tx, path })
                    .await
                    .context("Creating directory with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for directory creation with Fs")
                    .expect("fs actor died")
            }
            Self::Mock(_) => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "mkdir not supported in mock",
            )),
        }
    }

    /// Removes a directory
    pub async fn rmdir(&self, path: ArcPath) -> Result<(), io::Error> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(message::Message::RmDir { tx, path })
                    .await
                    .context("Removing directory with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for directory removal with Fs")
                    .expect("fs actor died")
            }
            Self::Mock(_) => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "rmdir not supported in mock",
            )),
        }
    }
}
