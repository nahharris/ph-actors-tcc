use std::ops::Deref;

use crate::ArcOsStr;

use super::{core::Core, Env};

#[tokio::test]
async fn test_mock_env_creation() {
    let env = Env::mock();
    assert!(matches!(env, Env::Mock(_)));
}

#[tokio::test]
async fn test_actual_env_creation() {
    let env = Env::spawn();
    assert!(matches!(env, Env::Actual(_)));
}

#[tokio::test]
async fn test_mock_env_operations() {
    let env = Env::mock();
    let key = ArcOsStr::from("TEST_MOCK_ENV");
    let value = "test_value";

    // Test set and get
    env.set_env(key.clone(), value).await;
    let result = env.env(key.clone()).await.unwrap();
    assert_eq!(result.deref(), value);

    // Test unset
    env.unset_env(key.clone()).await;
    let result = env.env(key).await;
    assert!(matches!(result, Err(std::env::VarError::NotPresent)));
}

#[tokio::test]
async fn test_actual_env_operations() {
    let env = Env::spawn();
    let key = ArcOsStr::from("TEST_ACTUAL_ENV");
    let value = "test_value";

    // Remove env var if it exists
    unsafe { std::env::remove_var(key.as_ref()) };

    // Verify it's not set in std::env
    assert!(std::env::var(key.as_ref()).is_err());

    // Test set and get
    env.set_env(key.clone(), value).await;
    let result = env.env(key.clone()).await.unwrap();
    assert_eq!(result.deref(), value);

    // Verify it's also set in std::env
    let std_result = std::env::var(key.as_ref()).unwrap();
    assert_eq!(std_result, value);

    // Test unset
    env.unset_env(key.clone()).await;
    let result = env.env(key.clone()).await;
    assert!(matches!(result, Err(std::env::VarError::NotPresent)));

    // Verify it's also unset in std::env
    let std_result = std::env::var(key.as_ref());
    assert!(matches!(std_result, Err(std::env::VarError::NotPresent)));
}

#[tokio::test]
async fn test_core_set_env() {
    let core = Core::new();
    let key = ArcOsStr::from("TEST_CORE_SET");
    let value = "test_value";

    // Remove env var if it exists
    unsafe { std::env::remove_var(key.as_ref()) };

    // Verify it's not set
    assert!(std::env::var(key.as_ref()).is_err());

    // Test set
    core.set_env(key.clone(), value.into());
    let result = std::env::var(key.as_ref()).unwrap();
    assert_eq!(result, value);

    // Cleanup
    unsafe { std::env::remove_var(key.as_ref()) };
}

#[tokio::test]
async fn test_core_unset_env() {
    let core = Core::new();
    let key = ArcOsStr::from("TEST_CORE_UNSET");
    let value = "test_value";

    // Set env var
    unsafe { std::env::set_var(key.as_ref(), value) };

    // Verify it's set
    assert_eq!(std::env::var(key.as_ref()).unwrap(), value);

    // Test unset
    core.unset_env(key.clone());
    let result = std::env::var(key.as_ref());
    assert!(matches!(result, Err(std::env::VarError::NotPresent)));
}

#[tokio::test]
async fn test_core_get_env() {
    let core = Core::new();
    let key = ArcOsStr::from("TEST_CORE_GET");
    let value = "test_value";

    // Set env var
    unsafe { std::env::set_var(key.as_ref(), value) };

    // Test get
    let (tx, rx) = tokio::sync::oneshot::channel();
    core.get_env(tx, key.clone());
    let result = rx.await.unwrap().unwrap();
    assert_eq!(result.deref(), value);

    // Cleanup
    unsafe { std::env::remove_var(key.as_ref()) };
}

#[tokio::test]
async fn test_core_spawn() {
    let core = Core::new();
    let (env, handle) = core.spawn();
    assert!(matches!(env, Env::Actual(_)));

    // Test the spawned actor
    let key = ArcOsStr::from("TEST_CORE_SPAWN");
    let value = "test_value";

    // Remove env var if it exists
    unsafe { std::env::remove_var(key.as_ref()) };

    // Test set and get through the actor
    env.set_env(key.clone(), value).await;
    let result = env.env(key.clone()).await.unwrap();
    assert_eq!(result.deref(), value);

    // Cleanup
    env.unset_env(key).await;
    handle.abort();
}

#[tokio::test]
async fn test_multiple_env_variables() {
    let env = Env::mock();
    let test_cases = vec![
        ("TEST_VAR_1", "value1"),
        ("TEST_VAR_2", "value2"),
        ("TEST_VAR_3", "value3"),
    ];

    // Test setting multiple variables
    for (key, value) in &test_cases {
        let key = ArcOsStr::from(*key);
        env.set_env(key.clone(), *value).await;
        let result = env.env(key).await.unwrap();
        assert_eq!(result.deref(), *value);
    }

    // Test unsetting multiple variables
    for (key, _) in &test_cases {
        let key = ArcOsStr::from(*key);
        env.unset_env(key.clone()).await;
        let result = env.env(key).await;
        assert!(matches!(result, Err(std::env::VarError::NotPresent)));
    }
}
