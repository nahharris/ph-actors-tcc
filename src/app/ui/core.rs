use anyhow::Result;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::ArcStr;
use crate::app::cache::{FeedCache, MailingListCache, PatchCache};
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
    /// Feed cache
    feed_cache: FeedCache,
    /// Patch cache
    patch_cache: PatchCache,
    /// Render actor
    render: Render,
}

impl Core {
    /// Create a new UI actor core
    pub fn new(
        log: Log,
        terminal: Terminal,
        mailing_list_cache: MailingListCache,
        feed_cache: FeedCache,
        patch_cache: PatchCache,
        render: Render,
    ) -> Self {
        Self {
            state: UiState::default(),
            log,
            terminal,
            mailing_list_cache,
            feed_cache,
            patch_cache,
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

        // Show loading screen immediately when transitioning to feed view
        self.terminal
            .show(Screen::Loading(ArcStr::from("Loading feed...")))
            .await?;

        // Ensure cache is loaded from disk
        self.feed_cache.ensure_loaded(list.clone()).await?;

        // Check if cache has data
        let is_empty = self.feed_cache.is_empty(list.clone()).await;
        if is_empty {
            self.log.info(
                SCOPE,
                &format!("Feed cache for '{}' is empty, will fetch data", list),
            );
        } else {
            self.log.info(
                SCOPE,
                &format!("Feed cache for '{}' has data, using cached version", list),
            );
        }

        // Render the feed - this will handle loading data if needed
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
                let new_page = self.state.list_page.saturating_sub(1);
                if new_page == self.state.list_page {
                    // Already at page 0, can't go back
                    return Ok(());
                }

                // Check if the previous page would have data
                let start = new_page * 20;
                let end = start + 20;
                let items = self.mailing_list_cache.get_slice(start..end).await?;
                if items.is_empty() {
                    // Previous page is empty, don't navigate
                    return Ok(());
                }

                self.state.list_page = new_page;
                self.state.list_selected = 0;
                self.render_lists().await
            }
            ViewKind::Feed => {
                let new_page = self.state.feed_page.saturating_sub(1);
                if new_page == self.state.feed_page {
                    // Already at page 0, can't go back
                    return Ok(());
                }

                if let Some(list) = self.state.feed_list.clone() {
                    // Navigate to the new page immediately
                    self.state.feed_page = new_page;
                    self.state.feed_selected = 0;

                    // Check if data is available in cache
                    let start = new_page * 20;
                    let end = start + 20;
                    let is_available = self.feed_cache.is_available(list.clone(), start..end).await;

                    if is_available {
                        // Data is available, render immediately
                        self.render_feed(list).await
                    } else {
                        // Data not available, show loading screen and fetch
                        self.log.info(
                            SCOPE,
                            &format!(
                                "Feed: page {} not in cache for '{}', showing loading screen",
                                new_page, list
                            ),
                        );

                        // Show loading screen
                        self.terminal
                            .show(Screen::Loading(ArcStr::from("Loading page...")))
                            .await?;

                        // Fetch the data (this will trigger on-demand fetching)
                        let items = self.feed_cache.get_slice(list.clone(), start..end).await?;

                        // Show the feed with fetched data
                        self.terminal
                            .show(Screen::Feed {
                                list,
                                items,
                                page: self.state.feed_page,
                                selected: self.state.feed_selected,
                            })
                            .await
                    }
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
                let new_page = self.state.list_page.saturating_add(1);

                // Check if the next page would have data
                let start = new_page * 20;
                let end = start + 20;
                let items = self.mailing_list_cache.get_slice(start..end).await?;
                if items.is_empty() {
                    // Next page is empty, don't navigate
                    return Ok(());
                }

                self.state.list_page = new_page;
                self.state.list_selected = 0;
                self.render_lists().await
            }
            ViewKind::Feed => {
                let new_page = self.state.feed_page.saturating_add(1);

                if let Some(list) = self.state.feed_list.clone() {
                    // Navigate to the new page immediately
                    self.state.feed_page = new_page;
                    self.state.feed_selected = 0;

                    // Check if data is available in cache
                    let start = new_page * 20;
                    let end = start + 20;
                    let is_available = self.feed_cache.is_available(list.clone(), start..end).await;

                    if is_available {
                        // Data is available, render immediately
                        self.render_feed(list).await
                    } else {
                        // Data not available, show loading screen and fetch
                        self.log.info(
                            SCOPE,
                            &format!(
                                "Feed: page {} not in cache for '{}', showing loading screen",
                                new_page, list
                            ),
                        );

                        // Show loading screen
                        self.terminal
                            .show(Screen::Loading(ArcStr::from("Loading page...")))
                            .await?;

                        // Fetch the data (this will trigger on-demand fetching)
                        let items = self.feed_cache.get_slice(list.clone(), start..end).await?;

                        // Show the feed with fetched data
                        self.terminal
                            .show(Screen::Feed {
                                list,
                                items,
                                page: self.state.feed_page,
                                selected: self.state.feed_selected,
                            })
                            .await
                    }
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
                    let items = self.feed_cache.get_slice(list.clone(), start..end).await?;
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

        if items.is_empty() {
            self.log.warn(
                SCOPE,
                &format!("Lists: empty page for page={}", self.state.list_page),
            );

            // Check if cache is empty and needs refresh
            let total_items = self.mailing_list_cache.len().await;
            if total_items == 0 {
                self.log.info(SCOPE, "Lists: cache empty, refreshing");

                // Use tokio::timeout to prevent hanging
                let refresh_result = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    self.mailing_list_cache.refresh(),
                )
                .await;

                match refresh_result {
                    Ok(Ok(())) => {
                        // Refresh succeeded, try again
                        let refreshed_items = self.mailing_list_cache.get_slice(start..end).await?;
                        if refreshed_items.is_empty() {
                            self.log.warn(
                                SCOPE,
                                &format!(
                                    "Lists: still empty after refresh for page={}",
                                    self.state.list_page
                                ),
                            );
                        } else {
                            self.log.info(
                                SCOPE,
                                &format!(
                                    "Lists: refreshed, fetched {} items",
                                    refreshed_items.len()
                                ),
                            );
                        }

                        self.terminal
                            .show(Screen::Lists {
                                items: refreshed_items,
                                page: self.state.list_page,
                                selected: self.state.list_selected,
                            })
                            .await
                    }
                    Ok(Err(e)) => {
                        // Refresh failed
                        self.log
                            .error(SCOPE, &format!("Lists: refresh failed: {}", e));
                        self.terminal
                            .show(Screen::Error(ArcStr::from("Failed to load mailing lists")))
                            .await
                    }
                    Err(_) => {
                        // Refresh timed out
                        self.log.error(SCOPE, "Lists: refresh timed out");
                        self.terminal
                            .show(Screen::Error(ArcStr::from(
                                "Mailing lists loading timed out",
                            )))
                            .await
                    }
                }
            } else {
                // Cache has data but this page is empty (e.g., page beyond available data)
                self.terminal
                    .show(Screen::Lists {
                        items,
                        page: self.state.list_page,
                        selected: self.state.list_selected,
                    })
                    .await
            }
        } else {
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
    }

    /// Render the feed view
    async fn render_feed(&self, list: ArcStr) -> Result<()> {
        let start = self.state.feed_page * 20;
        let end = start + 20;
        self.log.info(
            SCOPE,
            &format!("Feed: list={} range {}..{}", list, start, end),
        );

        // Try to get items from cache
        let items = self.feed_cache.get_slice(list.clone(), start..end).await?;

        if items.is_empty() {
            self.log.warn(
                SCOPE,
                &format!(
                    "Feed: empty page for list={} page={}",
                    list, self.state.feed_page
                ),
            );

            // Check if we need to load more data or if this is truly empty
            let total_items = self.feed_cache.len(list.clone()).await;
            if total_items == 0 {
                // Cache is empty, try to refresh it with timeout
                self.log.info(
                    SCOPE,
                    &format!("Feed: cache empty for '{}', refreshing", list),
                );

                // Use tokio::timeout to prevent hanging
                let refresh_result = tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    self.feed_cache.refresh(list.clone()),
                )
                .await;

                match refresh_result {
                    Ok(Ok(())) => {
                        // Refresh succeeded, try again
                        let refreshed_items =
                            self.feed_cache.get_slice(list.clone(), start..end).await?;
                        if refreshed_items.is_empty() {
                            self.log.warn(
                                SCOPE,
                                &format!(
                                    "Feed: still empty after refresh for list={} page={}",
                                    list, self.state.feed_page
                                ),
                            );
                        } else {
                            self.log.info(
                                SCOPE,
                                &format!(
                                    "Feed: refreshed, fetched {} items",
                                    refreshed_items.len()
                                ),
                            );
                            // Persist the cache after successful refresh
                            if let Err(e) = self.feed_cache.persist(list.clone()).await {
                                self.log.warn(
                                    SCOPE,
                                    &format!("Feed: failed to persist cache for '{}': {}", list, e),
                                );
                            }
                        }

                        self.terminal
                            .show(Screen::Feed {
                                list,
                                items: refreshed_items,
                                page: self.state.feed_page,
                                selected: self.state.feed_selected,
                            })
                            .await
                    }
                    Ok(Err(e)) => {
                        // Refresh failed
                        self.log.error(
                            SCOPE,
                            &format!("Feed: refresh failed for '{}': {}", list, e),
                        );
                        self.terminal
                            .show(Screen::Error(ArcStr::from("Failed to load feed data")))
                            .await
                    }
                    Err(_) => {
                        // Refresh timed out
                        self.log
                            .error(SCOPE, &format!("Feed: refresh timed out for '{}'", list));
                        self.terminal
                            .show(Screen::Error(ArcStr::from("Feed loading timed out")))
                            .await
                    }
                }
            } else {
                // Cache has data but this page is empty (e.g., page beyond available data)
                self.terminal
                    .show(Screen::Feed {
                        list,
                        items,
                        page: self.state.feed_page,
                        selected: self.state.feed_selected,
                    })
                    .await
            }
        } else {
            self.log
                .info(SCOPE, &format!("Feed: fetched {} items", items.len()));
            self.terminal
                .show(Screen::Feed {
                    list,
                    items,
                    page: self.state.feed_page,
                    selected: self.state.feed_selected,
                })
                .await
        }
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
        match self.patch_cache.get(list.clone(), message_id.clone()).await {
            Ok(patch) => {
                self.log.info(
                    SCOPE,
                    &format!(
                        "Patch: title='{}' version={} sequence={}",
                        patch.title, patch.version, patch.sequence
                    ),
                );
                match self.render.render_patch(patch).await {
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
                            self.feed_cache.invalidate(list).await?;
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
                        self.feed_cache.invalidate(list).await?;
                        self.terminal
                            .show(Screen::Error(ArcStr::from("Failed to render patch")))
                            .await
                    }
                }
            }
            Err(e) => {
                self.log.error(SCOPE, &format!("Patch: fetch error: {}", e));
                self.feed_cache.invalidate(list).await?;
                self.terminal
                    .show(Screen::Error(ArcStr::from("Failed to load patch")))
                    .await
            }
        }
    }
}
