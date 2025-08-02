use std::path::Path;

use clap::{Parser, Subcommand};
use ph::api::lore::LoreApi;
use ph::app::config::{Config, PathOpt, USizeOpt};
use ph::env::Env;
use ph::fs::Fs;
use ph::log::Log;
use ph::net::Net;
use ph::utils::install_panic_hook;

use ph::{ArcOsStr, ArcPath, ArcStr};

#[derive(Parser)]
#[command(name = "patch-hub")]
#[command(about = "A CLI tool for interacting with the Lore Kernel Archive")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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

    // Initialize actors
    let env = Env::spawn();
    let fs = Fs::spawn();

    let config_path = env.env(ArcOsStr::from("HOME")).await?;
    let config_path = Path::new(&config_path)
        .join(".config")
        .join("patch-hub")
        .join("config.toml");
    let config_path = ArcPath::from(&config_path);

    let config = Config::spawn(env.clone(), fs.clone(), config_path);
    let res = config.load().await;

    if res.is_err() {
        config.save().await?;
    }

    let log = Log::spawn(
        fs.clone(),
        config.log_level().await,
        config.usize(USizeOpt::MaxAge).await,
        config.path(PathOpt::LogDir).await,
    )
    .await?;

    let net = Net::spawn(config, log.clone()).await;
    let lore = LoreApi::spawn(net);

    log.info("Starting patch-hub CLI");

    match cli.command {
        Commands::Lists { page, count } => {
            handle_lists_command(&lore, page, count).await?;
        }
        Commands::Feed { list, page, count } => {
            handle_feed_command(&lore, list, page, count).await?;
        }
        Commands::Patch {
            list,
            message_id,
            html,
        } => {
            handle_patch_command(&lore, list, message_id, html).await?;
        }
    }

    Ok(())
}

/// Handle the lists command to display available mailing lists
async fn handle_lists_command(lore: &LoreApi, page: usize, count: usize) -> anyhow::Result<()> {
    println!("Fetching mailing lists (page {}, count {})...", page, count);

    let lists_page = lore.get_available_lists_page(page).await?;

    match lists_page {
        Some(page_data) => {
            let start_index = page * count;
            let end_index = (page * count + count).min(page_data.items.len());
            let items_to_show = &page_data.items[start_index..end_index];

            println!(
                "Mailing Lists (Page {}, showing items {} to {} of {}):",
                page,
                start_index + 1,
                end_index,
                page_data.items.len()
            );
            println!("Total items: {:?}", page_data.total_items);
            println!("Next page: {:?}", page_data.next_page_index);
            println!();

            for (i, list) in items_to_show.iter().enumerate() {
                let global_index = start_index + i + 1;
                println!("{}. {} - {}", global_index, list.name, list.description);
                println!(
                    "   Last update: {}",
                    list.last_update.format("%Y-%m-%d %H:%M:%S UTC")
                );
                println!();
            }
        }
        None => {
            println!("No mailing lists found for page {}", page);
        }
    }

    Ok(())
}

/// Handle the feed command to display patch feed for a mailing list
async fn handle_feed_command(
    lore: &LoreApi,
    list: String,
    page: usize,
    count: usize,
) -> anyhow::Result<()> {
    println!(
        "Fetching patch feed for '{}' (page {}, count {})...",
        list, page, count
    );

    let list_name = ArcStr::from(list.clone());
    let patch_feed = lore.get_patch_feed_page(list_name.clone(), page).await?;

    match patch_feed {
        Some(feed) => {
            let start_index = page * count;
            let end_index = (page * count + count).min(feed.items.len());
            let items_to_show = &feed.items[start_index..end_index];

            println!(
                "Patch Feed for '{}' (Page {}, showing items {} to {} of {}):",
                list,
                page,
                start_index + 1,
                end_index,
                feed.items.len()
            );
            println!("Total items: {:?}", feed.total_items);
            println!("Next page: {:?}", feed.next_page_index);
            println!();

            for (i, patch) in items_to_show.iter().enumerate() {
                let global_index = start_index + i + 1;
                println!("{}. {}", global_index, patch.title);
                println!("   Author: {} <{}>", patch.author, patch.email);
                println!(
                    "   Date: {}",
                    patch.last_update.format("%Y-%m-%d %H:%M:%S UTC")
                );
                println!("   Message ID: {}", patch.message_id);
                println!("   Link: {}", patch.link);
                println!();
            }
        }
        None => {
            println!("No patch feed found for '{}' on page {}", list, page);
        }
    }

    Ok(())
}

/// Handle the patch command to display patch content
async fn handle_patch_command(
    lore: &LoreApi,
    list: String,
    message_id: String,
    html: bool,
) -> anyhow::Result<()> {
    println!(
        "Fetching patch content for '{}' with message ID '{}'...",
        list, message_id
    );

    let list_name = ArcStr::from(list);
    let msg_id = ArcStr::from(message_id);

    let content = if html {
        lore.get_patch_html(list_name, msg_id).await?
    } else {
        lore.get_raw_patch(list_name, msg_id).await?
    };

    println!("Patch content:");
    println!("{}", "=".repeat(80));
    println!("{}", content);
    println!("{}", "=".repeat(80));

    Ok(())
}
