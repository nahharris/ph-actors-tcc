#[macro_use]
extern crate ph;
use ph::{log::Log, shell::Shell};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let log = Log::mock();

    // Initialize shell actor
    let shell = Shell::spawn(log.clone()).await?;

    println!("Shell Actor Example");
    println!("===================");

    // Example 1: Simple command execution
    println!("\n1. Executing 'echo hello world':");
    let result = shell
        .execute(arc_str!("echo"), arc_slice!["hello", "world"], None)
        .await?;
    println!("Command: {}", result.command.to_string());
    println!("Status: {}", result.status);
    println!("Stdout: {}", result.stdout.trim());
    if !result.stderr.is_empty() {
        println!("Stderr: {}", result.stderr);
    }

    // Example 2: Command with stdin input
    println!("\n2. Executing 'cat' with stdin input:");
    let result = shell
        .execute(
            arc_str!("cat"),
            arc_slice![],
            Some(arc_str!("Hello from stdin!")),
        )
        .await?;
    println!("Command: {}", result.command.to_string());
    println!("Status: {}", result.status);
    println!("Stdout: {}", result.stdout.trim());

    // Example 3: Command that might fail
    println!("\n3. Executing 'ls' on a non-existent directory:");
    let result = shell
        .execute(arc_str!("ls"), arc_slice!["/non/existent/path"], None)
        .await?;
    println!("Command: {}", result.command.to_string());
    println!("Status: {}", result.status);
    if !result.stdout.is_empty() {
        println!("Stdout: {}", result.stdout.trim());
    }
    if !result.stderr.is_empty() {
        println!("Stderr: {}", result.stderr.trim());
    }

    // Example 4: Using the result helper methods
    println!("\n4. Checking command results:");
    let success_result = shell.execute(arc_str!("true"), arc_slice![], None).await?;
    let failure_result = shell.execute(arc_str!("false"), arc_slice![], None).await?;

    println!("'true' command success: {}", success_result.is_success());
    println!("'true' command failure: {}", success_result.is_failure());
    println!("'true' exit code: {:?}", success_result.exit_code());

    println!("'false' command success: {}", failure_result.is_success());
    println!("'false' command failure: {}", failure_result.is_failure());
    println!("'false' exit code: {:?}", failure_result.exit_code());

    // Example 5: Using mock for testing
    println!("\n5. Using mock shell for testing:");
    let mock_shell = Shell::mock();
    let mock_result = mock_shell
        .execute(
            arc_str!("test_command"),
            arc_slice!["arg1", "arg2"],
            Some(arc_str!("mock input")),
        )
        .await?;
    println!("Mock command: {}", mock_result.command.to_string());
    println!("Mock result: {}", mock_result.stdout);

    // Get all commands from mock
    if let Some(commands) = mock_shell.get_commands().await {
        println!("Mock shell executed {} commands", commands.len());
        for (i, cmd) in commands.iter().enumerate() {
            println!("  {}. {}", i + 1, cmd.to_string());
        }
    }

    println!("\nExample completed successfully!");

    // Flush logs
    let _ = log.flush().await;

    Ok(())
}
