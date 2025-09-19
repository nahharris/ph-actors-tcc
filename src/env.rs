use std::{env::VarError, fmt::Display};

use anyhow::Context;
use tokio::sync::mpsc::Sender;

use crate::{ArcOsStr, ArcStr};

mod core;
mod mock;
mod message;
#[cfg(test)]
mod tests;

/// The environment actor that provides a thread-safe interface for environment variable operations.
///
/// This enum represents either a real environment actor or a mock implementation
/// for testing purposes. It provides a unified interface for environment variable operations
/// regardless of the underlying implementation.
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
/// copies the channel sender or mock reference.
#[derive(Debug, Clone)]
pub enum Env {
    /// A real environment variable actor that interacts with the system
    Actual(Sender<message::Message>),
    /// A mock implementation for testing
    Mock(mock::Mock),
}

impl Env {
    /// Creates a new environment instance and spawns its actor.
    ///
    /// # Returns
    /// A new environment instance with a spawned actor.
    pub fn spawn() -> Self {
        let (env, _) = core::Core::new().spawn();
        env
    }

    /// Creates a new mock environment instance for testing.
    ///
    /// # Returns
    /// A new mock environment instance that stores variables in memory.
    pub fn mock() -> Self {
        Self::Mock(mock::Mock::new())
    }

    /// Sets an environment variable
    pub async fn set_env<V>(&self, key: ArcOsStr, value: V)
    where
        V: Display,
    {
        match self {
            Self::Actual(sender) => {
                let value = format!("{value}").into();
                sender
                    .send(message::Message::Set { key, value })
                    .await
                    .context("Setting environment variable with Env")
                    .expect("env actor died")
            }
            Self::Mock(mock) => {
                mock.set_env(key, value).await;
            }
        }
    }

    /// Unsets an environment variable
    pub async fn unset_env(&self, key: ArcOsStr) {
        match self {
            Self::Actual(sender) => sender
                .send(message::Message::Unset { key })
                .await
                .context("Unsetting environment variable with Env")
                .expect("env actor died"),
            Self::Mock(mock) => {
                mock.unset_env(key).await;
            }
        }
    }

    /// Gets an environment variable
    pub async fn env(&self, key: ArcOsStr) -> Result<ArcStr, VarError> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(message::Message::Get { tx, key })
                    .await
                    .context("Getting environment variable with Env")
                    .expect("env actor died");
                rx.await
                    .context("Awaiting response for environment variable get with Env")
                    .expect("env actor died")
            }
            Self::Mock(mock) => {
                mock.env(key).await
            }
        }
    }
}
