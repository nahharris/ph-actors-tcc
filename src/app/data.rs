use crate::ArcStr;

/// Commands that can be executed by the App actor
#[derive(Debug, Clone)]
pub enum Command {
    /// List all available mailing lists
    Lists { page: usize, count: usize },
    /// Get the feed of a given mailing list
    Feed {
        list: ArcStr,
        page: usize,
        count: usize,
    },
    /// Get the content of a patch from the feed
    Patch {
        list: ArcStr,
        message_id: ArcStr,
        html: bool,
    },
}

/// Application state managed by the App actor
#[derive(Debug)]
pub struct AppState {
    /// Whether the application has been initialized
    pub initialized: bool,
    /// Current command being executed (if any)
    pub current_command: Option<Command>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            initialized: false,
            current_command: None,
        }
    }
}

/// Mock data for testing the App actor
#[derive(Debug, Default)]
pub struct MockData {
    /// Simulated application state
    pub state: AppState,
    /// Commands that have been executed
    pub executed_commands: Vec<Command>,
    /// Whether TUI mode has been run
    pub tui_run: bool,
    /// Whether shutdown has been called
    pub shutdown_called: bool,
}
