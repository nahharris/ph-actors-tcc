use crate::ArcStr;

/// Different view types in the TUI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewKind {
    /// Mailing lists view
    Lists,
    /// Patch feed view for a specific mailing list
    Feed,
    /// Individual patch content view
    Patch,
}

/// UI state managed by the UI actor
#[derive(Debug, Clone)]
pub struct UiState {
    /// Current view being displayed
    pub view: ViewKind,
    /// Current page in lists view
    pub list_page: usize,
    /// Currently selected item in lists view
    pub list_selected: usize,
    /// Current mailing list name (when in Feed view)
    pub feed_list: Option<ArcStr>,
    /// Current page in feed view
    pub feed_page: usize,
    /// Currently selected item in feed view
    pub feed_selected: usize,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            view: ViewKind::Lists,
            list_page: 0,
            list_selected: 0,
            feed_list: None,
            feed_page: 0,
            feed_selected: 0,
        }
    }
}

/// Mock data for testing the UI actor
#[derive(Debug, Clone, Default)]
pub struct MockData {
    /// Current UI state
    pub state: UiState,
    /// Track screens that have been rendered (for testing)
    pub rendered_screens: Vec<String>,
    /// Track navigation actions (for testing)
    pub navigation_actions: Vec<String>,
}
