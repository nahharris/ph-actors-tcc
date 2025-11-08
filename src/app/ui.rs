use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};

/// Actor name for error reporting.
pub const ACTOR_NAME: &'static str = "Ui";

use crate::{ArcStr, error::FatalActorError, error::UiError};

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
    pub async fn show_lists(&self, page: usize) -> Result<(), UiError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::ShowLists { page, tx })
            .await
            .map_err(|_e| {
                UiError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::ui::ACTOR_NAME,
                    operation: "show lists".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            UiError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::ui::ACTOR_NAME,
                operation: "show lists".to_string(),
                source: e,
            })
        })?
    }

    /// Show the patch feed view for a specific mailing list
    pub async fn show_feed(&self, list: ArcStr, page: usize) -> Result<(), UiError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::ShowFeed { list, page, tx })
            .await
            .map_err(|_e| {
                UiError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::ui::ACTOR_NAME,
                    operation: "show feed".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            UiError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::ui::ACTOR_NAME,
                operation: "show feed".to_string(),
                source: e,
            })
        })?
    }

    /// Show a specific patch content
    pub async fn show_patch(
        &self,
        list: ArcStr,
        message_id: ArcStr,
        title: ArcStr,
    ) -> Result<(), UiError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::ShowPatch {
                list,
                message_id,
                title,
                tx,
            })
            .await
            .map_err(|_e| {
                UiError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::ui::ACTOR_NAME,
                    operation: "show patch".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            UiError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::ui::ACTOR_NAME,
                operation: "show patch".to_string(),
                source: e,
            })
        })?
    }

    /// Update the current selection index
    pub async fn update_selection(&self, index: usize) {
        let _ = self.tx.send(Message::UpdateSelection { index }).await;
    }

    /// Navigate to the previous page
    pub async fn previous_page(&self) -> Result<(), UiError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::PreviousPage { tx })
            .await
            .map_err(|_e| {
                UiError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::ui::ACTOR_NAME,
                    operation: "previous page".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            UiError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::ui::ACTOR_NAME,
                operation: "previous page".to_string(),
                source: e,
            })
        })?
    }

    /// Navigate to the next page
    pub async fn next_page(&self) -> Result<(), UiError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Message::NextPage { tx }).await.map_err(|_e| {
            UiError::Fatal(FatalActorError::ActorSendFailed {
                actor_name: crate::app::ui::ACTOR_NAME,
                operation: "next page".to_string(),
            })
        })?;
        rx.await.map_err(|e| {
            UiError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::ui::ACTOR_NAME,
                operation: "next page".to_string(),
                source: e,
            })
        })?
    }

    /// Navigate back to previous view
    pub async fn navigate_back(&self) -> Result<(), UiError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::NavigateBack { tx })
            .await
            .map_err(|_e| {
                UiError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::ui::ACTOR_NAME,
                    operation: "navigate back".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            UiError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::ui::ACTOR_NAME,
                operation: "navigate back".to_string(),
                source: e,
            })
        })?
    }

    /// Submit/select the current item
    pub async fn submit_selection(&self) -> Result<Option<NavigationAction>, UiError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::SubmitSelection { tx })
            .await
            .map_err(|_e| {
                UiError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::app::ui::ACTOR_NAME,
                    operation: "submit selection".to_string(),
                })
            })?;
        rx.await.map_err(|e| {
            UiError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::ui::ACTOR_NAME,
                operation: "submit selection".to_string(),
                source: e,
            })
        })?
    }

    /// Get current UI state
    pub async fn get_state(&self) -> UiState {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Message::GetState { tx }).await;
        rx.await.unwrap_or_default()
    }
}
