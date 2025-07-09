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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::data::{LogLevel, LogMessage};

    #[test]
    fn test_message_log_variant() {
        let msg = LogMessage {
            level: LogLevel::Info,
            message: "test".to_string(),
        };
        let m = Message::Log(msg.clone());
        match m {
            Message::Log(inner) => assert_eq!(inner, msg),
            _ => panic!("Expected Log variant"),
        }
    }

    #[test]
    fn test_message_flush_variant() {
        let m = Message::Flush;
        match m {
            Message::Flush => (),
            _ => panic!("Expected Flush variant"),
        }
    }

    #[test]
    fn test_message_collect_garbage_variant() {
        let m = Message::CollectGarbage;
        match m {
            Message::CollectGarbage => (),
            _ => panic!("Expected CollectGarbage variant"),
        }
    }
}
