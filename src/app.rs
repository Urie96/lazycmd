use anyhow::{bail, Context, Result};
use crossterm::event::Event as CrosstermEvent;
use ratatui::{prelude::*, widgets::*};
use tracing::info;

use crate::{
    events::{self, Event, Events},
    term::{self, Term},
    widgets::{header::HeaderWidget, list::ListWidget},
    State, STATE,
};

#[derive(Debug)]
pub struct App {
    frame_count: usize,

    term: Term,
    quitting: bool,
}

impl App {
    pub fn new() -> Self {
        let term = term::init().unwrap();

        Self {
            term,
            frame_count: 0,
            quitting: false,
        }
    }

    /// Runs the main loop of the application, handling events and actions
    pub async fn run(&mut self, mut events: Events) -> Result<()> {
        loop {
            if let Some(e) = events.next().await {
                self.handle_event(e)?;
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
    fn handle_event(&mut self, e: Event) -> Result<()> {
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
                STATE.borrow_mut().tap_key(key)?;
            }
            Event::Command(command) => self.handle_command(command.as_str())?,
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
                    STATE.borrow_mut().scroll_up_by(num);
                } else {
                    STATE.borrow_mut().scroll_down_by(num);
                }
                events::emit(Event::Render)
            }
            _ => bail!("Unsupported command {}", command),
        };
        Ok(())
    }

    // Render the `AppWidget` as a stateful widget using `self` as the `State`
    fn draw(&mut self) -> Result<()> {
        self.term.draw(|frame| {
            frame.render_stateful_widget(AppWidget, frame.area(), &mut *STATE.borrow_mut());
            self.frame_count = frame.count()
        })?;
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
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
        let [header, main, footer] = Layout::vertical([Length(3), Min(3), Length(1)]).areas(area);
        let [list, preview] = Layout::horizontal([Percentage(50), Fill(1)]).areas(main);

        HeaderWidget.render(header, buf);
        let page = state.pages.entry(state.current_path.clone()).or_default();
        ListWidget.render(main, buf, page);
    }
}
