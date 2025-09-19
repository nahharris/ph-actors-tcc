use anyhow::Context;

mod core;
mod data;
mod mock;
pub mod message;

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
pub enum PatchCache {
    Actual(tokio::sync::mpsc::Sender<Message>),
    Mock(mock::Mock),
}

#[derive(Debug, Clone, Default)]
pub struct MockData {
    pub patches: std::collections::HashMap<String, String>,
}

impl PatchCache {
    /// Spawns a new PatchCache actor.
    pub async fn spawn(lore: LoreApi, fs: Fs, config: Config, log: Log) -> anyhow::Result<Self> {
        let core = core::Core::new(lore, fs, config, log).await?;
        let (state, _handle) = core.spawn();
        Ok(state)
    }

    /// Creates a new mock PatchCache actor for testing.
    pub fn mock(data: MockData) -> Self {
        Self::Mock(mock::Mock::new(data))
    }

    /// Fetches a patch by mailing list and message ID.
    pub async fn get(&self, list: ArcStr, message_id: ArcStr) -> anyhow::Result<String> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
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
            Self::Mock(mock) => {
                mock.get(list, message_id).await
            }
        }
    }

    /// Invalidates a specific patch.
    pub async fn invalidate(&self, list: ArcStr, message_id: ArcStr) -> anyhow::Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
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
            Self::Mock(mock) => {
                mock.invalidate(list, message_id).await
            }
        }
    }

    /// Checks if a patch is available in cache.
    pub async fn is_available(&self, list: ArcStr, message_id: ArcStr) -> bool {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
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
            Self::Mock(mock) => {
                mock.is_available(list, message_id).await
            }
        }
    }
}
