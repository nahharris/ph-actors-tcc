use tokio::sync::mpsc;

mod core;
mod data;
pub mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

/// Actor name for error reporting.
pub const ACTOR_NAME: &'static str = "PatchCache";

use crate::{ArcStr, error::CacheError, error::FatalActorError};
use message::Message;

#[cfg(not(test))]
use crate::{api::lore::LoreApi, app::config::Config, fs::Fs, log::Log};
#[cfg(test)]
use crate::{
    api::lore::mock::MockLoreApi as LoreApi, app::config::mock::MockConfig as Config,
    fs::mock::MockFs as Fs, log::mock::MockLog as Log,
};

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

    /// Fetches a patch by mailing list and message ID.
    pub async fn get(&self, list: ArcStr, message_id: ArcStr) -> Result<String, CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Get {
                list,
                message_id,
                tx,
            })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::patch::ACTOR_NAME,
                    operation: "get".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::patch::ACTOR_NAME,
                operation: "get".to_string(),
                source: e,
            })
        })?
    }

    /// Invalidates a specific patch.
    pub async fn invalidate(&self, list: ArcStr, message_id: ArcStr) -> Result<(), CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Invalidate {
                list,
                message_id,
                tx,
            })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::patch::ACTOR_NAME,
                    operation: "invalidate".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::patch::ACTOR_NAME,
                operation: "invalidate".to_string(),
                source: e,
            })
        })?
    }

    /// Checks if a patch is available in cache.
    pub async fn is_available(&self, list: ArcStr, message_id: ArcStr) -> Result<bool, CacheError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::IsAvailable {
                list,
                message_id,
                tx,
            })
            .await
            .map_err(|_e| {
                CacheError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::cache::patch::ACTOR_NAME,
                    operation: "is available".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            CacheError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::cache::patch::ACTOR_NAME,
                operation: "is available".to_string(),
                source: e,
            })
        })?
    }
}
