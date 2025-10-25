use super::mock::MockLog;

#[tokio::test]
async fn test_mock_log_operations() {
    let mut mock_log = MockLog::new();

    // Set up expectations for the methods that are actually mocked
    mock_log.expect_collect_garbage().times(1).returning(|| ());

    mock_log
        .expect_get_messages()
        .times(1)
        .returning(|| Some(vec![]));

    mock_log
        .expect_flush()
        .times(1)
        .returning(|| tokio::spawn(async {}));

    // Test the mock
    mock_log.collect_garbage().await;
    let messages = mock_log.get_messages().await;
    assert!(messages.is_some());

    let handle = mock_log.flush();
    handle.await.unwrap();
}
