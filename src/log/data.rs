use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

/// Describes a message to be logged.
///
/// Contains the message content, its associated log level, and a scope for categorization.
/// This struct is used internally by the logger to manage log entries.
///
/// # Examples
/// ```ignore
/// let msg = LogMessage {
///     level: LogLevel::Info,
///     scope: "app",
///     message: "Application started".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: LogLevel,
    pub scope: &'static str,
    pub message: String,
}

impl std::fmt::Display for LogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp = chrono::DateTime::from_timestamp(now.as_secs() as i64, now.subsec_nanos())
            .unwrap_or_default()
            .format("%Y-%m-%d %H:%M:%S UTC");
        write!(
            f,
            "[{}] [{}] [{}] {}",
            timestamp, self.level, self.scope, self.message
        )
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
/// ```ignore
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
            "warn" | "warning" => Ok(LogLevel::Warning),
            "error" => Ok(LogLevel::Error),
            _ => Err(anyhow::anyhow!("Invalid log level: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Info < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Error);
        assert!(LogLevel::Info < LogLevel::Error);
        assert_eq!(LogLevel::Info, LogLevel::Info);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Info.to_string(), "INFO");
        assert_eq!(LogLevel::Warning.to_string(), "WARN");
        assert_eq!(LogLevel::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("info").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("INFO").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("warn").unwrap(), LogLevel::Warning);
        assert_eq!(LogLevel::from_str("warning").unwrap(), LogLevel::Warning);
        assert_eq!(LogLevel::from_str("error").unwrap(), LogLevel::Error);
        assert!(LogLevel::from_str("notalevel").is_err());
    }

    #[test]
    fn test_log_message_display() {
        let msg = LogMessage {
            level: LogLevel::Error,
            scope: "test",
            message: "fail".to_string(),
        };
        let output = msg.to_string();
        // Check that the output contains the expected parts
        assert!(output.contains("[ERROR]"));
        assert!(output.contains("[test]"));
        assert!(output.contains("fail"));
        // Check the format: [timestamp] [level] [scope] message
        assert!(output.matches('[').count() == 3);
        assert!(output.matches(']').count() == 3);
    }

    #[test]
    fn test_log_message_ordering_and_equality() {
        let a = LogMessage {
            level: LogLevel::Info,
            scope: "test",
            message: "a".to_string(),
        };
        let b = LogMessage {
            level: LogLevel::Warning,
            scope: "test",
            message: "b".to_string(),
        };
        let c = LogMessage {
            level: LogLevel::Info,
            scope: "test",
            message: "a".to_string(),
        };
        assert!(a < b);
        assert_eq!(a, c);
    }
}
