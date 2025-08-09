use super::*;

#[tokio::test]
async fn test_log_info_warn_error() {
    let log = Log::mock();
    log.info("test", "info");
    log.warn("test", "warn");
    log.error("test", "error");
    // Should not panic or do anything
}

#[tokio::test]
async fn test_log_info_on_error() {
    let log = Log::mock();
    let ok: Result<u32, &str> = Ok(42);
    let err: Result<u32, &str> = Err("fail");
    assert_eq!(log.info_on_error("test", ok), Ok(42));
    assert!(log.info_on_error("test", err).is_err());
}

#[tokio::test]
async fn test_log_warn_on_error() {
    let log = Log::mock();
    let ok: Result<u32, &str> = Ok(42);
    let err: Result<u32, &str> = Err("fail");
    assert_eq!(log.warn_on_error("test", ok), Ok(42));
    assert!(log.warn_on_error("test", err).is_err());
}

#[tokio::test]
async fn test_log_error_on_error() {
    let log = Log::mock();
    let ok: Result<u32, &str> = Ok(42);
    let err: Result<u32, &str> = Err("fail");
    assert_eq!(log.error_on_error("test", ok), Ok(42));
    assert!(log.error_on_error("test", err).is_err());
}

#[tokio::test]
async fn test_log_flush() {
    let log = Log::mock();
    let _ = log.flush();
    // Should not panic or do anything
}

#[tokio::test]
async fn test_log_collect_garbage() {
    let log = Log::mock();
    log.collect_garbage().await;
    // Should not panic or do anything
}

#[tokio::test]
async fn test_log_get_messages() {
    let log = Log::mock();
    log.info("test", "test message");
    log.warn("test", "warning message");

    // Give some time for async operations to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let messages = log.get_messages().await;
    assert!(messages.is_some());
    let messages = messages.unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].level, LogLevel::Info);
    assert_eq!(messages[0].message, "test message");
    assert_eq!(messages[1].level, LogLevel::Warning);
    assert_eq!(messages[1].message, "warning message");
}
