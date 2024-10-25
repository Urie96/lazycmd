use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::{bail, Context, Result};
use crossterm::event::Event as CrosstermEvent;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc;

use crate::{
    events::{Event, EventName, Events},
    term::{self, Term},
    widgets::{header::HeaderWidget, list::ListWidget},
    Page, State, TapKeyAsyncCallback,
};

pub struct App {
    frame_count: usize,
    event_sender: mpsc::UnboundedSender<Event>,
    state: Rc<RefCell<State>>,
    term: Term,
    quitting: bool,
    event_hooks: HashMap<EventName, Vec<TapKeyAsyncCallback>>,
}

impl App {
    pub fn new(state: Rc<RefCell<State>>, event_sender: mpsc::UnboundedSender<Event>) -> Self {
        let term = term::init().unwrap();

        Self {
            event_sender,
            state,
            term,
            frame_count: Default::default(),
            quitting: Default::default(),
            event_hooks: Default::default(),
        }
    }

    /// Runs the main loop of the application, handling events and actions
    pub async fn run(&mut self, mut events: Events) -> Result<()> {
        self.event_sender.send(Event::Enter(Vec::new())).unwrap();
        loop {
            if let Some(e) = events.next().await {
                self.handle_event(e).await?;
            }
            if self.quitting {
                break;
            }
        }
        Ok(())
    }

    /// Handles an event by producing an optional `Action` that the application
    /// should perform in response.
    ///
    /// This method maps incoming events from the terminal user interface to
    /// specific `Action` that represents tasks or operations the
    /// application needs to carry out.
    async fn handle_event(&mut self, e: Event) -> Result<()> {
        if let Some(hooks) = self.event_hooks.get(&EventName::from(&e)) {
            for hook in hooks {
                hook().await;
            }
        }
        match e {
            Event::Quit => {
                self.quitting = true;
            }
            // Event::Tick => Some(Action::Tick),
            Event::Render => {
                self.draw()?;
            }
            // Event::Crossterm(CrosstermEvent::Resize(x, y)) => Some(Action::Resize(x, y)),
            Event::Crossterm(CrosstermEvent::Key(key)) => {
                let cb = { self.state.borrow_mut().tap_key(key)? };
                if let Some(cb) = cb {
                    cb().await;
                }
            }
            Event::Command(command) => self.handle_command(command.as_str())?,
            Event::AddKeymap(keymap) => self.state.borrow_mut().add_keymap(keymap),
            Event::PageSetEntries(entries) => {
                self.state.borrow_mut().current_page = Some(Page {
                    list: entries,
                    ..Default::default()
                })
            }
            Event::AddEventHook(name, cb) => self.event_hooks.entry(name).or_default().push(cb),
            Event::Enter(path) => {
                {
                    self.state.borrow_mut().current_path = path;
                }
                if let Some(cbs) = self.event_hooks.get(&EventName::Enter) {
                    for cb in cbs {
                        cb().await;
                    }
                }
            }
            _ => (),
        };
        Ok(())
    }

    fn handle_command(&mut self, command: &str) -> Result<()> {
        let splits = shell_words::split(command)?;
        if splits.is_empty() {
            bail!("Empty command {}", command)
        }
        let mut it = splits.iter();
        match it.next().unwrap().as_str() {
            "quit" => {
                self.quitting = true;
            }
            cmd @ ("scroll_up" | "scroll_down") => {
                let num = match it.next() {
                    Some(num) => num.parse::<u16>().context("wrong format for scroll")?,
                    None => 1,
                };
                if cmd == "scroll_up" {
                    self.state.borrow_mut().scroll_up_by(num);
                } else {
                    self.state.borrow_mut().scroll_down_by(num);
                }
                self.event_sender.send(Event::Render).unwrap();
            }
            _ => bail!("Unsupported command {}", command),
        };
        Ok(())
    }

    // Render the `AppWidget` as a stateful widget using `self` as the `State`
    fn draw(&mut self) -> Result<()> {
        self.term.draw(|frame| {
            let mut state = self.state.borrow_mut();
            frame.render_stateful_widget(AppWidget, frame.area(), &mut *state);
            self.frame_count = frame.count()
        })?;
        Ok(())
    }
}

struct AppWidget;

impl StatefulWidget for AppWidget {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut State) {
        // Background color
        Block::default()
            // .bg(config::get().color.base00)
            .render(area, buf);

        use Constraint::*;
        let [header, main, _footer] = Layout::vertical([Length(3), Min(3), Length(1)]).areas(area);
        let [list, _preview] = Layout::horizontal([Percentage(50), Fill(1)]).areas(main);

        HeaderWidget.render(header, buf);
        if let Some(page) = &mut state.current_page {
            ListWidget.render(list, buf, page);
        } else {
            Paragraph::new("loading...").render(list, buf)
        }
    }
}
