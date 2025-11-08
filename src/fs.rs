use std::collections::LinkedList;

use tokio::sync::mpsc::{self, Sender};

use crate::{ArcPath, error::FatalActorError, error::FsError};

mod core;
mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

/// Actor name for error reporting.
pub const ACTOR_NAME: &'static str = "Fs";

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
    pub async fn read_file(&self, path: ArcPath) -> Result<tokio::fs::File, FsError> {
        let path_str = path.to_string_lossy().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::ReadFile { tx, path })
            .await
            .map_err(|_e| {
                FsError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::fs::ACTOR_NAME,
                    operation: format!("read file: {}", path_str),
                })
            })?;
        rx.await.map_err(|e| {
            FsError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::fs::ACTOR_NAME,
                operation: format!("read file: {}", path_str),
                source: e,
            })
        })?
    }

    /// Opens a file for writing (truncates content, creates if needed).
    pub async fn write_file(&self, path: ArcPath) -> Result<tokio::fs::File, FsError> {
        let path_str = path.to_string_lossy().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::WriteFile { tx, path })
            .await
            .map_err(|_e| {
                FsError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::fs::ACTOR_NAME,
                    operation: format!("write file: {}", path_str),
                })
            })?;
        rx.await.map_err(|e| {
            FsError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::fs::ACTOR_NAME,
                operation: format!("write file: {}", path_str),
                source: e,
            })
        })?
    }

    /// Opens a file for appending (creates if needed).
    pub async fn append_file(&self, path: ArcPath) -> Result<tokio::fs::File, FsError> {
        let path_str = path.to_string_lossy().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::AppendFile { tx, path })
            .await
            .map_err(|_e| {
                FsError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::fs::ACTOR_NAME,
                    operation: format!("append file: {}", path_str),
                })
            })?;
        rx.await.map_err(|e| {
            FsError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::fs::ACTOR_NAME,
                operation: format!("append file: {}", path_str),
                source: e,
            })
        })?
    }

    /// Removes a file from the filesystem
    pub async fn remove_file(&self, path: ArcPath) -> Result<(), FsError> {
        let path_str = path.to_string_lossy().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::RemoveFile { tx, path })
            .await
            .map_err(|_e| {
                FsError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::fs::ACTOR_NAME,
                    operation: format!("remove file: {}", path_str),
                })
            })?;
        rx.await.map_err(|e| {
            FsError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::fs::ACTOR_NAME,
                operation: format!("remove file: {}", path_str),
                source: e,
            })
        })?
    }

    /// Reads a directory
    pub async fn read_dir(&self, path: ArcPath) -> Result<LinkedList<ArcPath>, FsError> {
        let path_str = path.to_string_lossy().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::ReadDir { tx, path })
            .await
            .map_err(|_e| {
                FsError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::fs::ACTOR_NAME,
                    operation: format!("read directory: {}", path_str),
                })
            })?;
        rx.await.map_err(|e| {
            FsError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::fs::ACTOR_NAME,
                operation: format!("read directory: {}", path_str),
                source: e,
            })
        })?
    }

    /// Creates a directory if it doesn't exist
    pub async fn mkdir(&self, path: ArcPath) -> Result<(), FsError> {
        let path_str = path.to_string_lossy().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::MkDir { tx, path })
            .await
            .map_err(|_e| {
                FsError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::fs::ACTOR_NAME,
                    operation: format!("create directory: {}", path_str),
                })
            })?;
        rx.await.map_err(|e| {
            FsError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::fs::ACTOR_NAME,
                operation: format!("create directory: {}", path_str),
                source: e,
            })
        })?
    }

    /// Removes a directory
    pub async fn rmdir(&self, path: ArcPath) -> Result<(), FsError> {
        let path_str = path.to_string_lossy().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::RmDir { tx, path })
            .await
            .map_err(|_e| {
                FsError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::fs::ACTOR_NAME,
                    operation: format!("remove directory: {}", path_str),
                })
            })?;
        rx.await.map_err(|e| {
            FsError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::fs::ACTOR_NAME,
                operation: format!("remove directory: {}", path_str),
                source: e,
            })
        })?
    }
}
