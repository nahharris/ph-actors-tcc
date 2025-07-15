use std::sync::Arc;
use tokio::sync::Mutex;

mod core;
pub mod message;

use crate::api::lore::{LorePatchMetadata, LoreApi};
use crate::{ArcStr};
use crate::fs::Fs;
use crate::app::config::Config;
use message::Message;

/// The PatchMetaState actor provides a demand-driven, cached map of patch metadata per mailing list.
///
/// Clients can fetch a single item by index or a slice of consecutive items for a given list. If an item is not cached, the actor fetches the next page from the API.
#[derive(Debug, Clone)]
pub enum PatchMetaState {
    Actual(tokio::sync::mpsc::Sender<Message>),
    Mock(Arc<Mutex<MockData>>),
}

#[derive(Debug, Default)]
pub struct MockData {
    pub patch_cache: std::collections::HashMap<ArcStr, Vec<LorePatchMetadata>>,
}

impl PatchMetaState {
    /// Spawns a new PatchMetaState actor.
    pub async fn spawn(lore: LoreApi, fs: Fs, config: Config, cache_path: crate::ArcPath) -> Self {
        let core = core::Core::new(lore, fs, config, cache_path);
        let (state, _handle) = core.spawn();
        state
    }

    /// Creates a new mock PatchMetaState actor for testing.
    pub fn mock(data: MockData) -> Self {
        Self::Mock(Arc::new(Mutex::new(data)))
    }

    /// Fetches a single patch metadata item by index for a given mailing list (demand-driven).
    pub async fn get(&self, list: ArcStr, index: usize) -> anyhow::Result<Option<LorePatchMetadata>> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender.send(Message::Get { list, index, tx }).await?;
                rx.await?
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                Ok(data.patch_cache.get(&list).and_then(|v| v.get(index)).cloned())
            }
        }
    }

    /// Fetches a slice of patch metadata items by range for a given mailing list (demand-driven).
    pub async fn get_slice(&self, list: ArcStr, range: std::ops::Range<usize>) -> anyhow::Result<Vec<LorePatchMetadata>> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender.send(Message::GetSlice { list, range, tx }).await?;
                rx.await?
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                Ok(data.patch_cache.get(&list).map(|v| v[range].to_vec()).unwrap_or_default())
            }
        }
    }

    /// Invalidates the current cache.
    pub async fn invalidate_cache(&self) {
        match self {
            Self::Actual(sender) => {
                let _ = sender.send(Message::InvalidateCache).await;
            }
            Self::Mock(data) => {
                let mut data = data.lock().await;
                data.patch_cache.clear();
            }
        }
    }

    /// Persists the cache to the filesystem.
    pub async fn persist_cache(&self) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender.send(Message::PersistCache { tx }).await?;
                rx.await?
            }
            Self::Mock(_) => Ok(()),
        }
    }

    /// Loads the cache from the filesystem.
    pub async fn load_cache(&self) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender.send(Message::LoadCache { tx }).await?;
                rx.await?
            }
            Self::Mock(_) => Ok(()),
        }
    }

    /// Returns the number of cached patch metadata items for a given mailing list.
    pub async fn len(&self, list: ArcStr) -> usize {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let _ = sender.send(message::Message::Len { list, tx }).await;
                rx.await.unwrap_or(0)
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                data.patch_cache.get(&list).map(|v| v.len()).unwrap_or(0)
            }
        }
    }

    /// Checks if the cache is still valid for a given mailing list by comparing the last_update field of the 0th item.
    pub async fn is_cache_valid(&self, list: ArcStr) -> anyhow::Result<bool> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender.send(message::Message::IsCacheValid { list, tx }).await?;
                rx.await?
            }
            Self::Mock(_) => {
                // Always true for mock
                Ok(true)
            }
        }
    }
}