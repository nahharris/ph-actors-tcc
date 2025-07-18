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

    pub fn spawn(self) -> (super::Fs, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel(crate::BUFFER_SIZE);
        let handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                use Message::*;
                match msg {
                    OpenFile { tx, path } => Self::open_file(tx, path).await,
                    RemoveFile { tx, path } => Self::remove_file(tx, path).await,
                    ReadDir { tx, path } => Self::read_dir(tx, path).await,
                    MkDir { tx, path } => Self::mkdir(tx, path).await,
                    RmDir { tx, path } => Self::rmdir(tx, path).await,
                }
            }
        });
        (super::Fs::Actual(tx), handle)
    }

    /// Opens a file (always opens a new handle).
    async fn open_file(
        tx: tokio::sync::oneshot::Sender<Result<tokio::fs::File, tokio::io::Error>>,
        path: ArcPath,
    ) {
        let res = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .await;
        let _ = tx.send(res);
    }

    async fn remove_file(
        tx: tokio::sync::oneshot::Sender<Result<(), tokio::io::Error>>,
        path: ArcPath,
    ) {
        let res = tokio::fs::remove_file(&path).await;
        let _ = tx.send(res);
    }

    async fn read_dir(
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

    async fn mkdir(tx: tokio::sync::oneshot::Sender<Result<(), io::Error>>, path: ArcPath) {
        let res = tokio::fs::create_dir_all(&path).await;
        let _ = tx.send(res);
    }

    async fn rmdir(tx: tokio::sync::oneshot::Sender<Result<(), io::Error>>, path: ArcPath) {
        let res = tokio::fs::remove_dir_all(&path).await;
        let _ = tx.send(res);
    }
}
