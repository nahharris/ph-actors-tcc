#[cfg(test)]
use mockall::mock;

#[cfg(test)]
use std::{collections::LinkedList, io};

#[cfg(test)]
use crate::ArcPath;

#[cfg(test)]
mock! {
    #[derive(Debug)]
    pub Fs {
        pub async fn read_file(&self, path: ArcPath) -> Result<tokio::fs::File, io::Error>;
        pub async fn write_file(&self, path: ArcPath) -> Result<tokio::fs::File, io::Error>;
        pub async fn append_file(&self, path: ArcPath) -> Result<tokio::fs::File, io::Error>;
        pub async fn remove_file(&self, path: ArcPath) -> Result<(), io::Error>;
        pub async fn read_dir(&self, path: ArcPath) -> Result<LinkedList<ArcPath>, io::Error>;
        pub async fn mkdir(&self, path: ArcPath) -> Result<(), io::Error>;
        pub async fn rmdir(&self, path: ArcPath) -> Result<(), io::Error>;
    }

    impl Clone for Fs {
        fn clone(&self) -> Self;
    }
}
