use super::data::Screen;

/// Messages that can be sent to the terminal actor.
#[derive(Debug)]
pub enum Message {
    /// Render the given screen
    Show(Screen),
    /// Quit the UI
    Quit,
}
