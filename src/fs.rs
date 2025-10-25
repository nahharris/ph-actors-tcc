use std::{collections::LinkedList, io};

use anyhow::Context;
use tokio::sync::mpsc::{self, Sender};

use crate::ArcPath;

mod core;
mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

/// The filesystem actor that provides a thread-safe interface for filesystem operations.
///
/// This struct provides a unified interface for filesystem operations
/// using message passing to a background actor.
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
/// copies the channel sender.
#[derive(Debug, Clone)]
pub struct Fs {
    tx: Sender<message::Message>,
}

impl Fs {
    /// Creates a new filesystem instance and spawns its actor.
    ///
    /// # Returns
    /// A new filesystem instance with a spawned actor.
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let _ = tokio::spawn(async move {
            core::Core::new().init(rx).await;
        });
        Self { tx }
    }

    /// Opens a file for reading only (does not create if it doesn't exist).
    pub async fn read_file(&self, path: ArcPath) -> Result<tokio::fs::File, io::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::ReadFile { tx, path })
            .await
            .context("Opening file for reading with Fs")
            .expect("fs actor died");
        rx.await
            .context("Awaiting response for file read with Fs")
            .expect("fs actor died")
    }

    /// Opens a file for writing (truncates content, creates if needed).
    pub async fn write_file(&self, path: ArcPath) -> Result<tokio::fs::File, io::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::WriteFile { tx, path })
            .await
            .context("Opening file for writing with Fs")
            .expect("fs actor died");
        rx.await
            .context("Awaiting response for file write with Fs")
            .expect("fs actor died")
    }

    /// Opens a file for appending (creates if needed).
    pub async fn append_file(&self, path: ArcPath) -> Result<tokio::fs::File, io::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::AppendFile { tx, path })
            .await
            .context("Opening file for appending with Fs")
            .expect("fs actor died");
        rx.await
            .context("Awaiting response for file append with Fs")
            .expect("fs actor died")
    }

    /// Removes a file from the filesystem
    pub async fn remove_file(&self, path: ArcPath) -> Result<(), io::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::RemoveFile { tx, path })
            .await
            .context("Removing file with Fs")
            .expect("fs actor died");
        rx.await
            .context("Awaiting response for file removal with Fs")
            .expect("fs actor died")
    }

    /// Reads a directory
    pub async fn read_dir(&self, path: ArcPath) -> Result<LinkedList<ArcPath>, io::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::ReadDir { tx, path })
            .await
            .context("Reading directory with Fs")
            .expect("fs actor died");
        rx.await
            .context("Awaiting response for directory read with Fs")
            .expect("fs actor died")
    }

    /// Creates a directory if it doesn't exist
    pub async fn mkdir(&self, path: ArcPath) -> Result<(), io::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::MkDir { tx, path })
            .await
            .context("Creating directory with Fs")
            .expect("fs actor died");
        rx.await
            .context("Awaiting response for directory creation with Fs")
            .expect("fs actor died")
    }

    /// Removes a directory
    pub async fn rmdir(&self, path: ArcPath) -> Result<(), io::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::RmDir { tx, path })
            .await
            .context("Removing directory with Fs")
            .expect("fs actor died");
        rx.await
            .context("Awaiting response for directory removal with Fs")
            .expect("fs actor died")
    }
}
