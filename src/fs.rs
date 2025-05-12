use std::{
    collections::{HashMap, LinkedList},
    io,
    sync::Arc,
};

use anyhow::Context;
use tokio::{fs::OpenOptions, sync::mpsc::Sender};

use crate::{ArcFile, ArcPath};

/// The core of the Fs actor, responsible for handling filesystem operations.
///
/// This struct provides thread-safe access to filesystem operations through an actor pattern.
/// It wraps tokio's filesystem functions and provides a safe interface for concurrent access.
/// Files are cached to avoid repeated opening of the same file.
///
/// # Examples
/// ```
/// let (fs, _) = FsCore::new().spawn();
/// let path = Arc::from(Path::new("example.txt"));
/// let file = fs.open_file(path).await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. File handles are shared
/// using `Arc` to avoid cloning file descriptors.
#[derive(Debug, Default)]
pub struct FsCore {
    /// The cache of open files, mapping paths to their file handles
    files: HashMap<ArcPath, ArcFile>,
}

impl FsCore {
    /// Creates a new Fs core instance.
    ///
    /// # Returns
    /// A new instance of `FsCore` with an empty file cache.
    pub fn new() -> Self {
        Default::default()
    }

    /// Transforms an instance of [`FsCore`] into an actor ready to receive messages.
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

/// Messages that can be sent to a [`FsCore`] actor.
///
/// This enum defines the different types of filesystem operations that can be performed
/// through the actor system.
#[derive(Debug)]
pub enum Message {
    /// Opens a file and returns its handle
    OpenFile {
        /// Channel to send the result back to the caller
        tx: tokio::sync::oneshot::Sender<Result<ArcFile, tokio::io::Error>>,
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
        tx: tokio::sync::oneshot::Sender<Result<(), tokio::io::Error>>,
        /// The path of the file to remove
        path: ArcPath,
    },
    /// Reads the contents of a directory
    ReadDir {
        /// Channel to send the result back to the caller
        tx: tokio::sync::oneshot::Sender<Result<LinkedList<ArcPath>, io::Error>>,
        /// The path of the directory to read
        path: ArcPath,
    },
    /// Creates a directory and its parents
    MkDir {
        /// Channel to send the result back to the caller
        tx: tokio::sync::oneshot::Sender<Result<(), io::Error>>,
        /// The path of the directory to create
        path: ArcPath,
    },
    /// Removes a directory and its contents
    RmDir {
        /// Channel to send the result back to the caller
        tx: tokio::sync::oneshot::Sender<Result<(), io::Error>>,
        /// The path of the directory to remove
        path: ArcPath,
    },
}

/// A mock implementation of the Fs actor, used for testing.
///
/// This implementation stores files and directories in memory instead of
/// interacting with the actual filesystem.
///
/// # Important
/// Files must be opened before being used, as this implementation won't attempt
/// to interact with the filesystem.
///
/// # Examples
/// ```
/// let mut files = HashMap::new();
/// let path = Arc::from(Path::new("test.txt"));
/// let file = Arc::new(RwLock::new(File::create("test.txt").await?));
/// files.insert(path.clone(), file);
/// let fs = Fs::mock(files);
/// ```
#[derive(Debug, Clone, Default)]
pub struct FsMock {
    /// In-memory storage for open files
    files: HashMap<ArcPath, ArcFile>,
    /// In-memory storage for directory contents
    dirs: HashMap<ArcPath, LinkedList<ArcPath>>,
}

/// The fs actor is responsible for handling filesystem operations.
///
/// This enum represents either a real filesystem actor or a mock implementation
/// for testing purposes. It provides a unified interface for filesystem operations
/// regardless of the underlying implementation.
///
/// # Examples
/// ```
/// let (fs, _) = FsCore::new().spawn();
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
    Actual(Sender<Message>),
    /// A mock implementation for testing
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
                let entries = lock.dirs.get(&path).ok_or_else(|| {
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
        let path = ArcPath::from(&file_path);

        // Create and write to file
        let file = File::create(&file_path).await.unwrap();
        drop(file);

        fs.open_file(path.clone()).await.unwrap();
        fs.close_file(path.clone()).await;

        // Cleanup
        fs.remove_file(path).await.unwrap();
        temp_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_fs_mkdir_rmdir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().join("test_fs_mkdir_rmdir");
        let path = ArcPath::from(&dir_path);

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

        let dir_path = ArcPath::from(&dir_path);
        let file_path = ArcPath::from(&file_path);

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
