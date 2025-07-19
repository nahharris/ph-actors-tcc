use std::ffi::OsString;

use crate::{ArcOsStr, ArcStr};

/// Messages that can be sent to an [`Env`] actor.
///
/// This enum defines the different types of operations that can be performed
/// on environment variables through the actor system.
#[derive(Debug)]
pub enum Message {
    /// Sets an environment variable to a specified value
    Set {
        /// The environment variable name
        key: ArcOsStr,
        /// The value to set
        value: OsString,
    },
    /// Unsets an environment variable
    Unset {
        /// The environment variable name to remove
        key: ArcOsStr,
    },
    /// Gets an environment variable
    Get {
        /// Channel to send the result back to the caller
        tx: tokio::sync::oneshot::Sender<Result<ArcStr, std::env::VarError>>,
        /// The environment variable name to retrieve
        key: ArcOsStr,
    },
}
