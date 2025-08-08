use anyhow::Context;

use crate::ArcStr;
use crate::api::lore::LoreApi;
use crate::app::cache::{mailing_list::MailingListCache, patch_meta::PatchMetaCache};
use crate::log::Log;
use crate::render::Render;
use crate::terminal::{Screen, Terminal, UiEvent};

/// High-level application actor managing TUI state and navigation.
#[derive(Debug, Clone)]
pub enum AppUi {
    Actual(tokio::sync::mpsc::Sender<Message>),
    #[allow(dead_code)]
    Mock,
}

#[derive(Debug)]
pub enum Message {
    Run,
    UiEvent(UiEvent),
}

#[derive(Debug, Clone, Copy)]
enum ViewKind {
    Lists,
    Feed,
    Patch,
}

#[derive(Debug)]
struct State {
    view: ViewKind,
    list_page: usize,
    list_selected: usize,
    feed_list: Option<ArcStr>,
    feed_page: usize,
    feed_selected: usize,
}

impl State {
    fn new() -> Self {
        Self {
            view: ViewKind::Lists,
            list_page: 0,
            list_selected: 0,
            feed_list: None,
            feed_page: 0,
            feed_selected: 0,
        }
    }
}

impl AppUi {
    /// Spawn the AppUi actor that orchestrates the TUI.
    pub fn spawn(
        log: Log,
        terminal: Terminal,
        mailing_list_cache: MailingListCache,
        patch_meta_cache: PatchMetaCache,
        lore: LoreApi,
        render: Render,
        mut ui_rx: tokio::sync::mpsc::Receiver<UiEvent>,
    ) -> (Self, tokio::task::JoinHandle<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(32);
        let tx_for_loop = tx.clone();

        let handle = tokio::spawn(async move {
            let mut st = State::new();

            // Initial screen load
            let _ = terminal
                .show(Screen::Loading(ArcStr::from("Loading mailing lists...")))
                .await;
            log.info("UI: show Lists screen");
            let _ = render_lists(&terminal, &mailing_list_cache, &log, &mut st).await;

            loop {
                tokio::select! {
                    Some(msg) = rx.recv() => {
                        match msg {
                            Message::Run => { /* already running */ }
                            Message::UiEvent(ev) => {
                                handle_event(ev, &log, &terminal, &mailing_list_cache, &patch_meta_cache, &lore, &render, &mut st).await;
                            }
                        }
                    }
                    Some(ev) = ui_rx.recv() => {
                        let _ = tx_for_loop.send(Message::UiEvent(ev)).await;
                    }
                    else => break,
                }
            }
        });

        (Self::Actual(tx), handle)
    }

    /// Entry point to start the UI loop; non-blocking.
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Actual(tx) => tx.send(Message::Run).await.context("sending Run to AppUi"),
            Self::Mock => Ok(()),
        }
    }
}

async fn render_lists(
    term: &Terminal,
    cache: &MailingListCache,
    log: &Log,
    st: &mut State,
) -> anyhow::Result<()> {
    let start = st.list_page * 20;
    let end = start + 20;
    log.info(format!("Lists: fetching items range {}..{}", start, end));
    let items = cache.get_slice(start..end).await?;
    log.info(format!("Lists: fetched {} items", items.len()));
    term.show(Screen::Lists {
        items,
        page: st.list_page,
        selected: st.list_selected,
    })
    .await
}

async fn render_feed(
    term: &Terminal,
    cache: &PatchMetaCache,
    log: &Log,
    st: &mut State,
    list: ArcStr,
) -> anyhow::Result<()> {
    let start = st.feed_page * 20;
    let end = start + 20;
    log.info(format!(
        "Feed: list={} range {}..{}",
        list.to_string(),
        start,
        end
    ));
    let items = cache.get_slice(list.clone(), start..end).await?;
    if items.is_empty() {
        log.warn(format!(
            "Feed: empty page for list={} page={}",
            list.to_string(),
            st.feed_page
        ));
        // Invalidate feed cache so next open refetches
        cache.invalidate_cache(list.clone()).await;
    } else {
        log.info(format!("Feed: fetched {} items", items.len()));
    }
    term.show(Screen::Feed {
        list,
        items,
        page: st.feed_page,
        selected: st.feed_selected,
    })
    .await
}

async fn render_patch(
    term: &Terminal,
    lore: &LoreApi,
    render: &Render,
    log: &Log,
    feed_cache: &PatchMetaCache,
    list: ArcStr,
    msg_id: ArcStr,
    title: ArcStr,
) -> anyhow::Result<()> {
    log.info(format!(
        "Patch: opening title='{}' list={} msg_id={}",
        title.to_string(),
        list.to_string(),
        msg_id.to_string()
    ));
    term.show(Screen::Loading(ArcStr::from("Loading patch...")))
        .await?;
    match lore.get_raw_patch(list.clone(), msg_id.clone()).await {
        Ok(raw) => {
            log.info(format!("Patch: raw chars={}", raw.len()));
            match render.render_patch(raw.clone()).await {
                Ok(rendered) => {
                    if rendered.is_empty() {
                        log.warn(format!(
                            "Patch: rendered empty content title='{}' list={} msg_id={}",
                            title.to_string(),
                            list.to_string(),
                            msg_id.to_string()
                        ));
                        // Invalidate cache so next attempt refetches
                        feed_cache.invalidate_cache(list.clone()).await;
                    } else {
                        log.info(format!("Patch: rendered chars={}", rendered.len()));
                    }
                    term.show(Screen::Patch {
                        title,
                        content: rendered,
                    })
                    .await
                }
                Err(e) => {
                    log.error(format!("Patch: render error: {}", e));
                    feed_cache.invalidate_cache(list.clone()).await;
                    term.show(Screen::Error(ArcStr::from("Failed to render patch")))
                        .await
                }
            }
        }
        Err(e) => {
            log.error(format!("Patch: fetch error: {}", e));
            feed_cache.invalidate_cache(list.clone()).await;
            term.show(Screen::Error(ArcStr::from("Failed to load patch")))
                .await
        }
    }
}

async fn handle_event(
    ev: UiEvent,
    _log: &Log,
    term: &Terminal,
    list_cache: &MailingListCache,
    feed_cache: &PatchMetaCache,
    lore: &LoreApi,
    render: &Render,
    st: &mut State,
) {
    match (st.view, ev) {
        (ViewKind::Lists, UiEvent::SelectionChange(idx)) => {
            st.list_selected = idx;
        }
        (ViewKind::Lists, UiEvent::Left) => {
            st.list_page = st.list_page.saturating_sub(1);
            st.list_selected = 0;
            let _ = render_lists(term, list_cache, _log, st).await;
        }
        (ViewKind::Lists, UiEvent::Right) => {
            st.list_page = st.list_page.saturating_add(1);
            st.list_selected = 0;
            let _ = render_lists(term, list_cache, _log, st).await;
        }
        (ViewKind::Lists, UiEvent::SelectionSubmit(idx)) => {
            // Resolve selected list name
            let start = st.list_page * 20;
            let end = start + 20;
            if let Ok(items) = list_cache.get_slice(start..end).await {
                if let Some(selected) = items.get(idx) {
                    _log.info(format!(
                        "UI: Lists -> Feed list={} (reset page/sel)",
                        selected.name
                    ));
                    st.view = ViewKind::Feed;
                    st.feed_page = 0;
                    st.feed_selected = 0;
                    st.feed_list = Some(ArcStr::from(selected.name.clone()));
                    let _ = term
                        .show(Screen::Loading(ArcStr::from("Loading feed...")))
                        .await;
                    let _ = render_feed(
                        term,
                        feed_cache,
                        _log,
                        st,
                        ArcStr::from(selected.name.clone()),
                    )
                    .await;
                }
            }
        }
        (ViewKind::Feed, UiEvent::SelectionChange(idx)) => {
            st.feed_selected = idx;
        }
        (ViewKind::Feed, UiEvent::Left) => {
            st.feed_page = st.feed_page.saturating_sub(1);
            st.feed_selected = 0;
            if let Some(list) = st.feed_list.clone() {
                let _ = render_feed(term, feed_cache, _log, st, list).await;
            }
        }
        (ViewKind::Feed, UiEvent::Right) => {
            st.feed_page = st.feed_page.saturating_add(1);
            st.feed_selected = 0;
            if let Some(list) = st.feed_list.clone() {
                let _ = render_feed(term, feed_cache, _log, st, list).await;
            }
        }
        (ViewKind::Feed, UiEvent::SelectionSubmit(idx)) => {
            if let Some(list) = st.feed_list.clone() {
                let start = st.feed_page * 20;
                let end = start + 20;
                if let Ok(items) = feed_cache.get_slice(list.clone(), start..end).await {
                    if let Some(sel) = items.get(idx) {
                        _log.info(format!(
                            "UI: Feed -> Patch title='{}' list={} msg_id={}",
                            sel.title,
                            list.to_string(),
                            sel.message_id
                        ));
                        st.view = ViewKind::Patch;
                        let _ = render_patch(
                            term,
                            lore,
                            render,
                            _log,
                            feed_cache,
                            list,
                            ArcStr::from(sel.message_id.clone()),
                            ArcStr::from(sel.title.clone()),
                        )
                        .await;
                    }
                }
            }
        }
        (ViewKind::Patch, UiEvent::Esc) => {
            _log.info("UI: Patch -> Feed");
            st.view = ViewKind::Feed;
            if let Some(list) = st.feed_list.clone() {
                let _ = render_feed(term, feed_cache, _log, st, list).await;
            }
        }
        (_, UiEvent::Esc) => {
            // From Lists -> quit; From Feed without previous screen -> back to Lists
            if matches!(st.view, ViewKind::Lists) {
                _log.info("UI: Lists -> Quit");
                let _ = term.quit().await;
            } else if matches!(st.view, ViewKind::Feed) {
                _log.info("UI: Feed -> Lists");
                st.view = ViewKind::Lists;
                let _ = render_lists(term, list_cache, _log, st).await;
            }
        }
        _ => {}
    }
}
