mod core;
pub mod data;
mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

/// Actor name for error reporting.
pub const ACTOR_NAME: &'static str = "Shell";

use tokio::sync::mpsc;

#[cfg(not(test))]
use crate::log::Log;
#[cfg(test)]
use crate::log::mock::MockLog as Log;

use crate::{error::ShellError, error::FatalActorError, ArcSlice, ArcStr};

/// The shell actor that provides a thread-safe interface for executing external programs.
///
/// This struct provides a unified interface for executing external programs through
/// an actor pattern. All shell operations are processed asynchronously and logged
/// for debugging and monitoring purposes.
///
/// # Examples
/// ```ignore
/// let shell = Shell::spawn();
/// let result = shell.execute(ArcStr::from("ls"), ArcSlice::from(&[ArcStr::from("-la")]), Some(ArcStr::from("input"))).await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender.
#[derive(Debug, Clone)]
pub struct Shell {
    tx: mpsc::Sender<message::Message>,
}

impl Shell {
    /// Creates a new shell instance and spawns its actor.
    ///
    /// # Arguments
    /// * `log` - The logging actor for logging shell operations
    ///
    /// # Returns
    /// A new shell instance with a spawned actor.
    pub async fn spawn(log: Log) -> Result<Self, ShellError> {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(log);
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Ok(Self { tx })
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
    ) -> Result<data::Result, crate::error::ShellError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let command = data::Command {
            program,
            args,
            stdin,
        };
        self.tx
            .send(message::Message::Execute { tx, command })
            .await
            .map_err(|_e| {
                ShellError::Fatal(FatalActorError::ActorSendFailed {
                    actor_name: crate::shell::ACTOR_NAME,
                    operation: "execute command".to_string(),
                })
            })?;
        rx.await
            .map_err(|e| {
                ShellError::Fatal(FatalActorError::ActorRecvFailed {
                    actor_name: crate::shell::ACTOR_NAME,
                    operation: "execute command".to_string(),
                    source: e,
                })
            })?
    }
}
