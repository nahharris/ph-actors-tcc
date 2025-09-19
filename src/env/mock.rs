use std::{collections::HashMap, env::VarError, ffi::OsString, fmt::Display, sync::Arc};
use tokio::sync::Mutex;

use crate::{ArcOsStr, ArcStr};

/// Mock implementation of the Env actor for testing purposes.
///
/// This struct contains predefined environment variables in memory,
/// allowing tests to run without affecting the actual system environment.
#[derive(Debug, Clone)]
pub struct Mock {
    variables: Arc<Mutex<HashMap<ArcOsStr, OsString>>>,
}

impl Mock {
    /// Creates a new mock instance with an empty environment variable store.
    pub fn new() -> Self {
        Self {
            variables: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Creates a new mock instance with the provided initial variables.
    ///
    /// # Arguments
    /// * `variables` - Initial environment variables to populate the mock with
    pub fn with_variables(variables: HashMap<ArcOsStr, OsString>) -> Self {
        Self {
            variables: Arc::new(Mutex::new(variables)),
        }
    }

    /// Sets an environment variable in the mock.
    ///
    /// # Arguments
    /// * `key` - The environment variable key
    /// * `value` - The environment variable value
    pub async fn set_env<V>(&self, key: ArcOsStr, value: V)
    where
        V: Display,
    {
        let value = format!("{value}").into();
        let mut variables = self.variables.lock().await;
        variables.insert(key, value);
    }

    /// Unsets an environment variable in the mock.
    ///
    /// # Arguments
    /// * `key` - The environment variable key to remove
    pub async fn unset_env(&self, key: ArcOsStr) {
        let mut variables = self.variables.lock().await;
        variables.remove(&key);
    }

    /// Gets an environment variable from the mock.
    ///
    /// # Arguments
    /// * `key` - The environment variable key
    ///
    /// # Returns
    /// The environment variable value if present, or `VarError::NotPresent` if not found.
    pub async fn env(&self, key: ArcOsStr) -> Result<ArcStr, VarError> {
        let variables = self.variables.lock().await;
        variables
            .get(&key)
            .map(|s| ArcStr::from(&s.to_string_lossy()))
            .ok_or(VarError::NotPresent)
    }
}
