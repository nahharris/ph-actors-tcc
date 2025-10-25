mod core;
mod message;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

use anyhow::Context;
use tokio::sync::mpsc::{self, Sender};

use crate::ArcStr;

// Actors it depends on
#[cfg(not(test))]
use crate::{app::config::Config, shell::Shell};
#[cfg(test)]
use crate::{app::config::mock::MockConfig as Config, shell::mock::MockShell as Shell};

/// The render actor that provides a thread-safe interface for rendering patch content.
///
/// This struct provides a unified interface for rendering patch content
/// using external programs like `bat` or `delta`.
///
/// # Examples
/// ```ignore
/// let render = Render::spawn(shell, config);
/// let rendered = render.render_patch(content).await?;
/// ```
///
/// # Thread Safety
/// This type is designed to be safely shared between threads. Cloning is cheap as it only
/// copies the channel sender.
#[derive(Debug, Clone)]
pub struct Render {
    tx: Sender<message::Message>,
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
    pub fn spawn(shell: Shell, config: Config) -> Self {
        let (tx, rx) = mpsc::channel(crate::BUFFER_SIZE);
        let core = core::Core::new(shell, config);
        let _ = tokio::spawn(async move {
            core.init(rx).await;
        });
        Self { tx }
    }

    /// Renders patch content using the configured renderer.
    ///
    /// # Arguments
    /// * `content` - The raw patch content to render (ArcStr)
    ///
    /// # Returns
    /// The rendered patch content as a string.
    pub async fn render_patch(&self, content: ArcStr) -> anyhow::Result<ArcStr> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.tx
            .send(message::Message::Render { tx, content })
            .await
            .context("Rendering patch with Render actor")
            .expect("render actor died");
        rx.await
            .context("Awaiting response for patch rendering with Render actor")
            .expect("render actor died")
    }
}
