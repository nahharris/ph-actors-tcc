//! Error types for the application.
//!
//! This module provides a comprehensive error handling system that distinguishes
//! between recoverable and unrecoverable errors, enabling better error handling
//! strategies and runtime decision-making.
//!
//! # Architecture
//!
//! Errors are organized hierarchically:
//! - **Fatal errors**: System-level failures that cannot be recovered from
//! - **Actor-specific errors**: Each actor defines only the errors it can emit
//! - **Composable**: All errors compose into `AppError` for unified handling

use miette::Diagnostic;
use thiserror::Error;

// ============================================================================
// Fatal (Unrecoverable) Errors
// ============================================================================

/// Fatal errors that indicate system-level failures.
///
/// These errors typically mean that an actor has died or communication has failed
/// in a way that cannot be recovered from without restarting the affected component.
#[derive(Debug, Error, Diagnostic)]
#[error("Fatal actor error")]
pub enum FatalActorError {
    /// Actor has died - communication channel closed.
    #[error("Actor '{actor_name}' has died - communication channel closed")]
    #[diagnostic(
        code(fatal::actor_died),
        help("The actor is no longer running. This is a fatal error that requires restarting the affected component.")
    )]
    ActorDied {
        /// Name of the actor that died
        actor_name: &'static str,
    },

    /// Failed to send message to actor.
    #[error("Failed to send message to actor '{actor_name}' for operation '{operation}'")]
    #[diagnostic(
        code(fatal::actor_send_failed),
        help("The message could not be sent to the actor. The actor may have died or the channel may be closed.")
    )]
    ActorSendFailed {
        /// Name of the actor
        actor_name: &'static str,
        /// Description of the operation being performed
        operation: String,
    },

    /// Failed to receive response from actor.
    #[error("Failed to receive response from actor '{actor_name}' for operation '{operation}'")]
    #[diagnostic(
        code(fatal::actor_recv_failed),
        help("The response could not be received from the actor. The actor may have died or the channel may be closed.")
    )]
    ActorRecvFailed {
        /// Name of the actor
        actor_name: &'static str,
        /// Description of the operation being performed
        operation: String,
        /// Source error from the receive operation
        #[source]
        source: tokio::sync::oneshot::error::RecvError,
    },
}

// ============================================================================
// Actor-Specific Error Types
// ============================================================================

/// Errors that can occur during network operations.
#[derive(Debug, Error, Diagnostic)]
#[error("Network operation error")]
pub enum NetError {
    /// Network request failed.
    #[error("Network request failed: {message}")]
    #[diagnostic(
        code(net::request_failed),
        help("The network request failed. Check your connection and try again.")
    )]
    RequestFailed {
        /// URL that was requested
        url: String,
        /// HTTP method used
        method: String,
        /// Whether this error is retryable
        retryable: bool,
        /// Underlying error source
        #[source]
        source: reqwest::Error,
        /// Human-readable error message
        message: String,
    },

    /// Fatal error that occurred during network actor operations.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

/// Errors that can occur during filesystem operations.
#[derive(Debug, Error, Diagnostic)]
#[error("Filesystem operation error")]
pub enum FsError {
    /// File or directory operation failed.
    #[error("IO operation failed: {operation}")]
    #[diagnostic(
        code(fs::operation_failed),
        help("The file system operation failed. Check file permissions and disk space.")
    )]
    OperationFailed {
        /// Path to the file or directory involved (if applicable)
        path: Option<String>,
        /// Description of the operation
        operation: String,
        /// Whether this error is retryable
        retryable: bool,
        /// Underlying error source
        #[source]
        source: std::io::Error,
    },

    /// Fatal error that occurred during filesystem actor operations.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

/// Errors that can occur during logging operations.
#[derive(Debug, Error, Diagnostic)]
#[error("Logging operation error")]
pub enum LogError {
    /// Log file operation failed.
    #[error("Log file operation failed: {operation}")]
    #[diagnostic(
        code(log::file_operation_failed),
        help("Failed to write to log file. Check file permissions and disk space.")
    )]
    FileOperationFailed {
        /// Path to the log file
        path: String,
        /// Description of the operation
        operation: String,
        /// Underlying error source
        #[source]
        source: std::io::Error,
    },

    /// Logger has been flushed and cannot be used.
    #[error("Logger has been flushed and cannot be used")]
    #[diagnostic(
        code(log::already_flushed),
        help("The logger has already been flushed and is no longer usable.")
    )]
    AlreadyFlushed,

    /// Fatal error that occurred during logging actor operations.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

/// Errors that can occur during shell command execution.
#[derive(Debug, Error, Diagnostic)]
#[error("Shell command execution error")]
pub enum ShellError {
    /// Command execution failed.
    #[error("Command execution failed: {message}")]
    #[diagnostic(
        code(shell::execution_failed),
        help("The shell command failed to execute. Check the command and try again.")
    )]
    ExecutionFailed {
        /// Command that was executed
        command: String,
        /// Arguments passed to the command
        args: Vec<String>,
        /// Human-readable error message
        message: String,
        /// Exit code (if available)
        exit_code: Option<i32>,
    },

    /// Command output encoding error.
    #[error("Failed to decode command output as UTF-8")]
    #[diagnostic(
        code(shell::encoding_failed),
        help("The command output could not be decoded as UTF-8.")
    )]
    EncodingFailed {
        /// Command that was executed
        command: String,
    },

    /// Fatal error that occurred during shell actor operations.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

/// Errors that can occur during terminal operations.
#[derive(Debug, Error, Diagnostic)]
#[error("Terminal operation error")]
pub enum TerminalError {
    /// Terminal initialization failed.
    #[error("Terminal initialization failed: {message}")]
    #[diagnostic(
        code(terminal::init_failed),
        help("Failed to initialize the terminal. Check terminal settings.")
    )]
    InitFailed {
        /// Human-readable error message
        message: String,
    },

    /// Terminal operation failed.
    #[error("Terminal operation failed: {operation}")]
    #[diagnostic(
        code(terminal::operation_failed),
        help("The terminal operation failed. Check terminal state.")
    )]
    OperationFailed {
        /// Description of the operation
        operation: String,
        /// Underlying error source
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Fatal error that occurred during terminal actor operations.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

/// Errors that can occur during rendering operations.
#[derive(Debug, Error, Diagnostic)]
#[error("Rendering operation error")]
pub enum RenderError {
    /// Patch rendering failed.
    #[error("Patch rendering failed: {message}")]
    #[diagnostic(
        code(render::rendering_failed),
        help("Failed to render the patch. Check the renderer configuration.")
    )]
    RenderingFailed {
        /// Human-readable error message
        message: String,
        /// Underlying error source (if available)
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Fatal error that occurred during render actor operations.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

/// Errors that can occur during configuration operations.
#[derive(Debug, Error, Diagnostic)]
#[error("Configuration operation error")]
pub enum ConfigError {
    /// Configuration file parsing failed.
    #[error("Configuration file parsing failed: {message}")]
    #[diagnostic(
        code(config::parse_failed),
        help("The TOML configuration file is invalid. Check the syntax and try again.")
    )]
    ParseFailed {
        /// Configuration key being accessed (if applicable)
        key: Option<String>,
        /// Underlying TOML error
        #[source]
        source: toml::de::Error,
        /// Human-readable error message
        message: String,
    },

    /// Configuration value not found or invalid.
    #[error("Configuration error: {message}")]
    #[diagnostic(
        code(config::invalid_value),
        help("The configuration value is invalid. Check the configuration file or environment variables.")
    )]
    InvalidValue {
        /// Configuration key being accessed (if applicable)
        key: Option<String>,
        /// Human-readable error message
        message: String,
    },

    /// Configuration file operation failed.
    #[error("Configuration file operation failed: {operation}")]
    #[diagnostic(
        code(config::file_operation_failed),
        help("Failed to read or write the configuration file. Check file permissions.")
    )]
    FileOperationFailed {
        /// Path to the configuration file
        path: String,
        /// Description of the operation
        operation: String,
        /// Underlying error source
        #[source]
        source: std::io::Error,
    },

    /// Fatal error that occurred during configuration actor operations.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

/// Errors that can occur during Lore API operations.
#[derive(Debug, Error, Diagnostic)]
#[error("Lore API operation error")]
pub enum LoreApiError {
    /// API request failed.
    #[error("API request failed: {message}")]
    #[diagnostic(
        code(lore::request_failed),
        help("The Lore API request failed. Check your network connection and try again.")
    )]
    RequestFailed {
        /// Endpoint that was requested
        endpoint: String,
        /// Human-readable error message
        message: String,
        /// Whether this error is retryable
        retryable: bool,
    },

    /// Response parsing failed.
    #[error("Response parsing failed: {details}")]
    #[diagnostic(
        code(lore::parse_failed),
        help("Failed to parse the API response. The response format may have changed.")
    )]
    ParseFailed {
        /// Format being parsed (e.g., "HTML", "XML", "JSON")
        format: String,
        /// Description of the operation
        operation: String,
        /// Human-readable error details
        details: String,
    },

    /// Fatal error that occurred during Lore API actor operations.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

/// Errors that can occur during cache operations.
#[derive(Debug, Error, Diagnostic)]
#[error("Cache operation error")]
pub enum CacheError {
    /// Cache file operation failed.
    #[error("Cache file operation failed: {operation}")]
    #[diagnostic(
        code(cache::file_operation_failed),
        help("Failed to read or write the cache file. Check file permissions and disk space.")
    )]
    FileOperationFailed {
        /// Path to the cache file
        path: String,
        /// Description of the operation
        operation: String,
        /// Underlying error source
        #[source]
        source: std::io::Error,
    },

    /// Cache serialization/deserialization failed.
    #[error("Cache serialization failed: {message}")]
    #[diagnostic(
        code(cache::serialization_failed),
        help("Failed to serialize or deserialize cache data. The cache file may be corrupted.")
    )]
    SerializationFailed {
        /// Human-readable error message
        message: String,
        /// Underlying error source (if available)
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Fatal error that occurred during cache actor operations.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

/// Errors that can occur during environment variable operations.
#[derive(Debug, Error, Diagnostic)]
#[error("Environment variable operation error")]
pub enum EnvError {
    /// Environment variable not found.
    #[error("Environment variable not found: {name}")]
    #[diagnostic(
        code(env::not_found),
        help("The requested environment variable is not set.")
    )]
    NotFound {
        /// Name of the environment variable
        name: String,
    },

    /// Fatal error that occurred during environment actor operations.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

/// Errors that can occur during UI operations.
#[derive(Debug, Error, Diagnostic)]
#[error("UI operation error")]
pub enum UiError {
    /// UI operation failed.
    #[error("UI operation failed: {operation}")]
    #[diagnostic(
        code(ui::operation_failed),
        help("The UI operation failed. Check the UI state and try again.")
    )]
    OperationFailed {
        /// Description of the operation
        operation: String,
        /// Human-readable error message
        message: String,
    },

    /// Fatal error that occurred during UI actor operations.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

/// Errors that can occur during application operations.
#[derive(Debug, Error, Diagnostic)]
#[error("Application operation error")]
pub enum AppOperationError {
    /// Application operation failed.
    #[error("Application operation failed: {message}")]
    #[diagnostic(
        code(app::operation_failed),
        help("The application operation failed. Check the error details and try again.")
    )]
    OperationFailed {
        /// Human-readable error message
        message: String,
    },

    /// Fatal error that occurred during application actor operations.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),
}

// ============================================================================
// Top-Level App Error
// ============================================================================

/// Top-level application error type.
///
/// This error type composes all actor-specific errors, allowing callers to
/// match on specific error types for precise handling while maintaining
/// unified error propagation.
#[derive(Debug, Error, Diagnostic)]
#[error("Application error")]
pub enum AppError {
    /// Fatal error that requires system-level changes.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fatal(#[from] FatalActorError),

    /// Network operation error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Net(#[from] NetError),

    /// Filesystem operation error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Fs(#[from] FsError),

    /// Logging operation error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Log(#[from] LogError),

    /// Shell command execution error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Shell(#[from] ShellError),

    /// Terminal operation error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Terminal(#[from] TerminalError),

    /// Rendering operation error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Render(#[from] RenderError),

    /// Configuration operation error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Config(#[from] ConfigError),

    /// Lore API operation error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    LoreApi(#[from] LoreApiError),

    /// Cache operation error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Cache(#[from] CacheError),

    /// Environment variable operation error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Env(#[from] EnvError),

    /// UI operation error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    Ui(#[from] UiError),

    /// Application operation error.
    #[error(transparent)]
    #[diagnostic(transparent)]
    App(#[from] AppOperationError),
}

impl FatalActorError {
    /// Returns a human-readable suggestion for handling this fatal error.
    pub fn suggested_action(&self) -> String {
        match self {
            Self::ActorDied { actor_name } => {
                format!("Restart the {} actor or the application", actor_name)
            }
            Self::ActorSendFailed { actor_name, .. } | Self::ActorRecvFailed { actor_name, .. } => {
                format!("Check if the {} actor is still running", actor_name)
            }
        }
    }
}

impl AppError {
    /// Returns `true` if this error is recoverable.
    ///
    /// Fatal errors are never recoverable. All other errors are recoverable.
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Fatal(_) => false,
            Self::Net(NetError::Fatal(_))
            | Self::Fs(FsError::Fatal(_))
            | Self::Log(LogError::Fatal(_))
            | Self::Shell(ShellError::Fatal(_))
            | Self::Terminal(TerminalError::Fatal(_))
            | Self::Render(RenderError::Fatal(_))
            | Self::Config(ConfigError::Fatal(_))
            | Self::LoreApi(LoreApiError::Fatal(_))
            | Self::Cache(CacheError::Fatal(_))
            | Self::Env(EnvError::Fatal(_))
            | Self::Ui(UiError::Fatal(_))
            | Self::App(AppOperationError::Fatal(_)) => false,
            _ => true,
        }
    }

    /// Returns `true` if this error is unrecoverable.
    ///
    /// Only fatal actor errors are unrecoverable.
    pub fn is_unrecoverable(&self) -> bool {
        !self.is_recoverable()
    }

    /// Returns `true` if this error indicates the operation can be retried.
    ///
    /// This checks if the error is recoverable and specifically indicates
    /// that a retry might succeed.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Fatal(_) => false,
            Self::Net(NetError::RequestFailed { retryable, .. }) => *retryable,
            Self::Fs(FsError::OperationFailed { retryable, .. }) => *retryable,
            _ => false,
        }
    }

    /// Returns a suggested retry delay if this error is retryable.
    ///
    /// Returns `None` if the error is not retryable or if no delay is recommended.
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            Self::Net(NetError::RequestFailed { retryable, .. }) if *retryable => {
                Some(std::time::Duration::from_secs(1))
            }
            Self::Fs(FsError::OperationFailed { retryable, .. }) if *retryable => {
                Some(std::time::Duration::from_millis(100))
            }
            _ => None,
        }
    }

    /// Returns a human-readable suggestion for handling this error.
    pub fn suggested_action(&self) -> String {
        match self {
            // Direct fatal errors
            Self::Fatal(fatal) => fatal.suggested_action(),
            // Nested fatal errors in actor-specific errors
            Self::Net(NetError::Fatal(fatal)) => fatal.suggested_action(),
            Self::Fs(FsError::Fatal(fatal)) => fatal.suggested_action(),
            Self::Log(LogError::Fatal(fatal)) => fatal.suggested_action(),
            Self::Shell(ShellError::Fatal(fatal)) => fatal.suggested_action(),
            Self::Terminal(TerminalError::Fatal(fatal)) => fatal.suggested_action(),
            Self::Render(RenderError::Fatal(fatal)) => fatal.suggested_action(),
            Self::Config(ConfigError::Fatal(fatal)) => fatal.suggested_action(),
            Self::LoreApi(LoreApiError::Fatal(fatal)) => fatal.suggested_action(),
            Self::Cache(CacheError::Fatal(fatal)) => fatal.suggested_action(),
            Self::Env(EnvError::Fatal(fatal)) => fatal.suggested_action(),
            Self::Ui(UiError::Fatal(fatal)) => fatal.suggested_action(),
            Self::App(AppOperationError::Fatal(fatal)) => fatal.suggested_action(),
            // Domain-specific errors
            Self::Net(NetError::RequestFailed { retryable, .. }) if *retryable => {
                "Retry the request after a brief delay".to_string()
            }
            Self::Net(NetError::RequestFailed { .. }) => {
                "Check the request parameters and network connectivity".to_string()
            }
            Self::Fs(FsError::OperationFailed { retryable, .. }) if *retryable => {
                "Retry the operation after a brief delay".to_string()
            }
            Self::Fs(FsError::OperationFailed { .. }) => {
                "Check file permissions and disk space".to_string()
            }
            Self::Config(ConfigError::ParseFailed { .. }) => {
                "Check the TOML syntax in the configuration file".to_string()
            }
            Self::Config(ConfigError::InvalidValue { .. }) => {
                "Check and fix the configuration file".to_string()
            }
            Self::Config(ConfigError::FileOperationFailed { .. }) => {
                "Check file permissions for the configuration file".to_string()
            }
            Self::LoreApi(LoreApiError::ParseFailed { .. }) => {
                "Check the data format and try again".to_string()
            }
            Self::LoreApi(LoreApiError::RequestFailed { retryable, .. }) if *retryable => {
                "Retry the API request after a brief delay".to_string()
            }
            Self::LoreApi(LoreApiError::RequestFailed { .. }) => {
                "Check your network connection and API endpoint".to_string()
            }
            Self::Cache(CacheError::SerializationFailed { .. }) => {
                "The cache file may be corrupted. Consider clearing the cache".to_string()
            }
            Self::Cache(CacheError::FileOperationFailed { .. }) => {
                "Check file permissions and disk space for the cache directory".to_string()
            }
            Self::Env(EnvError::NotFound { name }) => {
                format!("Set the {} environment variable", name)
            }
            Self::Log(LogError::FileOperationFailed { .. }) => {
                "Check file permissions and disk space for the log directory".to_string()
            }
            Self::Log(LogError::AlreadyFlushed) => {
                "The logger has been flushed and cannot be used anymore".to_string()
            }
            Self::Shell(ShellError::ExecutionFailed { .. }) => {
                "Check the command and its arguments".to_string()
            }
            Self::Shell(ShellError::EncodingFailed { .. }) => {
                "The command output contains invalid UTF-8. Check the command output".to_string()
            }
            Self::Terminal(TerminalError::InitFailed { .. }) => {
                "Check terminal settings and try again".to_string()
            }
            Self::Terminal(TerminalError::OperationFailed { .. }) => {
                "Check terminal state and try again".to_string()
            }
            Self::Render(RenderError::RenderingFailed { .. }) => {
                "Check the renderer configuration and patch format".to_string()
            }
            Self::Ui(UiError::OperationFailed { .. }) => {
                "Check the UI state and try again".to_string()
            }
            Self::App(AppOperationError::OperationFailed { .. }) => {
                "Check the error details and try again".to_string()
            }
        }
    }
}

// ============================================================================
// Helper Traits and Functions
// ============================================================================

/// Helper trait to determine if a reqwest error is retryable.
pub trait ReqwestErrorExt {
    /// Returns `true` if this error suggests the request should be retried.
    fn is_retryable(&self) -> bool;

    /// Returns a human-readable error message.
    fn error_message(&self) -> String;
}

impl ReqwestErrorExt for reqwest::Error {
    fn is_retryable(&self) -> bool {
        // Connection errors, timeouts, and 5xx errors are retryable
        if self.is_timeout() || self.is_connect() || self.is_request() {
            return true;
        }

        // Check status code if available
        if let Some(status) = self.status() {
            // 5xx server errors are retryable
            if status.is_server_error() {
                return true;
            }
            // 4xx client errors are not retryable (except 408, 429)
            if status.is_client_error() {
                return status.as_u16() == 408 || status.as_u16() == 429;
            }
        }

        false
    }

    fn error_message(&self) -> String {
        if let Some(status) = self.status() {
            format!("HTTP {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown"))
        } else if self.is_timeout() {
            "Request timed out".to_string()
        } else if self.is_connect() {
            "Connection failed".to_string()
        } else {
            self.to_string()
        }
    }
}

/// Helper trait to determine if an IO error is retryable.
pub trait IoErrorExt {
    /// Returns `true` if this error suggests the operation should be retried.
    fn is_retryable(&self) -> bool;
}

impl IoErrorExt for std::io::Error {
    fn is_retryable(&self) -> bool {
        use std::io::ErrorKind;
        matches!(
            self.kind(),
            ErrorKind::TimedOut
                | ErrorKind::WouldBlock
                | ErrorKind::Interrupted
                | ErrorKind::ResourceBusy
                | ErrorKind::UnexpectedEof
        )
    }
}

/// Helper function to create a NetError from a reqwest error.
pub fn network_error(
    url: impl Into<String>,
    method: impl Into<String>,
    source: reqwest::Error,
) -> NetError {
    let url_str = url.into();
    let method_str = method.into();
    let retryable = source.is_retryable();
    let message = source.error_message();
    NetError::RequestFailed {
        url: url_str,
        method: method_str,
        retryable,
        source,
        message,
    }
}

/// Helper function to create a FsError from an std::io::Error.
pub fn fs_error(
    path: Option<impl Into<String>>,
    operation: impl Into<String>,
    source: std::io::Error,
) -> FsError {
    FsError::OperationFailed {
        path: path.map(|p| p.into()),
        operation: operation.into(),
        retryable: source.is_retryable(),
        source,
    }
}
