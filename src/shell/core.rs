use tokio::process::Command;
use tokio::sync::mpsc;

use super::data::{Command as ShellCommand, Result, Status};
use super::message::Message;
use crate::{error::ShellError, ArcStr};
#[cfg(not(test))]
use crate::log::Log;
#[cfg(test)]
use crate::log::mock::MockLog as Log;

const SCOPE: &str = "shell";

/// The core of the Shell actor, responsible for executing external programs.
///
/// This struct provides thread-safe access to shell operations through an actor pattern.
/// It wraps tokio's process functions and provides a safe interface for concurrent access.
/// All shell operations are logged for debugging and monitoring purposes.
#[derive(Debug)]
pub struct Core {
    /// The logging actor for logging shell operations
    log: Log,
}

impl Core {
    /// Creates a new Shell core instance.
    ///
    /// # Arguments
    /// * `log` - The logging actor for logging shell operations
    ///
    /// # Returns
    /// A new instance of `Core`.
    pub fn new(log: Log) -> Self {
        Self { log }
    }

    /// Initializes the shell actor message receiver.
    ///
    /// This method processes messages from the receiver in a loop, handling each message
    /// using pattern matching.
    ///
    /// # Arguments
    /// * `rx` - A receiver for messages to process
    pub async fn init(mut self, mut rx: mpsc::Receiver<Message>) {
        while let Some(msg) = rx.recv().await {
            use Message::*;
            match msg {
                Execute { tx, command } => self.handle_execute(tx, command).await,
            }
        }
    }

    /// Executes an external program with the given command.
    ///
    /// This method creates a new process, provides optional stdin input,
    /// captures stdout and stderr, and returns a structured result.
    ///
    /// # Arguments
    /// * `tx` - A oneshot channel sender to receive the result
    /// * `command` - The command to execute
    ///
    /// # Errors
    /// The function will return an error if the command cannot be executed or if there
    /// are any issues with the channel communication.
    async fn handle_execute(
        &mut self,
        tx: tokio::sync::oneshot::Sender<std::result::Result<Result, ShellError>>,
        command: ShellCommand,
    ) {
        let command_str = command.to_string();
        self.log
            .info(SCOPE, format!("Executing command: {command_str}"));

        let mut cmd = Command::new(&command.program);
        for arg in command.args.iter() {
            cmd.arg(arg);
        }

        // Set up stdin if provided
        let has_stdin = command.stdin.is_some();
        if has_stdin {
            cmd.stdin(std::process::Stdio::piped());
        }

        // Set up stdout and stderr capture
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let result = match cmd.spawn() {
            Ok(mut child) => {
                // Write to stdin if provided
                if let Some(stdin_data) = &command.stdin {
                    if let Some(stdin) = child.stdin.take() {
                        if let Err(e) = tokio::io::AsyncWriteExt::write_all(
                            &mut tokio::io::BufWriter::new(stdin),
                            stdin_data.as_bytes(),
                        )
                        .await
                        {
                            self.log
                                .error(SCOPE, format!("Failed to write to stdin: {e}"));
                            let _ = tx.send(Err(ShellError::ExecutionFailed {
                                command: command.program.to_string(),
                                args: command.args.iter().map(|a| a.to_string()).collect(),
                                message: format!("Failed to write to stdin: {}", e),
                                exit_code: None,
                            }));
                            return;
                        }
                    }
                }

                // Wait for the process to complete
                match child.wait_with_output().await {
                    Ok(output) => {
                        let status = Status::from(output.status);
                        let result = Result::new(
                            ArcStr::from(String::from_utf8_lossy(&output.stdout).to_string()),
                            ArcStr::from(String::from_utf8_lossy(&output.stderr).to_string()),
                            status,
                            command,
                        );

                        match &result.status {
                            Status::Success(0) => {
                                self.log.info(
                                    SCOPE,
                                    format!("Command completed successfully: {command_str}"),
                                );
                            }
                            Status::Success(code) => {
                                self.log.warn(
                                    SCOPE,
                                    format!(
                                        "Command completed with exit code {code}: {command_str}"
                                    ),
                                );
                            }
                            Status::Signal(signal) => {
                                self.log.error(
                                    SCOPE,
                                    format!("Command terminated by signal {signal}: {command_str}"),
                                );
                            }
                            Status::Failed(reason) => {
                                self.log.error(
                                    SCOPE,
                                    format!("Command failed: {command_str} - {reason}"),
                                );
                            }
                        }

                        Ok(result)
                    }
                    Err(e) => {
                        self.log.error(
                            SCOPE,
                            format!("Failed to wait for command: {command_str} - {e}"),
                        );
                        Err(ShellError::ExecutionFailed {
                            command: command.program.to_string(),
                            args: command.args.iter().map(|a| a.to_string()).collect(),
                            message: format!("Failed to wait for command: {}", e),
                            exit_code: None,
                        })
                    }
                }
            }
            Err(e) => {
                self.log.error(
                    SCOPE,
                    format!("Failed to spawn command: {command_str} - {e}"),
                );
                Err(ShellError::ExecutionFailed {
                    command: command.program.to_string(),
                    args: command.args.iter().map(|a| a.to_string()).collect(),
                    message: format!("Failed to spawn command: {}", e),
                    exit_code: None,
                })
            }
        };

        let _ = tx.send(result);
    }
}
