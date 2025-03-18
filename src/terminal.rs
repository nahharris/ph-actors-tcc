use anyhow::bail;
use ratatui::{
    crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
};
use std::io::{Stdout, stdout};
use tokio::task::JoinHandle;

/// The core of a terminal actor, responsible for the low level operations to
/// manage the terminal (drawing, cursor movement, alternate screen, etc.)
#[derive(Debug)]
pub struct TerminalCore {
    /// Indicates if the terminal has been taken over
    take_over: bool,
    /// The terminal backend
    terminal: ratatui::Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalCore {
    /// Creates a new terminal core
    ///
    /// # Errors
    /// Fails if the terminal cannot be created (ratatui)
    pub fn build() -> anyhow::Result<Self> {
        let terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout()))?;
        let core = Self {
            take_over: false,
            terminal,
        };

        Ok(core)
    }

    /// Transforms an instance of [`TerminalCore`] into an actor ready to receive
    /// messages
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

    /// Takes over the terminal by entering the alternate screen and enabling raw
    /// mode. This is necessary to draw on the terminal screen and to handle input
    /// events.
    /// This is the opposite of [`TerminalCore::release`] and should be called
    /// just after the progrma starts or before the execution of a command that
    /// needs the terminal screen.
    fn take_over(&mut self) -> anyhow::Result<()> {
        if self.take_over {
            bail!("Terminal already taken over");
        }
        self.take_over = true;
        execute!(self.terminal.backend_mut(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        Ok(())
    }

    /// Releases the terminal by leaving the alternate screen and disabling raw
    /// mode. This is the opposite of [`TerminalCore::take_over`] and should be
    /// called before the program exits or before the execution of a command that
    /// needs the terminal screen.
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

/// Messages that can be sent to a [`TerminalCore`]
pub enum Message {
    TakeOver(tokio::sync::oneshot::Sender<anyhow::Result<()>>),
    Release(tokio::sync::oneshot::Sender<anyhow::Result<()>>),
}

/// The terminal actor, responsible for managing the terminal. This is a
/// transmitter to a running terminal actor.
///
/// Cloning this is cheap and can be done to send to multiple actors.
///
/// Instantiate a new terminal actor with [`TerminalCore::spawn`]
pub enum Terminal {
    Actual(tokio::sync::mpsc::Sender<Message>),
    #[allow(dead_code)]
    Mock,
}

#[allow(dead_code)]
impl Terminal {
    pub fn mock() -> Self {
        Terminal::Mock
    }

    pub async fn take_over(&self) -> anyhow::Result<()> {
        match self {
            Terminal::Actual(tx) => {
                let (res_tx, res_rx) = tokio::sync::oneshot::channel();
                let _ = tx.send(Message::TakeOver(res_tx)).await;
                res_rx.await?
            }
            Terminal::Mock => Ok(()),
        }
    }
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
