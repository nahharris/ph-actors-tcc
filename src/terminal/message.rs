use super::data::{Screen, UiEvent};
use tokio::sync::oneshot;

/// Messages that can be sent to the terminal actor.
#[derive(Debug)]
pub enum Message {
    /// Render the given screen
    Show(Screen),
    /// Get the next UI event from the queue (pops the head)
    GetUiEvent {
        tx: oneshot::Sender<Option<UiEvent>>,
    },
    /// Clear all UI events from the queue
    ClearUiEvents { tx: oneshot::Sender<()> },
    /// Quit the UI
    Quit { tx: oneshot::Sender<()> },
}
