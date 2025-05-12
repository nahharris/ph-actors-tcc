use anyhow::{Context, bail};
use ratatui::{
    crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
};
use std::io::{Stdout, stdout};
use tokio::task::JoinHandle;

use crate::log::Log;

/// The core of a terminal actor, responsible for managing terminal operations.
///
/// This struct provides thread-safe access to terminal operations through an actor pattern.
/// It handles low-level terminal operations such as drawing, cursor movement, and screen management
/// using the ratatui library.
///
/// # Features
/// - Thread-safe terminal operations through actor pattern
/// - Alternate screen management
/// - Raw mode control
/// - Terminal state tracking
///
/// # Examples
/// ```
/// let terminal = TerminalCore::build(log)?;
/// let (terminal, _) = terminal.spawn();
/// terminal.take_over().await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads through the actor pattern.
/// All terminal operations are handled sequentially to ensure consistency.
#[derive(Debug)]
pub struct TerminalCore {
    /// Logging interface for operation logging
    #[allow(dead_code)]
    log: Log,
    /// Indicates if the terminal has been taken over
    take_over: bool,
    /// The terminal backend for drawing operations
    terminal: ratatui::Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalCore {
    /// Creates a new terminal instance.
    ///
    /// # Arguments
    /// * `log` - The logging actor for operation logging
    ///
    /// # Returns
    /// A new instance of `TerminalCore` with a fresh terminal backend.
    ///
    /// # Errors
    /// Returns an error if the terminal cannot be created or initialized.
    pub fn build(log: Log) -> anyhow::Result<Self> {
        let terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout()))?;
        let core = Self {
            take_over: false,
            terminal,
            log,
        };

        Ok(core)
    }

    /// Transforms the terminal core instance into an actor.
    ///
    /// This method spawns a new task that will handle terminal operations
    /// asynchronously through a message channel. All operations are processed
    /// sequentially to ensure consistency.
    ///
    /// # Returns
    /// A tuple containing:
    /// - A [`Terminal`] instance that can be used to send messages to the actor
    /// - A join handle for the spawned task
    ///
    /// # Panics
    /// This function will panic if the underlying task fails to spawn.
    pub fn spawn(mut self) -> (Terminal, JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    Message::TakeOver(tx) => {
                        let res = self.take_over();
                        let _ = tx.send(res);
                    }
                    Message::Release(tx) => {
                        let res = self.release();
                        let _ = tx.send(res);
                    }
                }
            }
        });
        (Terminal::Actual(tx), handle)
    }

    /// Takes over the terminal by entering the alternate screen and enabling raw mode.
    ///
    /// This method is necessary to draw on the terminal screen and handle input events.
    /// It should be called at the start of the program or before any command that
    /// needs terminal screen control.
    ///
    /// # Returns
    /// `Ok(())` if the terminal was successfully taken over.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The terminal is already taken over
    /// - The alternate screen cannot be entered
    /// - Raw mode cannot be enabled
    fn take_over(&mut self) -> anyhow::Result<()> {
        if self.take_over {
            bail!("Terminal already taken over");
        }
        self.take_over = true;
        execute!(self.terminal.backend_mut(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        Ok(())
    }

    /// Releases the terminal by leaving the alternate screen and disabling raw mode.
    ///
    /// This method should be called before the program exits or before executing
    /// a command that needs the normal terminal screen.
    ///
    /// # Returns
    /// `Ok(())` if the terminal was successfully released.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The terminal is not taken over
    /// - The alternate screen cannot be left
    /// - Raw mode cannot be disabled
    fn release(&mut self) -> anyhow::Result<()> {
        if !self.take_over {
            bail!("Terminal not taken over");
        }
        self.take_over = false;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
}

/// Messages that can be sent to a [`TerminalCore`] actor.
///
/// This enum defines the different types of terminal operations that can be performed
/// through the terminal actor system.
#[derive(Debug)]
pub enum Message {
    /// Takes over the terminal by entering alternate screen and enabling raw mode
    TakeOver(tokio::sync::oneshot::Sender<anyhow::Result<()>>),
    /// Releases the terminal by leaving alternate screen and disabling raw mode
    Release(tokio::sync::oneshot::Sender<anyhow::Result<()>>),
}

/// The terminal actor that provides a thread-safe interface for terminal operations.
///
/// This enum represents either a real terminal actor or a mock implementation
/// for testing purposes. It provides a unified interface for terminal operations
/// regardless of the underlying implementation.
///
/// # Examples
/// ```
/// let (terminal, _) = TerminalCore::build(log)?.spawn();
/// terminal.take_over().await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender.
#[derive(Debug)]
pub enum Terminal {
    /// A real terminal actor that manages the terminal screen
    Actual(tokio::sync::mpsc::Sender<Message>),
    /// A mock implementation for testing that does nothing
    #[allow(dead_code)]
    Mock,
}

#[allow(dead_code)]
impl Terminal {
    /// Creates a new mock terminal instance for testing.
    ///
    /// This implementation does nothing and always succeeds, making it suitable
    /// for testing code that uses the terminal.
    ///
    /// # Returns
    /// A new mock terminal instance.
    pub fn mock() -> Self {
        Terminal::Mock
    }

    /// Takes over the terminal by entering alternate screen and enabling raw mode.
    ///
    /// # Returns
    /// `Ok(())` if the terminal was successfully taken over.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The terminal is already taken over
    /// - The alternate screen cannot be entered
    /// - Raw mode cannot be enabled
    /// - The terminal actor has died
    pub async fn take_over(&self) -> anyhow::Result<()> {
        match self {
            Terminal::Actual(tx) => {
                let (res_tx, res_rx) = tokio::sync::oneshot::channel();
                tx.send(Message::TakeOver(res_tx))
                    .await
                    .context("Failed to take over")
                    .expect("Terminal actor is dead");
                res_rx
                    .await
                    .context("Failed to take over")
                    .expect("Terminal actor is dead")
            }
            Terminal::Mock => Ok(()),
        }
    }

    /// Releases the terminal by leaving alternate screen and disabling raw mode.
    ///
    /// # Returns
    /// `Ok(())` if the terminal was successfully released.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The terminal is not taken over
    /// - The alternate screen cannot be left
    /// - Raw mode cannot be disabled
    /// - The terminal actor has died
    pub async fn release(&self) -> anyhow::Result<()> {
        match self {
            Terminal::Actual(tx) => {
                let (res_tx, res_rx) = tokio::sync::oneshot::channel();
                let _ = tx.send(Message::Release(res_tx)).await;
                res_rx.await?
            }
            Terminal::Mock => Ok(()),
        }
    }
}
