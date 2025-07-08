use anyhow::Context;
use tokio::sync::mpsc::Sender;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

use crate::{net::message::Message, ArcStr};

mod core;
mod message;

/// The networking actor that provides a thread-safe interface for network operations.
///
/// This enum represents either a real networking actor or a mock implementation
/// for testing purposes. It provides a unified interface for network operations
/// regardless of the underlying implementation.
///
/// # Examples
/// ```
/// let net = Net::spawn(config, log);
/// let response = net.get(url).await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender or mock reference.
#[derive(Debug, Clone)]
pub enum Net {
    /// A real networking actor that performs HTTP requests
    Actual(Sender<Message>),
    /// A mock implementation for testing
    Mock(Arc<Mutex<HashMap<ArcStr, ArcStr>>>),
}

impl Net {
    /// Creates a new networking instance and spawns its actor.
    ///
    /// # Arguments
    /// * `config` - The configuration actor for settings
    /// * `log` - The logging actor for operation logging
    ///
    /// # Returns
    /// A new networking instance with a spawned actor.
    pub fn spawn(config: crate::config::Config, log: crate::log::Log) -> Self {
        let (net, _) = core::Core::new(config, log).spawn();
        net
    }

    /// Creates a new mock networking instance for testing.
    ///
    /// # Arguments
    /// * `responses` - Optional initial response cache. If None, an empty cache will be used.
    ///
    /// # Returns
    /// A new mock networking instance that returns predefined responses.
    pub fn mock(responses: HashMap<ArcStr, ArcStr>) -> Self {
        Self::Mock(Arc::new(Mutex::new(responses)))
    }

    pub async fn get(&self, url: ArcStr) -> Result<ArcStr, anyhow::Error> {
        match self {
            Net::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender.send(Message::Get { url, tx }).await.context("Sending message to Net actor")?;
                rx.await.context("Receiving response from Net actor")?
            }
            Net::Mock(responses) => {
                let responses = responses.lock().await;
                responses.get(&url)
                    .map(ArcStr::clone)
                    .ok_or_else(|| anyhow::anyhow!("URL not found in mock responses: {}", url))
            }
        }
    }
}