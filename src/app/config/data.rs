use serde::{Deserialize, Serialize};

use crate::{ArcPath, log::LogLevel};

/// Available renderers for patch content.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Renderer {
    /// Do not use an external renderer; show raw content
    #[default]
    None,
    /// Use the `bat` program for syntax highlighting
    Bat,
    /// Use the `delta` program for diff highlighting
    Delta,
}

impl Renderer {
    /// Gets the program name for this renderer.
    ///
    /// # Returns
    /// The program name as a string.
    pub fn program_name(&self) -> &'static str {
        match self {
            Renderer::None => "",
            Renderer::Bat => "bat",
            Renderer::Delta => "delta",
        }
    }

    /// Gets the default arguments for this renderer.
    ///
    /// # Returns
    /// A vector of default arguments for the renderer.
    pub fn default_args(&self) -> Vec<&'static str> {
        match self {
            Renderer::None => vec![],
            Renderer::Bat => vec![
                "--language=diff",
                "--paging=never",
                "--style=numbers",
                "--color=always",
            ],
            Renderer::Delta => vec!["--paging=never"],
        }
    }
}

/// Options for path-based configuration values that can be accessed and modified.
#[derive(Debug, Clone, Copy)]
pub enum PathOpt {
    /// Directory where log files are stored
    LogDir,
    /// Path to the cache file
    CachePath,
}

/// Options for numeric configuration values that can be accessed and modified.
#[derive(Debug, Clone, Copy)]
pub enum USizeOpt {
    /// Maximum age of log files in days before they are deleted
    MaxAge,
    /// Timeout for network requests in seconds
    Timeout,
}

/// Options for renderer configuration values that can be accessed and modified.
#[derive(Debug, Clone, Copy)]
pub enum RendererOpt {
    /// The renderer to use for patch content
    PatchRenderer,
}

/// The configuration data structure that holds all configurable values.
///
/// This struct is responsible for storing and managing all configuration values.
/// It provides methods to access and modify these values in a type-safe manner.
///
/// # Thread Safety
/// This type is designed to be safely shared between threads when wrapped in an `Arc<Mutex<>>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    /// Path to the cache file
    cache_path: ArcPath,
    /// Directory where log files are stored
    log_dir: ArcPath,
    /// Current log level
    log_level: LogLevel,
    /// Maximum age of log files in days before they are deleted
    max_age: usize,
    /// Timeout for network requests in seconds
    timeout: usize,
    /// The renderer to use for patch content
    patch_renderer: Renderer,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            cache_path: ArcPath::from("/tmp/patch-hub/cache"),
            log_dir: ArcPath::from("/tmp/patch-hub/logs"),
            log_level: LogLevel::Warning,
            max_age: 0,
            timeout: 30,
            patch_renderer: Renderer::default(),
        }
    }
}

impl Data {
    /// Gets a path-based configuration value.
    ///
    /// # Arguments
    /// * `opt` - The path option to retrieve
    ///
    /// # Returns
    /// The requested path value.
    pub fn path(&self, opt: PathOpt) -> ArcPath {
        match opt {
            PathOpt::LogDir => self.log_dir.clone(),
            PathOpt::CachePath => self.cache_path.clone(),
        }
    }

    /// Sets a path-based configuration value.
    ///
    /// # Arguments
    /// * `opt` - The path option to set
    /// * `path` - The new path value
    pub fn set_path(&mut self, opt: PathOpt, path: ArcPath) {
        match opt {
            PathOpt::LogDir => self.log_dir = path,
            PathOpt::CachePath => self.cache_path = path,
        }
    }

    /// Gets the current log level.
    ///
    /// # Returns
    /// The current log level.
    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    /// Sets the log level.
    ///
    /// # Arguments
    /// * `level` - The new log level value
    pub fn set_log_level(&mut self, level: LogLevel) {
        self.log_level = level;
    }

    /// Gets a numeric configuration value.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to retrieve
    ///
    /// # Returns
    /// The requested numeric value.
    pub fn usize(&self, opt: USizeOpt) -> usize {
        match opt {
            USizeOpt::MaxAge => self.max_age,
            USizeOpt::Timeout => self.timeout,
        }
    }

    /// Sets a numeric configuration value.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to set
    /// * `value` - The new numeric value
    pub fn set_usize(&mut self, opt: USizeOpt, value: usize) {
        match opt {
            USizeOpt::MaxAge => self.max_age = value,
            USizeOpt::Timeout => self.timeout = value,
        }
    }

    /// Gets a renderer configuration value.
    ///
    /// # Arguments
    /// * `opt` - The renderer option to retrieve
    ///
    /// # Returns
    /// The requested renderer value.
    pub fn renderer(&self, opt: RendererOpt) -> Renderer {
        match opt {
            RendererOpt::PatchRenderer => self.patch_renderer,
        }
    }

    /// Sets a renderer configuration value.
    ///
    /// # Arguments
    /// * `opt` - The renderer option to set
    /// * `renderer` - The new renderer value
    pub fn set_renderer(&mut self, opt: RendererOpt, renderer: Renderer) {
        match opt {
            RendererOpt::PatchRenderer => self.patch_renderer = renderer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_default_values() {
        let data = Data::default();
        assert_eq!(data.log_level(), LogLevel::Warning);
        assert_eq!(
            data.path(PathOpt::LogDir).to_str().unwrap(),
            "/tmp/patch-hub/logs"
        );
        assert_eq!(data.usize(USizeOpt::MaxAge), 0);
        assert_eq!(data.usize(USizeOpt::Timeout), 30);
        assert_eq!(
            data.path(PathOpt::CachePath).to_str().unwrap(),
            "/tmp/patch-hub/cache"
        );
    }

    #[test]
    fn test_data_setters_and_getters() {
        let mut data = Data::default();

        // Test log level
        data.set_log_level(LogLevel::Info);
        assert_eq!(data.log_level(), LogLevel::Info);

        // Test path
        let new_path = ArcPath::from("/var/log");
        data.set_path(PathOpt::LogDir, new_path.clone());
        assert_eq!(data.path(PathOpt::LogDir), new_path);

        let new_path = ArcPath::from("/var/cache");
        data.set_path(PathOpt::CachePath, new_path.clone());
        assert_eq!(data.path(PathOpt::CachePath), new_path);

        // Test max age
        data.set_usize(USizeOpt::MaxAge, 60);
        assert_eq!(data.usize(USizeOpt::MaxAge), 60);

        // Test timeout
        data.set_usize(USizeOpt::Timeout, 120);
        assert_eq!(data.usize(USizeOpt::Timeout), 120);
    }

    #[test]
    fn test_data_serialization() {
        let mut data = Data::default();
        data.set_log_level(LogLevel::Error);
        data.set_path(PathOpt::LogDir, ArcPath::from("/custom/log"));
        data.set_path(PathOpt::CachePath, ArcPath::from("/custom/cache"));
        data.set_usize(USizeOpt::MaxAge, 45);
        data.set_usize(USizeOpt::Timeout, 180);

        let toml = toml::to_string_pretty(&data).unwrap();
        let deserialized: Data = toml::from_str(&toml).unwrap();

        assert_eq!(data.log_level(), deserialized.log_level());
        assert_eq!(
            data.path(PathOpt::LogDir),
            deserialized.path(PathOpt::LogDir)
        );
        assert_eq!(
            data.path(PathOpt::CachePath),
            deserialized.path(PathOpt::CachePath)
        );
        assert_eq!(
            data.usize(USizeOpt::MaxAge),
            deserialized.usize(USizeOpt::MaxAge)
        );
        assert_eq!(
            data.usize(USizeOpt::Timeout),
            deserialized.usize(USizeOpt::Timeout)
        );
    }
}
