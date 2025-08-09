use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc::Sender, oneshot};

pub mod cache;
pub mod config;
pub mod ui;

mod core;
mod data;
mod message;

pub use data::{AppState, Command, MockData};
use message::Message;

/// App actor - Central coordinator for the entire application
///
/// This actor manages application state and coordinates all other actors.
/// It handles command execution, cache management, and UI coordination.
#[derive(Debug)]
pub enum App {
    /// Ready to be used (built but not spawned)
    Ready(Arc<core::Core>),
    /// Real implementation using message passing (spawned)
    Actual(Sender<Message>),
    /// Mock implementation for testing
    Mock(Arc<Mutex<MockData>>),
}

impl App {
    /// Create a new App actor with full initialization (but not spawned)
    ///
    /// This performs all necessary setup including:
    /// - Actor initialization (env, fs, config, log, net, lore, shell, render)
    /// - Configuration loading
    /// - Cache initialization and loading
    ///
    /// Returns the App actor ready for resolve() or spawn()
    pub async fn build() -> Result<Self> {
        let core = core::Core::build().await?;
        Ok(Self::Ready(Arc::new(core)))
    }

    /// Create a mock App actor for testing
    pub fn mock(data: MockData) -> Self {
        Self::Mock(Arc::new(Mutex::new(data)))
    }

    /// Execute a CLI command and exit (resolve mode)
    ///
    /// Handles Lists, Feed, and Patch commands by coordinating with
    /// appropriate actors and caches. This is for one-shot CLI execution.
    pub async fn resolve(&self, command: Command) -> Result<()> {
        match self {
            Self::Ready(core) => {
                // Execute command directly without spawning actor
                let core_ref = Arc::clone(core);
                match command {
                    Command::Lists { page, count } => {
                        core_ref.handle_lists_command(page, count).await
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
            Self::Actual(_) => Err(anyhow::anyhow!("App already spawned, cannot resolve")),
            Self::Mock(data) => {
                let mut mock_data = data.lock().await;
                mock_data.executed_commands.push(command.clone());
                mock_data.state.current_command = Some(command);
                Ok(())
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
