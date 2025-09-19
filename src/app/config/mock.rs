use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{ArcPath, log::LogLevel};
use crate::app::config::{Data, PathOpt, Renderer, RendererOpt, USizeOpt};

/// Mock implementation of the Config actor for testing purposes.
///
/// This struct stores configuration data in memory,
/// allowing tests to run without creating actual configuration files.
#[derive(Debug, Clone)]
pub struct Mock {
    data: Arc<Mutex<Data>>,
}

impl Mock {
    /// Creates a new mock instance with the provided configuration data.
    ///
    /// # Arguments
    /// * `data` - Initial configuration data
    pub fn new(data: Data) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }

    /// Loads the configuration from the file.
    /// Mock implementation is a no-op that always succeeds.
    ///
    /// # Returns
    /// Ok(()) always
    pub async fn load(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Saves the current configuration to the file.
    /// Mock implementation is a no-op that always succeeds.
    ///
    /// # Returns
    /// Ok(()) always
    pub async fn save(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Gets a path-based configuration value.
    /// Mock implementation retrieves the value from stored data.
    ///
    /// # Arguments
    /// * `opt` - The path option to retrieve
    ///
    /// # Returns
    /// The requested path value
    pub async fn path(&self, opt: PathOpt) -> ArcPath {
        let data = self.data.lock().await;
        data.path(opt)
    }

    /// Sets a path-based configuration value.
    /// Mock implementation updates the stored data.
    ///
    /// # Arguments
    /// * `opt` - The path option to set
    /// * `path` - The new path value
    pub async fn set_path(&self, opt: PathOpt, path: ArcPath) {
        let mut data = self.data.lock().await;
        data.set_path(opt, path);
    }

    /// Gets the current log level.
    /// Mock implementation retrieves the value from stored data.
    ///
    /// # Returns
    /// The current log level
    pub async fn log_level(&self) -> LogLevel {
        let data = self.data.lock().await;
        data.log_level()
    }

    /// Sets the log level.
    /// Mock implementation updates the stored data.
    ///
    /// # Arguments
    /// * `level` - The new log level value
    pub async fn set_log_level(&self, level: LogLevel) {
        let mut data = self.data.lock().await;
        data.set_log_level(level);
    }

    /// Gets a numeric configuration value.
    /// Mock implementation retrieves the value from stored data.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to retrieve
    ///
    /// # Returns
    /// The requested numeric value
    pub async fn usize(&self, opt: USizeOpt) -> usize {
        let data = self.data.lock().await;
        data.usize(opt)
    }

    /// Sets a numeric configuration value.
    /// Mock implementation updates the stored data.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to set
    /// * `value` - The new numeric value
    pub async fn set_usize(&self, opt: USizeOpt, value: usize) {
        let mut data = self.data.lock().await;
        data.set_usize(opt, value);
    }

    /// Gets a renderer configuration value.
    /// Mock implementation retrieves the value from stored data.
    ///
    /// # Arguments
    /// * `opt` - The renderer option to retrieve
    ///
    /// # Returns
    /// The requested renderer value
    pub async fn renderer(&self, opt: RendererOpt) -> Renderer {
        let data = self.data.lock().await;
        data.renderer(opt)
    }

    /// Sets a renderer configuration value.
    /// Mock implementation updates the stored data.
    ///
    /// # Arguments
    /// * `opt` - The renderer option to set
    /// * `renderer` - The new renderer value
    pub async fn set_renderer(&self, opt: RendererOpt, renderer: Renderer) {
        let mut data = self.data.lock().await;
        data.set_renderer(opt, renderer);
    }

    /// Gets the mock data for inspection in tests.
    ///
    /// # Returns
    /// A copy of the current mock data
    pub async fn get_data(&self) -> Data {
        let data = self.data.lock().await;
        data.clone()
    }
}
