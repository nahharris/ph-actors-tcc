use anyhow::Context;
use reqwest::Client;
use std::collections::HashMap;

#[cfg(not(test))]
use crate::{app::config::Config, log::Log};
#[cfg(test)]
use crate::{app::config::mock::MockConfig as Config, log::mock::MockLog as Log};

use crate::{ArcStr, net::message::Message};

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
/// ```ignore
/// let core = Core::new();
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads through the actor pattern.
/// All network operations are handled sequentially to ensure consistency.
#[derive(Debug)]
pub struct Core {
    /// Configuration interface for settings
    config: Config,
    /// Logging interface for operation logging
    log: Log,
    /// HTTP client for making requests
    client: Client,
}

impl Core {
    /// Creates a new networking instance.
    ///
    /// # Arguments
    /// * `config` - The configuration actor for settings
    /// * `log` - The logging actor for operation logging
    ///
    /// # User Agent
    /// All requests use the user agent string.
    ///
    /// # Timeout
    /// The timeout for network requests is configured via the config (default: 30 seconds).
    ///
    /// # Returns
    /// A new instance of `Core` with a fresh HTTP client.
    pub fn new(config: Config, log: Log) -> Self {
        // Get timeout from config - this will be async, so we use a simple default for now
        // In a real implementation, we might need to make this async or use a lazy approach
        let timeout_secs = 30u64; // Default timeout

        let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .use_rustls_tls()
            .build()
            .expect("Failed to build reqwest client with user agent and timeout");

        Self {
            config,
            log,
            client,
        }
    }

    /// Initializes the networking actor message receiver.
    ///
    /// This method processes messages from the receiver in a loop, handling each message
    /// using pattern matching.
    ///
    /// # Arguments
    /// * `rx` - A receiver for messages to process
    pub async fn init(self, mut rx: tokio::sync::mpsc::Receiver<Message>) {
        while let Some(msg) = rx.recv().await {
            use Message::*;
            match msg {
                Get { url, headers, tx } => {
                    let response = self
                        .handle_get_request(url.clone(), headers)
                        .await
                        .with_context(|| format!("GET request failed for URL: {url}"));
                    let _ = tx.send(response);
                }
                Post {
                    url,
                    headers,
                    body,
                    tx,
                } => {
                    let response = self
                        .handle_post_request(url.clone(), headers, body)
                        .await
                        .with_context(|| format!("POST request failed for URL: {url}"));
                    let _ = tx.send(response);
                }
                Put {
                    url,
                    headers,
                    body,
                    tx,
                } => {
                    let response = self
                        .handle_put_request(url.clone(), headers, body)
                        .await
                        .with_context(|| format!("PUT request failed for URL: {url}"));
                    let _ = tx.send(response);
                }
                Delete { url, headers, tx } => {
                    let response = self
                        .handle_delete_request(url.clone(), headers)
                        .await
                        .with_context(|| format!("DELETE request failed for URL: {url}"));
                    let _ = tx.send(response);
                }
                Patch {
                    url,
                    headers,
                    body,
                    tx,
                } => {
                    let response = self
                        .handle_patch_request(url.clone(), headers, body)
                        .await
                        .with_context(|| format!("PATCH request failed for URL: {url}"));
                    let _ = tx.send(response);
                }
            }
        }
    }

    /// Handles GET requests with optional headers
    async fn handle_get_request(
        &self,
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
    ) -> anyhow::Result<ArcStr> {
        let mut request = self.client.get::<&str>(url.as_ref());

        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(
                    <ArcStr as AsRef<str>>::as_ref(&key),
                    <ArcStr as AsRef<str>>::as_ref(&value),
                );
            }
        }

        let response = request.send().await.context("Sending GET request")?;
        let text = response.text().await.context("Reading response body")?;
        Ok(ArcStr::from(&text))
    }

    /// Handles POST requests with optional headers and body
    async fn handle_post_request(
        &self,
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
        body: Option<ArcStr>,
    ) -> anyhow::Result<ArcStr> {
        let mut request = self.client.post::<&str>(url.as_ref());

        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(
                    <ArcStr as AsRef<str>>::as_ref(&key),
                    <ArcStr as AsRef<str>>::as_ref(&value),
                );
            }
        }

        if let Some(body) = body {
            request = request.body(<ArcStr as AsRef<str>>::as_ref(&body).to_string());
        }

        let response = request.send().await.context("Sending POST request")?;
        let text = response.text().await.context("Reading response body")?;
        Ok(ArcStr::from(&text))
    }

    /// Handles PUT requests with optional headers and body
    async fn handle_put_request(
        &self,
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
        body: Option<ArcStr>,
    ) -> anyhow::Result<ArcStr> {
        let mut request = self.client.put::<&str>(url.as_ref());

        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(
                    <ArcStr as AsRef<str>>::as_ref(&key),
                    <ArcStr as AsRef<str>>::as_ref(&value),
                );
            }
        }

        if let Some(body) = body {
            request = request.body(<ArcStr as AsRef<str>>::as_ref(&body).to_string());
        }

        let response = request.send().await.context("Sending PUT request")?;
        let text = response.text().await.context("Reading response body")?;
        Ok(ArcStr::from(&text))
    }

    /// Handles DELETE requests with optional headers
    async fn handle_delete_request(
        &self,
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
    ) -> anyhow::Result<ArcStr> {
        let mut request = self.client.delete::<&str>(url.as_ref());

        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(
                    <ArcStr as AsRef<str>>::as_ref(&key),
                    <ArcStr as AsRef<str>>::as_ref(&value),
                );
            }
        }

        let response = request.send().await.context("Sending DELETE request")?;
        let text = response.text().await.context("Reading response body")?;
        Ok(ArcStr::from(&text))
    }

    /// Handles PATCH requests with optional headers and body
    async fn handle_patch_request(
        &self,
        url: ArcStr,
        headers: Option<HashMap<ArcStr, ArcStr>>,
        body: Option<ArcStr>,
    ) -> anyhow::Result<ArcStr> {
        let mut request = self.client.patch::<&str>(url.as_ref());

        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(
                    <ArcStr as AsRef<str>>::as_ref(&key),
                    <ArcStr as AsRef<str>>::as_ref(&value),
                );
            }
        }

        if let Some(body) = body {
            request = request.body(<ArcStr as AsRef<str>>::as_ref(&body).to_string());
        }

        let response = request.send().await.context("Sending PATCH request")?;
        let text = response.text().await.context("Reading response body")?;
        Ok(ArcStr::from(&text))
    }
}
