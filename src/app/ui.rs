use anyhow::{Context, Result};
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};

use crate::ArcStr;

#[cfg(not(test))]
use crate::{
    app::cache::{feed::FeedCache, mailing_list::MailingListCache, patch::PatchCache},
    log::Log,
    render::Render,
    terminal::Terminal,
};

#[cfg(test)]
use crate::{
    app::cache::{
        feed::mock::MockFeedCache as FeedCache,
        mailing_list::mock::MockMailingListCache as MailingListCache,
        patch::mock::MockPatchCache as PatchCache,
    },
    log::mock::MockLog as Log,
    render::mock::MockRender as Render,
    terminal::mock::MockTerminal as Terminal,
};

mod core;
mod data;
mod message;
#[cfg(test)]
pub mod mock;

pub use data::{MockData, UiState, ViewKind};
pub use message::{Message, NavigationAction};

/// UI actor - Manages TUI state and rendering
///
/// This actor handles the terminal user interface, managing view state,
/// navigation, and rendering different screens (Lists, Feed, Patch).
#[derive(Debug, Clone)]
pub struct Ui {
    tx: Sender<Message>,
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
    ) -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(
            log,
            terminal,
            mailing_list_cache,
            feed_cache,
            patch_cache,
            render,
        );
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }

    /// Show the mailing lists view
    pub async fn show_lists(&self, page: usize) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::ShowLists { page, tx })
            .await
            .context("Sending show lists message to UI actor")
            .expect("UI actor died");
        rx.await
            .context("Awaiting response for show lists from UI actor")
            .expect("UI actor died")
    }

    /// Show the patch feed view for a specific mailing list
    pub async fn show_feed(&self, list: ArcStr, page: usize) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::ShowFeed { list, page, tx })
            .await
            .context("Sending show feed message to UI actor")
            .expect("UI actor died");
        rx.await
            .context("Awaiting response for show feed from UI actor")
            .expect("UI actor died")
    }

    /// Show a specific patch content
    pub async fn show_patch(&self, list: ArcStr, message_id: ArcStr, title: ArcStr) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
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

    /// Update the current selection index
    pub async fn update_selection(&self, index: usize) {
        let _ = self.tx.send(Message::UpdateSelection { index }).await;
    }

    /// Navigate to the previous page
    pub async fn previous_page(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::PreviousPage { tx })
            .await
            .context("Sending previous page message to UI actor")
            .expect("UI actor died");
        rx.await
            .context("Awaiting response for previous page from UI actor")
            .expect("UI actor died")
    }

    /// Navigate to the next page
    pub async fn next_page(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::NextPage { tx })
            .await
            .context("Sending next page message to UI actor")
            .expect("UI actor died");
        rx.await
            .context("Awaiting response for next page from UI actor")
            .expect("UI actor died")
    }

    /// Navigate back to previous view
    pub async fn navigate_back(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::NavigateBack { tx })
            .await
            .context("Sending navigate back message to UI actor")
            .expect("UI actor died");
        rx.await
            .context("Awaiting response for navigate back from UI actor")
            .expect("UI actor died")
    }

    /// Submit/select the current item
    pub async fn submit_selection(&self) -> Result<Option<NavigationAction>> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::SubmitSelection { tx })
            .await
            .context("Sending submit selection message to UI actor")
            .expect("UI actor died");
        rx.await
            .context("Awaiting response for submit selection from UI actor")
            .expect("UI actor died")
    }

    /// Get current UI state
    pub async fn get_state(&self) -> UiState {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Message::GetState { tx }).await;
        rx.await.unwrap_or_default()
    }
}
