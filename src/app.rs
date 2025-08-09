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
#[derive(Debug, Clone)]
pub enum App {
    /// Real implementation using message passing
    Actual(Sender<Message>),
    /// Mock implementation for testing
    Mock(Arc<Mutex<MockData>>),
}

impl App {
    /// Create a new App actor with full initialization
    ///
    /// This performs all necessary setup including:
    /// - Actor initialization (env, fs, config, log, net, lore, shell, render)
    /// - Configuration loading
    /// - Cache initialization and loading
    ///
    /// Returns the App actor and a JoinHandle for the actor task
    pub async fn build() -> Result<(Self, tokio::task::JoinHandle<()>)> {
        let core = core::Core::build().await?;
        Ok(core.spawn())
    }

    /// Create a mock App actor for testing
    pub fn mock(data: MockData) -> Self {
        Self::Mock(Arc::new(Mutex::new(data)))
    }

    /// Execute a CLI command
    ///
    /// Handles Lists, Feed, and Patch commands by coordinating with
    /// appropriate actors and caches.
    pub async fn execute_command(&self, command: Command) -> Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = oneshot::channel();
                sender
                    .send(Message::ExecuteCommand { command, tx })
                    .await
                    .context("Sending execute command message to App actor")
                    .expect("App actor died");
                rx.await
                    .context("Awaiting response for execute command from App actor")
                    .expect("App actor died")
            }
            Self::Mock(data) => {
                let mut mock_data = data.lock().await;
                mock_data.executed_commands.push(command.clone());
                mock_data.state.current_command = Some(command);
                Ok(())
            }
        }
    }

    /// Run the TUI (Terminal User Interface) mode
    ///
    /// Starts the interactive TUI interface for browsing mailing lists
    /// and patches. Blocks until the user exits the TUI.
    pub async fn run_tui(&self) -> Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = oneshot::channel();
                sender
                    .send(Message::RunTui { tx })
                    .await
                    .context("Sending run TUI message to App actor")
                    .expect("App actor died");
                rx.await
                    .context("Awaiting response for run TUI from App actor")
                    .expect("App actor died")
            }
            Self::Mock(data) => {
                let mut mock_data = data.lock().await;
                mock_data.tui_run = true;
                Ok(())
            }
        }
    }

    /// Shutdown the application gracefully
    ///
    /// Persists cache data and performs cleanup before shutting down.
    /// This should be called before the application exits.
    pub async fn shutdown(&self) -> Result<()> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = oneshot::channel();
                sender
                    .send(Message::Shutdown { tx })
                    .await
                    .context("Sending shutdown message to App actor")
                    .expect("App actor died");
                rx.await
                    .context("Awaiting response for shutdown from App actor")
                    .expect("App actor died")
            }
            Self::Mock(data) => {
                let mut mock_data = data.lock().await;
                mock_data.shutdown_called = true;
                Ok(())
            }
        }
    }
}
