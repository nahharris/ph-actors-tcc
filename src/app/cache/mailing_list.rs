use tokio::sync::mpsc;

mod core;
mod data;
pub mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

/// Actor name for error reporting.
pub const ACTOR_NAME: &'static str = "MailingListCache";

use crate::{api::lore::LoreMailingList, error::CacheError, error::FatalActorError};
use message::Message;

#[cfg(not(test))]
use crate::{api::LoreApi, app::config::Config, fs::Fs, log::Log};
#[cfg(test)]
use crate::{
    api::lore::mock::MockLoreApi as LoreApi, app::config::mock::MockConfig as Config,
    fs::mock::MockFs as Fs, log::mock::MockLog as Log,
};

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

    /// Fetches a single mailing list by index.
    pub async fn get(&self, index: usize) -> Result<Option<LoreMailingList>, CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Get { index, tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                    operation: "get".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                operation: "get".to_string(),
                source: e,
            })
        })?
    }

    /// Fetches a slice of mailing lists by range.
    pub async fn get_slice(
        &self,
        range: std::ops::Range<usize>,
    ) -> Result<Vec<LoreMailingList>, CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::GetSlice { range, tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                    operation: "get slice".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                operation: "get slice".to_string(),
                source: e,
            })
        })?
    }

    /// Refreshes the cache by fetching from the API.
    pub async fn refresh(&self) -> Result<(), CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Message::Refresh { tx }).await.map_err(|_e| {
            CacheError::Fatal(FatalActorError::ActorSendFailed {
                actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                operation: "refresh".to_string(),
            })
        })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                operation: "refresh".to_string(),
                source: e,
            })
        })?
    }

    /// Invalidates the current cache.
    pub async fn invalidate(&self) -> Result<(), CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Invalidate { tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                    operation: "invalidate".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                operation: "invalidate".to_string(),
                source: e,
            })
        })?
    }

    /// Checks if the requested range is available in cache.
    pub async fn is_available(&self, range: std::ops::Range<usize>) -> Result<bool, CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::IsAvailable { range, tx })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                    operation: "is available".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                operation: "is available".to_string(),
                source: e,
            })
        })?
    }

    /// Returns the number of cached mailing lists.
    pub async fn len(&self) -> Result<usize, CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Message::Len { tx }).await.map_err(|_e| {
            CacheError::Fatal(FatalActorError::ActorSendFailed {
                actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                operation: "len".to_string(),
            })
        })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                operation: "len".to_string(),
                source: e,
            })
        })?
    }

    /// Persists the cache to the filesystem.
    pub async fn persist(&self) -> Result<(), CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Message::Persist { tx }).await.map_err(|_e| {
            CacheError::Fatal(FatalActorError::ActorSendFailed {
                actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                operation: "persist".to_string(),
            })
        })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                operation: "persist".to_string(),
                source: e,
            })
        })?
    }

    /// Loads the cache from the filesystem.
    pub async fn load(&self) -> Result<(), CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx.send(Message::Load { tx }).await.map_err(|_e| {
            CacheError::Fatal(FatalActorError::ActorSendFailed {
                actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                operation: "load".to_string(),
            })
        })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::mailing_list::ACTOR_NAME,
                operation: "load".to_string(),
                source: e,
            })
        })?
    }
}
