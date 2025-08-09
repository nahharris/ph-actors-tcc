use clap::{Parser, Subcommand};
use ph::ArcStr;
use ph::app::{App, Command};
use ph::utils::install_panic_hook;

#[derive(Parser)]
#[command(name = "patch-hub")]
#[command(about = "A CLI tool for interacting with the Lore Kernel Archive")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available mailing lists
    Lists {
        /// Page number (0-based)
        #[arg(short, long, default_value = "0")]
        page: usize,
        /// Number of items per page
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
    /// Get the feed of a given mailing list
    Feed {
        /// The mailing list name (e.g., "amd-gfx", "linux-kernel")
        #[arg(required = true)]
        list: String,
        /// Page number (0-based)
        #[arg(short, long, default_value = "0")]
        page: usize,
        /// Number of items per page
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
    /// Get the content of a patch from the feed
    Patch {
        /// The mailing list name
        #[arg(required = true)]
        list: String,
        /// The message ID of the patch
        #[arg(required = true)]
        message_id: String,
        /// Get HTML patch content instead of raw
        #[arg(long)]
        html: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    install_panic_hook()?;

    let cli = Cli::parse();

    // Initialize the App actor with all dependencies
    let (app, _handle) = App::build().await?;

    // Execute the appropriate command or run TUI
    match cli.command {
        Some(Commands::Lists { page, count }) => {
            let command = Command::Lists { page, count };
            app.execute_command(command).await?;
        }
        Some(Commands::Feed { list, page, count }) => {
            let command = Command::Feed {
                list: ArcStr::from(list),
                page,
                count,
            };
            app.execute_command(command).await?;
        }
        Some(Commands::Patch {
            list,
            message_id,
            html,
        }) => {
            let command = Command::Patch {
                list: ArcStr::from(list),
                message_id: ArcStr::from(message_id),
                html,
            };
            app.execute_command(command).await?;
        }
        None => {
            // TUI mode
            app.run_tui().await?;
        }
    }

    // Graceful shutdown
    app.shutdown().await?;

    Ok(())
}
