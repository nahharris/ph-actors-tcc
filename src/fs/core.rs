use std::{collections::LinkedList, io};

use tokio::{fs::OpenOptions, sync::mpsc};

use crate::ArcPath;

use super::message::Message;

/// The core of the Fs actor, responsible for handling filesystem operations.
///
/// This struct provides thread-safe access to filesystem operations through an actor pattern.
/// It wraps tokio's filesystem functions and provides a safe interface for concurrent access.
#[derive(Debug, Default)]
pub struct Core;

impl Core {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn init(self, mut rx: mpsc::Receiver<Message>) {
        while let Some(msg) = rx.recv().await {
            use Message::*;
            match msg {
                ReadFile { tx, path } => self.handle_read_file(tx, path).await,
                WriteFile { tx, path } => self.handle_write_file(tx, path).await,
                AppendFile { tx, path } => self.handle_append_file(tx, path).await,
                RemoveFile { tx, path } => self.handle_remove_file(tx, path).await,
                ReadDir { tx, path } => self.handle_read_dir(tx, path).await,
                MkDir { tx, path } => self.handle_mkdir(tx, path).await,
                RmDir { tx, path } => self.handle_rmdir(tx, path).await,
            }
        }
    }

    /// Opens a file for reading only (does not create if it doesn't exist).
    async fn handle_read_file(
        &self,
        tx: tokio::sync::oneshot::Sender<Result<tokio::fs::File, tokio::io::Error>>,
        path: ArcPath,
    ) {
        let res = OpenOptions::new().read(true).open(&path).await;
        let _ = tx.send(res);
    }

    /// Opens a file for writing (truncates content, creates if needed).
    async fn handle_write_file(
        &self,
        tx: tokio::sync::oneshot::Sender<Result<tokio::fs::File, tokio::io::Error>>,
        path: ArcPath,
    ) {
        // Ensure parent directories exist before creating the file
        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        let res = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .await;
        let _ = tx.send(res);
    }

    /// Opens a file for appending (creates if needed).
    async fn handle_append_file(
        &self,
        tx: tokio::sync::oneshot::Sender<Result<tokio::fs::File, tokio::io::Error>>,
        path: ArcPath,
    ) {
        // Ensure parent directories exist before creating the file
        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        let res = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&path)
            .await;
        let _ = tx.send(res);
    }

    async fn handle_remove_file(
        &self,
        tx: tokio::sync::oneshot::Sender<Result<(), tokio::io::Error>>,
        path: ArcPath,
    ) {
        let res = tokio::fs::remove_file(&path).await;
        let _ = tx.send(res);
    }

    async fn handle_read_dir(
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

    async fn handle_mkdir(
        &self,
        tx: tokio::sync::oneshot::Sender<Result<(), io::Error>>,
        path: ArcPath,
    ) {
        let res = tokio::fs::create_dir_all(&path).await;
        let _ = tx.send(res);
    }

    async fn handle_rmdir(
        &self,
        tx: tokio::sync::oneshot::Sender<Result<(), io::Error>>,
        path: ArcPath,
    ) {
        let res = tokio::fs::remove_dir_all(&path).await;
        let _ = tx.send(res);
    }
}
