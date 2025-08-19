use tokio::sync::mpsc::{self, Receiver};
use tokio::task::JoinHandle;

use crate::api::lore::data::LorePatch;
use crate::{ArcSlice, ArcStr};

use super::message::Message;

/// The core implementation of the render actor.
///
/// This struct contains the internal state and logic for the render actor,
/// including the shell actor for executing external programs and the configuration
/// for renderer settings.
pub struct Core {
    /// The shell actor for executing external programs
    shell: crate::shell::Shell,
    /// The configuration actor for renderer settings
    config: crate::app::config::Config,
}

impl Core {
    /// Creates a new render actor core.
    ///
    /// # Arguments
    /// * `shell` - The shell actor for executing external programs
    /// * `config` - The configuration actor for renderer settings
    ///
    /// # Returns
    /// A new render actor core instance.
    pub fn new(shell: crate::shell::Shell, config: crate::app::config::Config) -> Self {
        Self { shell, config }
    }

    /// Spawns the render actor and returns the handle and join handle.
    ///
    /// # Returns
    /// A tuple containing the render actor handle and the join handle for the spawned task.
    pub fn spawn(self) -> (super::Render, JoinHandle<anyhow::Result<()>>) {
        let (tx, rx) = mpsc::channel(32);
        let handle = super::Render::Actual(tx);
        let join_handle = tokio::spawn(self.run(rx));
        (handle, join_handle)
    }

    /// Runs the render actor event loop.
    ///
    /// # Arguments
    /// * `mut rx` - The message receiver
    ///
    /// # Returns
    /// Result indicating success or failure of the actor
    async fn run(self, mut rx: Receiver<Message>) -> anyhow::Result<()> {
        while let Some(message) = rx.recv().await {
            match message {
                Message::Render { tx, patch } => {
                    let result = self.handle_render_request(patch).await;
                    let _ = tx.send(result);
                }
            }
        }
        Ok(())
    }

    /// Handles a render request by executing the appropriate external program.
    ///
    /// # Arguments
    /// * `patch` - The patch to render
    ///
    /// # Returns
    /// The rendered content or an error
    async fn handle_render_request(&self, patch: LorePatch) -> anyhow::Result<ArcStr> {
        // Get the renderer from config
        let renderer = self
            .config
            .renderer(crate::app::config::RendererOpt::PatchRenderer)
            .await;

        if matches!(renderer, crate::app::config::Renderer::None) {
            // No external renderer: return just the diff content
            return Ok(patch.diff);
        }

        // Always use only the diff content for rendering, regardless of renderer
        // This focuses on the most important part of the patch
        let content = format!("{}", patch.diff);

        // Get the program name and default arguments
        let program = ArcStr::from(renderer.program_name());
        let default_args = renderer.default_args();

        // Convert default args to ArcSlice<ArcStr>
        let args: Vec<ArcStr> = default_args.into_iter().map(ArcStr::from).collect();
        let args = ArcSlice::from(args);

        // Execute the renderer program with the content as stdin
        let result = self
            .shell
            .execute(program, args, Some(ArcStr::from(content)))
            .await?;

        if result.is_success() {
            Ok(result.stdout)
        } else {
            Err(anyhow::anyhow!(
                "Renderer '{}' failed with status: {}, stderr: {}",
                renderer.program_name(),
                result.status,
                result.stderr
            ))
        }
    }
}
