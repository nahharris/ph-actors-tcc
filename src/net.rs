use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

use crate::{ArcStr, error::FatalActorError, error::NetError, net::core::Core};
#[cfg(not(test))]
use crate::{app::config::Config, log::Log};
#[cfg(test)]
use crate::{app::config::mock::MockConfig as Config, log::mock::MockLog as Log};

mod core;
pub mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

/// Actor name for error reporting.
pub const ACTOR_NAME: &'static str = "Net";

/// The networking actor that provides a thread-safe interface for network operations.
///
/// This struct provides a unified interface for network operations
/// using message passing to a background actor.
///
/// # Examples
/// ```ignore
/// let net = Net::spawn();
/// let response = net.get(url).await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender.
#[derive(Debug, Clone)]
pub struct Net {
    tx: Sender<message::Message>,
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
    pub fn spawn(config: Config, log: Log) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(crate::BUFFER_SIZE);
        let core = Core::new(config, log);
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
    ) -> Result<ArcStr, NetError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::Get {
                url: url.clone(),
                headers,
                tx,
            })
            .await
            .map_err(|_e| {
                NetError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::net::ACTOR_NAME,
                    operation: "GET request".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            NetError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::net::ACTOR_NAME,
                operation: format!("GET request to {}", url),
                source: e,
            })
        })?
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
    ) -> Result<ArcStr, NetError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::Post {
                url: url.clone(),
                headers,
                body,
                tx,
            })
            .await
            .map_err(|_e| {
                NetError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::net::ACTOR_NAME,
                    operation: "POST request".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            NetError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::net::ACTOR_NAME,
                operation: format!("POST request to {}", url),
                source: e,
            })
        })?
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
    ) -> Result<ArcStr, NetError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::Put {
                url: url.clone(),
                headers,
                body,
                tx,
            })
            .await
            .map_err(|_e| {
                NetError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::net::ACTOR_NAME,
                    operation: "PUT request".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            NetError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::net::ACTOR_NAME,
                operation: format!("PUT request to {}", url),
                source: e,
            })
        })?
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
    ) -> Result<ArcStr, NetError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::Delete {
                url: url.clone(),
                headers,
                tx,
            })
            .await
            .map_err(|_e| {
                NetError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::net::ACTOR_NAME,
                    operation: "DELETE request".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            NetError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::net::ACTOR_NAME,
                operation: format!("DELETE request to {}", url),
                source: e,
            })
        })?
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
    ) -> Result<ArcStr, NetError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::Patch {
                url: url.clone(),
                headers,
                body,
                tx,
            })
            .await
            .map_err(|_e| {
                NetError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::net::ACTOR_NAME,
                    operation: "PATCH request".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            NetError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::net::ACTOR_NAME,
                operation: format!("PATCH request to {}", url),
                source: e,
            })
        })?
    }
}
