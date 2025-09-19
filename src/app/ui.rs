use anyhow::{Context, Result};
use tokio::sync::{mpsc::Sender, oneshot};

use crate::ArcStr;
use crate::app::cache::{FeedCache, MailingListCache, PatchCache};
use crate::log::Log;
use crate::render::Render;
use crate::terminal::Terminal;

mod core;
mod data;
mod mock;
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
    Mock(mock::Mock),
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
        Self::Mock(mock::Mock::new(data))
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
            Self::Mock(mock) => {
                mock.show_lists(page).await
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
            Self::Mock(mock) => {
                mock.show_feed(list, page).await
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
            Self::Mock(mock) => {
                mock.show_patch(list, message_id, title).await
            }
        }
    }

    /// Update the current selection index
    pub async fn update_selection(&self, index: usize) {
        match self {
            Self::Actual(sender) => {
                let _ = sender.send(Message::UpdateSelection { index }).await;
            }
            Self::Mock(mock) => {
                mock.update_selection(index).await
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
            Self::Mock(mock) => {
                mock.previous_page().await
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
            Self::Mock(mock) => {
                mock.next_page().await
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
            Self::Mock(mock) => {
                mock.navigate_back().await
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
            Self::Mock(mock) => {
                mock.submit_selection().await
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
            Self::Mock(mock) => {
                mock.get_state().await
            }
        }
    }
}
