use anyhow::Context;
use tokio::sync::mpsc;

mod core;
mod data;
pub mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

use crate::ArcStr;
use crate::api::lore::LorePatchMetadata;
use message::Message;

#[cfg(not(test))]
use crate::{api::lore::LoreApi, app::config::Config, fs::Fs, log::Log};
#[cfg(test)]
use crate::{
    api::lore::mock::MockLoreApi as LoreApi, app::config::mock::MockConfig as Config,
    fs::mock::MockFs as Fs, log::mock::MockLog as Log,
};

/// The Feed Actor provides per-mailing-list caching of patch metadata.
///
/// This actor caches patch metadata for each mailing list separately, providing
/// smart pagination and cache validation. It fetches data on demand and maintains
/// cache validity based on the 0-th item's updated time.
#[derive(Debug, Clone)]
pub struct FeedCache {
    tx: tokio::sync::mpsc::Sender<Message>,
}

impl FeedCache {
    /// Creates a new FeedCache actor.
    pub fn new(tx: tokio::sync::mpsc::Sender<Message>) -> Self {
        Self { tx }
    }

    /// Spawns a new FeedCache actor.
    pub async fn spawn(lore: LoreApi, fs: Fs, config: Config, log: Log) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(lore, fs, config, log).await?;
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Ok(Self { tx })
    }

    /// Fetches a single patch metadata item by index for a given mailing list.
    pub async fn get(
        &self,
        list: ArcStr,
        index: usize,
    ) -> anyhow::Result<Option<LorePatchMetadata>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Get { list, index, tx })
            .await
            .context("Sending message to FeedCache actor")
            .expect("FeedCache actor died");
        rx.await
            .context("Awaiting response from FeedCache actor")
            .expect("FeedCache actor died")
    }

    /// Fetches a slice of patch metadata items by range for a given mailing list.
    pub async fn get_slice(
        &self,
        list: ArcStr,
        range: std::ops::Range<usize>,
    ) -> anyhow::Result<Vec<LorePatchMetadata>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::GetSlice { list, range, tx })
            .await
            .context("Sending message to FeedCache actor")
            .expect("FeedCache actor died");
        rx.await
            .context("Awaiting response from FeedCache actor")
            .expect("FeedCache actor died")
    }

    /// Refreshes the cache for a specific mailing list.
    pub async fn refresh(&self, list: ArcStr) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Refresh { list, tx })
            .await
            .context("Sending message to FeedCache actor")
            .expect("FeedCache actor died");
        rx.await
            .context("Awaiting response from FeedCache actor")
            .expect("FeedCache actor died")
    }

    /// Invalidates the cache for a specific mailing list.
    pub async fn invalidate(&self, list: ArcStr) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Invalidate { list, tx })
            .await
            .context("Sending message to FeedCache actor")
            .expect("FeedCache actor died");
        rx.await
            .context("Awaiting response from FeedCache actor")
            .expect("FeedCache actor died")
    }

    /// Checks if the requested range is available in cache for a mailing list.
    pub async fn is_available(&self, list: ArcStr, range: std::ops::Range<usize>) -> bool {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::IsAvailable { list, range, tx })
            .await
            .context("Sending message to FeedCache actor")
            .expect("FeedCache actor died");
        rx.await
            .context("Awaiting response from FeedCache actor")
            .expect("FeedCache actor died")
    }

    /// Returns the number of cached items for a given mailing list.
    pub async fn len(&self, list: ArcStr) -> usize {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Len { list, tx })
            .await
            .context("Sending message to FeedCache actor")
            .expect("FeedCache actor died");
        rx.await
            .context("Awaiting response from FeedCache actor")
            .expect("FeedCache actor died")
    }

    /// Checks if the cache has been loaded from disk for a given mailing list.
    /// This is different from is_empty() - a cache can be loaded but empty.
    pub async fn is_loaded(&self, list: ArcStr) -> bool {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::IsLoaded { list, tx })
            .await
            .context("Sending message to FeedCache actor")
            .expect("FeedCache actor died");
        rx.await
            .context("Awaiting response from FeedCache actor")
            .expect("FeedCache actor died")
    }

    /// Ensures the cache is loaded for a given mailing list.
    /// This will load from disk if not already loaded.
    pub async fn ensure_loaded(&self, list: ArcStr) -> anyhow::Result<()> {
        if !self.is_loaded(list.clone()).await {
            self.load(list).await?;
        }
        Ok(())
    }

    /// Persists the cache for a specific mailing list to the filesystem.
    pub async fn persist(&self, list: ArcStr) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Persist { list, tx })
            .await
            .context("Sending message to FeedCache actor")
            .expect("FeedCache actor died");
        rx.await
            .context("Awaiting response from FeedCache actor")
            .expect("FeedCache actor died")
    }

    /// Loads the cache for a specific mailing list from the filesystem.
    pub async fn load(&self, list: ArcStr) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Load { list, tx })
            .await
            .context("Sending message to FeedCache actor")
            .expect("FeedCache actor died");
        rx.await
            .context("Awaiting response from FeedCache actor")
            .expect("FeedCache actor died")
    }
}
