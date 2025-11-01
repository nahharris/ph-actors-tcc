use anyhow::{Context, Result};
use tokio::sync::{mpsc, oneshot};

pub mod cache;
pub mod config;
pub mod ui;

mod core;
mod data;
mod message;

#[cfg(not(test))]
use crate::{
    api::lore::LoreApi,
    app::cache::{feed::FeedCache, mailing_list::MailingListCache, patch::PatchCache},
    app::config::Config,
    app::ui::Ui,
    env::Env,
    fs::Fs,
    log::Log,
    render::Render,
    shell::Shell,
    terminal::Terminal,
};
#[cfg(test)]
use crate::{
    api::lore::mock::MockLoreApi as LoreApi, app::cache::feed::mock::MockFeedCache as FeedCache,
    app::cache::mailing_list::mock::MockMailingListCache as MailingListCache,
    app::cache::patch::mock::MockPatchCache as PatchCache, app::config::mock::MockConfig as Config,
    app::ui::mock::MockUi as Ui, env::mock::MockEnv as Env, fs::mock::MockFs as Fs,
    log::mock::MockLog as Log, render::mock::MockRender as Render, shell::mock::MockShell as Shell,
    terminal::mock::MockTerminal as Terminal,
};

pub use data::{AppState, MockData};
use message::Message;

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
        fs: Fs,
        env: Env,
        config: Config,
        lore: LoreApi,
        shell: Shell,
        render: Render,
        mailing_list_cache: MailingListCache,
        feed_cache: FeedCache,
        patch_cache: PatchCache,
        terminal: Terminal,
        ui: Ui,
    ) -> (Self, tokio::task::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(
            log,
            fs,
            env,
            config,
            lore,
            shell,
            render,
            mailing_list_cache,
            feed_cache,
            patch_cache,
            terminal,
            ui,
        );
        let handle = tokio::spawn(async move {
            core.init(rx).await;
        });
        (Self { tx }, handle)
    }

    /// Shutdown the App actor gracefully
    pub async fn shutdown(&self) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Message::Shutdown { tx })
            .await
            .context("Sending shutdown message to App actor")
            .expect("App actor died");
        rx.await
            .context("Awaiting response for shutdown from App actor")
            .expect("App actor died")
    }
}
