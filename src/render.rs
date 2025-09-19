mod core;
mod mock;
mod message;
#[cfg(test)]
mod tests;

use anyhow::Context;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

use crate::ArcStr;

/// The render actor that provides a thread-safe interface for rendering patch content.
///
/// This enum represents either a real render actor or a mock implementation
/// for testing purposes. It provides a unified interface for rendering patch content
/// using external programs like `bat` or `delta`.
///
/// # Examples
/// ```ignore
/// let render = Render::spawn(shell, config).await?;
/// let rendered = render.render_patch(content, Renderer::Bat).await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender or mock reference.
#[derive(Debug, Clone)]
pub enum Render {
    /// A real render actor that uses external programs
    Actual(Sender<message::Message>),
    /// A mock implementation for testing that returns predefined content
    Mock(mock::Mock),
}

/// Re-export the renderer type from config for convenience
pub use crate::app::config::Renderer;

impl Render {
    /// Creates a new render instance and spawns its actor.
    ///
    /// # Arguments
    /// * `shell` - The shell actor for executing external programs
    /// * `config` - The configuration actor for renderer settings
    ///
    /// # Returns
    /// A new render instance with a spawned actor.
    pub async fn spawn(
        shell: crate::shell::Shell,
        config: crate::app::config::Config,
    ) -> anyhow::Result<Self> {
        let (render, _) = core::Core::new(shell, config).spawn();
        Ok(render)
    }

    /// Creates a new mock render instance for testing.
    ///
    /// # Arguments
    /// * `content` - Initial response cache mapping patch content to rendered output
    ///
    /// # Returns
    /// A new mock render instance that stores requests in memory.
    pub fn mock(content: HashMap<ArcStr, ArcStr>) -> Self {
        Self::Mock(mock::Mock::new(content))
    }

    /// Renders patch content using the configured renderer.
    ///
    /// # Arguments
    /// * `content` - The raw patch content to render (ArcStr)
    ///
    /// # Returns
    /// The rendered patch content as a string.
    pub async fn render_patch(&self, content: ArcStr) -> anyhow::Result<ArcStr> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(message::Message::Render { tx, content })
                    .await
                    .context("Rendering patch with Render actor")
                    .expect("render actor died");
                rx.await
                    .context("Awaiting response for patch rendering with Render actor")
                    .expect("render actor died")
            }
            Self::Mock(mock) => {
                mock.render_patch(content).await
            }
        }
    }
}
