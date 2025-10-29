use anyhow::Context;
use tokio::sync::mpsc;

mod core;
mod data;
pub mod message;
#[cfg(test)]
pub mod mock;

use crate::{
    api::{LoreApi, lore::LoreMailingList},
    app::config::Config,
    fs::Fs,
    log::Log,
};
use message::Message;

/// The Mailing List Actor provides a cached list of mailing lists sorted alphabetically.
///
/// This actor fetches all mailing lists from the API, sorts them alphabetically,
/// and provides fast access to individual items or ranges. The cache is validated
/// based on the last updated time of the 0-th item from the API.
#[derive(Debug, Clone)]
pub struct MailingListCache {
    tx: tokio::sync::mpsc::Sender<Message>,
}

impl MailingListCache {
    /// Spawns a new MailingListCache actor.
    pub fn new(tx: tokio::sync::mpsc::Sender<Message>) -> Self {
        Self { tx }
    }

    /// Creates a new MailingListCache actor.
    pub async fn spawn(lore: LoreApi, fs: Fs, config: Config, log: Log) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(lore, fs, config, log).await?;
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Ok(Self { tx })
    }

    /// Fetches a single mailing list by index.
    pub async fn get(&self, index: usize) -> anyhow::Result<Option<LoreMailingList>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Get { index, tx })
            .await
            .context("Sending message to MailingListCache actor")
            .expect("MailingListCache actor died");
        rx.await
            .context("Awaiting response from MailingListCache actor")
            .expect("MailingListCache actor died")
    }

    /// Fetches a slice of mailing lists by range.
    pub async fn get_slice(
        &self,
        range: std::ops::Range<usize>,
    ) -> anyhow::Result<Vec<LoreMailingList>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::GetSlice { range, tx })
            .await
            .context("Sending message to MailingListCache actor")
            .expect("MailingListCache actor died");
        rx.await
            .context("Awaiting response from MailingListCache actor")
            .expect("MailingListCache actor died")
    }

    /// Refreshes the cache by fetching from the API.
    pub async fn refresh(&self) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Refresh { tx })
            .await
            .context("Sending message to MailingListCache actor")
            .expect("MailingListCache actor died");
        rx.await
            .context("Awaiting response from MailingListCache actor")
            .expect("MailingListCache actor died")
    }

    /// Invalidates the current cache.
    pub async fn invalidate(&self) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Invalidate { tx })
            .await
            .context("Sending message to MailingListCache actor")
            .expect("MailingListCache actor died");
        rx.await
            .context("Awaiting response from MailingListCache actor")
            .expect("MailingListCache actor died")
    }

    /// Checks if the requested range is available in cache.
    pub async fn is_available(&self, range: std::ops::Range<usize>) -> bool {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::IsAvailable { range, tx })
            .await
            .context("Sending message to MailingListCache actor")
            .expect("MailingListCache actor died");
        rx.await
            .context("Awaiting response from MailingListCache actor")
            .expect("MailingListCache actor died")
    }

    /// Returns the number of cached mailing lists.
    pub async fn len(&self) -> usize {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Len { tx })
            .await
            .context("Sending message to MailingListCache actor")
            .expect("MailingListCache actor died");
        rx.await
            .context("Awaiting response from MailingListCache actor")
            .expect("MailingListCache actor died")
    }

    /// Persists the cache to the filesystem.
    pub async fn persist(&self) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Persist { tx })
            .await
            .context("Sending message to MailingListCache actor")
            .expect("MailingListCache actor died");
        rx.await
            .context("Awaiting response from MailingListCache actor")
            .expect("MailingListCache actor died")
    }

    /// Loads the cache from the filesystem.
    pub async fn load(&self) -> anyhow::Result<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Load { tx })
            .await
            .context("Sending message to MailingListCache actor")
            .expect("MailingListCache actor died");
        rx.await
            .context("Awaiting response from MailingListCache actor")
            .expect("MailingListCache actor died")
    }
}
