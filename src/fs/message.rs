use std::collections::LinkedList;

use tokio::sync::oneshot;

use crate::{ArcFile, ArcPath};

/// Messages that can be sent to a [`Fs`] actor.
///
/// This enum defines the different types of filesystem operations that can be performed
/// through the actor system.
#[derive(Debug)]
pub enum Message {
    /// Opens a file and returns its handle
    OpenFile {
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<Result<ArcFile, tokio::io::Error>>,
        /// The path of the file to open
        path: ArcPath,
    },
    /// Removes a file handle from the cache
    CloseFile {
        /// The path of the file to remove from cache
        path: ArcPath,
    },
    /// Removes a file from the filesystem
    RemoveFile {
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<Result<(), tokio::io::Error>>,
        /// The path of the file to remove
        path: ArcPath,
    },
    /// Reads the contents of a directory
    ReadDir {
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<Result<LinkedList<ArcPath>, std::io::Error>>,
        /// The path of the directory to read
        path: ArcPath,
    },
    /// Creates a directory and its parents
    MkDir {
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<Result<(), std::io::Error>>,
        /// The path of the directory to create
        path: ArcPath,
    },
    /// Removes a directory and its contents
    RmDir {
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<Result<(), std::io::Error>>,
        /// The path of the directory to remove
        path: ArcPath,
    },
}
