use tokio::sync::oneshot::Sender;

use crate::ArcStr;


/// Messages that can be sent to a [`NetCore`] actor.
///
/// This enum defines the different types of network operations that can be performed
/// through the networking actor system.
#[derive(Debug)]
pub enum Message {
    /// Performs an HTTP GET request to the specified URL
    Get{
        url: ArcStr,
        tx: Sender<anyhow::Result<ArcStr>>,
    },
}