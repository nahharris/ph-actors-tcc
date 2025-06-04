use std::{env::VarError, ffi::OsString};

use tokio::sync::mpsc;

use crate::{ArcOsStr, ArcStr};

use super::message::Message;

/// The core of the Env actor, responsible for handling environment variable operations.
///
/// This struct provides thread-safe access to environment variables through an actor pattern.
/// It wraps the standard library's environment variable functions and provides a safe interface
/// for concurrent access.
#[derive(Debug, Default)]
pub struct Core {}

impl Core {
    /// Creates a new Env core instance.
    ///
    /// # Returns
    /// A new instance of `Core` with default values.
    pub fn new() -> Self {
        Default::default()
    }

    /// Transforms an instance of [`Core`] into an actor ready to receive messages.
    ///
    /// This method spawns a new task that will handle environment variable operations
    /// asynchronously through a message channel.
    ///
    /// # Returns
    /// A tuple containing:
    /// - An [`Env`] instance that can be used to send messages to the actor
    /// - A join handle for the spawned task
    ///
    /// # Panics
    /// This function will panic if the underlying task fails to spawn.
    pub fn spawn(self) -> (super::Env, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel(crate::BUFFER_SIZE);
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

        (super::Env::Actual(tx), handle)
    }

    /// Sets an environment variable using the standard library.
    ///
    /// # Arguments
    /// * `key` - The environment variable name
    /// * `value` - The value to set
    ///
    /// # Safety
    /// This function is unsafe because it modifies global state. The caller must ensure
    /// that no other thread is concurrently modifying the same environment variable.
    pub fn set_env(&self, key: ArcOsStr, value: OsString) {
        unsafe {
            std::env::set_var(key, value);
        }
    }

    /// Removes an environment variable using the standard library.
    ///
    /// # Arguments
    /// * `key` - The environment variable name to remove
    ///
    /// # Safety
    /// This function is unsafe because it modifies global state. The caller must ensure
    /// that no other thread is concurrently modifying the same environment variable.
    pub fn unset_env(&self, key: ArcOsStr) {
        unsafe {
            std::env::remove_var(key);
        }
    }

    /// Gets an environment variable using the standard library and sends the result
    /// through the provided channel.
    ///
    /// # Arguments
    /// * `tx` - A oneshot channel sender to receive the result
    /// * `key` - The environment variable name to retrieve
    ///
    /// # Errors
    /// The function will return an error if the environment variable is not found
    /// or if there are any issues with the channel communication.
    pub fn get_env(
        &self,
        tx: tokio::sync::oneshot::Sender<Result<ArcStr, VarError>>,
        key: ArcOsStr,
    ) {
        let _ = tx.send(std::env::var(key).map(|s| ArcStr::from(&s)));
    }
}
