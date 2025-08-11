use anyhow::Context;
use std::sync::Arc;
use tokio::sync::Mutex;

mod core;
mod data;
pub mod message;

use crate::api::lore::{LoreApi, LoreMailingList};
use crate::app::config::Config;
use crate::fs::Fs;
use crate::log::Log;
use message::Message;

/// The Mailing List Actor provides a cached list of mailing lists sorted alphabetically.
///
/// This actor fetches all mailing lists from the API, sorts them alphabetically,
/// and provides fast access to individual items or ranges. The cache is validated
/// based on the last updated time of the 0-th item from the API.
#[derive(Debug, Clone)]
pub enum MailingListCache {
    Actual(tokio::sync::mpsc::Sender<Message>),
    Mock(Arc<Mutex<MockData>>),
}

#[derive(Debug, Default)]
pub struct MockData {
    pub mailing_lists: Vec<LoreMailingList>,
}

impl MailingListCache {
    /// Spawns a new MailingListCache actor.
    pub async fn spawn(lore: LoreApi, fs: Fs, config: Config, log: Log) -> anyhow::Result<Self> {
        let core = core::Core::new(lore, fs, config, log).await?;
        let (state, _handle) = core.spawn();
        Ok(state)
    }

    /// Creates a new mock MailingListCache actor for testing.
    pub fn mock(data: MockData) -> Self {
        Self::Mock(Arc::new(Mutex::new(data)))
    }

    /// Fetches a single mailing list by index.
    pub async fn get(&self, index: usize) -> anyhow::Result<Option<LoreMailingList>> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Get { index, tx })
                    .await
                    .context("Sending message to MailingListCache actor")
                    .expect("MailingListCache actor died");
                rx.await
                    .context("Awaiting response from MailingListCache actor")
                    .expect("MailingListCache actor died")
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                Ok(data.mailing_lists.get(index).cloned())
            }
        }
    }

    /// Fetches a slice of mailing lists by range.
    pub async fn get_slice(
        &self,
        range: std::ops::Range<usize>,
    ) -> anyhow::Result<Vec<LoreMailingList>> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::GetSlice { range, tx })
                    .await
                    .context("Sending message to MailingListCache actor")
                    .expect("MailingListCache actor died");
                rx.await
                    .context("Awaiting response from MailingListCache actor")
                    .expect("MailingListCache actor died")
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                Ok(data.mailing_lists[range].to_vec())
            }
        }
    }

    /// Refreshes the cache by fetching from the API.
    pub async fn refresh(&self) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Refresh { tx })
                    .await
                    .context("Sending message to MailingListCache actor")
                    .expect("MailingListCache actor died");
                rx.await
                    .context("Awaiting response from MailingListCache actor")
                    .expect("MailingListCache actor died")
            }
            Self::Mock(_) => Ok(()),
        }
    }

    /// Invalidates the current cache.
    pub async fn invalidate(&self) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Invalidate { tx })
                    .await
                    .context("Sending message to MailingListCache actor")
                    .expect("MailingListCache actor died");
                rx.await
                    .context("Awaiting response from MailingListCache actor")
                    .expect("MailingListCache actor died")
            }
            Self::Mock(data) => {
                let mut data = data.lock().await;
                data.mailing_lists.clear();
                Ok(())
            }
        }
    }

    /// Checks if the requested range is available in cache.
    pub async fn is_available(&self, range: std::ops::Range<usize>) -> bool {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::IsAvailable { range, tx })
                    .await
                    .context("Sending message to MailingListCache actor")
                    .expect("MailingListCache actor died");
                rx.await
                    .context("Awaiting response from MailingListCache actor")
                    .expect("MailingListCache actor died")
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                range.end <= data.mailing_lists.len()
            }
        }
    }

    /// Returns the number of cached mailing lists.
    pub async fn len(&self) -> usize {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Len { tx })
                    .await
                    .context("Sending message to MailingListCache actor")
                    .expect("MailingListCache actor died");
                rx.await
                    .context("Awaiting response from MailingListCache actor")
                    .expect("MailingListCache actor died")
            }
            Self::Mock(data) => {
                let data = data.lock().await;
                data.mailing_lists.len()
            }
        }
    }

    /// Returns true if the cache is empty.
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    /// Persists the cache to the filesystem.
    pub async fn persist(&self) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Persist { tx })
                    .await
                    .context("Sending message to MailingListCache actor")
                    .expect("MailingListCache actor died");
                rx.await
                    .context("Awaiting response from MailingListCache actor")
                    .expect("MailingListCache actor died")
            }
            Self::Mock(_) => Ok(()),
        }
    }

    /// Loads the cache from the filesystem.
    pub async fn load(&self) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Load { tx })
                    .await
                    .context("Sending message to MailingListCache actor")
                    .expect("MailingListCache actor died");
                rx.await
                    .context("Awaiting response from MailingListCache actor")
                    .expect("MailingListCache actor died")
            }
            Self::Mock(_) => Ok(()),
        }
    }
}
