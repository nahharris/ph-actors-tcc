use anyhow::Context;
use reqwest::Client;
use std::collections::HashMap;
use tokio::task::JoinHandle;

use crate::{
    config::{Config, USizeOpt}, log::Log, net::{message::Message, Net}, ArcStr
};

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
/// let core = Core::new(config, log);
/// let (net, _) = core.spawn();
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
    pub async fn new(config: Config, log: Log) -> Self {
        // Try to get timeout from config synchronously if possible
        let timeout_secs = config.usize(USizeOpt::Timeout).await as u64;
        let user_agent = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        );
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

    /// Transforms the networking core instance into an actor.
    ///
    /// This method spawns a new task that will handle network operations
    /// asynchronously through a message channel. All operations are processed
    /// sequentially to ensure consistency.
    ///
    /// # Returns
    /// A tuple containing:
    /// - The `Net` interface
    /// - A join handle for the spawned task
    ///
    /// # Panics
    /// This function will panic if the underlying task fails to spawn.
    pub fn spawn(self) -> (Net, JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let handle = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    Message::Get { url, headers, tx } => {
                        let response = self
                            .handle_get_request(url.clone(), headers)
                            .await
                            .with_context(|| format!("GET request failed for URL: {}", url));
                        let _ = tx.send(response);
                    }
                    Message::Post {
                        url,
                        headers,
                        body,
                        tx,
                    } => {
                        let response = self
                            .handle_post_request(url.clone(), headers, body)
                            .await
                            .with_context(|| format!("POST request failed for URL: {}", url));
                        let _ = tx.send(response);
                    }
                    Message::Put {
                        url,
                        headers,
                        body,
                        tx,
                    } => {
                        let response = self
                            .handle_put_request(url.clone(), headers, body)
                            .await
                            .with_context(|| format!("PUT request failed for URL: {}", url));
                        let _ = tx.send(response);
                    }
                    Message::Delete { url, headers, tx } => {
                        let response = self
                            .handle_delete_request(url.clone(), headers)
                            .await
                            .with_context(|| format!("DELETE request failed for URL: {}", url));
                        let _ = tx.send(response);
                    }
                    Message::Patch {
                        url,
                        headers,
                        body,
                        tx,
                    } => {
                        let response = self
                            .handle_patch_request(url.clone(), headers, body)
                            .await
                            .with_context(|| format!("PATCH request failed for URL: {}", url));
                        let _ = tx.send(response);
                    }
                }
            }
        });

        (Net::Actual(tx), handle)
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
