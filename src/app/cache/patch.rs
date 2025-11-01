use anyhow::Context;
use tokio::sync::mpsc;

mod core;
mod data;
pub mod message;
#[cfg(test)]
pub mod mock;

use crate::ArcStr;
use crate::api::lore::LoreApi;
use crate::app::config::Config;
use crate::fs::Fs;
use crate::log::Log;
use message::Message;

/// The Patch Actor provides caching for individual patch content.
///
/// This actor caches raw patch content with permanent validity. Once a patch
/// is cached, it's considered valid forever. It provides a small in-memory
/// buffer for fast access to recently used patches.
#[derive(Debug, Clone)]
pub struct PatchCache {
    /// The sender for sending messages to the PatchCache actor.
    tx: tokio::sync::mpsc::Sender<Message>,
}

impl PatchCache {
    /// Creates a new PatchCache actor.
    pub fn new(tx: tokio::sync::mpsc::Sender<Message>) -> Self {
        Self { tx }
    }

    /// Spawns a new PatchCache actor.
    pub async fn spawn(lore: LoreApi, fs: Fs, config: Config, log: Log) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(lore, fs, config, log).await?;
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Ok(Self { tx })
    }

    /// Fetches a patch by mailing list and message ID.
    pub async fn get(&self, list: ArcStr, message_id: ArcStr) -> anyhow::Result<String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Get {
                list,
                message_id,
                tx,
            })
            .await
            .context("Sending message to PatchCache actor")
            .expect("PatchCache actor died");
        rx.await
            .context("Awaiting response from PatchCache actor")
            .expect("PatchCache actor died")
    }

    /// Invalidates a specific patch.
    pub async fn invalidate(&self, list: ArcStr, message_id: ArcStr) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Invalidate {
                list,
                message_id,
                tx,
            })
            .await
            .context("Sending message to PatchCache actor")
            .expect("PatchCache actor died");
        rx.await
            .context("Awaiting response from PatchCache actor")
            .expect("PatchCache actor died")
    }

    /// Checks if a patch is available in cache.
    pub async fn is_available(&self, list: ArcStr, message_id: ArcStr) -> bool {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::IsAvailable {
                list,
                message_id,
                tx,
            })
            .await
            .context("Sending message to PatchCache actor")
            .expect("PatchCache actor died");
        rx.await
            .context("Awaiting response from PatchCache actor")
            .expect("PatchCache actor died")
    }
}
