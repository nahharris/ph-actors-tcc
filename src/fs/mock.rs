#[cfg(test)]
use mockall::mock;

#[cfg(test)]
use std::collections::LinkedList;

#[cfg(test)]
use crate::{ArcPath, error::FsError};

#[cfg(test)]
mock! {
    #[derive(Debug)]
    pub Fs {
        pub async fn read_file(&self, path: ArcPath) -> Result<tokio::fs::File, FsError>;
        pub async fn write_file(&self, path: ArcPath) -> Result<tokio::fs::File, FsError>;
        pub async fn append_file(&self, path: ArcPath) -> Result<tokio::fs::File, FsError>;
        pub async fn remove_file(&self, path: ArcPath) -> Result<(), FsError>;
        pub async fn read_dir(&self, path: ArcPath) -> Result<LinkedList<ArcPath>, FsError>;
        pub async fn mkdir(&self, path: ArcPath) -> Result<(), FsError>;
        pub async fn rmdir(&self, path: ArcPath) -> Result<(), FsError>;
    }

    impl Clone for Fs {
        fn clone(&self) -> Self;
    }
}
