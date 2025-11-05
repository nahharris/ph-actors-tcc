use tokio::sync::mpsc;

use crate::error::AppError;

use crate::ArcStr;
#[cfg(not(test))]
use crate::{
    api::lore::LoreApi,
    app::{
        cache::{feed::FeedCache, mailing_list::MailingListCache, patch::PatchCache},
        config::Config,
        ui::Ui,
    },
    env::Env,
    fs::Fs,
    log::Log,
    render::Render,
    shell::Shell,
    terminal::Terminal,
};
#[cfg(test)]
use crate::{
    api::lore::mock::MockLoreApi as LoreApi,
    app::{
        cache::{
            feed::mock::MockFeedCache as FeedCache,
            mailing_list::mock::MockMailingListCache as MailingListCache,
            patch::mock::MockPatchCache as PatchCache,
        },
        config::mock::MockConfig as Config,
        ui::mock::MockUi as Ui,
    },
    env::mock::MockEnv as Env,
    fs::mock::MockFs as Fs,
    log::mock::MockLog as Log,
    render::mock::MockRender as Render,
    shell::mock::MockShell as Shell,
    terminal::mock::MockTerminal as Terminal,
};
use crate::{app::ui::NavigationAction, terminal::UiEvent};

use super::data::AppState;
use super::message::Message;

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
    /// Lore API actor
    lore: LoreApi,
    /// Shell actor
    shell: Shell,
    /// Render actor
    render: Render,
    /// Mailing list cache actor
    mailing_list_cache: MailingListCache,
    /// Feed cache actor
    feed_cache: FeedCache,
    /// Patch cache actor
    patch_cache: PatchCache,
    /// Terminal actor
    terminal: Terminal,
    /// Terminal task handle for monitoring completion
    terminal_handle: tokio::task::JoinHandle<()>,
    /// UI actor
    ui: Ui,
}

impl Core {
    /// Build a new App actor core with full initialization
    pub fn new(
        log: Log,
        fs: Fs,
        env: Env,
        config: Config,
        lore: LoreApi,
        shell: Shell,
        render: Render,
        mailing_list_cache: MailingListCache,
        feed_cache: FeedCache,
        patch_cache: PatchCache,
        terminal: Terminal,
        terminal_handle: tokio::task::JoinHandle<()>,
        ui: Ui,
    ) -> Self {
        Self {
            state: AppState {
                initialized: true,
                current_command: None,
            },
            env,
            fs,
            config,
            log,
            lore,
            shell,
            render,
            mailing_list_cache,
            feed_cache,
            patch_cache,
            terminal,
            terminal_handle,
            ui,
        }
    }

    /// Initialize the App actor message receiver
    ///
    /// This method processes messages from the receiver in a loop using tokio::select!
    /// to handle both incoming messages and UI events from the terminal.
    pub async fn init(self, mut rx: mpsc::Receiver<Message>) {
        let Self {
            terminal,
            terminal_handle,
            ui,
            log,
            mailing_list_cache,
            feed_cache,
            ..
        } = self;
        let terminal_for_events = terminal.clone();

        // Pin the handle so we can await it in the select! loop
        tokio::pin!(terminal_handle);

        // Start with lists view
        let _ = ui.show_lists(0).await;

        loop {
            tokio::select! {
                Some(msg) = rx.recv() => {
                    use Message::*;
                    match msg {
                        Shutdown { tx } => {
                            let result = Self::handle_shutdown_static(
                                &log,
                                &mailing_list_cache,
                                &feed_cache,
                            ).await;
                            let _ = tx.send(result);
                            break;
                        }
                    }
                }
                event = terminal_for_events.get_ui_event() => {
                    match event {
                        Ok(Some(event)) => {
                            Self::handle_key_event_static(&ui, event).await;
                        }
                        Ok(None) => {
                            // No event, continue polling
                        }
                        Err(e) => {
                            log.error(SCOPE, format!("Error getting UI event: {}", e));
                        }
                    }
                }
                result = terminal_handle.as_mut() => {
                    // Terminal exited, check for panics
                    if let Err(e) = result {
                        log.error(SCOPE, format!("Terminal task panicked: {:?}", e));
                    }
                    // Shutdown the application
                    let _ = Self::handle_shutdown_static(
                        &log,
                        &mailing_list_cache,
                        &feed_cache,
                    ).await;
                    break;
                }
            }
        }
    }

    /// Handle key events from the terminal
    async fn handle_key_event_static(ui: &Ui, event: UiEvent) {
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
    async fn handle_shutdown_static(
        log: &Log,
        mailing_list_cache: &MailingListCache,
        feed_cache: &FeedCache,
    ) -> Result<(), AppError> {
        log.info(SCOPE, "Shutting down application".to_string());

        // Persist cache data before exiting
        if let Err(e) = mailing_list_cache.persist().await {
            log.warn(
                SCOPE,
                format!("Failed to persist mailing list cache: {}", e),
            );
        }
        if let Err(e) = feed_cache.persist(ArcStr::from("")).await {
            log.warn(
                SCOPE,
                format!("Failed to persist patch metadata cache: {}", e),
            );
        }

        log.info(SCOPE, "Application shutdown complete".to_string());
        Ok(())
    }
}
