use anyhow::{anyhow, bail};
use ratatui::{
    crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    prelude::CrosstermBackend,
};
use std::io::{Stdout, stdout};
use tokio::task::JoinHandle;

pub struct TerminalCore {
    take_over: bool,
    terminal: ratatui::Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalCore {
    pub fn build() -> anyhow::Result<Self> {
        let terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout()))?;
        let core = Self {
            take_over: false,
            terminal,
        };

        Ok(core)
    }

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

    fn take_over(&mut self) -> anyhow::Result<()> {
        if self.take_over {
            bail!("Terminal already taken over");
        }
        self.take_over = true;
        execute!(self.terminal.backend_mut(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        Ok(())
    }

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

pub enum Message {
    TakeOver(tokio::sync::oneshot::Sender<anyhow::Result<()>>),
    Release(tokio::sync::oneshot::Sender<anyhow::Result<()>>),
}

pub enum Terminal {
    Actual(tokio::sync::mpsc::Sender<Message>),
    Mock,
}

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
