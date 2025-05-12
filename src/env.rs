use std::{env::VarError, ffi::OsString, fmt::Display, sync::Arc};

use anyhow::Context;
use tokio::sync::mpsc::Sender;

use crate::{ArcOsStr, ArcStr};

/// The core of the Env actor, responsible for handling environment variable operations
///
/// Interactions with environment variables are done through the standard library.
/// This struct is not meant to be used directly, but rather through [`Env`]
#[derive(Debug, Default)]
pub struct EnvCore {}

impl EnvCore {
    /// Creates a new Env core
    pub fn new() -> Self {
        Default::default()
    }

    /// Transforms an instance of [`EnvCore`] into an actor ready to receive messages
    pub fn spawn(self) -> (Env, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(crate::BUFFER_SIZE);
        let handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                use Message::*;
                match msg {
                    SetEnv { key, value } => self.set_env(key, value),
                    UnsetEnv { key } => self.unset_env(key),
                    GetEnv { tx, key } => self.get_env(tx, key),
                }
            }
        });

        (Env::Actual(tx), handle)
    }

    /// Sets an environment variable using the standard library
    ///
    /// # Safety
    /// This function is unsafe because it modifies global state
    fn set_env(&self, key: ArcOsStr, value: OsString) {
        unsafe {
            std::env::set_var(key, value);
        }
    }

    /// Removes an environment variable using the standard library
    ///
    /// # Safety
    /// This function is unsafe because it modifies global state
    fn unset_env(&self, key: ArcOsStr) {
        unsafe {
            std::env::remove_var(key);
        }
    }

    /// Gets an environment variable using the standard library and sends the result
    /// through the provided channel
    fn get_env(&self, tx: tokio::sync::oneshot::Sender<Result<ArcStr, VarError>>, key: ArcOsStr) {
        let _ = tx.send(std::env::var(key).map(Arc::from));
    }
}

/// Messages that can be sent to an [`EnvCore`]
#[derive(Debug)]
pub enum Message {
    SetEnv {
        key: ArcOsStr,
        value: OsString,
    },
    UnsetEnv {
        key: ArcOsStr,
    },
    GetEnv {
        tx: tokio::sync::oneshot::Sender<Result<ArcStr, VarError>>,
        key: ArcOsStr,
    },
}

/// A mock implementation of the Env actor, used for testing
/// This implementation won't actually interact with the OS, but rather store
/// the state in memory.
#[derive(Debug, Clone, Default)]
pub struct EnvMock {
    env: std::collections::HashMap<ArcOsStr, OsString>,
}

/// The env actor is responsible for handling environment variable operations
/// This is a transmitter to a running actor, and can be used to send messages
///
/// Cloning this actor is cheap, since it's just a transmitter
///
/// Instantiate a new env actor with [`EnvCore::spawn`]
#[derive(Debug, Clone)]
pub enum Env {
    Actual(Sender<Message>),
    Mock(Arc<tokio::sync::Mutex<EnvMock>>),
}

impl From<EnvCore> for Env {
    fn from(core: EnvCore) -> Self {
        let (env, _) = core.spawn();
        env
    }
}

use Env::*;
#[allow(dead_code)]
impl Env {
    /// Creates a new mock instance of the Env actor for testing
    pub fn mock() -> Self {
        Mock(Arc::new(tokio::sync::Mutex::new(EnvMock::default())))
    }

    /// Sets an environment variable
    pub async fn set_env<V>(&self, key: ArcOsStr, value: V)
    where
        V: Display,
    {
        let value = format!("{}", value).into();
        match self {
            Actual(sender) => sender
                .send(Message::SetEnv { key, value })
                .await
                .context("Setting environment variable with Env")
                .expect("env actor died"),

            Mock(lock) => {
                let mut lock = lock.lock().await;
                lock.env.insert(key, value);
            }
        }
    }

    /// Unsets an environment variable
    pub async fn unset_env(&self, key: ArcOsStr) {
        match self {
            Actual(sender) => sender
                .send(Message::UnsetEnv { key })
                .await
                .context("Unsetting environment variable with Env")
                .expect("env actor died"),
            Mock(lock) => {
                let mut lock = lock.lock().await;
                lock.env.remove(&key);
            }
        }
    }

    /// Gets an environment variable
    pub async fn env(&self, key: ArcOsStr) -> Result<ArcStr, VarError> {
        match self {
            Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::GetEnv { tx, key })
                    .await
                    .context("Getting environment variable with Env")
                    .expect("env actor died");
                rx.await
                    .context("Awaiting response for environment variable get with Env")
                    .expect("env actor died")
            }
            Mock(lock) => {
                let lock = lock.lock().await;
                lock.env
                    .get(&key)
                    .map(|s| s.to_string_lossy().to_string().into())
                    .ok_or(VarError::NotPresent)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_env_set_get() {
        let (env, _) = EnvCore::new().spawn();
        let key: ArcOsStr = Arc::from(OsString::from("TEST_ENV_SET_GET"));
        let value = "test_value";

        // Remove env var if it exists
        unsafe { std::env::remove_var(key.as_ref()) };
        
        // Verify it's not set in std::env
        assert!(std::env::var(key.as_ref()).is_err());

        // Set and verify through our Env actor
        env.set_env(key.clone(), value).await;
        let result = env.env(key.clone()).await.unwrap();
        assert_eq!(result.as_ref(), value);

        // Verify it's also set in std::env
        let std_result = std::env::var(key.as_ref()).unwrap();
        assert_eq!(std_result, value);
    }

    #[tokio::test]
    async fn test_env_unset() {
        let env: Env = EnvCore::new().into();
        let key: ArcOsStr = Arc::from(OsString::from("TEST_ENV_UNSET"));
        let value = "test_value";

        unsafe { std::env::set_var(key.as_ref(), value) };
        env.unset_env(key.clone()).await;
        let result = env.env(key.clone()).await;
        assert!(matches!(result, Err(VarError::NotPresent)));

        // Verify it's also unset in std::env
        let std_result = std::env::var(key.as_ref());
        assert!(matches!(std_result, Err(VarError::NotPresent)));
    }
}
