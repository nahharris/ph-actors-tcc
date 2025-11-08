use tokio::fs::File;

use crate::{ArcPath, error::FsError};

use super::Fs;
use super::mock::MockFs;

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

    let _ = fs.read_file(path.clone()).await.unwrap();

    // Cleanup
    fs.remove_file(path).await.unwrap();
    temp_dir.close().unwrap();
}

#[tokio::test]
async fn test_mock_fs_operations() {
    let mut mock_fs = MockFs::new();
    let path = ArcPath::from("test_file.txt");

    // Set up expectations
    mock_fs
        .expect_read_file()
        .with(mockall::predicate::eq(path.clone()))
        .times(1)
        .returning(|_| {
            Err(FsError::OperationFailed {
                path: None,
                operation: "read file".to_string(),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
                retryable: false,
            })
        });
    mock_fs
        .expect_write_file()
        .with(mockall::predicate::eq(path.clone()))
        .times(1)
        .returning(|_| {
            Ok(File::from_std(
                std::fs::File::create("temp").expect("Creating temp file to succeed"),
            ))
        });

    // Test the mock
    let result = mock_fs.read_file(path.clone()).await;
    assert!(result.is_err());

    let result = mock_fs.write_file(path).await;
    assert!(result.is_ok());

    // Cleanup temp file
    std::fs::remove_file("temp").ok();
}

#[tokio::test]
async fn test_fs_mkdir_rmdir() {
    let temp_dir = tempfile::tempdir().expect("Creating temp directory to succeed");
    let dir_path = temp_dir.path().join("test_fs_mkdir_rmdir");
    let path = ArcPath::from(&dir_path);

    let fs = Fs::spawn();

    fs.mkdir(path.clone())
        .await
        .expect("Creating directory to succeed");
    let entries = fs
        .read_dir(path.clone())
        .await
        .expect("Reading directory to succeed");
    assert!(entries.is_empty());

    let path_to_read = path.clone();
    fs.rmdir(path.clone())
        .await
        .expect("Removing directory to succeed");
    let Err(FsError::OperationFailed {
        path: error_path,
        operation,
        retryable,
        source,
    }) = fs.read_dir(path_to_read.clone()).await
    else {
        panic!("Reading directory to fail");
    };
    assert_eq!(error_path, Some(path_to_read.to_string_lossy().to_string()));
    assert_eq!(operation, "read directory");
    assert_eq!(retryable, false);
    assert!(matches!(source, e if e.kind() == std::io::ErrorKind::NotFound));

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
    let _ = fs.write_file(file_path.clone()).await.unwrap();

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
async fn test_fs_file_operations() {
    let fs = Fs::spawn();
    let path = ArcPath::from("test_file_operations.txt");

    // Test write_file - should create file
    let _ = fs.write_file(path.clone()).await.unwrap();
    assert!(fs.read_file(path.clone()).await.is_ok());

    // Test append_file - should append to existing file
    let _ = fs.append_file(path.clone()).await.unwrap();
    assert!(fs.read_file(path.clone()).await.is_ok());

    // Test read_file on non-existent file - should fail
    let non_existent_path = ArcPath::from("non_existent.txt");
    assert!(fs.read_file(non_existent_path).await.is_err());

    // Cleanup
    fs.remove_file(path).await.unwrap();
}
