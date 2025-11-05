use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::{error::TerminalError, error::FatalActorError, log::Log};

mod core;
mod data;
mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

pub use data::{Screen, UiEvent};
use message::Message;

pub const ACTOR_NAME: &'static str = "Terminal";

/// The terminal actor that owns the Cursive event loop and exposes a message-based API.
#[derive(Debug, Clone)]
pub struct Terminal {
    tx: mpsc::Sender<Message>,
}

impl Terminal {
    /// Spawns a terminal actor using the Cursive `crossterm` backend.
    ///
    /// The actor stores `UiEvent`s in an internal FIFO queue and accepts `Message`s to update the UI.
    /// Returns a tuple of the terminal interface and the join handle for awaiting terminal completion.
    pub fn spawn(log: Log) -> (Self, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(log);
        let handle = tokio::spawn(async move {
            core.init(rx).await;
        });
        (Self { tx }, handle)
    }

    /// Requests the terminal to show a specific screen.
    pub async fn show(&self, screen: Screen) -> Result<(), TerminalError> {
        self.tx
            .send(Message::Show(screen))
            .await
            .map_err(|_e| {
                TerminalError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::terminal::ACTOR_NAME,
                    operation: "show screen".to_string(),
                })
            })
    }

    /// Get the next UI event from the terminal's internal queue (pops the head).
    pub async fn get_ui_event(&self) -> Result<Option<UiEvent>, TerminalError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::GetUiEvent { tx })
            .await
            .map_err(|_e| {
                TerminalError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::terminal::ACTOR_NAME,
                    operation: "get ui event".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                TerminalError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::terminal::ACTOR_NAME,
                    operation: "get ui event".to_string(),
                    source: e,
                })
            })
    }

    /// Clear all UI events from the terminal's internal queue.
    pub async fn clear_ui_events(&self) -> Result<(), TerminalError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::ClearUiEvents { tx })
            .await
            .map_err(|_e| {
                TerminalError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::terminal::ACTOR_NAME,
                    operation: "clear ui events".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                TerminalError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::terminal::ACTOR_NAME,
                    operation: "clear ui events".to_string(),
                    source: e,
                })
            })
    }

    /// Requests the terminal to quit.
    pub async fn quit(&self) -> Result<(), TerminalError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::Quit { tx })
            .await
            .map_err(|_e| {
                TerminalError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::terminal::ACTOR_NAME,
                    operation: "quit".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                TerminalError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::terminal::ACTOR_NAME,
                    operation: "quit".to_string(),
                    source: e,
                })
            })
    }
}
