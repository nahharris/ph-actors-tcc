use tokio::sync::oneshot;

use super::data::Command;
use crate::error::ShellError;

/// Messages that can be sent to a [`Shell`] actor.
///
/// This enum defines the different types of shell operations that can be performed
/// through the actor system.
#[derive(Debug)]
pub enum Message {
    /// Executes an external program with the given command
    Execute {
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<Result<super::data::Result, ShellError>>,
        /// The command to execute
        command: Command,
    },
}
