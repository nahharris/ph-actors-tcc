use anyhow::Context;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

use crate::{
    ArcStr,
    app::config::Config,
    net::{
        core::Core,
        message::Message,
    },
};

mod core;
mod mock;
pub mod message;

/// The networking actor that provides a thread-safe interface for network operations.
///
/// This enum represents either a real networking actor or a mock implementation
/// for testing purposes. It provides a unified interface for network operations
/// regardless of the underlying implementation.
///
/// # Examples
/// ```ignore
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
    Mock(mock::Mock),
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
    pub async fn spawn(config: Config, log: crate::log::Log) -> Self {
        let (net, _) = Core::new(config, log).await.spawn();
        net
    }

    /// Creates a new mock networking instance for testing.
    ///
    /// # Arguments
    /// * `responses` - Initial response cache mapping HTTP method + URL pairs to responses
    ///
    /// # Returns
    /// A new mock networking instance that returns predefined responses.
    pub fn mock(responses: HashMap<crate::net::message::MockRequestKey, ArcStr>) -> Self {
        Self::Mock(mock::Mock::new(responses))
    }

    /// Creates a new empty mock networking instance for testing.
    ///
    /// # Returns
    /// A new mock networking instance with an empty response cache.
    pub fn mock_empty() -> Self {
        Self::Mock(mock::Mock::empty())
    }

    /// Performs an HTTP GET request to the specified URL.
    ///
    /// # Arguments
    /// * `url` - The URL to send the GET request to
    /// * `headers` - Optional headers to include in the request
    ///
    /// # Returns
    /// The response body as a string, or an error if the request fails.
    pub async fn get(
        &self,
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
    ) -> Result<ArcStr, anyhow::Error> {
        match self {
            Net::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Get { url, headers, tx })
                    .await
                    .context("Sending message to Net actor")
                    .expect("Net actor died");
                rx.await
                    .context("Awaiting response from Net actor")
                    .expect("Net actor died")
            }
            Net::Mock(mock) => {
                mock.get(url, headers).await
            }
        }
    }

    /// Performs an HTTP POST request to the specified URL.
    ///
    /// # Arguments
    /// * `url` - The URL to send the POST request to
    /// * `headers` - Optional headers to include in the request
    /// * `body` - Optional body content to send with the request
    ///
    /// # Returns
    /// The response body as a string, or an error if the request fails.
    pub async fn post(
        &self,
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
        body: Option<ArcStr>,
    ) -> Result<ArcStr, anyhow::Error> {
        match self {
            Net::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Post {
                        url,
                        headers,
                        body,
                        tx,
                    })
                    .await
                    .context("Sending message to Net actor")?;
                rx.await.context("Receiving response from Net actor")?
            }
            Net::Mock(mock) => {
                mock.post(url, headers, body).await
            }
        }
    }

    /// Performs an HTTP PUT request to the specified URL.
    ///
    /// # Arguments
    /// * `url` - The URL to send the PUT request to
    /// * `headers` - Optional headers to include in the request
    /// * `body` - Optional body content to send with the request
    ///
    /// # Returns
    /// The response body as a string, or an error if the request fails.
    pub async fn put(
        &self,
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
        body: Option<ArcStr>,
    ) -> Result<ArcStr, anyhow::Error> {
        match self {
            Net::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Put {
                        url,
                        headers,
                        body,
                        tx,
                    })
                    .await
                    .context("Sending message to Net actor")?;
                rx.await.context("Receiving response from Net actor")?
            }
            Net::Mock(mock) => {
                mock.put(url, headers, body).await
            }
        }
    }

    /// Performs an HTTP DELETE request to the specified URL.
    ///
    /// # Arguments
    /// * `url` - The URL to send the DELETE request to
    /// * `headers` - Optional headers to include in the request
    ///
    /// # Returns
    /// The response body as a string, or an error if the request fails.
    pub async fn delete(
        &self,
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
    ) -> Result<ArcStr, anyhow::Error> {
        match self {
            Net::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Delete { url, headers, tx })
                    .await
                    .context("Sending message to Net actor")?;
                rx.await.context("Receiving response from Net actor")?
            }
            Net::Mock(mock) => {
                mock.delete(url, headers).await
            }
        }
    }

    /// Performs an HTTP PATCH request to the specified URL.
    ///
    /// # Arguments
    /// * `url` - The URL to send the PATCH request to
    /// * `headers` - Optional headers to include in the request
    /// * `body` - Optional body content to send with the request
    ///
    /// # Returns
    /// The response body as a string, or an error if the request fails.
    pub async fn patch(
        &self,
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
        body: Option<ArcStr>,
    ) -> Result<ArcStr, anyhow::Error> {
        match self {
            Net::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(Message::Patch {
                        url,
                        headers,
                        body,
                        tx,
                    })
                    .await
                    .context("Sending message to Net actor")?;
                rx.await.context("Receiving response from Net actor")?
            }
            Net::Mock(mock) => {
                mock.patch(url, headers, body).await
            }
        }
    }
}
