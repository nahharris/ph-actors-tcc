use super::data::LogMessage;

/// Messages that can be sent to a [`LogCore`] actor.
///
/// This enum defines the different types of operations that can be performed
/// through the logging actor system.
#[derive(Debug)]
pub enum Message {
    /// Logs a message with the specified level and content
    Log(LogMessage),
    /// Flushes the logger by writing buffered messages to stderr and destroying the instance
    Flush,
    /// Runs the log garbage collector to delete old log files
    CollectGarbage,
}
