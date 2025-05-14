use std::{collections::{HashMap, LinkedList}, io, sync::Arc};

use tokio::{fs::OpenOptions, sync::mpsc};

use crate::{ArcFile, ArcPath};

use super::message::Message;

/// The core of the Fs actor, responsible for handling filesystem operations.
///
/// This struct provides thread-safe access to filesystem operations through an actor pattern.
/// It wraps tokio's filesystem functions and provides a safe interface for concurrent access.
/// Files are cached to avoid repeated opening of the same file.
#[derive(Debug, Default)]
pub struct Core {
    /// The cache of open files, mapping paths to their file handles
    files: HashMap<ArcPath, ArcFile>,
}

impl Core {
    /// Creates a new Fs core instance.
    ///
    /// # Returns
    /// A new instance of `Core` with an empty file cache.
    pub fn new() -> Self {
        Default::default()
    }

    /// Transforms an instance of [`Core`] into an actor ready to receive messages.
    ///
    /// This method spawns a new task that will handle filesystem operations
    /// asynchronously through a message channel.
    ///
    /// # Returns
    /// A tuple containing:
    /// - A [`Fs`] instance that can be used to send messages to the actor
    /// - A join handle for the spawned task
    ///
    /// # Panics
    /// This function will panic if the underlying task fails to spawn.
    pub fn spawn(mut self) -> (super::Fs, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel(crate::BUFFER_SIZE);
        let handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                use Message::*;
                match msg {
                    OpenFile { tx, path } => self.open_file(tx, path).await,
                    CloseFile { path } => self.close_file(path),
                    RemoveFile { tx, path } => self.remove_file(tx, path).await,
                    ReadDir { tx, path } => self.read_dir(tx, path).await,
                    MkDir { tx, path } => self.mkdir(tx, path).await,
                    RmDir { tx, path } => self.rmdir(tx, path).await,
                }
            }
        });

        (super::Fs::Actual(tx), handle)
    }

    /// Opens a file or returns a cached file handle if one exists.
    ///
    /// The file is opened with write and create permissions. If the file is already
    /// open, a reference to the existing handle is returned.
    ///
    /// # Arguments
    /// * `tx` - A oneshot channel sender to receive the result
    /// * `path` - The path to the file to open
    ///
    /// # Errors
    /// The function will return an error if the file cannot be opened or if there
    /// are any issues with the channel communication.
    async fn open_file(
        &mut self,
        tx: tokio::sync::oneshot::Sender<Result<ArcFile, tokio::io::Error>>,
        path: ArcPath,
    ) {
        let f = match self.files.get(&path) {
            Some(f) => f.clone(),
            None => match OpenOptions::new()
                .write(true)
                .create(true)
                .open(&path)
                .await
            {
                Ok(f) => {
                    let f = ArcFile::from(f);
                    self.files.insert(path, f.clone());
                    f
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                    return;
                }
            },
        };
        let _ = tx.send(Ok(f));
    }

    /// Removes a file handle from the cache.
    ///
    /// This doesn't close the file immediately - all `Arc` references must be dropped
    /// before the file is actually closed.
    ///
    /// # Arguments
    /// * `path` - The path of the file to remove from the cache
    fn close_file(&mut self, path: ArcPath) {
        self.files.remove(&path);
    }

    /// Removes a file from the filesystem.
    ///
    /// # Arguments
    /// * `tx` - A oneshot channel sender to receive the result
    /// * `path` - The path of the file to remove
    ///
    /// # Errors
    /// The function will return an error if the file cannot be removed or if there
    /// are any issues with the channel communication.
    async fn remove_file(
        &self,
        tx: tokio::sync::oneshot::Sender<Result<(), tokio::io::Error>>,
        path: ArcPath,
    ) {
        let res = tokio::fs::remove_file(&path).await;
        let _ = tx.send(res);
    }

    /// Reads the contents of a directory.
    ///
    /// Returns a list of paths to all entries in the directory, each wrapped in an `Arc`.
    ///
    /// # Arguments
    /// * `tx` - A oneshot channel sender to receive the result
    /// * `path` - The path of the directory to read
    ///
    /// # Errors
    /// The function will return an error if the directory cannot be read or if there
    /// are any issues with the channel communication.
    async fn read_dir(
        &self,
        tx: tokio::sync::oneshot::Sender<Result<LinkedList<ArcPath>, io::Error>>,
        path: ArcPath,
    ) {
        match tokio::fs::read_dir(&path).await {
            Ok(mut rd) => {
                let mut entries = LinkedList::new();
                let res = loop {
                    match rd.next_entry().await {
                        Ok(Some(entry)) => entries.push_back(ArcPath::from(&entry.path())),
                        Ok(None) => break Ok(entries),
                        Err(e) => break Err(e),
                    }
                };

                let _ = tx.send(res);
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        }
    }

    /// Creates a directory and all its parent directories if they don't exist.
    ///
    /// # Arguments
    /// * `tx` - A oneshot channel sender to receive the result
    /// * `path` - The path of the directory to create
    ///
    /// # Errors
    /// The function will return an error if the directory cannot be created or if there
    /// are any issues with the channel communication.
    async fn mkdir(&self, tx: tokio::sync::oneshot::Sender<Result<(), io::Error>>, path: ArcPath) {
        let res = tokio::fs::create_dir_all(&path).await;
        let _ = tx.send(res);
    }

    /// Removes a directory and all its contents recursively.
    ///
    /// # Arguments
    /// * `tx` - A oneshot channel sender to receive the result
    /// * `path` - The path of the directory to remove
    ///
    /// # Errors
    /// The function will return an error if the directory cannot be removed or if there
    /// are any issues with the channel communication.
    async fn rmdir(&self, tx: tokio::sync::oneshot::Sender<Result<(), io::Error>>, path: ArcPath) {
        let res = tokio::fs::remove_dir_all(&path).await;
        let _ = tx.send(res);
    }
} 