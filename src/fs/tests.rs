use std::collections::HashMap;

use tempfile::tempfile;
use tokio::fs::File;

use crate::{ArcFile, ArcPath};

use super::Fs;

#[tokio::test]
async fn test_fs_open_close() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test_fs_open_close.txt");

    // Create the actual filesystem handler
    let fs = Fs::spawn();
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

    let fs = Fs::spawn();

    fs.mkdir(path.clone()).await.unwrap();
    let entries = fs.read_dir(path.clone()).await.unwrap();
    assert!(entries.is_empty());

    fs.rmdir(path.clone()).await.unwrap();
    let result = fs.read_dir(path).await;
    assert!(matches!(result, Err(e) if e.kind() == std::io::ErrorKind::NotFound));

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

    let fs = Fs::spawn();

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

#[tokio::test]
async fn test_fs_mock() {
    let mut files = HashMap::new();
    let path = ArcPath::from("test.txt");
    let std_file = tempfile().expect("Failed to create a temporary file");
    let tokio_file = tokio::fs::File::from_std(std_file);
    let file = ArcFile::from(tokio_file);
    files.insert(path.clone(), file);

    let fs = Fs::mock(files);

    // Test file operations
    let _ = fs.open_file(path.clone()).await.unwrap();
    fs.close_file(path.clone()).await;

    // Test unsupported operations
    assert!(matches!(
        fs.read_dir(ArcPath::from("test")).await,
        Err(e) if e.kind() == std::io::ErrorKind::Unsupported
    ));
    assert!(matches!(
        fs.mkdir(ArcPath::from("test")).await,
        Err(e) if e.kind() == std::io::ErrorKind::Unsupported
    ));
    assert!(matches!(
        fs.rmdir(ArcPath::from("test")).await,
        Err(e) if e.kind() == std::io::ErrorKind::Unsupported
    ));
} 