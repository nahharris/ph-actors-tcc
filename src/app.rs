use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, oneshot};

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
    env::Env,
    fs::Fs,
    log::Log,
    render::Render,
    shell::Shell,
};
#[cfg(test)]
use crate::{
    api::lore::mock::MockLoreApi as LoreApi, app::cache::feed::mock::MockFeedCache as FeedCache,
    app::cache::mailing_list::mock::MockMailingListCache as MailingListCache,
    app::cache::patch::mock::MockPatchCache as PatchCache, app::config::mock::MockConfig as Config,
    env::mock::MockEnv as Env, fs::mock::MockFs as Fs, log::mock::MockLog as Log,
    render::mock::MockRender as Render, shell::mock::MockShell as Shell,
};

pub use data::{AppState, Command, MockData};
use message::Message;

/// App actor - Central coordinator for the entire application
///
/// This actor manages application state and coordinates all other actors.
/// It handles command execution, cache management, and UI coordination.
#[derive(Debug, Clone)]
pub struct App {
    tx: mpsc::Sender<Message>,
}

impl App {
    pub fn new(
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
    ) -> Self {
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
        );
        core.spawn_interactive().0
    }

    /// Execute a CLI command and exit (resolve mode)
    ///
    /// Handles Lists, Feed, and Patch commands by coordinating with
    /// appropriate actors and caches. This is for one-shot CLI execution.
    pub async fn resolve(&self, command: Command) -> Result<()> {
        match command {
            Command::Lists { page, count } => {
                self.handle_lists_command(page, count).await
                    }
                    Command::Feed { list, page, count } => {
                        core_ref.handle_feed_command(list, page, count).await
                    }
                    Command::Patch {
                        list,
                        message_id,
                        html,
                    } => core_ref.handle_patch_command(list, message_id, html).await,
                }?;
                // Persist caches before exiting
                core_ref.handle_shutdown().await
            }
        }
    }

    /// Spawn the App actor for interactive mode
    ///
    /// Starts the interactive TUI interface and returns a handle for
    /// sending key events and a JoinHandle for the actor task.
    pub fn spawn(self) -> Result<(AppHandle, tokio::task::JoinHandle<()>)> {
        match self {
            Self::Ready(core) => {
                let core = Arc::try_unwrap(core)
                    .map_err(|_| anyhow::anyhow!("Core still has references"))?;
                let (app, handle) = core.spawn_interactive()?;
                Ok((AppHandle { app }, handle))
            }
            Self::Actual(_) => Err(anyhow::anyhow!("App already spawned")),
            Self::Mock(data) => {
                // Return dummy handle for mock
                let handle = tokio::spawn(async {});
                Ok((
                    AppHandle {
                        app: App::Mock(data),
                    },
                    handle,
                ))
            }
        }
    }
}

/// Handle for interacting with a spawned App actor
pub struct AppHandle {
    app: App,
}

impl AppHandle {
    /// Send a key event to the spawned App actor
    pub async fn send_key_event(&self, event: crate::terminal::UiEvent) -> Result<()> {
        match &self.app {
            App::Actual(sender) => {
                sender
                    .send(Message::KeyEvent { event })
                    .await
                    .context("Sending key event to App actor")?;
                Ok(())
            }
            App::Mock(_) => Ok(()), // Mock doesn't need to handle events
            App::Ready(_) => Err(anyhow::anyhow!("App not properly spawned")),
        }
    }

    /// Shutdown the spawned App actor
    pub async fn shutdown(&self) -> Result<()> {
        match &self.app {
            App::Actual(sender) => {
                let (tx, rx) = oneshot::channel();
                sender
                    .send(Message::Shutdown { tx })
                    .await
                    .context("Sending shutdown message to App actor")?;
                rx.await
                    .context("Awaiting response for shutdown from App actor")?
            }
            App::Mock(data) => {
                let mut mock_data = data.lock().await;
                mock_data.shutdown_called = true;
                Ok(())
            }
            App::Ready(_) => Err(anyhow::anyhow!("App not properly spawned")),
        }
    }
}
