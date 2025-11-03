use std::ops::Deref;

use crate::{ArcOsStr, EnvError};

use super::Env;

#[tokio::test]
async fn test_env_operations() {
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
    assert!(matches!(result, Err(EnvError::NotFound { name: _ })));

    // Verify it's also unset in std::env
    let std_result = std::env::var(key.as_ref());
    assert!(matches!(std_result, Err(std::env::VarError::NotPresent)));
}

#[tokio::test]
async fn test_set_env() {
    let env = Env::spawn();
    let key = ArcOsStr::from("TEST_CORE_SET");
    let value = "test_value";

    // Remove env var if it exists
    unsafe { std::env::remove_var(key.as_ref()) };

    // Verify it's not set in the actor
    assert!(env.env(key.clone()).await.is_err());

    // Test set
    env.set_env(key.clone(), value).await;
    let result = env.env(key.clone()).await.expect("Getting environment variable to succeed");
    assert_eq!(result.deref(), value);

    // Cleanup
    env.unset_env(key.clone()).await;
}

#[tokio::test]
async fn test_unset_env() {
    let env = Env::spawn();
    let key = ArcOsStr::from("TEST_CORE_UNSET");
    let value = "test_value";

    // Set env var
    env.set_env(key.clone(), value).await;

    // Verify it's set in the actor
    let result = env.env(key.clone()).await.expect("Getting environment variable to succeed");
    assert_eq!(result.deref(), value);

    // Test unset
    env.unset_env(key.clone()).await;
    let result = env.env(key.clone()).await;
    assert!(matches!(result, Err(EnvError::NotFound { name: _ })));
}

#[tokio::test]
async fn test_get_env() {
    let env = Env::spawn();
    let key = ArcOsStr::from("TEST_CORE_GET");
    let value = "test_value";

    // Set env var
    unsafe { std::env::set_var(key.as_ref(), value) };

    // Test get
    let result = env.env(key.clone()).await.expect("Getting environment variable to succeed");
    assert_eq!(result.deref(), value);

    // Cleanup
    unsafe { std::env::remove_var(key.as_ref()) };
}
