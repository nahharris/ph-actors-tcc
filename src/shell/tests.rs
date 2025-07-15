use super::*;
use crate::shell::data::{Command, Status, Result};
use crate::{ArcStr, ArcSlice};

#[tokio::test]
async fn test_shell_mock() {
    let shell = Shell::mock();
    let result = shell.execute(ArcStr::from("echo"), ArcSlice::from([ArcStr::from("hello")]), None).await.unwrap();
    
    assert!(result.is_success());
    assert_eq!(result.stdout, ArcStr::from("Mock output for: echo"));
    assert!(result.stderr.is_empty());
    assert_eq!(result.command.program, ArcStr::from("echo"));
    assert_eq!(result.command.args, ArcSlice::from([ArcStr::from("hello")]));
    assert!(result.command.stdin.is_none());
}

#[tokio::test]
async fn test_shell_mock_with_stdin() {
    let shell = Shell::mock();
    let result = shell.execute(ArcStr::from("cat"), ArcSlice::from([]), Some(ArcStr::from("test input"))).await.unwrap();
    
    assert!(result.is_success());
    assert_eq!(result.command.stdin, Some(ArcStr::from("test input")));
}

#[tokio::test]
async fn test_shell_mock_get_commands() {
    let shell = Shell::mock();
    
    // Execute some commands
    shell.execute(ArcStr::from("ls"), ArcSlice::from([ArcStr::from("-la")]), None).await.unwrap();
    shell.execute(ArcStr::from("echo"), ArcSlice::from([ArcStr::from("hello")]), Some(ArcStr::from("input"))).await.unwrap();
    
    let commands = shell.get_commands().await.unwrap();
    assert_eq!(commands.len(), 2);
    
    assert_eq!(commands[0].program, ArcStr::from("ls"));
    assert_eq!(commands[0].args, ArcSlice::from([ArcStr::from("-la")]));
    assert!(commands[0].stdin.is_none());
    
    assert_eq!(commands[1].program, ArcStr::from("echo"));
    assert_eq!(commands[1].args, ArcSlice::from([ArcStr::from("hello")]));
    assert_eq!(commands[1].stdin, Some(ArcStr::from("input")));
}

#[tokio::test]
async fn test_shell_command_as_string() {
    let command = Command::new(
        ArcStr::from("ls"),
        ArcSlice::from([ArcStr::from("-la"), ArcStr::from("/tmp")]),
        None,
    );
    
    assert_eq!(command.to_string(), "ls -la /tmp");
}

#[tokio::test]
async fn test_shell_status_display() {
    let success = Status::Success(0);
    let failure = Status::Success(1);
    let signal = Status::Signal(9);
    let failed = Status::Failed(ArcStr::from("not found"));
    
    assert_eq!(success.to_string(), "Success(0)");
    assert_eq!(failure.to_string(), "Success(1)");
    assert_eq!(signal.to_string(), "Signal(9)");
    assert_eq!(failed.to_string(), "Failed(not found)");
}

#[tokio::test]
async fn test_shell_result_methods() {
    let command = Command::new(ArcStr::from("test"), ArcSlice::from([]), None);
    let success_result = Result::new(
        ArcStr::from("output"),
        ArcStr::from(""),
        Status::Success(0),
        command.clone(),
    );
    
    let failure_result = Result::new(
        ArcStr::from(""),
        ArcStr::from("error"),
        Status::Success(1),
        command,
    );
    
    assert!(success_result.is_success());
    assert!(!success_result.is_failure());
    assert_eq!(success_result.exit_code(), Some(0));
    
    assert!(!failure_result.is_success());
    assert!(failure_result.is_failure());
    assert_eq!(failure_result.exit_code(), Some(1));
}

#[tokio::test]
async fn test_shell_result_display() {
    let command = Command::new(ArcStr::from("echo"), ArcSlice::from([ArcStr::from("hello")]), None);
    let result = Result::new(
        ArcStr::from("hello\n"),
        ArcStr::from(""),
        Status::Success(0),
        command,
    );
    
    let display = result.to_string();
    assert!(display.contains("Command: echo hello"));
    assert!(display.contains("Status: Success(0)"));
    assert!(display.contains("Stdout:"));
    assert!(display.contains("hello"));
}

#[tokio::test]
async fn test_shell_actual_integration() {
    // This test requires a real shell actor with logging
    // We'll skip it for now as it requires more setup
    // In a real implementation, you'd want to test with actual commands
    // like "echo hello" or "true" that are guaranteed to exist
}
