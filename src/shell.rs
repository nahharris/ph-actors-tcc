mod core;
mod data;
mod mock;
mod message;
#[cfg(test)]
mod tests;

use anyhow::Context;
use tokio::sync::mpsc::Sender;

use crate::{ArcSlice, ArcStr};

/// The shell actor that provides a thread-safe interface for executing external programs.
///
/// This enum represents either a real shell actor or a mock implementation
/// for testing purposes. It provides a unified interface for executing external
/// programs regardless of the underlying implementation.
///
/// # Examples
/// ```ignore
/// let shell = Shell::spawn(log).await?;
/// let result = shell.execute(ArcStr::from("ls"), ArcSlice::from(&[ArcStr::from("-la")]), Some(ArcStr::from("input"))).await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender or mock reference.
#[derive(Debug, Clone)]
pub enum Shell {
    /// A real shell actor that executes external programs
    Actual(Sender<message::Message>),
    /// A mock implementation for testing that stores commands in memory
    Mock(mock::Mock),
}

impl Shell {
    /// Creates a new shell instance and spawns its actor.
    ///
    /// # Arguments
    /// * `log` - The logging actor for logging shell operations
    ///
    /// # Returns
    /// A new shell instance with a spawned actor.
    pub async fn spawn(log: crate::log::Log) -> anyhow::Result<Self> {
        let (shell, _) = core::Core::new(log).spawn();
        Ok(shell)
    }

    /// Creates a new mock shell instance for testing.
    ///
    /// # Returns
    /// A new mock shell instance that stores commands in memory.
    pub fn mock() -> Self {
        Self::Mock(mock::Mock::new())
    }

    /// Executes an external program with the given arguments and optional stdin.
    ///
    /// # Arguments
    /// * `program` - The name or path of the program to execute (ArcStr)
    /// * `args` - Command line arguments to pass to the program (ArcSlice<ArcStr>)
    /// * `stdin` - Optional input to provide to the program's stdin (Option<ArcStr>)
    ///
    /// # Returns
    /// A structured result containing stdout, stderr, and exit status.
    pub async fn execute(
        &self,
        program: ArcStr,
        args: ArcSlice<ArcStr>,
        stdin: Option<ArcStr>,
    ) -> anyhow::Result<data::Result> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let command = data::Command {
                    program,
                    args,
                    stdin,
                };
                sender
                    .send(message::Message::Execute { tx, command })
                    .await
                    .context("Executing command with Shell")
                    .expect("shell actor died");
                rx.await
                    .context("Awaiting response for command execution with Shell")
                    .expect("shell actor died")
            }
            Self::Mock(mock) => {
                mock.execute(program, args, stdin).await
            }
        }
    }

    /// Gets all executed commands from the mock implementation.
    /// This method is only available for mock instances and is useful for testing.
    ///
    /// # Returns
    /// A vector of all executed commands, or None if this is not a mock instance.
    pub async fn get_commands(&self) -> Option<Vec<data::Command>> {
        match self {
            Self::Mock(mock) => {
                Some(mock.get_commands().await)
            }
            Self::Actual(_) => None,
        }
    }
}
