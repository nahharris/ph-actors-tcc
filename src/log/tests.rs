use super::*;

#[tokio::test]
async fn test_log_message_creation() {
    // Test that we can create log messages
    let msg = LogMessage {
        level: LogLevel::Info,
        scope: "test",
        message: "test message".to_string(),
    };
    
    assert_eq!(msg.level, LogLevel::Info);
    assert_eq!(msg.scope, "test");
    assert_eq!(msg.message, "test message");
}

#[tokio::test]
async fn test_log_level_ordering() {
    // Test that log levels are ordered correctly
    assert!(LogLevel::Info < LogLevel::Warning);
    assert!(LogLevel::Warning < LogLevel::Error);
    assert!(LogLevel::Info < LogLevel::Error);
}

#[tokio::test]
async fn test_log_level_display() {
    // Test that log levels display correctly
    assert_eq!(LogLevel::Info.to_string(), "INFO");
    assert_eq!(LogLevel::Warning.to_string(), "WARN");
    assert_eq!(LogLevel::Error.to_string(), "ERROR");
}
