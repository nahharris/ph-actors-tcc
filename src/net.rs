use anyhow::Context;
use tokio::sync::mpsc::Sender;

use crate::{net::message::Message, ArcStr};

mod core;
mod message;

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

impl Net {
    pub async fn get(&self, url: ArcStr) -> Result<ArcStr, anyhow::Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        match self {
            Net::Actual(sender) => {
                sender.send(Message::Get { url, tx }).await.context("Sending message to Net actor")?;
            }
            _ => return Err(anyhow::anyhow!("Not a real networking actor")),
        }

        let response = rx.await.context("Receiving response from Net actor")?;
        response
    }
}