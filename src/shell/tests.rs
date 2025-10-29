use super::*;
use crate::shell::data::{Command, Result, Status};
use crate::{ArcSlice, ArcStr};

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
    let command = Command::new(
        ArcStr::from("echo"),
        ArcSlice::from([ArcStr::from("hello")]),
        None,
    );
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
    // Create a mock log for testing
    let mut log = crate::log::mock::MockLog::new();
    
    // Set up expectations for the log calls that will be made during command execution
    log.expect_info()
        .withf(|scope, message| scope == "shell" && message.contains("Executing command: echo hello"))
        .times(1)
        .returning(|_, _| ());
    
    log.expect_info()
        .withf(|scope, message| scope == "shell" && message.contains("Command completed successfully: echo hello"))
        .times(1)
        .returning(|_, _| ());
    
    let shell = Shell::spawn(log).await.unwrap();
    let result = shell
        .execute(
            ArcStr::from("echo"),
            ArcSlice::from([ArcStr::from("hello")]),
            None,
        )
        .await
        .unwrap();

    assert!(result.is_success());
    assert_eq!(result.stdout.trim(), "hello");
    assert!(result.stderr.is_empty());
    assert_eq!(result.command.program, ArcStr::from("echo"));
    assert_eq!(result.command.args, ArcSlice::from([ArcStr::from("hello")]));
    assert!(result.command.stdin.is_none());
}
