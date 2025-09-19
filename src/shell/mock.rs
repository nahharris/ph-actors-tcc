use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{ArcSlice, ArcStr};
use crate::shell::data::{Command, Result, Status};

/// Mock implementation of the Shell actor for testing purposes.
///
/// This struct stores executed commands in memory and returns predefined results,
/// allowing tests to run without executing actual external programs.
#[derive(Debug, Clone)]
pub struct Mock {
    commands: Arc<Mutex<Vec<Command>>>,
}

impl Mock {
    /// Creates a new mock instance with an empty command store.
    pub fn new() -> Self {
        Self {
            commands: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Executes an external program with the given arguments and optional stdin.
    /// Mock implementation stores the command and returns a success result.
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
    ) -> anyhow::Result<Result> {
        let mut lock = self.commands.lock().await;
        let command = Command {
            program: program.clone(),
            args: args.clone(),
            stdin: stdin.clone(),
        };
        lock.push(command.clone());

        // Mock implementation returns a success result
        Ok(Result {
            stdout: ArcStr::from(format!("Mock output for: {}", command.program).as_str()),
            stderr: ArcStr::from(""),
            status: Status::Success(0),
            command,
        })
    }

    /// Gets all executed commands from the mock implementation.
    ///
    /// # Returns
    /// A vector of all executed commands.
    pub async fn get_commands(&self) -> Vec<Command> {
        let lock = self.commands.lock().await;
        lock.clone()
    }
}
