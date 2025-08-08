use anyhow::Context;
use cursive::Cursive;
use cursive::event::{Event, Key};
use cursive::traits::*;
use cursive::views::{Dialog, SelectView, TextView};
use std::thread;

use crate::ArcStr;
use crate::api::lore::{LoreMailingList, LorePatchMetadata};
use crate::log::Log;

/// UI key events emitted by the terminal.
#[derive(Debug, Clone, Copy)]
pub enum UiEvent {
    Left,
    Right,
    Esc,
    SelectionChange(usize),
    SelectionSubmit(usize),
}

/// A high-level description of the screen to render.
#[derive(Debug, Clone)]
pub enum Screen {
    /// Lists screen: shows mailing lists, with current page and selection
    Lists {
        items: Vec<LoreMailingList>,
        page: usize,
        selected: usize,
    },
    /// Feed screen: shows patches for a mailing list
    Feed {
        list: ArcStr,
        items: Vec<LorePatchMetadata>,
        page: usize,
        selected: usize,
    },
    /// Patch screen: shows rendered patch content
    Patch { title: ArcStr, content: ArcStr },
    /// Loading screen with a message
    Loading(ArcStr),
    /// Error screen with a message
    Error(ArcStr),
}

/// Messages that can be sent to the terminal actor.
#[derive(Debug)]
pub enum Message {
    /// Render the given screen
    Show(Screen),
    /// Quit the UI
    Quit,
}

/// The terminal actor that owns the Cursive event loop and exposes a message-based API.
#[derive(Debug, Clone)]
pub enum Terminal {
    Actual(tokio::sync::mpsc::Sender<Message>),
    #[allow(dead_code)]
    Mock,
}

impl Terminal {
    /// Spawns a terminal actor using the Cursive `crossterm` backend.
    ///
    /// The actor sends `UiEvent`s to `ui_events` and accepts `Message`s to update the UI.
    pub fn spawn(
        log: Log,
        ui_events: tokio::sync::mpsc::Sender<UiEvent>,
    ) -> anyhow::Result<(Self, tokio::sync::oneshot::Receiver<()>)> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Message>(16);

        // Spawn the Cursive loop in its own thread and obtain its callback sink.
        let (sink_tx, sink_rx) = std::sync::mpsc::channel();
        let ui_events_async = ui_events.clone();
        let (exit_tx, exit_rx) = tokio::sync::oneshot::channel::<()>();
        thread::spawn(move || {
            let mut siv = cursive::crossterm();

            // Install global key callbacks to forward to app actor.
            let fwd = |ev: UiEvent| {
                let tx = ui_events.clone();
                move |_s: &mut Cursive| {
                    let _ = tx.try_send(ev);
                }
            };

            siv.add_global_callback(Event::Key(Key::Left), fwd(UiEvent::Left));
            siv.add_global_callback(Event::Key(Key::Right), fwd(UiEvent::Right));
            siv.add_global_callback(Event::Key(Key::Esc), fwd(UiEvent::Esc));

            let cb_sink = siv.cb_sink().clone();
            let _ = sink_tx.send(cb_sink);

            // Run the event loop
            siv.add_layer(Dialog::around(TextView::new("Starting...")));
            siv.run();
            // Notify async side that UI exited
            let _ = exit_tx.send(());
        });

        // Receive the Cursive callback sink from the UI thread
        let cb_sink = sink_rx
            .recv()
            .expect("failed to initialize cursive callback sink");

        // Async task that receives Messages and schedules UI updates via cb_sink
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    Message::Show(screen) => {
                        let ev_tx_outer = ui_events_async.clone();
                        let _ = cb_sink.send(Box::new(move |s: &mut Cursive| match screen {
                            Screen::Loading(text) => {
                                s.pop_layer();
                                let msg = text.to_string();
                                s.add_layer(Dialog::around(TextView::new(msg)).title("Loading"));
                            }
                            Screen::Error(text) => {
                                s.pop_layer();
                                let msg = text.to_string();
                                s.add_layer(Dialog::around(TextView::new(msg)).title("Error"));
                            }
                            Screen::Lists {
                                items,
                                page,
                                selected,
                            } => {
                                s.pop_layer();
                                let mut list = SelectView::<usize>::new();
                                for (i, it) in items.into_iter().enumerate() {
                                    let label = format!("{} - {}", it.name, it.description);
                                    list.add_item(label, i);
                                }
                                let tx_sel = ev_tx_outer.clone();
                                list.set_on_select(move |_siv, idx| {
                                    let _ = tx_sel.try_send(UiEvent::SelectionChange(*idx));
                                });
                                let tx_submit = ev_tx_outer.clone();
                                list.set_on_submit(move |_siv, idx| {
                                    let _ = tx_submit.try_send(UiEvent::SelectionSubmit(*idx));
                                });
                                let len = list.len();
                                let idx = selected.min(len.saturating_sub(1));
                                let _ = list.set_selection(idx);
                                s.add_layer(
                                    Dialog::around(list)
                                        .title(format!("Mailing Lists - Page {}", page + 1)),
                                );
                            }
                            Screen::Feed {
                                list,
                                items,
                                page,
                                selected,
                            } => {
                                s.pop_layer();
                                let mut listv = SelectView::<usize>::new();
                                for (i, p) in items.into_iter().enumerate() {
                                    let label = format!("{} — {} <{}>", p.title, p.author, p.email);
                                    listv.add_item(label, i);
                                }
                                let tx_sel = ev_tx_outer.clone();
                                listv.set_on_select(move |_siv, idx| {
                                    let _ = tx_sel.try_send(UiEvent::SelectionChange(*idx));
                                });
                                let tx_submit = ev_tx_outer.clone();
                                listv.set_on_submit(move |_siv, idx| {
                                    let _ = tx_submit.try_send(UiEvent::SelectionSubmit(*idx));
                                });
                                let len = listv.len();
                                let idx = selected.min(len.saturating_sub(1));
                                let _ = listv.set_selection(idx);
                                s.add_layer(Dialog::around(listv).title(format!(
                                    "Feed: {} — Page {}",
                                    list.to_string(),
                                    page + 1
                                )));
                            }
                            Screen::Patch { title, content } => {
                                s.pop_layer();
                                let text = TextView::new(content.to_string()).scrollable();
                                s.add_layer(
                                    Dialog::around(text)
                                        .title(format!("Patch: {}", title.to_string())),
                                );
                            }
                        }));
                    }
                    Message::Quit => {
                        let _ = cb_sink.send(Box::new(|s: &mut Cursive| s.quit()));
                        break;
                    }
                }
            }
        });

        log.info("Terminal actor spawned");
        Ok((Terminal::Actual(tx), exit_rx))
    }

    /// Requests the terminal to show a specific screen.
    pub async fn show(&self, screen: Screen) -> anyhow::Result<()> {
        match self {
            Terminal::Actual(tx) => tx
                .send(Message::Show(screen))
                .await
                .context("Sending Show message to terminal"),
            Terminal::Mock => Ok(()),
        }
    }

    /// Requests the terminal to quit.
    pub async fn quit(&self) -> anyhow::Result<()> {
        match self {
            Terminal::Actual(tx) => tx
                .send(Message::Quit)
                .await
                .context("Sending Quit message to terminal"),
            Terminal::Mock => Ok(()),
        }
    }
}
