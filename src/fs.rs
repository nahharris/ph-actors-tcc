use std::{
    collections::{HashMap, LinkedList},
    io,
    sync::Arc,
};

use anyhow::Context;
use tokio::{
    fs::OpenOptions,
    sync::{RwLock, mpsc::Sender},
};

use crate::{ArcFile, ArcPath};

/// The core of the Fs actor, responsible for handling filesystem operations
///
/// Filesystem operations are done through tokio and are async. We also cache open
/// files to avoid opening the same file multiple times. Open files are shared
/// using `Arc` to avoid cloning the file descriptor.
///
/// This struct is not meant to be used directly, but rather through [`Fs`]
#[derive(Debug, Default)]
pub struct FsCore {
    /// The cache of open files
    files: HashMap<ArcPath, ArcFile>,
}

impl FsCore {
    /// Creates a new Fs core
    pub fn new() -> Self {
        Default::default()
    }

    /// Transforms an instance of [`FsCore`] into an actor ready to receive messages
    pub fn spawn(mut self) -> (Fs, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(crate::BUFFER_SIZE);
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

        (Fs::Actual(tx), handle)
    }

    /// Opens a file or returns a cached file handle if one exists
    /// The file is opened with write and create permissions
    /// The result is sent through the provided channel
    async fn open_file(
        &mut self,
        tx: tokio::sync::oneshot::Sender<Result<ArcFile, tokio::io::Error>>,
        path: ArcPath,
    ) {
        let f = match self.files.get(path.as_ref()) {
            Some(f) => Arc::clone(f),
            None => match OpenOptions::new()
                .write(true)
                .create(true)
                .open(path.as_ref())
                .await
            {
                Ok(f) => {
                    let f = Arc::new(RwLock::new(f));
                    self.files.insert(path, Arc::clone(&f));
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

    /// Removes a file handle from the cache
    /// This doesn't close the file immediately - all Arc references must be dropped first
    fn close_file(&mut self, path: ArcPath) {
        self.files.remove(path.as_ref());
    }

    /// Removes a file from the filesystem and sends the result through the provided channel
    async fn remove_file(
        &self,
        tx: tokio::sync::oneshot::Sender<Result<(), tokio::io::Error>>,
        path: ArcPath,
    ) {
        let res = tokio::fs::remove_file(path.as_ref()).await;
        let _ = tx.send(res);
    }

    /// Reads the contents of a directory and sends the list of entries through the provided channel
    /// Each entry is wrapped in an Arc
    async fn read_dir(
        &self,
        tx: tokio::sync::oneshot::Sender<Result<LinkedList<ArcPath>, io::Error>>,
        path: ArcPath,
    ) {
        match tokio::fs::read_dir(path.as_ref()).await {
            Ok(mut rd) => {
                let mut entries = LinkedList::new();
                let res = loop {
                    match rd.next_entry().await {
                        Ok(Some(entry)) => entries.push_back(Arc::from(entry.path())),
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

    /// Creates a directory and all its parent directories if they don't exist
    /// Sends the result through the provided channel
    async fn mkdir(&self, tx: tokio::sync::oneshot::Sender<Result<(), io::Error>>, path: ArcPath) {
        let res = tokio::fs::create_dir_all(path.as_ref()).await;
        let _ = tx.send(res);
    }

    /// Removes a directory and all its contents recursively
    /// Sends the result through the provided channel
    async fn rmdir(&self, tx: tokio::sync::oneshot::Sender<Result<(), io::Error>>, path: ArcPath) {
        let res = tokio::fs::remove_dir_all(path.as_ref()).await;
        let _ = tx.send(res);
    }
}

/// Messages that can be sent to a [`FsCore`]
#[derive(Debug)]
pub enum Message {
    OpenFile {
        tx: tokio::sync::oneshot::Sender<Result<ArcFile, tokio::io::Error>>,
        path: ArcPath,
    },
    CloseFile {
        path: ArcPath,
    },
    RemoveFile {
        tx: tokio::sync::oneshot::Sender<Result<(), tokio::io::Error>>,
        path: ArcPath,
    },
    ReadDir {
        tx: tokio::sync::oneshot::Sender<Result<LinkedList<ArcPath>, io::Error>>,
        path: ArcPath,
    },
    MkDir {
        tx: tokio::sync::oneshot::Sender<Result<(), io::Error>>,
        path: ArcPath,
    },
    RmDir {
        tx: tokio::sync::oneshot::Sender<Result<(), io::Error>>,
        path: ArcPath,
    },
}

/// A mock implementation of the Fs actor, used for testing
/// This implementation won't actually interact with the OS, but rather store
/// the state in memory.
///
/// Files should be opened before being used, since it won't attempt to interact
/// with the filesystem.
#[derive(Debug, Clone, Default)]
pub struct FsMock {
    files: HashMap<ArcPath, ArcFile>,
    dirs: HashMap<ArcPath, LinkedList<ArcPath>>,
}

/// The fs actor is responsible for handling filesystem operations
/// This is a transmitter to a running actor, and can be used to send messages
///
/// Cloning this actor is cheap, since it's just a transmitter
///
/// Instantiate a new fs actor with [`FsCore::spawn`]
#[derive(Debug, Clone)]
pub enum Fs {
    Actual(Sender<Message>),
    Mock(Arc<tokio::sync::Mutex<FsMock>>),
}

impl From<FsCore> for Fs {
    fn from(core: FsCore) -> Self {
        let (fs, _) = core.spawn();
        fs
    }
}

use Fs::*;
#[allow(dead_code)]
impl Fs {
    /// Creates a new mock instance of the Fs actor for testing
    ///
    /// # Important
    /// Mocks cannot open new files, so you need to open your files beforehands
    /// and pass them to the mock
    pub fn mock(files: HashMap<ArcPath, ArcFile>) -> Self {
        let mock = FsMock {
            files,
            ..FsMock::default()
        };

        Mock(Arc::new(tokio::sync::Mutex::new(mock)))
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
            Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::OpenFile { tx, path })
                    .await
                    .context("Opening file with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for file open with Fs")
                    .expect("fs actor died")
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
    pub async fn close_file(&self, path: ArcPath) {
        match self {
            Actual(sender) => sender
                .send(Message::CloseFile { path })
                .await
                .context("Closing file with Fs")
                .expect("fs actor died"),
            Mock(mutex) => {
                let mut lock = mutex.lock().await;
                lock.files.remove(&path);
            }
        }
    }

    /// Removes a file from the filesystem
    pub async fn remove_file(&self, path: ArcPath) -> Result<(), io::Error> {
        match self {
            Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::RemoveFile { tx, path })
                    .await
                    .context("Removing file with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for file removal with Fs")
                    .expect("fs actor died")
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

    /// Reads a directory
    pub async fn read_dir(&self, path: ArcPath) -> Result<LinkedList<ArcPath>, io::Error> {
        match self {
            Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::ReadDir { tx, path })
                    .await
                    .context("Reading directory with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for directory read with Fs")
                    .expect("fs actor died")
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

    /// Creates a directory if it doesn't exist
    pub async fn mkdir(&self, path: ArcPath) -> Result<(), io::Error> {
        match self {
            Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::MkDir { tx, path })
                    .await
                    .context("Creating directory with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for directory creation with Fs")
                    .expect("fs actor died")
            }
            Mock(lock) => {
                let mut lock = lock.lock().await;
                lock.dirs.insert(path, LinkedList::new());
                Ok(())
            }
        }
    }

    /// Removes a directory
    pub async fn rmdir(&self, path: ArcPath) -> Result<(), io::Error> {
        match self {
            Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::RmDir { tx, path })
                    .await
                    .context("Removing directory with Fs")
                    .expect("fs actor died");
                rx.await
                    .context("Awaiting response for directory removal with Fs")
                    .expect("fs actor died")
            }
            Mock(lock) => {
                let mut lock = lock.lock().await;
                lock.dirs.remove(&path);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs::File;

    #[tokio::test]
    async fn test_fs_open_close() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_fs_open_close.txt");
        
        // Create the actual filesystem handler
        let (fs, _) = FsCore::new().spawn();
        let path: ArcPath = Arc::from(file_path.clone());

        // Create and write to file
        let file = File::create(&file_path).await.unwrap();
        drop(file);

        fs.open_file(path.clone()).await.unwrap();
        fs.close_file(path).await;

        // Cleanup
        fs.remove_file(Arc::from(file_path)).await.unwrap();
        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_fs_mkdir_rmdir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().join("test_fs_mkdir_rmdir");
        let path: ArcPath = Arc::from(dir_path.clone());

        let (fs, _) = FsCore::new().spawn();

        fs.mkdir(path.clone()).await.unwrap();
        let entries = fs.read_dir(path.clone()).await.unwrap();
        assert!(entries.is_empty());

        fs.rmdir(path.clone()).await.unwrap();
        let result = fs.read_dir(path).await;
        assert!(matches!(result, Err(e) if e.kind() == io::ErrorKind::NotFound));

        // Cleanup
        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_fs_remove_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().join("test_fs_remove_file");
        let file_path = dir_path.join("test_fs_remove_file.txt");
        
        let dir_path: ArcPath = Arc::from(dir_path);
        let file_path: ArcPath = Arc::from(file_path);

        let (fs, _) = FsCore::new().spawn();

        // Create directory and file
        fs.mkdir(dir_path.clone()).await.unwrap();
        fs.open_file(file_path.clone()).await.unwrap();
        fs.close_file(file_path.clone()).await;

        // Verify file exists in directory
        let entries = fs.read_dir(dir_path.clone()).await.unwrap();
        assert!(!entries.is_empty());
        assert_eq!(entries.len(), 1);

        // Remove file
        fs.remove_file(file_path).await.unwrap();

        // Verify directory is now empty
        let entries = fs.read_dir(dir_path.clone()).await.unwrap();
        assert!(entries.is_empty());

        // Cleanup
        fs.rmdir(dir_path).await.unwrap();
        temp_dir.close().unwrap();
    }
}
