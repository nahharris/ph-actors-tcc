use anyhow::Context;
use std::sync::Arc;
use tokio::sync::Mutex;

mod core;
mod data;
pub mod message;

use crate::ArcStr;
use crate::api::lore::{LoreApi, LorePatchMetadata};
use crate::app::config::Config;
use crate::fs::Fs;
use crate::log::Log;
use message::Message;

/// The Feed Actor provides per-mailing-list caching of patch metadata.
///
/// This actor caches patch metadata for each mailing list separately, providing
/// smart pagination and cache validation. It fetches data on demand and maintains
/// cache validity based on the 0-th item's updated time.
#[derive(Debug, Clone)]
pub enum FeedCache {
    Actual(tokio::sync::mpsc::Sender<Message>),
    Mock(Arc<Mutex<MockData>>),
}

#[derive(Debug, Default)]
pub struct MockData {
    pub feeds: std::collections::HashMap<ArcStr, Vec<LorePatchMetadata>>,
}

impl FeedCache {
    /// Spawns a new FeedCache actor.
    pub async fn spawn(lore: LoreApi, fs: Fs, config: Config, log: Log) -> anyhow::Result<Self> {
        let core = core::Core::new(lore, fs, config, log).await?;
        let (state, _handle) = core.spawn();
        Ok(state)
    }

    /// Creates a new mock FeedCache actor for testing.
    pub fn mock(data: MockData) -> Self {
        Self::Mock(Arc::new(Mutex::new(data)))
    }

    /// Fetches a single patch metadata item by index for a given mailing list.
    pub async fn get(
        &self,
        list: ArcStr,
        index: usize,
    ) -> anyhow::Result<Option<LorePatchMetadata>> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Get { list, index, tx })
                    .await
                    .context("Sending message to FeedCache actor")
                    .expect("FeedCache actor died");
                rx.await
                    .context("Awaiting response from FeedCache actor")
                    .expect("FeedCache actor died")
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                Ok(data.feeds.get(&list).and_then(|v| v.get(index)).cloned())
            }
        }
    }

    /// Fetches a slice of patch metadata items by range for a given mailing list.
    pub async fn get_slice(
        &self,
        list: ArcStr,
        range: std::ops::Range<usize>,
    ) -> anyhow::Result<Vec<LorePatchMetadata>> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::GetSlice { list, range, tx })
                    .await
                    .context("Sending message to FeedCache actor")
                    .expect("FeedCache actor died");
                rx.await
                    .context("Awaiting response from FeedCache actor")
                    .expect("FeedCache actor died")
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                Ok(data
                    .feeds
                    .get(&list)
                    .map(|v| v[range].to_vec())
                    .unwrap_or_default())
            }
        }
    }

    /// Refreshes the cache for a specific mailing list.
    pub async fn refresh(&self, list: ArcStr) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Refresh { list, tx })
                    .await
                    .context("Sending message to FeedCache actor")
                    .expect("FeedCache actor died");
                rx.await
                    .context("Awaiting response from FeedCache actor")
                    .expect("FeedCache actor died")
            }
            Self::Mock(_) => Ok(()),
        }
    }

    /// Invalidates the cache for a specific mailing list.
    pub async fn invalidate(&self, list: ArcStr) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Invalidate { list, tx })
                    .await
                    .context("Sending message to FeedCache actor")
                    .expect("FeedCache actor died");
                rx.await
                    .context("Awaiting response from FeedCache actor")
                    .expect("FeedCache actor died")
            }
            Self::Mock(data) => {
                let mut data = data.lock().await;
                data.feeds.remove(&list);
                Ok(())
            }
        }
    }

    /// Checks if the requested range is available in cache for a mailing list.
    pub async fn is_available(&self, list: ArcStr, range: std::ops::Range<usize>) -> bool {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::IsAvailable { list, range, tx })
                    .await
                    .context("Sending message to FeedCache actor")
                    .expect("FeedCache actor died");
                rx.await
                    .context("Awaiting response from FeedCache actor")
                    .expect("FeedCache actor died")
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                data.feeds
                    .get(&list)
                    .map(|v| v.len() >= range.end)
                    .unwrap_or(false)
            }
        }
    }

    /// Returns the number of cached items for a given mailing list.
    pub async fn len(&self, list: ArcStr) -> usize {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Len { list, tx })
                    .await
                    .context("Sending message to FeedCache actor")
                    .expect("FeedCache actor died");
                rx.await
                    .context("Awaiting response from FeedCache actor")
                    .expect("FeedCache actor died")
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                data.feeds.get(&list).map(|v| v.len()).unwrap_or(0)
            }
        }
    }

    /// Returns true if the cache is empty for a given mailing list.
    pub async fn is_empty(&self, list: ArcStr) -> bool {
        self.len(list).await == 0
    }

    /// Checks if the cache has been loaded from disk for a given mailing list.
    /// This is different from is_empty() - a cache can be loaded but empty.
    pub async fn is_loaded(&self, list: ArcStr) -> bool {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::IsLoaded { list, tx })
                    .await
                    .context("Sending message to FeedCache actor")
                    .expect("FeedCache actor died");
                rx.await
                    .context("Awaiting response from FeedCache actor")
                    .expect("FeedCache actor died")
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                data.feeds.contains_key(&list)
            }
        }
    }

    /// Ensures the cache is loaded for a given mailing list.
    /// This will load from disk if not already loaded.
    pub async fn ensure_loaded(&self, list: ArcStr) -> anyhow::Result<()> {
        if !self.is_loaded(list.clone()).await {
            self.load(list.clone()).await?;
        }
        Ok(())
    }

    /// Persists the cache for a specific mailing list to the filesystem.
    pub async fn persist(&self, list: ArcStr) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Persist { list, tx })
                    .await
                    .context("Sending message to FeedCache actor")
                    .expect("FeedCache actor died");
                rx.await
                    .context("Awaiting response from FeedCache actor")
                    .expect("FeedCache actor died")
            }
            Self::Mock(_) => Ok(()),
        }
    }

    /// Loads the cache for a specific mailing list from the filesystem.
    pub async fn load(&self, list: ArcStr) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Load { list, tx })
                    .await
                    .context("Sending message to FeedCache actor")
                    .expect("FeedCache actor died");
                rx.await
                    .context("Awaiting response from FeedCache actor")
                    .expect("FeedCache actor died")
            }
            Self::Mock(_) => Ok(()),
        }
    }
}
