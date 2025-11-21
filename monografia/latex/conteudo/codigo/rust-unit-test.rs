#[tokio::test]
async fn test_my_actor_operation() {
    let mut mock_dep = MockOtherActor::new();
    
    // Configura o comportamento esperado do mock
    mock_dep 
        .expect_get_value()
        .times(1)
        .returning(|| Ok(42));
    
    let actor = App::spawn(mock_dep);

    let result = actor.operation().await;
    
    assert_eq!(result, Ok(42));
}

