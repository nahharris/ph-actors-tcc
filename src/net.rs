use anyhow::Context;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

use crate::{
    ArcStr,
    app::config::Config,
    net::{core::Core, message::Message},
};

mod core;
pub mod message;
mod mock;

/// The networking actor that provides a thread-safe interface for network operations.
///
/// This struct provides a unified interface for network operations
/// using message passing to a background actor.
///
/// # Examples
/// ```ignore
/// let net = Net::spawn(config, log);
/// let response = net.get(url).await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender.
#[derive(Debug, Clone)]
pub struct Net {
    tx: Sender<Message>,
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
        let (tx, rx) = tokio::sync::mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new(config, log).await;
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
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
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Get { url, headers, tx })
            .await
            .context("Sending message to Net actor")
            .expect("Net actor died");
        rx.await
            .context("Awaiting response from Net actor")
            .expect("Net actor died")
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
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
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
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
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
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(Message::Delete { url, headers, tx })
            .await
            .context("Sending message to Net actor")?;
        rx.await.context("Receiving response from Net actor")?
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
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
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
}
