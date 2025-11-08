use tokio::sync::mpsc;

mod core;
mod data;
pub mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

/// Actor name for error reporting.
pub const ACTOR_NAME: &'static str = "FeedCache";

use crate::api::lore::LorePatchMetadata;
use crate::{ArcStr, error::CacheError, error::FatalActorError};
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
    pub async fn spawn(
        lore: LoreApi,
        fs: Fs,
        config: Config,
        log: Log,
    ) -> Result<Self, CacheError> {
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
    ) -> Result<Option<LorePatchMetadata>, CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Get { list, index, tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::feed::ACTOR_NAME,
                    operation: "get patch metadata".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::feed::ACTOR_NAME,
                operation: "get patch metadata".to_string(),
                source: e,
            })
        })?
    }

    /// Fetches a slice of patch metadata items by range for a given mailing list.
    pub async fn get_slice(
        &self,
        list: ArcStr,
        range: std::ops::Range<usize>,
    ) -> Result<Vec<LorePatchMetadata>, CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::GetSlice { list, range, tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::feed::ACTOR_NAME,
                    operation: "get slice".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::feed::ACTOR_NAME,
                operation: "get slice".to_string(),
                source: e,
            })
        })?
    }

    /// Refreshes the cache for a specific mailing list.
    pub async fn refresh(&self, list: ArcStr) -> Result<(), CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Refresh { list, tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::feed::ACTOR_NAME,
                    operation: "refresh".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::feed::ACTOR_NAME,
                operation: "refresh".to_string(),
                source: e,
            })
        })?
    }

    /// Invalidates the cache for a specific mailing list.
    pub async fn invalidate(&self, list: ArcStr) -> Result<(), CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Invalidate { list, tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::feed::ACTOR_NAME,
                    operation: "invalidate".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::feed::ACTOR_NAME,
                operation: "invalidate".to_string(),
                source: e,
            })
        })?
    }

    /// Checks if the requested range is available in cache for a mailing list.
    pub async fn is_available(
        &self,
        list: ArcStr,
        range: std::ops::Range<usize>,
    ) -> Result<bool, CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::IsAvailable { list, range, tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::feed::ACTOR_NAME,
                    operation: "is available".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::feed::ACTOR_NAME,
                operation: "is available".to_string(),
                source: e,
            })
        })?
    }

    /// Returns the number of cached items for a given mailing list.
    pub async fn len(&self, list: ArcStr) -> Result<usize, CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Len { list, tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::feed::ACTOR_NAME,
                    operation: "len".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::feed::ACTOR_NAME,
                operation: "len".to_string(),
                source: e,
            })
        })?
    }

    /// Checks if the cache has been loaded from disk for a given mailing list.
    /// This is different from is_empty() - a cache can be loaded but empty.
    pub async fn is_loaded(&self, list: ArcStr) -> Result<bool, CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::IsLoaded { list, tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::feed::ACTOR_NAME,
                    operation: "is loaded".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::feed::ACTOR_NAME,
                operation: "is loaded".to_string(),
                source: e,
            })
        })?
    }

    /// Ensures the cache is loaded for a given mailing list.
    /// This will load from disk if not already loaded.
    pub async fn ensure_loaded(&self, list: ArcStr) -> Result<(), CacheError> {
        if !self.is_loaded(list.clone()).await? {
            self.load(list).await?;
        }
        Ok(())
    }

    /// Persists the cache for a specific mailing list to the filesystem.
    pub async fn persist(&self, list: ArcStr) -> Result<(), CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Persist { list, tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::feed::ACTOR_NAME,
                    operation: "persist".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::feed::ACTOR_NAME,
                operation: "persist".to_string(),
                source: e,
            })
        })?
    }

    /// Loads the cache for a specific mailing list from the filesystem.
    pub async fn load(&self, list: ArcStr) -> Result<(), CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Load { list, tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::feed::ACTOR_NAME,
                    operation: "load".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::feed::ACTOR_NAME,
                operation: "load".to_string(),
                source: e,
            })
        })?
    }
}
