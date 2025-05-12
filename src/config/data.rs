use serde::{Deserialize, Serialize};

use crate::{ArcPath, log::LogLevel};

/// Options for path-based configuration values that can be accessed and modified.
#[derive(Debug, Clone, Copy)]
pub enum PathOpt {
    /// Directory where log files are stored
    LogDir,
}

/// Options for numeric configuration values that can be accessed and modified.
#[derive(Debug, Clone, Copy)]
pub enum USizeOpt {
    /// Maximum age of log files in days before they are deleted
    MaxAge,
}

/// The configuration data structure that holds all configurable values.
///
/// This struct is responsible for storing and managing all configuration values.
/// It provides methods to access and modify these values in a type-safe manner.
///
/// # Thread Safety
/// This type is designed to be safely shared between threads when wrapped in an `Arc<Mutex<>>`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct Data {
    /// Directory where log files are stored
    log_dir: ArcPath,
    /// Current log level
    log_level: LogLevel,
    /// Maximum age of log files in days before they are deleted
    max_age: usize,
}

impl Data {
    /// Gets a path-based configuration value.
    ///
    /// # Arguments
    /// * `opt` - The path option to retrieve
    ///
    /// # Returns
    /// The requested path value.
    pub(crate) fn path(&self, opt: PathOpt) -> ArcPath {
        match opt {
            PathOpt::LogDir => self.log_dir.clone(),
        }
    }

    /// Sets a path-based configuration value.
    ///
    /// # Arguments
    /// * `opt` - The path option to set
    /// * `path` - The new path value
    pub(crate) fn set_path(&mut self, opt: PathOpt, path: ArcPath) {
        match opt {
            PathOpt::LogDir => self.log_dir = path,
        }
    }

    /// Gets the current log level.
    ///
    /// # Returns
    /// The current log level.
    pub(crate) fn log_level(&self) -> LogLevel {
        self.log_level
    }

    /// Sets the log level.
    ///
    /// # Arguments
    /// * `level` - The new log level value
    pub(crate) fn set_log_level(&mut self, level: LogLevel) {
        self.log_level = level;
    }

    /// Gets a numeric configuration value.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to retrieve
    ///
    /// # Returns
    /// The requested numeric value.
    pub(crate) fn usize(&self, opt: USizeOpt) -> usize {
        match opt {
            USizeOpt::MaxAge => self.max_age,
        }
    }

    /// Sets a numeric configuration value.
    ///
    /// # Arguments
    /// * `opt` - The numeric option to set
    /// * `value` - The new numeric value
    pub(crate) fn set_usize(&mut self, opt: USizeOpt, value: usize) {
        match opt {
            USizeOpt::MaxAge => self.max_age = value,
        }
    }
}
