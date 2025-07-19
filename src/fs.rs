use std::{collections::LinkedList, io, sync::Arc};

use anyhow::Context;
use tokio::sync::mpsc::Sender;

use crate::ArcPath;
use std::path::PathBuf;
use tempfile::TempDir;

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
/// ```ignore
/// let fs = Fs::spawn();
/// let path = Arc::from(Path::new("example.txt"));
/// let file = fs.read_file(path).await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender or mock reference.
#[derive(Debug, Clone)]
pub enum Fs {
    /// A real filesystem actor that interacts with the system
    Actual(Sender<message::Message>),
    /// A mock implementation for testing (uses a temp dir)
    Mock(Arc<tokio::sync::Mutex<TempDir>>),
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
    /// # Returns
    /// A new mock filesystem instance that returns errors for all operations.
    pub fn mock() -> Self {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir for Fs mock");
        Self::Mock(Arc::new(tokio::sync::Mutex::new(temp_dir)))
    }

    async fn mock_path(&self, path: &ArcPath) -> PathBuf {
        match self {
            Fs::Mock(temp_dir) => {
                let temp_dir = temp_dir.lock().await;
                temp_dir.path().join(path.as_ref() as &std::path::Path)
            }
            _ => unreachable!(),
        }
    }

    /// Opens a file for reading only (does not create if it doesn't exist).
    pub async fn read_file(&self, path: ArcPath) -> Result<tokio::fs::File, io::Error> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(message::Message::ReadFile { tx, path })
                    .await
                    .context("Opening file for reading with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for file read with Fs")
                    .expect("fs actor died")
            }
            Self::Mock(_) => {
                let real_path = self.mock_path(&path).await;
                tokio::fs::OpenOptions::new()
                    .read(true)
                    .open(real_path)
                    .await
            }
        }
    }

    /// Opens a file for writing (truncates content, creates if needed).
    pub async fn write_file(&self, path: ArcPath) -> Result<tokio::fs::File, io::Error> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(message::Message::WriteFile { tx, path })
                    .await
                    .context("Opening file for writing with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for file write with Fs")
                    .expect("fs actor died")
            }
            Self::Mock(_) => {
                let real_path = self.mock_path(&path).await;
                tokio::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(real_path)
                    .await
            }
        }
    }

    /// Opens a file for appending (creates if needed).
    pub async fn append_file(&self, path: ArcPath) -> Result<tokio::fs::File, io::Error> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(message::Message::AppendFile { tx, path })
                    .await
                    .context("Opening file for appending with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for file append with Fs")
                    .expect("fs actor died")
            }
            Self::Mock(_) => {
                let real_path = self.mock_path(&path).await;
                tokio::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(real_path)
                    .await
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
            Self::Mock(_) => {
                let real_path = self.mock_path(&path).await;
                tokio::fs::remove_file(real_path).await
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
            Self::Mock(_) => {
                let real_path = self.mock_path(&path).await;
                let mut entries = LinkedList::new();
                let mut rd = tokio::fs::read_dir(real_path).await?;
                while let Some(entry) = rd.next_entry().await? {
                    let path = entry.path();
                    entries.push_back(ArcPath::from(&path));
                }
                Ok(entries)
            }
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
            Self::Mock(_) => {
                let real_path = self.mock_path(&path).await;
                tokio::fs::create_dir_all(real_path).await
            }
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
            Self::Mock(_) => {
                let real_path = self.mock_path(&path).await;
                tokio::fs::remove_dir_all(real_path).await
            }
        }
    }
}
