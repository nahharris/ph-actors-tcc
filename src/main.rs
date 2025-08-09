use std::path::Path;

use clap::{Parser, Subcommand};
use ph::api::lore::LoreApi;
use ph::app::cache::{mailing_list::MailingListCache, patch_meta::PatchMetaCache};
use ph::app::config::{Config, PathOpt, USizeOpt};
use ph::app::ui::AppUi;
use ph::env::Env;
use ph::fs::Fs;
use ph::log::Log;
use ph::net::Net;
use ph::render::Render;
use ph::shell::Shell;
use ph::terminal::Terminal;
use ph::utils::install_panic_hook;

use ph::{ArcOsStr, ArcPath, ArcStr};

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

    let net = Net::spawn(config.clone(), log.clone()).await;
    let lore = LoreApi::spawn(net);

    // Initialize shell and render actors
    let shell = Shell::spawn(log.clone()).await?;
    let render = Render::spawn(shell, config.clone()).await?;

    // Initialize cache actors
    let mailing_list_cache = MailingListCache::spawn(lore.clone(), fs.clone(), config.clone());
    let patch_meta_cache = PatchMetaCache::spawn(lore.clone(), fs.clone(), config.clone());

    // Load existing cache data
    if let Err(e) = mailing_list_cache.load_cache().await {
        log.warn(&format!("Failed to load mailing list cache: {}", e));
    }
    if let Err(e) = patch_meta_cache.load_cache().await {
        log.warn(&format!("Failed to load patch metadata cache: {}", e));
    }

    log.info("Starting patch-hub CLI");

    match cli.command {
        Some(Commands::Lists { page, count }) => {
            handle_lists_command(&mailing_list_cache, page, count).await?;
        }
        Some(Commands::Feed { list, page, count }) => {
            handle_feed_command(&patch_meta_cache, list, page, count).await?;
        }
        Some(Commands::Patch {
            list,
            message_id,
            html,
        }) => {
            handle_patch_command(&lore, &render, list, message_id, html).await?;
        }
        None => {
            // TUI mode
            let (ui_tx, ui_rx) = tokio::sync::mpsc::channel(64);
            let (terminal, ui_exit) = Terminal::spawn(log.clone(), ui_tx.clone());
            let (app, _handle) = AppUi::spawn(
                log.clone(),
                terminal,
                mailing_list_cache.clone(),
                patch_meta_cache.clone(),
                lore.clone(),
                render.clone(),
                ui_rx,
            );
            app.run().await?;
            // Block main until UI exits (Esc), then continue to persist caches
            let _ = ui_exit.await;
        }
    }

    // Persist cache data before exiting
    if let Err(e) = mailing_list_cache.persist_cache().await {
        log.warn(&format!("Failed to persist mailing list cache: {}", e));
    }
    if let Err(e) = patch_meta_cache.persist_cache().await {
        log.warn(&format!("Failed to persist patch metadata cache: {}", e));
    }

    Ok(())
}

// no-op helper removed

/// Handle the lists command to display available mailing lists using cache
async fn handle_lists_command(
    cache: &MailingListCache,
    page: usize,
    count: usize,
) -> anyhow::Result<()> {
    println!("Fetching mailing lists (page {}, count {})...", page, count);

    let start_index = page * count;
    let end_index = start_index + count;
    let range = start_index..end_index;

    let lists = cache.get_slice(range).await?;

    if lists.is_empty() {
        println!("No mailing lists found for page {}", page);
        return Ok(());
    }

    println!(
        "Mailing Lists (Page {}, showing items {} to {}):",
        page,
        start_index + 1,
        start_index + lists.len()
    );
    println!();

    for (i, list) in lists.iter().enumerate() {
        let global_index = start_index + i + 1;
        println!("{}. {} - {}", global_index, list.name, list.description);
        println!(
            "   Last update: {}",
            list.last_update.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!();
    }

    Ok(())
}

/// Handle the feed command to display patch feed for a mailing list using cache
async fn handle_feed_command(
    cache: &PatchMetaCache,
    list: String,
    page: usize,
    count: usize,
) -> anyhow::Result<()> {
    println!(
        "Fetching patch feed for '{}' (page {}, count {})...",
        list, page, count
    );

    let list_name = ArcStr::from(list.clone());
    let start_index = page * count;
    let end_index = start_index + count;
    let range = start_index..end_index;

    let patches = cache.get_slice(list_name.clone(), range).await?;

    if patches.is_empty() {
        println!("No patch feed found for '{}' on page {}", list, page);
        return Ok(());
    }

    println!(
        "Patch Feed for '{}' (Page {}, showing items {} to {}):",
        list,
        page,
        start_index + 1,
        start_index + patches.len()
    );
    println!();

    for (i, patch) in patches.iter().enumerate() {
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

    Ok(())
}

/// Handle the patch command to display patch content
async fn handle_patch_command(
    lore: &LoreApi,
    render: &Render,
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

    if html {
        println!("Patch content:");
        println!("{}", "=".repeat(80));
        println!("{}", content);
        println!("{}", "=".repeat(80));
    } else {
        // Use the render actor to render the raw patch content
        let rendered_content = render.render_patch(content).await?;
        println!("{}", rendered_content);
    }

    Ok(())
}
