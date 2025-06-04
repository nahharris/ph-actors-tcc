use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

/// Describes a message to be logged.
///
/// Contains both the message content and its associated log level.
/// This struct is used internally by the logger to manage log entries.
///
/// # Examples
/// ```
/// let msg = LogMessage {
///     level: LogLevel::Info,
///     message: "Application started".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: LogLevel,
    pub message: String,
}

impl std::fmt::Display for LogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.level, self.message)
    }
}

/// Describes the log level of a message.
///
/// This enum is used to determine the severity of a log message so the logger
/// can handle it according to the configured verbosity level.
///
/// # Ordering
/// The levels are ordered by severity: `Info` < `Warning` < `Error`
///
/// # Examples
/// ```
/// let level = LogLevel::Info;
/// assert!(level < LogLevel::Warning);
/// assert!(level < LogLevel::Error);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum LogLevel {
    #[default]
    /// The lowest level, dedicated to regular information that is not critical.
    /// Used for general operational messages and debugging information.
    Info,
    /// Mid level, used to indicate when something went wrong but it's not
    /// critical. Used for recoverable errors or potential issues.
    Warning,
    /// The highest level, used to indicate critical errors that require attention
    /// but are not severe enough to crash the program.
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl FromStr for LogLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warning),
            "error" => Ok(LogLevel::Error),
            _ => Err(anyhow::anyhow!("Invalid log level: {}", s)),
        }
    }
}
