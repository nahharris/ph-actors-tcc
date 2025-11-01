use anyhow::Context;
use std::sync::Arc;
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
    /// Join handle that completes when the terminal exits
    handle: Arc<JoinHandle<()>>,
}

impl Terminal {
    /// Spawns a terminal actor using the Cursive `crossterm` backend.
    ///
    /// The actor stores `UiEvent`s in an internal FIFO queue and accepts `Message`s to update the UI.
    /// Returns the terminal interface with the join handle stored as a field.
    pub fn spawn(log: Log) -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(log);
        let handle = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self {
            tx,
            handle: Arc::new(handle),
        }
    }

    /// Get a reference to the join handle for awaiting terminal completion
    pub fn handle(&self) -> &Arc<JoinHandle<()>> {
        &self.handle
    }

    /// Requests the terminal to show a specific screen.
    pub async fn show(&self, screen: Screen) {
        self.tx
            .send(Message::Show(screen))
            .await
            .context("Sending Show message to terminal")
            .expect("Terminal actor died");
    }

    /// Get the next UI event from the terminal's internal queue (pops the head).
    pub async fn get_ui_event(&self) -> Option<UiEvent> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::GetUiEvent { tx })
            .await
            .context("Sending GetUiEvent message to terminal")
            .expect("Terminal actor died");
        rx.await
            .context("Awaiting response for GetUiEvent from terminal")
            .expect("Terminal actor died")
    }

    /// Clear all UI events from the terminal's internal queue.
    pub async fn clear_ui_events(&self) {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::ClearUiEvents { tx })
            .await
            .context("Sending ClearUiEvents message to terminal")
            .expect("Terminal actor died");
        rx.await
            .context("Awaiting response for ClearUiEvents from terminal")
            .expect("Terminal actor died");
    }

    /// Requests the terminal to quit.
    pub async fn quit(&self) {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::Quit { tx })
            .await
            .context("Sending Quit message to terminal")
            .expect("Terminal actor died");
        rx.await
            .context("Awaiting response for Quit message from terminal")
            .expect("Terminal actor died");
    }
}
