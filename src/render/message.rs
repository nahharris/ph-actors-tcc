use tokio::sync::oneshot::Sender;

use crate::ArcStr;

/// Messages that can be sent to the render actor.
///
/// This enum defines all the possible messages that can be sent to the render actor,
/// along with their associated response channels.
#[derive(Debug)]
pub enum Message {
    /// Render patch content using the specified renderer
    Render {
        /// Response channel for the rendered content
        tx: Sender<anyhow::Result<ArcStr>>,
        /// The render request containing content and renderer
        content: ArcStr,
    },
}
