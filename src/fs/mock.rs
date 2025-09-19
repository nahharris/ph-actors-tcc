use std::{collections::LinkedList, io, sync::Arc};
use tokio::sync::Mutex;
use tempfile::TempDir;

use crate::ArcPath;

/// Mock implementation of the Fs actor for testing purposes.
///
/// This struct uses a temporary directory to simulate filesystem operations,
/// allowing tests to run without affecting the actual filesystem.
#[derive(Debug)]
pub struct Mock {
    temp_dir: Arc<Mutex<TempDir>>,
}

impl Clone for Mock {
    fn clone(&self) -> Self {
        Self {
            temp_dir: self.temp_dir.clone(),
        }
    }
}

impl Mock {
    /// Creates a new mock instance with a temporary directory.
    pub fn new() -> Self {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir for Fs mock");
        Self {
            temp_dir: Arc::new(Mutex::new(temp_dir)),
        }
    }

    /// Converts a relative path to a full path within the mock's temporary directory.
    async fn mock_path(&self, path: &ArcPath) -> std::path::PathBuf {
        let temp_dir = self.temp_dir.lock().await;
        temp_dir.path().join(path.as_ref() as &std::path::Path)
    }

    /// Opens a file for reading only (does not create if it doesn't exist).
    ///
    /// # Arguments
    /// * `path` - The file path to open for reading
    ///
    /// # Returns
    /// The opened file or an error if the file doesn't exist or cannot be opened.
    pub async fn read_file(&self, path: ArcPath) -> Result<tokio::fs::File, io::Error> {
        let real_path = self.mock_path(&path).await;
        tokio::fs::OpenOptions::new()
            .read(true)
            .open(real_path)
            .await
    }

    /// Opens a file for writing (truncates content, creates if needed).
    ///
    /// # Arguments
    /// * `path` - The file path to open for writing
    ///
    /// # Returns
    /// The opened file or an error if the file cannot be created or opened.
    pub async fn write_file(&self, path: ArcPath) -> Result<tokio::fs::File, io::Error> {
        let real_path = self.mock_path(&path).await;
        tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(real_path)
            .await
    }

    /// Opens a file for appending (creates if needed).
    ///
    /// # Arguments
    /// * `path` - The file path to open for appending
    ///
    /// # Returns
    /// The opened file or an error if the file cannot be created or opened.
    pub async fn append_file(&self, path: ArcPath) -> Result<tokio::fs::File, io::Error> {
        let real_path = self.mock_path(&path).await;
        tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(real_path)
            .await
    }

    /// Removes a file from the filesystem.
    ///
    /// # Arguments
    /// * `path` - The file path to remove
    ///
    /// # Returns
    /// Ok(()) if successful, or an error if the file cannot be removed.
    pub async fn remove_file(&self, path: ArcPath) -> Result<(), io::Error> {
        let real_path = self.mock_path(&path).await;
        tokio::fs::remove_file(real_path).await
    }

    /// Reads a directory.
    ///
    /// # Arguments
    /// * `path` - The directory path to read
    ///
    /// # Returns
    /// A list of directory entries or an error if the directory cannot be read.
    pub async fn read_dir(&self, path: ArcPath) -> Result<LinkedList<ArcPath>, io::Error> {
        let real_path = self.mock_path(&path).await;
        let mut entries = LinkedList::new();
        let mut rd = tokio::fs::read_dir(real_path).await?;
        while let Some(entry) = rd.next_entry().await? {
            let path = entry.path();
            entries.push_back(ArcPath::from(&path));
        }
        Ok(entries)
    }

    /// Creates a directory if it doesn't exist.
    ///
    /// # Arguments
    /// * `path` - The directory path to create
    ///
    /// # Returns
    /// Ok(()) if successful, or an error if the directory cannot be created.
    pub async fn mkdir(&self, path: ArcPath) -> Result<(), io::Error> {
        let real_path = self.mock_path(&path).await;
        tokio::fs::create_dir_all(real_path).await
    }

    /// Removes a directory.
    ///
    /// # Arguments
    /// * `path` - The directory path to remove
    ///
    /// # Returns
    /// Ok(()) if successful, or an error if the directory cannot be removed.
    pub async fn rmdir(&self, path: ArcPath) -> Result<(), io::Error> {
        let real_path = self.mock_path(&path).await;
        tokio::fs::remove_dir_all(real_path).await
    }
}
