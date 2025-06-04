use std::{env::VarError, ffi::OsString};

use tokio::sync::oneshot;

use crate::{ArcOsStr, ArcStr};

/// Messages that can be sent to an [`Env`] actor.
///
/// This enum defines the different types of operations that can be performed
/// on environment variables through the actor system.
#[derive(Debug)]
pub enum Message {
    /// Sets an environment variable to a specified value
    SetEnv {
        /// The environment variable name
        key: ArcOsStr,
        /// The value to set
        value: OsString,
    },
    /// Removes an environment variable
    UnsetEnv {
        /// The environment variable name to remove
        key: ArcOsStr,
    },
    /// Retrieves the value of an environment variable
    GetEnv {
        /// Channel to send the result back to the caller
        tx: oneshot::Sender<Result<ArcStr, VarError>>,
        /// The environment variable name to retrieve
        key: ArcOsStr,
    },
}
