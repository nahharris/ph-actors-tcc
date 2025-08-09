use anyhow::Result;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::ArcStr;
use crate::api::lore::LoreApi;
use crate::app::cache::{mailing_list::MailingListCache, patch_meta::PatchMetaCache};
use crate::log::Log;
use crate::render::Render;
use crate::terminal::{Screen, Terminal};

use super::data::{UiState, ViewKind};
use super::message::{Message, NavigationAction};

const BUFFER_SIZE: usize = 32;
const SCOPE: &str = "app.ui";

/// Core implementation of the UI actor
pub struct Core {
    /// UI state
    state: UiState,
    /// Logging actor
    log: Log,
    /// Terminal actor for rendering
    terminal: Terminal,
    /// Mailing list cache
    mailing_list_cache: MailingListCache,
    /// Patch metadata cache
    patch_meta_cache: PatchMetaCache,
    /// Lore API actor
    lore: LoreApi,
    /// Render actor
    render: Render,
}

impl Core {
    /// Create a new UI actor core
    pub fn new(
        log: Log,
        terminal: Terminal,
        mailing_list_cache: MailingListCache,
        patch_meta_cache: PatchMetaCache,
        lore: LoreApi,
        render: Render,
    ) -> Self {
        Self {
            state: UiState::default(),
            log,
            terminal,
            mailing_list_cache,
            patch_meta_cache,
            lore,
            render,
        }
    }

    /// Spawn the UI actor
    pub fn spawn(self) -> (super::Ui, JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel(BUFFER_SIZE);
        let handle = tokio::spawn(async move {
            let mut core = self;

            // Show initial loading screen and render lists
            let _ = core
                .terminal
                .show(Screen::Loading(ArcStr::from("Loading mailing lists...")))
                .await;
            core.log.info(SCOPE, "Initial render of Lists screen");
            let _ = core.render_lists().await;

            while let Some(message) = rx.recv().await {
                match message {
                    Message::ShowLists { page, tx } => {
                        let result = core.handle_show_lists(page).await;
                        let _ = tx.send(result);
                    }
                    Message::ShowFeed { list, page, tx } => {
                        let result = core.handle_show_feed(list, page).await;
                        let _ = tx.send(result);
                    }
                    Message::ShowPatch {
                        list,
                        message_id,
                        title,
                        tx,
                    } => {
                        let result = core.handle_show_patch(list, message_id, title).await;
                        let _ = tx.send(result);
                    }
                    Message::UpdateSelection { index } => {
                        core.handle_update_selection(index);
                    }
                    Message::PreviousPage { tx } => {
                        let result = core.handle_previous_page().await;
                        let _ = tx.send(result);
                    }
                    Message::NextPage { tx } => {
                        let result = core.handle_next_page().await;
                        let _ = tx.send(result);
                    }
                    Message::NavigateBack { tx } => {
                        let result = core.handle_navigate_back().await;
                        let _ = tx.send(result);
                    }
                    Message::SubmitSelection { tx } => {
                        let result = core.handle_submit_selection().await;
                        let _ = tx.send(result);
                    }
                    Message::GetState { tx } => {
                        let _ = tx.send(core.state.clone());
                    }
                }
            }
        });
        (super::Ui::Actual(tx), handle)
    }

    /// Handle showing lists view
    async fn handle_show_lists(&mut self, page: usize) -> Result<()> {
        self.state.view = ViewKind::Lists;
        self.state.list_page = page;
        self.state.list_selected = 0;
        self.render_lists().await
    }

    /// Handle showing feed view
    async fn handle_show_feed(&mut self, list: ArcStr, page: usize) -> Result<()> {
        self.state.view = ViewKind::Feed;
        self.state.feed_list = Some(list.clone());
        self.state.feed_page = page;
        self.state.feed_selected = 0;

        // Show loading screen before fetching feed data
        self.terminal
            .show(Screen::Loading(ArcStr::from("Loading feed...")))
            .await?;

        self.render_feed(list).await
    }

    /// Handle showing patch view
    async fn handle_show_patch(
        &mut self,
        list: ArcStr,
        message_id: ArcStr,
        title: ArcStr,
    ) -> Result<()> {
        self.state.view = ViewKind::Patch;
        self.render_patch(list, message_id, title).await
    }

    /// Handle selection update
    fn handle_update_selection(&mut self, index: usize) {
        match self.state.view {
            ViewKind::Lists => self.state.list_selected = index,
            ViewKind::Feed => self.state.feed_selected = index,
            ViewKind::Patch => {} // No selection in patch view
        }
    }

    /// Handle previous page navigation
    async fn handle_previous_page(&mut self) -> Result<()> {
        match self.state.view {
            ViewKind::Lists => {
                self.state.list_page = self.state.list_page.saturating_sub(1);
                self.state.list_selected = 0;

                // Check if cache contains the new page range
                let start = self.state.list_page * 20;
                let end = start + 20;
                let range = start..end;

                if !self.mailing_list_cache.contains_range(range.clone()).await {
                    // Show loading screen if data not cached
                    self.terminal
                        .show(Screen::Loading(ArcStr::from("Loading mailing lists...")))
                        .await?;
                }

                self.render_lists().await
            }
            ViewKind::Feed => {
                self.state.feed_page = self.state.feed_page.saturating_sub(1);
                self.state.feed_selected = 0;
                if let Some(list) = self.state.feed_list.clone() {
                    // Check if cache contains the new page range
                    let start = self.state.feed_page * 20;
                    let end = start + 20;
                    let range = start..end;

                    if !self
                        .patch_meta_cache
                        .contains_range(list.clone(), range.clone())
                        .await
                    {
                        // Show loading screen if data not cached
                        self.terminal
                            .show(Screen::Loading(ArcStr::from("Loading feed...")))
                            .await?;
                    }

                    self.render_feed(list).await
                } else {
                    Ok(())
                }
            }
            ViewKind::Patch => Ok(()), // No pagination in patch view
        }
    }

    /// Handle next page navigation
    async fn handle_next_page(&mut self) -> Result<()> {
        match self.state.view {
            ViewKind::Lists => {
                self.state.list_page = self.state.list_page.saturating_add(1);
                self.state.list_selected = 0;

                // Check if cache contains the new page range
                let start = self.state.list_page * 20;
                let end = start + 20;
                let range = start..end;

                if !self.mailing_list_cache.contains_range(range.clone()).await {
                    // Show loading screen if data not cached
                    self.terminal
                        .show(Screen::Loading(ArcStr::from("Loading mailing lists...")))
                        .await?;
                }

                self.render_lists().await
            }
            ViewKind::Feed => {
                self.state.feed_page = self.state.feed_page.saturating_add(1);
                self.state.feed_selected = 0;
                if let Some(list) = self.state.feed_list.clone() {
                    // Check if cache contains the new page range
                    let start = self.state.feed_page * 20;
                    let end = start + 20;
                    let range = start..end;

                    if !self
                        .patch_meta_cache
                        .contains_range(list.clone(), range.clone())
                        .await
                    {
                        // Show loading screen if data not cached
                        self.terminal
                            .show(Screen::Loading(ArcStr::from("Loading feed...")))
                            .await?;
                    }

                    self.render_feed(list).await
                } else {
                    Ok(())
                }
            }
            ViewKind::Patch => Ok(()), // No pagination in patch view
        }
    }

    /// Handle back navigation
    async fn handle_navigate_back(&mut self) -> Result<()> {
        match self.state.view {
            ViewKind::Lists => {
                // From lists, we quit
                self.terminal.quit().await
            }
            ViewKind::Feed => {
                // From feed back to lists
                self.log.info(SCOPE, "Feed -> Lists");
                self.state.view = ViewKind::Lists;
                self.render_lists().await
            }
            ViewKind::Patch => {
                // From patch back to feed
                self.log.info(SCOPE, "Patch -> Feed");
                self.state.view = ViewKind::Feed;
                if let Some(list) = self.state.feed_list.clone() {
                    self.render_feed(list).await
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Handle selection submission
    async fn handle_submit_selection(&mut self) -> Result<Option<NavigationAction>> {
        match self.state.view {
            ViewKind::Lists => {
                let start = self.state.list_page * 20;
                let end = start + 20;
                let items = self.mailing_list_cache.get_slice(start..end).await?;
                if let Some(selected) = items.get(self.state.list_selected) {
                    self.log.info(
                        SCOPE,
                        &format!("Lists -> Feed list={} (reset page/sel)", selected.name),
                    );
                    Ok(Some(NavigationAction::OpenFeed {
                        list: ArcStr::from(selected.name.clone()),
                    }))
                } else {
                    Ok(None)
                }
            }
            ViewKind::Feed => {
                if let Some(list) = self.state.feed_list.clone() {
                    let start = self.state.feed_page * 20;
                    let end = start + 20;
                    let items = self
                        .patch_meta_cache
                        .get_slice(list.clone(), start..end)
                        .await?;
                    if let Some(selected) = items.get(self.state.feed_selected) {
                        self.log.info(
                            SCOPE,
                            &format!(
                                "Feed -> Patch title='{}' list={} msg_id={}",
                                selected.title, list, selected.message_id
                            ),
                        );
                        Ok(Some(NavigationAction::OpenPatch {
                            list,
                            message_id: ArcStr::from(selected.message_id.clone()),
                            title: ArcStr::from(selected.title.clone()),
                        }))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            ViewKind::Patch => Ok(None), // No submission in patch view
        }
    }

    /// Render the lists view
    async fn render_lists(&self) -> Result<()> {
        let start = self.state.list_page * 20;
        let end = start + 20;
        self.log.info(
            SCOPE,
            &format!("Lists: fetching items range {}..{}", start, end),
        );
        let items = self.mailing_list_cache.get_slice(start..end).await?;
        self.log
            .info(SCOPE, &format!("Lists: fetched {} items", items.len()));
        self.terminal
            .show(Screen::Lists {
                items,
                page: self.state.list_page,
                selected: self.state.list_selected,
            })
            .await
    }

    /// Render the feed view
    async fn render_feed(&self, list: ArcStr) -> Result<()> {
        let start = self.state.feed_page * 20;
        let end = start + 20;
        self.log.info(
            SCOPE,
            &format!("Feed: list={} range {}..{}", list, start, end),
        );
        let items = self
            .patch_meta_cache
            .get_slice(list.clone(), start..end)
            .await?;
        if items.is_empty() {
            self.log.warn(
                SCOPE,
                &format!(
                    "Feed: empty page for list={} page={}",
                    list, self.state.feed_page
                ),
            );
            // Invalidate feed cache so next open refetches
            self.patch_meta_cache.invalidate_cache(list.clone()).await;
        } else {
            self.log
                .info(SCOPE, &format!("Feed: fetched {} items", items.len()));
        }
        self.terminal
            .show(Screen::Feed {
                list,
                items,
                page: self.state.feed_page,
                selected: self.state.feed_selected,
            })
            .await
    }

    /// Render the patch view
    async fn render_patch(&self, list: ArcStr, message_id: ArcStr, title: ArcStr) -> Result<()> {
        self.log.info(
            SCOPE,
            &format!(
                "Patch: opening title='{}' list={} msg_id={}",
                title, list, message_id
            ),
        );
        self.terminal
            .show(Screen::Loading(ArcStr::from("Loading patch...")))
            .await?;
        match self
            .lore
            .get_raw_patch(list.clone(), message_id.clone())
            .await
        {
            Ok(raw) => {
                self.log
                    .info(SCOPE, &format!("Patch: raw chars={}", raw.len()));
                match self.render.render_patch(raw).await {
                    Ok(rendered) => {
                        if rendered.is_empty() {
                            self.log.warn(
                                SCOPE,
                                &format!(
                                    "Patch: rendered empty content title='{}' list={} msg_id={}",
                                    title, list, message_id
                                ),
                            );
                            // Invalidate cache so next attempt refetches
                            self.patch_meta_cache.invalidate_cache(list).await;
                        } else {
                            self.log
                                .info(SCOPE, &format!("Patch: rendered chars={}", rendered.len()));
                        }
                        self.terminal
                            .show(Screen::Patch {
                                title,
                                content: rendered,
                            })
                            .await
                    }
                    Err(e) => {
                        self.log
                            .error(SCOPE, &format!("Patch: render error: {}", e));
                        self.patch_meta_cache.invalidate_cache(list).await;
                        self.terminal
                            .show(Screen::Error(ArcStr::from("Failed to render patch")))
                            .await
                    }
                }
            }
            Err(e) => {
                self.log.error(SCOPE, &format!("Patch: fetch error: {}", e));
                self.patch_meta_cache.invalidate_cache(list).await;
                self.terminal
                    .show(Screen::Error(ArcStr::from("Failed to load patch")))
                    .await
            }
        }
    }
}
