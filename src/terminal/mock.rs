use std::sync::Arc;
use tokio::sync::Mutex;

use crate::terminal::data::{MockData, Screen};

/// Mock implementation of the Terminal actor for testing purposes.
///
/// This struct stores terminal state and operations in memory,
/// allowing tests to run without creating actual terminal interfaces.
#[derive(Debug, Clone)]
pub struct Mock {
    data: Arc<Mutex<MockData>>,
}

impl Mock {
    /// Creates a new mock instance with the provided mock data.
    ///
    /// # Arguments
    /// * `data` - Initial mock data containing terminal state
    pub fn new(data: MockData) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }

    /// Requests the terminal to show a specific screen.
    /// Mock implementation stores the screen in mock data.
    ///
    /// # Arguments
    /// * `screen` - The screen to show
    ///
    /// # Returns
    /// Ok(()) if successful
    pub async fn show(&self, screen: Screen) -> anyhow::Result<()> {
        let mut mock_data = self.data.lock().await;
        mock_data.last_screen = Some(screen);
        Ok(())
    }

    /// Requests the terminal to quit.
    /// Mock implementation sets the quit flag in mock data.
    ///
    /// # Returns
    /// Ok(()) if successful
    pub async fn quit(&self) -> anyhow::Result<()> {
        let mut mock_data = self.data.lock().await;
        mock_data.quit_called = true;
        Ok(())
    }

    /// Gets the mock data for inspection in tests.
    ///
    /// # Returns
    /// A copy of the current mock data
    pub async fn get_data(&self) -> MockData {
        let mock_data = self.data.lock().await;
        mock_data.clone()
    }
}
