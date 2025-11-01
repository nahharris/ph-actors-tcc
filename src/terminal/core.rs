use cursive::Cursive;
use cursive::event::{Event, Key};
use cursive::traits::*;
use cursive::views::{Dialog, SelectView, TextView};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc;

use super::data::{Screen, UiEvent};
use super::message::Message;
use crate::log::Log;

const SCOPE: &str = "terminal";

/// Core implementation of the terminal actor that manages the Cursive UI.
pub struct Core {
    log: Log,
    /// Internal FIFO queue for storing UI events
    ui_events_queue: Arc<Mutex<VecDeque<UiEvent>>>,
}

impl Core {
    /// Creates a new terminal core with the required dependencies.
    ///
    /// # Arguments
    /// * `log` - Logging actor for recording terminal events
    pub fn new(log: Log) -> Self {
        Self {
            log,
            ui_events_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Initializes the terminal actor message receiver.
    ///
    /// This method processes messages from the receiver in a loop, handling each message
    /// using pattern matching.
    ///
    /// # Arguments
    /// * `rx` - A receiver for messages to process
    pub async fn init(self, mut rx: mpsc::Receiver<Message>) {
        // Spawn the Cursive loop in its own thread and obtain its callback sink
        let (sink_tx, sink_rx) = std::sync::mpsc::channel();
        let ui_events_queue = self.ui_events_queue.clone();

        // We need to spawn Cursive in a thread because it requires blocking I/O
        // But the actor itself is still a tokio task
        thread::spawn(move || {
            let mut siv = cursive::crossterm();

            // Install global key callbacks to store events in the queue
            let fwd = |ev: UiEvent| {
                let queue = ui_events_queue.clone();
                move |_s: &mut Cursive| {
                    if let Ok(mut q) = queue.lock() {
                        q.push_back(ev);
                    }
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

        self.log.info(SCOPE, "Terminal actor spawned".to_string());

        // Message handling loop - this is the actual actor behavior
        while let Some(msg) = rx.recv().await {
            use Message::*;
            match msg {
                Show(screen) => {
                    self.handle_show_screen(screen, &cb_sink);
                }
                GetUiEvent { tx } => {
                    let event = {
                        if let Ok(mut queue) = self.ui_events_queue.lock() {
                            queue.pop_front()
                        } else {
                            None
                        }
                    };
                    let _ = tx.send(event);
                }
                ClearUiEvents { tx } => {
                    if let Ok(mut queue) = self.ui_events_queue.lock() {
                        queue.clear();
                    }
                    let _ = tx.send(());
                }
                Quit { tx } => {
                    let _ = cb_sink.send(Box::new(|s: &mut Cursive| s.quit()));
                    let _ = tx.send(());
                    break;
                }
            }
        }
    }

    /// Handles the Show message by updating the UI with the given screen.
    fn handle_show_screen(&self, screen: Screen, cb_sink: &cursive::CbSink) {
        let ui_events_queue = self.ui_events_queue.clone();
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
                let queue_sel = ui_events_queue.clone();
                list.set_on_select(move |_siv, idx| {
                    if let Ok(mut q) = queue_sel.lock() {
                        q.push_back(UiEvent::SelectionChange(*idx));
                    }
                });
                let queue_submit = ui_events_queue.clone();
                list.set_on_submit(move |_siv, idx| {
                    if let Ok(mut q) = queue_submit.lock() {
                        q.push_back(UiEvent::SelectionSubmit(*idx));
                    }
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
                let queue_sel = ui_events_queue.clone();
                listv.set_on_select(move |_siv, idx| {
                    if let Ok(mut q) = queue_sel.lock() {
                        q.push_back(UiEvent::SelectionChange(*idx));
                    }
                });
                let queue_submit = ui_events_queue.clone();
                listv.set_on_submit(move |_siv, idx| {
                    if let Ok(mut q) = queue_submit.lock() {
                        q.push_back(UiEvent::SelectionSubmit(*idx));
                    }
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
                s.add_layer(Dialog::around(text).title(format!("Patch: {}", title.to_string())));
            }
        }));
    }
}
