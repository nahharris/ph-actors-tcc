use anyhow::Result;
use std::path::Path;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::api::lore::LoreApi;
use crate::app::cache::{mailing_list::MailingListCache, patch_meta::PatchMetaCache};
use crate::app::config::{Config, PathOpt, USizeOpt};
use crate::app::ui::{NavigationAction, Ui};
use crate::env::Env;
use crate::fs::Fs;
use crate::log::Log;
use crate::net::Net;
use crate::render::Render;
use crate::shell::Shell;
use crate::terminal::{Terminal, UiEvent};
use crate::{ArcOsStr, ArcPath, ArcStr};

use super::data::{AppState, Command};
use super::message::Message;

const BUFFER_SIZE: usize = 64;
const SCOPE: &str = "app";

/// Core implementation of the App actor
#[derive(Debug)]
pub struct Core {
    /// Application state
    state: AppState,
    /// Environment actor
    env: Env,
    /// Filesystem actor
    fs: Fs,
    /// Configuration actor
    config: Config,
    /// Logging actor
    log: Log,
    /// Network actor
    net: Net,
    /// Lore API actor
    lore: LoreApi,
    /// Shell actor
    shell: Shell,
    /// Render actor
    render: Render,
    /// Mailing list cache actor
    mailing_list_cache: MailingListCache,
    /// Patch metadata cache actor
    patch_meta_cache: PatchMetaCache,
}

impl Core {
    /// Build a new App actor core with full initialization
    pub async fn build() -> Result<Self> {
        // Initialize basic actors
        let env = Env::spawn();
        let fs = Fs::spawn();

        // Set up configuration
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

        // Initialize logging actor
        let log = Log::spawn(
            fs.clone(),
            config.log_level().await,
            config.usize(USizeOpt::MaxAge).await,
            config.path(PathOpt::LogDir).await,
        )
        .await?;

        // Initialize network and API actors
        let net = Net::spawn(config.clone(), log.clone()).await;
        let lore = LoreApi::spawn(net.clone());

        // Initialize shell and render actors
        let shell = Shell::spawn(log.clone()).await?;
        let render = Render::spawn(shell.clone(), config.clone()).await?;

        // Initialize cache actors
        let mailing_list_cache =
            MailingListCache::spawn(lore.clone(), fs.clone(), config.clone(), log.clone());
        let patch_meta_cache =
            PatchMetaCache::spawn(lore.clone(), fs.clone(), config.clone(), log.clone());

        // Load existing cache data
        if let Err(e) = mailing_list_cache.load_cache().await {
            log.warn(SCOPE, &format!("Failed to load mailing list cache: {}", e));
        } else {
            // Validate mailing list cache after loading
            match mailing_list_cache.is_cache_valid().await {
                Ok(true) => {
                    log.info(SCOPE, "Mailing list cache is valid");
                }
                Ok(false) => {
                    log.warn(SCOPE, "Mailing list cache is outdated, invalidating");
                    mailing_list_cache.invalidate_cache().await;
                }
                Err(e) => {
                    log.warn(SCOPE, &format!("Failed to validate mailing list cache: {}, invalidating", e));
                    mailing_list_cache.invalidate_cache().await;
                }
            }
        }

        if let Err(e) = patch_meta_cache.load_cache().await {
            log.warn(
                SCOPE,
                &format!("Failed to load patch metadata cache: {}", e),
            );
        } else {
            log.info(SCOPE, "Patch metadata cache loaded successfully, validation will be done per mailing list on demand");
        }

        log.info(SCOPE, "App actor initialized successfully");

        Ok(Self {
            state: AppState {
                initialized: true,
                current_command: None,
            },
            env,
            fs,
            config,
            log,
            net,
            lore,
            shell,
            render,
            mailing_list_cache,
            patch_meta_cache,
        })
    }

    /// Spawn the App actor for interactive mode
    pub fn spawn_interactive(self) -> Result<(super::App, JoinHandle<()>)> {
        // Create Terminal and UI actors for interactive mode
        let (ui_tx, ui_rx) = tokio::sync::mpsc::channel(64);
        let (terminal, ui_exit) = Terminal::spawn(self.log.clone(), ui_tx.clone());
        let (ui, _ui_handle) = Ui::spawn(
            self.log.clone(),
            terminal,
            self.mailing_list_cache.clone(),
            self.patch_meta_cache.clone(),
            self.lore.clone(),
            self.render.clone(),
        );

        let (tx, mut rx) = mpsc::channel(BUFFER_SIZE);
        let handle = tokio::spawn(async move {
            let mut core = self;
            let mut ui_event_rx = ui_rx;
            let mut ui_exit = ui_exit;

            // Start with lists view
            let _ = ui.show_lists(0).await;

            loop {
                tokio::select! {
                    Some(message) = rx.recv() => {
                        match message {
                            Message::ExecuteCommand { command, tx } => {
                                let result = core.handle_execute_command(command).await;
                                let _ = tx.send(result);
                            }
                            Message::KeyEvent { event } => {
                                core.handle_key_event(&ui, event).await;
                            }
                            Message::Shutdown { tx } => {
                                let result = core.handle_shutdown().await;
                                let _ = tx.send(result);
                                break; // Exit the message loop
                            }
                        }
                    }
                    Some(ui_event) = ui_event_rx.recv() => {
                        // Forward UI events to key event handler
                        core.handle_key_event(&ui, ui_event).await;
                    }
                    _ = &mut ui_exit => {
                        // UI exited, shutdown
                        let _ = core.handle_shutdown().await;
                        break;
                    }
                }
            }
        });
        Ok((super::App::Actual(tx), handle))
    }

    /// Handle command execution
    async fn handle_execute_command(&mut self, command: Command) -> Result<()> {
        self.state.current_command = Some(command.clone());

        match command {
            Command::Lists { page, count } => self.handle_lists_command(page, count).await,
            Command::Feed { list, page, count } => {
                self.handle_feed_command(list, page, count).await
            }
            Command::Patch {
                list,
                message_id,
                html,
            } => self.handle_patch_command(list, message_id, html).await,
        }
    }

    /// Handle key events from the terminal
    async fn handle_key_event(&self, ui: &Ui, event: UiEvent) {
        match event {
            UiEvent::SelectionChange(index) => {
                ui.update_selection(index).await;
            }
            UiEvent::Left => {
                let _ = ui.previous_page().await;
            }
            UiEvent::Right => {
                let _ = ui.next_page().await;
            }
            UiEvent::SelectionSubmit(_) => {
                if let Ok(Some(action)) = ui.submit_selection().await {
                    match action {
                        NavigationAction::OpenFeed { list } => {
                            let _ = ui.show_feed(list, 0).await;
                        }
                        NavigationAction::OpenPatch {
                            list,
                            message_id,
                            title,
                        } => {
                            let _ = ui.show_patch(list, message_id, title).await;
                        }
                        NavigationAction::Quit => {
                            // Terminal will handle quit
                        }
                    }
                }
            }
            UiEvent::Esc => {
                let _ = ui.navigate_back().await;
            }
        }
    }

    /// Handle graceful shutdown
    pub async fn handle_shutdown(&self) -> Result<()> {
        self.log.info(SCOPE, "Shutting down application");

        // Persist cache data before exiting
        if let Err(e) = self.mailing_list_cache.persist_cache().await {
            self.log.warn(
                SCOPE,
                &format!("Failed to persist mailing list cache: {}", e),
            );
        }
        if let Err(e) = self.patch_meta_cache.persist_cache().await {
            self.log.warn(
                SCOPE,
                &format!("Failed to persist patch metadata cache: {}", e),
            );
        }

        self.log.info(SCOPE, "Application shutdown complete");
        Ok(())
    }

    /// Handle the lists command to display available mailing lists using cache
    pub async fn handle_lists_command(&self, page: usize, count: usize) -> Result<()> {
        println!("Fetching mailing lists (page {}, count {})...", page, count);

        let start_index = page * count;
        let end_index = start_index + count;
        let range = start_index..end_index;

        let lists = self.mailing_list_cache.get_slice(range).await?;

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
    pub async fn handle_feed_command(&self, list: ArcStr, page: usize, count: usize) -> Result<()> {
        println!(
            "Fetching patch feed for '{}' (page {}, count {})...",
            list, page, count
        );

        let start_index = page * count;
        let end_index = start_index + count;
        let range = start_index..end_index;

        let patches = self.patch_meta_cache.get_slice(list.clone(), range).await?;

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
    pub async fn handle_patch_command(
        &self,
        list: ArcStr,
        message_id: ArcStr,
        html: bool,
    ) -> Result<()> {
        println!(
            "Fetching patch content for '{}' with message ID '{}'...",
            list, message_id
        );

        let content = if html {
            self.lore.get_patch_html(list, message_id).await?
        } else {
            self.lore.get_raw_patch(list, message_id).await?
        };

        if html {
            println!("Patch content:");
            println!("{}", "=".repeat(80));
            println!("{}", content);
            println!("{}", "=".repeat(80));
        } else {
            // Use the render actor to render the raw patch content
            let rendered_content = self.render.render_patch(content).await?;
            println!("{}", rendered_content);
        }

        Ok(())
    }
}
