use anyhow::Context;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ArcStr;

/// Mock implementation of the Render actor for testing purposes.
///
/// This struct contains predefined rendered content for various patch inputs,
/// allowing tests to run without executing external rendering programs.
#[derive(Debug, Clone)]
pub struct Mock {
    responses: Arc<Mutex<HashMap<ArcStr, ArcStr>>>,
}

impl Mock {
    /// Creates a new mock instance with the provided responses.
    ///
    /// # Arguments
    /// * `responses` - Initial response cache mapping patch content to rendered output
    pub fn new(responses: HashMap<ArcStr, ArcStr>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
        }
    }

    /// Renders patch content using mock responses.
    ///
    /// # Arguments
    /// * `content` - The raw patch content to render
    ///
    /// # Returns
    /// The rendered patch content as a string, or an error if not found in mock responses.
    pub async fn render_patch(&self, content: ArcStr) -> anyhow::Result<ArcStr> {
        let lock = self.responses.lock().await;
        lock.get(&content)
            .context("No more mocked responses")
            .cloned()
    }
}
