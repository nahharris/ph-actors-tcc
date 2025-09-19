use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ArcStr;
use crate::app::ui::{MockData, NavigationAction, UiState, ViewKind};

/// Mock implementation of the UI actor for testing purposes.
///
/// This struct stores UI state and operations in memory,
/// allowing tests to run without creating actual terminal interfaces.
#[derive(Debug, Clone)]
pub struct Mock {
    data: Arc<Mutex<MockData>>,
}

impl Mock {
    /// Creates a new mock instance with the provided mock data.
    ///
    /// # Arguments
    /// * `data` - Initial mock data containing UI state
    pub fn new(data: MockData) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }

    /// Show the mailing lists view.
    /// Mock implementation updates the UI state and records rendered screens.
    ///
    /// # Arguments
    /// * `page` - The page number to show
    ///
    /// # Returns
    /// Ok(()) if successful
    pub async fn show_lists(&self, page: usize) -> anyhow::Result<()> {
        let mut mock_data = self.data.lock().await;
        mock_data.state.view = ViewKind::Lists;
        mock_data.state.list_page = page;
        mock_data.state.list_selected = 0;
        mock_data
            .rendered_screens
            .push(format!("Lists(page={})", page));
        Ok(())
    }

    /// Show the patch feed view for a specific mailing list.
    /// Mock implementation updates the UI state and records rendered screens.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    /// * `page` - The page number to show
    ///
    /// # Returns
    /// Ok(()) if successful
    pub async fn show_feed(&self, list: ArcStr, page: usize) -> anyhow::Result<()> {
        let mut mock_data = self.data.lock().await;
        mock_data.state.view = ViewKind::Feed;
        mock_data.state.feed_list = Some(list.clone());
        mock_data.state.feed_page = page;
        mock_data.state.feed_selected = 0;
        mock_data
            .rendered_screens
            .push(format!("Feed(list={}, page={})", list, page));
        Ok(())
    }

    /// Show a specific patch content.
    /// Mock implementation updates the UI state and records rendered screens.
    ///
    /// # Arguments
    /// * `list` - The mailing list name
    /// * `message_id` - The message ID of the patch
    /// * `title` - The title of the patch
    ///
    /// # Returns
    /// Ok(()) if successful
    pub async fn show_patch(&self, list: ArcStr, message_id: ArcStr, title: ArcStr) -> anyhow::Result<()> {
        let mut mock_data = self.data.lock().await;
        mock_data.state.view = ViewKind::Patch;
        mock_data.rendered_screens.push(format!(
            "Patch(list={}, msg_id={}, title={})",
            list, message_id, title
        ));
        Ok(())
    }

    /// Update the current selection index.
    /// Mock implementation updates the selection based on the current view.
    ///
    /// # Arguments
    /// * `index` - The new selection index
    pub async fn update_selection(&self, index: usize) {
        let mut mock_data = self.data.lock().await;
        match mock_data.state.view {
            ViewKind::Lists => mock_data.state.list_selected = index,
            ViewKind::Feed => mock_data.state.feed_selected = index,
            ViewKind::Patch => {} // No selection in patch view
        }
    }

    /// Navigate to the previous page.
    /// Mock implementation records navigation actions and updates pagination.
    ///
    /// # Returns
    /// Ok(()) if successful
    pub async fn previous_page(&self) -> anyhow::Result<()> {
        let mut mock_data = self.data.lock().await;
        mock_data
            .navigation_actions
            .push("PreviousPage".to_string());
        match mock_data.state.view {
            ViewKind::Lists => {
                mock_data.state.list_page = mock_data.state.list_page.saturating_sub(1);
                mock_data.state.list_selected = 0;
            }
            ViewKind::Feed => {
                mock_data.state.feed_page = mock_data.state.feed_page.saturating_sub(1);
                mock_data.state.feed_selected = 0;
            }
            ViewKind::Patch => {} // No pagination in patch view
        }
        Ok(())
    }

    /// Navigate to the next page.
    /// Mock implementation records navigation actions and updates pagination.
    ///
    /// # Returns
    /// Ok(()) if successful
    pub async fn next_page(&self) -> anyhow::Result<()> {
        let mut mock_data = self.data.lock().await;
        mock_data.navigation_actions.push("NextPage".to_string());
        match mock_data.state.view {
            ViewKind::Lists => {
                mock_data.state.list_page = mock_data.state.list_page.saturating_add(1);
                mock_data.state.list_selected = 0;
            }
            ViewKind::Feed => {
                mock_data.state.feed_page = mock_data.state.feed_page.saturating_add(1);
                mock_data.state.feed_selected = 0;
            }
            ViewKind::Patch => {} // No pagination in patch view
        }
        Ok(())
    }

    /// Navigate back to previous view.
    /// Mock implementation records navigation actions and updates the view state.
    ///
    /// # Returns
    /// Ok(()) if successful
    pub async fn navigate_back(&self) -> anyhow::Result<()> {
        let mut mock_data = self.data.lock().await;
        mock_data
            .navigation_actions
            .push("NavigateBack".to_string());
        match mock_data.state.view {
            ViewKind::Lists => {} // From lists, we quit (handled elsewhere)
            ViewKind::Feed => mock_data.state.view = ViewKind::Lists,
            ViewKind::Patch => mock_data.state.view = ViewKind::Feed,
        }
        Ok(())
    }

    /// Submit/select the current item.
    /// Mock implementation returns appropriate navigation actions based on current view.
    ///
    /// # Returns
    /// Ok(Some(NavigationAction)) if navigation should occur, Ok(None) otherwise
    pub async fn submit_selection(&self) -> anyhow::Result<Option<NavigationAction>> {
        let mock_data = self.data.lock().await;
        match mock_data.state.view {
            ViewKind::Lists => Ok(Some(NavigationAction::OpenFeed {
                list: ArcStr::from("test-list"),
            })),
            ViewKind::Feed => Ok(Some(NavigationAction::OpenPatch {
                list: ArcStr::from("test-list"),
                message_id: ArcStr::from("test-msg-id"),
                title: ArcStr::from("test-title"),
            })),
            ViewKind::Patch => Ok(None),
        }
    }

    /// Get current UI state.
    /// Mock implementation returns the current UI state.
    ///
    /// # Returns
    /// The current UI state
    pub async fn get_state(&self) -> UiState {
        let mock_data = self.data.lock().await;
        mock_data.state.clone()
    }

    /// Gets the mock data for inspection in tests.
    ///
    /// # Returns
    /// A copy of the current mock data
    pub async fn get_data(&self) -> MockData {
        let mock_data = self.data.lock().await;
        mock_data.clone()
    }
}
