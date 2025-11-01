use anyhow::Context;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::log::Log;

mod core;
mod data;
mod message;
#[cfg(test)]
pub mod mock;

pub use data::{Screen, UiEvent};
use message::Message;

/// The terminal actor that owns the Cursive event loop and exposes a message-based API.
#[derive(Debug, Clone)]
pub struct Terminal {
    tx: mpsc::Sender<Message>,
}

impl Terminal {
    /// Spawns a terminal actor using the Cursive `crossterm` backend.
    ///
    /// The actor sends `UiEvent`s to `ui_events` and accepts `Message`s to update the UI.
    /// Returns the terminal interface and a JoinHandle that completes when the UI exits.
    pub fn spawn(log: Log, ui_events: mpsc::Sender<UiEvent>) -> (Self, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(log, ui_events);
        let handle = tokio::spawn(async move {
            core.init(rx).await;
        });
        (Self { tx }, handle)
    }

    /// Requests the terminal to show a specific screen.
    pub async fn show(&self, screen: Screen) {
        self.tx.send(Message::Show(screen))
            .await
            .context("Sending Show message to terminal")
            .expect("Terminal actor died");
    }

    /// Requests the terminal to quit.
    pub async fn quit(&self) {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Message::Quit { tx })
            .await
            .context("Sending Quit message to terminal")
            .expect("Terminal actor died");
        rx.await
            .context("Awaiting response for Quit message from terminal")
            .expect("Terminal actor died");
    }
}
