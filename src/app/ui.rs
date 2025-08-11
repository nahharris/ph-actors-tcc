use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc::Sender, oneshot};

use crate::ArcStr;
use crate::app::cache::{FeedCache, MailingListCache, PatchCache};
use crate::log::Log;
use crate::render::Render;
use crate::terminal::Terminal;

mod core;
mod data;
mod message;

pub use data::{MockData, UiState, ViewKind};
pub use message::{Message, NavigationAction};

/// UI actor - Manages TUI state and rendering
///
/// This actor handles the terminal user interface, managing view state,
/// navigation, and rendering different screens (Lists, Feed, Patch).
#[derive(Debug, Clone)]
pub enum Ui {
    /// Real implementation using message passing
    Actual(Sender<Message>),
    /// Mock implementation for testing
    Mock(Arc<Mutex<MockData>>),
}

impl Ui {
    /// Create a new UI actor
    pub fn spawn(
        log: Log,
        terminal: Terminal,
        mailing_list_cache: MailingListCache,
        feed_cache: FeedCache,
        patch_cache: PatchCache,
        render: Render,
    ) -> (Self, tokio::task::JoinHandle<()>) {
        let core = core::Core::new(
            log,
            terminal,
            mailing_list_cache,
            feed_cache,
            patch_cache,
            render,
        );
        core.spawn()
    }

    /// Create a mock UI actor for testing
    pub fn mock(data: MockData) -> Self {
        Self::Mock(Arc::new(Mutex::new(data)))
    }

    /// Show the mailing lists view
    pub async fn show_lists(&self, page: usize) -> Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = oneshot::channel();
                sender
                    .send(Message::ShowLists { page, tx })
                    .await
                    .context("Sending show lists message to UI actor")
                    .expect("UI actor died");
                rx.await
                    .context("Awaiting response for show lists from UI actor")
                    .expect("UI actor died")
            }
            Self::Mock(data) => {
                let mut mock_data = data.lock().await;
                mock_data.state.view = ViewKind::Lists;
                mock_data.state.list_page = page;
                mock_data.state.list_selected = 0;
                mock_data
                    .rendered_screens
                    .push(format!("Lists(page={})", page));
                Ok(())
            }
        }
    }

    /// Show the patch feed view for a specific mailing list
    pub async fn show_feed(&self, list: ArcStr, page: usize) -> Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = oneshot::channel();
                sender
                    .send(Message::ShowFeed { list, page, tx })
                    .await
                    .context("Sending show feed message to UI actor")
                    .expect("UI actor died");
                rx.await
                    .context("Awaiting response for show feed from UI actor")
                    .expect("UI actor died")
            }
            Self::Mock(data) => {
                let mut mock_data = data.lock().await;
                mock_data.state.view = ViewKind::Feed;
                mock_data.state.feed_list = Some(list.clone());
                mock_data.state.feed_page = page;
                mock_data.state.feed_selected = 0;
                mock_data
                    .rendered_screens
                    .push(format!("Feed(list={}, page={})", list, page));
                Ok(())
            }
        }
    }

    /// Show a specific patch content
    pub async fn show_patch(&self, list: ArcStr, message_id: ArcStr, title: ArcStr) -> Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = oneshot::channel();
                sender
                    .send(Message::ShowPatch {
                        list,
                        message_id,
                        title,
                        tx,
                    })
                    .await
                    .context("Sending show patch message to UI actor")
                    .expect("UI actor died");
                rx.await
                    .context("Awaiting response for show patch from UI actor")
                    .expect("UI actor died")
            }
            Self::Mock(data) => {
                let mut mock_data = data.lock().await;
                mock_data.state.view = ViewKind::Patch;
                mock_data.rendered_screens.push(format!(
                    "Patch(list={}, msg_id={}, title={})",
                    list, message_id, title
                ));
                Ok(())
            }
        }
    }

    /// Update the current selection index
    pub async fn update_selection(&self, index: usize) {
        match self {
            Self::Actual(sender) => {
                let _ = sender.send(Message::UpdateSelection { index }).await;
            }
            Self::Mock(data) => {
                let mut mock_data = data.lock().await;
                match mock_data.state.view {
                    ViewKind::Lists => mock_data.state.list_selected = index,
                    ViewKind::Feed => mock_data.state.feed_selected = index,
                    ViewKind::Patch => {} // No selection in patch view
                }
            }
        }
    }

    /// Navigate to the previous page
    pub async fn previous_page(&self) -> Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = oneshot::channel();
                sender
                    .send(Message::PreviousPage { tx })
                    .await
                    .context("Sending previous page message to UI actor")
                    .expect("UI actor died");
                rx.await
                    .context("Awaiting response for previous page from UI actor")
                    .expect("UI actor died")
            }
            Self::Mock(data) => {
                let mut mock_data = data.lock().await;
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
        }
    }

    /// Navigate to the next page
    pub async fn next_page(&self) -> Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = oneshot::channel();
                sender
                    .send(Message::NextPage { tx })
                    .await
                    .context("Sending next page message to UI actor")
                    .expect("UI actor died");
                rx.await
                    .context("Awaiting response for next page from UI actor")
                    .expect("UI actor died")
            }
            Self::Mock(data) => {
                let mut mock_data = data.lock().await;
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
        }
    }

    /// Navigate back to previous view
    pub async fn navigate_back(&self) -> Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = oneshot::channel();
                sender
                    .send(Message::NavigateBack { tx })
                    .await
                    .context("Sending navigate back message to UI actor")
                    .expect("UI actor died");
                rx.await
                    .context("Awaiting response for navigate back from UI actor")
                    .expect("UI actor died")
            }
            Self::Mock(data) => {
                let mut mock_data = data.lock().await;
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
        }
    }

    /// Submit/select the current item
    pub async fn submit_selection(&self) -> Result<Option<NavigationAction>> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = oneshot::channel();
                sender
                    .send(Message::SubmitSelection { tx })
                    .await
                    .context("Sending submit selection message to UI actor")
                    .expect("UI actor died");
                rx.await
                    .context("Awaiting response for submit selection from UI actor")
                    .expect("UI actor died")
            }
            Self::Mock(data) => {
                let mock_data = data.lock().await;
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
        }
    }

    /// Get current UI state
    pub async fn get_state(&self) -> UiState {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = oneshot::channel();
                let _ = sender.send(Message::GetState { tx }).await;
                rx.await.unwrap_or_default()
            }
            Self::Mock(data) => {
                let mock_data = data.lock().await;
                mock_data.state.clone()
            }
        }
    }
}
