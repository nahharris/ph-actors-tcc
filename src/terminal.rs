use anyhow::Context;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::log::Log;

mod core;
mod data;
mod mock;
mod message;

use data::MockData;
pub use data::{Screen, UiEvent};
use message::Message;

/// The terminal actor that owns the Cursive event loop and exposes a message-based API.
#[derive(Debug, Clone)]
pub enum Terminal {
    Actual(mpsc::Sender<Message>),
    Mock(mock::Mock),
}

impl Terminal {
    /// Spawns a terminal actor using the Cursive `crossterm` backend.
    ///
    /// The actor sends `UiEvent`s to `ui_events` and accepts `Message`s to update the UI.
    /// Returns the terminal interface and a JoinHandle that completes when the UI exits.
    pub fn spawn(log: Log, ui_events: mpsc::Sender<UiEvent>) -> (Self, JoinHandle<()>) {
        let core = core::Core::new(log, ui_events);
        core.spawn()
    }

    /// Creates a mock terminal for testing.
    pub fn mock(data: MockData) -> Self {
        Self::Mock(mock::Mock::new(data))
    }

    /// Requests the terminal to show a specific screen.
    pub async fn show(&self, screen: Screen) -> anyhow::Result<()> {
        match self {
            Terminal::Actual(tx) => {
                tx.send(Message::Show(screen))
                    .await
                    .context("Sending Show message to terminal")
                    .expect("Terminal actor died");
                Ok(())
            }
            Terminal::Mock(mock) => {
                mock.show(screen).await
            }
        }
    }

    /// Requests the terminal to quit.
    pub async fn quit(&self) -> anyhow::Result<()> {
        match self {
            Terminal::Actual(tx) => {
                tx.send(Message::Quit)
                    .await
                    .context("Sending Quit message to terminal")
                    .expect("Terminal actor died");
                Ok(())
            }
            Terminal::Mock(mock) => {
                mock.quit().await
            }
        }
    }
}
