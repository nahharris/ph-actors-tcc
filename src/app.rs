use tokio::sync::{mpsc, oneshot};

use crate::error::{AppError, FatalActorError};

pub mod cache;
pub mod config;
pub mod ui;

mod core;
mod data;
mod message;

#[cfg(not(test))]
use crate::{
    app::cache::{feed::FeedCache, mailing_list::MailingListCache},
    app::ui::Ui,
    log::Log,
    terminal::Terminal,
};
#[cfg(test)]
use crate::{
    app::cache::{
        feed::mock::MockFeedCache as FeedCache,
        mailing_list::mock::MockMailingListCache as MailingListCache,
    },
    app::ui::mock::MockUi as Ui,
    log::mock::MockLog as Log,
    terminal::mock::MockTerminal as Terminal,
};

pub use data::{AppState, MockData};
use message::Message;

pub const ACTOR_NAME: &'static str = "App";

/// App actor - Central coordinator for the entire application
///
/// This actor manages application state and coordinates all other actors.
/// It handles UI coordination and cache management.
#[derive(Debug, Clone)]
pub struct App {
    tx: mpsc::Sender<Message>,
}

impl App {
    /// Spawn the App actor
    ///
    /// Starts the interactive TUI interface and returns the actor interface
    /// and a JoinHandle for the actor task.
    pub fn spawn(
        log: Log,
        mailing_list_cache: MailingListCache,
        feed_cache: FeedCache,
        terminal: Terminal,
        terminal_handle: tokio::task::JoinHandle<()>,
        ui: Ui,
    ) -> (Self, tokio::task::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(
            log,
            mailing_list_cache,
            feed_cache,
            terminal,
            terminal_handle,
            ui,
        );
        let handle = tokio::spawn(async move {
            core.init(rx).await;
        });
        (Self { tx }, handle)
    }

    /// Shutdown the App actor gracefully
    pub async fn shutdown(&self) -> Result<(), AppError> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Message::Shutdown { tx }).await.map_err(|_e| {
            AppError::Fatal(FatalActorError::ActorSendFailed {
                actor_name: crate::app::ACTOR_NAME,
                operation: "shutdown".to_string(),
            })
        })?;
        rx.await.map_err(|e| {
            AppError::Fatal(FatalActorError::ActorRecvFailed {
                actor_name: crate::app::ACTOR_NAME,
                operation: "shutdown".to_string(),
                source: e,
            })
        })?
    }
}
