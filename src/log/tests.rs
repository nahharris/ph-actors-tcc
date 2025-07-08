use super::*;

#[tokio::test]
async fn test_log_info_warn_error() {
    let log = Log::Mock;
    log.info("info");
    log.warn("warn");
    log.error("error");
    // Should not panic or do anything
}

#[tokio::test]
async fn test_log_info_on_error() {
    let log = Log::Mock;
    let ok: Result<u32, &str> = Ok(42);
    let err: Result<u32, &str> = Err("fail");
    assert_eq!(log.info_on_error(ok), Ok(42));
    assert!(log.info_on_error(err).is_err());
}

#[tokio::test]
async fn test_log_warn_on_error() {
    let log = Log::Mock;
    let ok: Result<u32, &str> = Ok(42);
    let err: Result<u32, &str> = Err("fail");
    assert_eq!(log.warn_on_error(ok), Ok(42));
    assert!(log.warn_on_error(err).is_err());
}

#[tokio::test]
async fn test_log_error_on_error() {
    let log = Log::Mock;
    let ok: Result<u32, &str> = Ok(42);
    let err: Result<u32, &str> = Err("fail");
    assert_eq!(log.error_on_error(ok), Ok(42));
    assert!(log.error_on_error(err).is_err());
}

#[tokio::test]
async fn test_log_flush() {
    let log = Log::Mock;
    let _ = log.flush();
    // Should not panic or do anything
}

#[tokio::test]
async fn test_log_collect_garbage() {
    let log = Log::Mock;
    log.collect_garbage().await;
    // Should not panic or do anything
}
