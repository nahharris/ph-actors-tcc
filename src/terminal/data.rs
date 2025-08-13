use crate::ArcStr;
use crate::api::lore::{LoreFeedItem, LoreMailingList};

/// UI key events emitted by the terminal.
#[derive(Debug, Clone, Copy)]
pub enum UiEvent {
    Left,
    Right,
    Esc,
    SelectionChange(usize),
    SelectionSubmit(usize),
}

/// A high-level description of the screen to render.
#[derive(Debug, Clone)]
pub enum Screen {
    /// Lists screen: shows mailing lists, with current page and selection
    Lists {
        items: Vec<LoreMailingList>,
        page: usize,
        selected: usize,
    },
    /// Feed screen: shows patches for a mailing list
    Feed {
        list: ArcStr,
        items: Vec<LoreFeedItem>,
        page: usize,
        selected: usize,
    },
    /// Patch screen: shows rendered patch content
    Patch { title: ArcStr, content: ArcStr },
    /// Loading screen with a message
    Loading(ArcStr),
    /// Error screen with a message
    Error(ArcStr),
}

/// Mock data for testing terminal operations
#[derive(Debug, Clone, Default)]
pub struct MockData {
    /// Last screen that was requested to be shown
    pub last_screen: Option<Screen>,
    /// Whether quit was called
    pub quit_called: bool,
}
