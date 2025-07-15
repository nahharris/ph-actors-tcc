use std::process::ExitStatus;

use crate::{ArcStr, ArcSlice};

/// Represents a shell command to be executed.
///
/// This struct contains all the information needed to execute an external program,
/// including the program name, arguments, and optional stdin input.
#[derive(Debug, Clone)]
pub struct Command {
    /// The name or path of the program to execute
    pub program: ArcStr,
    /// Command line arguments to pass to the program
    pub args: ArcSlice<ArcStr>,
    /// Optional input to provide to the program's stdin
    pub stdin: Option<ArcStr>,
}

impl Command {
    /// Creates a new shell command.
    ///
    /// # Arguments
    /// * `program` - The name or path of the program to execute
    /// * `args` - Command line arguments to pass to the program
    /// * `stdin` - Optional input to provide to the program's stdin
    ///
    /// # Returns
    /// A new shell command instance.
    pub fn new(program: ArcStr, args: ArcSlice<ArcStr>, stdin: Option<ArcStr>) -> Self {
        Self {
            program,
            args,
            stdin,
        }
    }

    /// Returns the full command as a string for display purposes.
    ///
    /// # Returns
    /// A string representation of the command with arguments.
    pub fn to_string(&self) -> String {
        let mut cmd = self.program.to_string();
        for arg in self.args.iter() {
            cmd.push(' ');
            cmd.push_str(arg);
        }
        cmd
    }
}

/// Represents the exit status of a shell command.
///
/// This enum provides a structured way to handle different types of command termination.
#[derive(Debug, Clone)]
pub enum Status {
    /// The command completed successfully with the given exit code
    Success(i32),
    /// The command was terminated by a signal with the given signal number
    Signal(i32),
    /// The command failed to start or execute
    Failed(ArcStr),
}

impl From<ExitStatus> for Status {
    fn from(status: ExitStatus) -> Self {
        if status.success() {
            Status::Success(status.code().unwrap_or(0))
        } else {
            match status.code() {
                Some(code) => Status::Success(code),
                None => Status::Signal(-1), // Unknown signal
            }
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Success(code) => write!(f, "Success({})", code),
            Status::Signal(signal) => write!(f, "Signal({})", signal),
            Status::Failed(reason) => write!(f, "Failed({})", reason),
        }
    }
}

/// Represents the result of executing a shell command.
///
/// This struct contains the complete output and status information from
/// executing an external program.
#[derive(Debug, Clone)]
pub struct Result {
    /// The standard output of the command
    pub stdout: ArcStr,
    /// The standard error output of the command
    pub stderr: ArcStr,
    /// The exit status of the command
    pub status: Status,
    /// The original command that was executed
    pub command: Command,
}

impl Result {
    /// Creates a new shell result.
    ///
    /// # Arguments
    /// * `stdout` - The standard output of the command
    /// * `stderr` - The standard error output of the command
    /// * `status` - The exit status of the command
    /// * `command` - The original command that was executed
    ///
    /// # Returns
    /// A new shell result instance.
    pub fn new(
        stdout: ArcStr,
        stderr: ArcStr,
        status: Status,
        command: Command,
    ) -> Self {
        Self {
            stdout,
            stderr,
            status,
            command,
        }
    }

    /// Returns true if the command executed successfully.
    ///
    /// # Returns
    /// True if the command completed with a success status.
    pub fn is_success(&self) -> bool {
        matches!(self.status, Status::Success(0))
    }

    /// Returns true if the command failed or was terminated.
    ///
    /// # Returns
    /// True if the command did not complete successfully.
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }

    /// Returns the exit code if the command completed normally.
    ///
    /// # Returns
    /// Some(exit_code) if the command completed normally, None otherwise.
    pub fn exit_code(&self) -> Option<i32> {
        match self.status {
            Status::Success(code) => Some(code),
            _ => None,
        }
    }
}

impl std::fmt::Display for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Command: {}", self.command.to_string())?;
        writeln!(f, "Status: {}", self.status)?;
        if !self.stdout.is_empty() {
            writeln!(f, "Stdout:")?;
            write!(f, "{}", self.stdout)?;
        }
        if !self.stderr.is_empty() {
            writeln!(f, "Stderr:")?;
            write!(f, "{}", self.stderr)?;
        }
        Ok(())
    }
}
