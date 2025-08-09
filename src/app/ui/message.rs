use anyhow::Result;
use tokio::sync::oneshot;

use crate::ArcStr;

/// Messages for communicating with the UI actor
#[derive(Debug)]
pub enum Message {
    /// Show the mailing lists view
    ShowLists {
        page: usize,
        tx: oneshot::Sender<Result<()>>,
    },
    /// Show the patch feed view for a specific mailing list
    ShowFeed {
        list: ArcStr,
        page: usize,
        tx: oneshot::Sender<Result<()>>,
    },
    /// Show a specific patch content
    ShowPatch {
        list: ArcStr,
        message_id: ArcStr,
        title: ArcStr,
        tx: oneshot::Sender<Result<()>>,
    },
    /// Update the current selection index
    UpdateSelection { index: usize },
    /// Navigate to the previous page
    PreviousPage { tx: oneshot::Sender<Result<()>> },
    /// Navigate to the next page
    NextPage { tx: oneshot::Sender<Result<()>> },
    /// Navigate back to previous view
    NavigateBack { tx: oneshot::Sender<Result<()>> },
    /// Submit/select the current item
    SubmitSelection {
        tx: oneshot::Sender<Result<Option<NavigationAction>>>,
    },
    /// Get current UI state
    GetState {
        tx: oneshot::Sender<super::data::UiState>,
    },
}

/// Actions that result from UI navigation
#[derive(Debug, Clone)]
pub enum NavigationAction {
    /// Navigate to feed for a specific list
    OpenFeed { list: ArcStr },
    /// Navigate to patch with specific details
    OpenPatch {
        list: ArcStr,
        message_id: ArcStr,
        title: ArcStr,
    },
    /// Quit the application
    Quit,
}
