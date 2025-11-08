use tokio::sync::mpsc;

use crate::{ArcSlice, ArcStr, error::RenderError};

use super::message::Message;

// Actors it depends on
#[cfg(not(test))]
use crate::{app::config::Config, shell::Shell};
#[cfg(test)]
use crate::{app::config::mock::MockConfig as Config, shell::mock::MockShell as Shell};

/// The core implementation of the render actor.
///
/// This struct contains the internal state and logic for the render actor,
/// including the shell actor for executing external programs and the configuration
/// for renderer settings.
pub struct Core {
    /// The shell actor for executing external programs
    shell: Shell,
    /// The configuration actor for renderer settings
    config: Config,
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
    pub fn new(shell: Shell, config: Config) -> Self {
        Self { shell, config }
    }

    /// Initializes the render actor message receiver.
    ///
    /// This method processes messages from the receiver in a loop, handling each message
    /// using pattern matching.
    ///
    /// # Arguments
    /// * `rx` - A receiver for messages to process
    pub async fn init(self, mut rx: mpsc::Receiver<Message>) {
        while let Some(msg) = rx.recv().await {
            use Message::*;
            match msg {
                Render { tx, content } => {
                    let result = self.handle_render_request(content).await;
                    let _ = tx.send(result);
                }
            }
        }
    }

    /// Handles a render request by executing the appropriate external program.
    ///
    /// # Arguments
    /// * `request` - The render request containing content
    ///
    /// # Returns
    /// The rendered content or an error
    async fn handle_render_request(&self, content: ArcStr) -> Result<ArcStr, RenderError> {
        // Get the renderer from config
        let renderer = self
            .config
            .renderer(crate::app::config::RendererOpt::PatchRenderer)
            .await
            .map_err(|e| match e {
                crate::error::ConfigError::Fatal(fatal) => RenderError::Fatal(fatal),
                crate::error::ConfigError::FileOperationFailed { .. }
                | crate::error::ConfigError::ParseFailed { .. }
                | crate::error::ConfigError::InvalidValue { .. } => RenderError::RenderingFailed {
                    message: format!("Failed to get renderer configuration: {}", e),
                    source: Some(Box::new(e)),
                },
            })?;

        if matches!(renderer, crate::app::config::Renderer::None) {
            // No external renderer: return raw content
            return Ok(content);
        }

        // Get the program name and default arguments
        let program = ArcStr::from(renderer.program_name());
        let default_args = renderer.default_args();

        // Convert default args to ArcSlice<ArcStr>
        let args: Vec<ArcStr> = default_args.into_iter().map(ArcStr::from).collect();
        let args = ArcSlice::from(args);

        // Execute the renderer program with the content as stdin
        let result = self
            .shell
            .execute(program, args, Some(content))
            .await
            .map_err(|e| match e {
                crate::error::ShellError::Fatal(fatal) => RenderError::Fatal(fatal),
                crate::error::ShellError::ExecutionFailed { ref message, .. } => {
                    RenderError::RenderingFailed {
                        message: format!("Renderer execution failed: {}", message),
                        source: Some(Box::new(e)),
                    }
                }
                crate::error::ShellError::EncodingFailed { ref command, .. } => {
                    RenderError::RenderingFailed {
                        message: format!("Renderer encoding failed for command: {}", command),
                        source: Some(Box::new(e)),
                    }
                }
            })?;

        if result.is_success() {
            Ok(result.stdout)
        } else {
            Err(RenderError::RenderingFailed {
                message: format!(
                    "Renderer '{}' failed with status: {}, stderr: {}",
                    renderer.program_name(),
                    result.status,
                    result.stderr
                ),
                source: None,
            })
        }
    }
}
