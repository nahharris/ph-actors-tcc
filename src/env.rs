use std::fmt::Display;

use tokio::sync::mpsc::{self, Sender};

use crate::{error::EnvError, error::FatalActorError, ArcOsStr, ArcStr};

mod core;
mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

/// Actor name for error reporting.
pub const ACTOR_NAME: &'static str = "Env";

/// The environment actor that provides a thread-safe interface for environment variable operations.
///
/// This struct provides a unified interface for environment variable operations
/// using message passing to a background actor.
///
/// # Examples
/// ```ignore
/// let env = Env::spawn();
/// let key = Arc::from(OsString::from("TEST_KEY"));
/// env.set_env(key.clone(), "test_value").await;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender.
#[derive(Debug, Clone)]
pub struct Env {
    tx: Sender<message::Message>,
}

impl Env {
    /// Creates a new environment instance and spawns its actor.
    ///
    /// # Returns
    /// A new environment instance with a spawned actor.
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let _ = tokio::spawn(async move {
            core::Core::new().init(rx).await;
        });
        Self { tx }
    }

    /// Sets an environment variable
    pub async fn set_env<V>(&self, key: ArcOsStr, value: V) -> Result<(), EnvError>
    where
        V: Display,
    {
        let value = format!("{value}").into();
        self.tx
            .send(message::Message::Set { key, value })
            .await
            .map_err(|_e| {
                EnvError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::env::ACTOR_NAME,
                    operation: "set environment variable".to_string(),
                })
            })
    }

    /// Unsets an environment variable
    pub async fn unset_env(&self, key: ArcOsStr) -> Result<(), EnvError> {
        self.tx
            .send(message::Message::Unset { key })
            .await
            .map_err(|_e| {
                EnvError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::env::ACTOR_NAME,
                    operation: "unset environment variable".to_string(),
                })
            })
    }

    /// Gets an environment variable
    pub async fn env(&self, key: ArcOsStr) -> Result<ArcStr, EnvError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::Get { tx, key })
            .await
            .map_err(|_e| {
                EnvError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::env::ACTOR_NAME,
                    operation: "get environment variable".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                EnvError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::env::ACTOR_NAME,
                    operation: "get environment variable".to_string(),
                    source: e,
                })
            })?
    }
}
