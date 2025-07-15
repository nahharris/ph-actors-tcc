use std::sync::Arc;
use tokio::sync::Mutex;

mod core;
pub mod message;

use crate::api::lore::{LoreMailingList, LoreApi};
use crate::fs::Fs;
use crate::app::config::Config;
use message::Message;

/// The MailingListState actor provides a demand-driven, cached list of mailing lists.
///
/// Clients can fetch a single item by index or a slice of consecutive items. If an item is not cached, the actor fetches the next page from the API.
#[derive(Debug, Clone)]
pub enum MailingListState {
    Actual(tokio::sync::mpsc::Sender<Message>),
    Mock(Arc<Mutex<MockData>>),
}

#[derive(Debug, Default)]
pub struct MockData {
    pub mailing_lists: Vec<LoreMailingList>,
}

impl MailingListState {
    /// Spawns a new MailingListState actor.
    pub fn spawn(lore: LoreApi, fs: Fs, config: Config, cache_path: crate::ArcPath) -> Self {
        let core = core::Core::new(lore, fs, config, cache_path);
        let (state, _handle) = core.spawn();
        state
    }

    /// Creates a new mock MailingListState actor for testing.
    pub fn mock(data: MockData) -> Self {
        Self::Mock(Arc::new(Mutex::new(data)))
    }

    /// Fetches a single mailing list by index (demand-driven).
    pub async fn get(&self, index: usize) -> anyhow::Result<Option<LoreMailingList>> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender.send(Message::Get { index, tx }).await?;
                rx.await?
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                Ok(data.mailing_lists.get(index).cloned())
            }
        }
    }

    /// Fetches a slice of mailing lists by range (demand-driven).
    pub async fn get_slice(&self, range: std::ops::Range<usize>) -> anyhow::Result<Vec<LoreMailingList>> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender.send(Message::GetSlice { range, tx }).await?;
                rx.await?
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                Ok(data.mailing_lists[range].to_vec())
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
                data.mailing_lists.clear();
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

    /// Returns the number of cached mailing lists.
    pub async fn len(&self) -> usize {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let _ = sender.send(message::Message::Len { tx }).await;
                rx.await.unwrap_or(0)
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                data.mailing_lists.len()
            }
        }
    }

    /// Checks if the cache is still valid by comparing the last_update field of the 0th item.
    pub async fn is_cache_valid(&self) -> anyhow::Result<bool> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender.send(message::Message::IsCacheValid { tx }).await?;
                rx.await?
            }
            Self::Mock(_) => {
                // Always true for mock
                Ok(true)
            }
        }
    }
} 