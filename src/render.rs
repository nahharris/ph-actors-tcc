mod core;
mod message;
#[cfg(test)]
mod tests;

use anyhow::Context;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;

use crate::ArcStr;
use crate::api::lore::data::LorePatch;

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
    Mock(Arc<Mutex<HashMap<ArcStr, ArcStr>>>),
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
    /// # Returns
    /// A new mock render instance that stores requests in memory.
    pub fn mock(content: HashMap<ArcStr, ArcStr>) -> Self {
        Self::Mock(Arc::new(Mutex::new(content)))
    }

    /// Renders patch content using the configured renderer.
    ///
    /// # Arguments
    /// * `patch` - The patch to render
    ///
    /// # Returns
    /// The rendered patch content as a string.
    pub async fn render_patch(&self, patch: LorePatch) -> anyhow::Result<ArcStr> {
        match self {
            Self::Actual(sender) => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                sender
                    .send(message::Message::Render { tx, patch })
                    .await
                    .context("Rendering patch with Render actor")
                    .expect("render actor died");
                rx.await
                    .context("Awaiting response for patch rendering with Render actor")
                    .expect("render actor died")
            }
            Self::Mock(requests) => {
                let lock = requests.lock().await;
                // Mock now uses diff content as key since render logic only renders diff
                let diff_content = format!("{}", patch.diff);
                lock.get(&ArcStr::from(diff_content))
                    .context("No more mocked responses")
                    .cloned()
            }
        }
    }
}
