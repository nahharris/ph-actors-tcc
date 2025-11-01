use crate::ArcStr;
use crate::api::lore::{LoreMailingList, LorePatchMetadata};

/// UI key events emitted by the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        items: Vec<LorePatchMetadata>,
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
