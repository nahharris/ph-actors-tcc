use cursive::theme::Theme;
use cursive::utils::markup::ansi;
use cursive::Cursive;
use cursive::event::{Event, Key};
use cursive::traits::*;
use cursive::views::{Dialog, SelectView, TextView};
use std::thread;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::Terminal;
use super::data::{Screen, UiEvent};
use super::message::Message;
use crate::log::Log;

const SCOPE: &str = "terminal";

/// Core implementation of the terminal actor that manages the Cursive UI.
pub struct Core {
    log: Log,
    ui_events: mpsc::Sender<UiEvent>,
}

impl Core {
    /// Creates a new terminal core with the required dependencies.
    ///
    /// # Arguments
    /// * `log` - Logging actor for recording terminal events
    /// * `ui_events` - Channel sender for forwarding UI events to the application
    pub fn new(log: Log, ui_events: mpsc::Sender<UiEvent>) -> Self {
        Self { log, ui_events }
    }

    /// Spawns the terminal actor and returns the public interface and join handle.
    ///
    /// This method takes ownership of the core and starts both the Cursive UI thread
    /// and the async message handling task.
    pub fn spawn(self) -> (Terminal, JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel::<Message>(crate::BUFFER_SIZE);

        // The main actor task - this follows the tokio task requirement
        let handle = tokio::spawn(async move {
            // Spawn the Cursive loop in its own thread and obtain its callback sink
            let (sink_tx, sink_rx) = std::sync::mpsc::channel();
            let ui_events = self.ui_events.clone();

            // We need to spawn Cursive in a thread because it requires blocking I/O
            // But the actor itself is still a tokio task
            thread::spawn(move || {
                let mut siv = cursive::crossterm();
                siv.set_theme(Theme::terminal_default());

                // Install global key callbacks to forward to app actor
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
            });

            // Receive the Cursive callback sink from the UI thread
            let cb_sink = sink_rx
                .recv()
                .expect("failed to initialize cursive callback sink");

            self.log.info(SCOPE, "Terminal actor spawned");

            // Message handling loop - this is the actual actor behavior
            while let Some(msg) = rx.recv().await {
                match msg {
                    Message::Show(screen) => {
                        Self::handle_show_screen(screen, &cb_sink, self.ui_events.clone());
                    }
                    Message::Quit => {
                        let _ = cb_sink.send(Box::new(|s: &mut Cursive| s.quit()));
                        break;
                    }
                }
            }
        });

        (Terminal::Actual(tx), handle)
    }

    /// Handles the Show message by updating the UI with the given screen.
    fn handle_show_screen(
        screen: Screen,
        cb_sink: &cursive::CbSink,
        ui_events: mpsc::Sender<UiEvent>,
    ) {
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
                let tx_sel = ui_events.clone();
                list.set_on_select(move |_siv, idx| {
                    let _ = tx_sel.try_send(UiEvent::SelectionChange(*idx));
                });
                let tx_submit = ui_events.clone();
                list.set_on_submit(move |_siv, idx| {
                    let _ = tx_submit.try_send(UiEvent::SelectionSubmit(*idx));
                });
                let len = list.len();
                let idx = selected.min(len.saturating_sub(1));
                let _ = list.set_selection(idx);
                s.add_layer(
                    Dialog::around(list).title(format!("Mailing Lists - Page {}", page + 1)),
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
                let tx_sel = ui_events.clone();
                listv.set_on_select(move |_siv, idx| {
                    let _ = tx_sel.try_send(UiEvent::SelectionChange(*idx));
                });
                let tx_submit = ui_events.clone();
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
                // TODO: remove this workaround
                let content = content.replace("\x1b[0K", "");
                let content = regex::Regex::new(r"\x1b\[(\d+);(\d+);(\d+);(\d+);(\d+);(\d+)m")
                    .unwrap()
                    .replace_all(&content, "\x1b[$1;$2;${3}m\x1b[$4;$5;${6}m");
                let content = ansi::parse(content.to_string());
                let text = TextView::new(content).scrollable();
                s.add_layer(Dialog::around(text).title(format!("Patch: {}", title.to_string())));
            }
        }));
    }
}
