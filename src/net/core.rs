use anyhow::Context;
use reqwest::Client;

use crate::{config::Config, log::Log, net::{message::Message, Net}, ArcStr};

/// The core of the networking system that handles HTTP requests.
///
/// This struct provides thread-safe access to network operations through an actor pattern.
/// It wraps the reqwest HTTP client and provides a safe interface for making HTTP requests.
///
/// # Features
/// - Thread-safe network operations through actor pattern
/// - HTTP client with automatic connection pooling
/// - Integration with logging system
/// - Configuration-based settings
///
/// # Examples
/// ```
/// let net = NetCore::new(config, log);
/// let (net, _) = net.spawn()?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads through the actor pattern.
/// All network operations are handled sequentially to ensure consistency.
#[derive(Debug)]
pub struct NetCore {
    /// Configuration interface for settings
    config: Config,
    /// Logging interface for operation logging
    log: Log,
    /// HTTP client for making requests
    client: Client,
}

impl NetCore {
    /// Creates a new networking instance.
    ///
    /// # Arguments
    /// * `config` - The configuration actor for settings
    /// * `log` - The logging actor for operation logging
    ///
    /// # Returns
    /// A new instance of `NetCore` with a fresh HTTP client.
    pub fn new(config: Config, log: Log) -> Self {
        let client = Client::new();

        Self {
            config,
            log,
            client,
        }
    }

    /// Transforms the networking core instance into an actor.
    ///
    /// This method spawns a new task that will handle network operations
    /// asynchronously through a message channel. All operations are processed
    /// sequentially to ensure consistency.
    ///
    /// # Returns
    /// A tuple containing:
    /// - The `NetCore` instance
    /// - A join handle for the spawned task
    ///
    /// # Errors
    /// Returns an error if the task fails to spawn.
    ///
    /// # Panics
    /// This function will panic if the underlying task fails to spawn.
    pub fn spawn(self) -> anyhow::Result<(Net, tokio::task::JoinHandle<anyhow::Result<()>>)> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        
        let handle = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    Message::Get{url, tx} => {
                        let response = self.client.get::<&str>(url.as_ref())
                            .send()
                            .await
                            .context("Sending GET request")?;

                        let body = response.text()
                            .await
                            .context("Reading response body")?;
                        
                        tx.send(Ok(ArcStr::from(&body)));
                    }
                }
            }
            Ok(())
        });

        Ok((Net::Actual(tx), handle))
    }
}