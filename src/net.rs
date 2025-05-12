use reqwest::Client;
use tokio::sync::mpsc::Sender;

use crate::{config::Config, log::Log};

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
    pub fn spawn(mut self) -> anyhow::Result<(Self, tokio::task::JoinHandle<anyhow::Result<()>>)> {
        let handle = tokio::spawn(async move {
            while let Some(message) = self.receiver.recv().await {
                match message {
                    Message::Get(url) => {
                        let response = self.client.get(url).send().await?;
                    }
                }
            }
            Ok(())
        });
        Ok((self, handle))
    }
}

/// Messages that can be sent to a [`NetCore`] actor.
///
/// This enum defines the different types of network operations that can be performed
/// through the networking actor system.
#[derive(Debug)]
pub enum Message {
    /// Performs an HTTP GET request to the specified URL
    Get(String),
}

/// The networking actor that provides a thread-safe interface for network operations.
///
/// This enum represents a networking actor that can be used to perform HTTP requests
/// in a thread-safe manner.
///
/// # Examples
/// ```
/// let (net, _) = NetCore::new(config, log).spawn()?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender.
#[derive(Debug)]
pub enum Net {
    /// A real networking actor that performs HTTP requests
    Actual(Sender<Message>),
}
