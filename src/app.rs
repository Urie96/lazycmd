use anyhow::Result;
use crossterm::event::Event as CrosstermEvent;
use ratatui::{prelude::*, widgets::*};

use crate::{
    events::{Event, Events},
    term::{self, Term},
    widgets::{
        header::HeaderWidget,
        list::{List, ListWidget},
    },
    STATE,
};

#[derive(Debug)]
pub struct App {
    /// frame counter
    frame_count: usize,

    term: Term,

    list: List,

    quitting: bool,
}

impl App {
    pub fn new() -> Self {
        let term = term::init().unwrap();

        let mut s = Self {
            term,
            ..Default::default()
        };

        s.list.items = (0..100).map(|v| v.to_string()).collect();
        s
    }

    /// Runs the main loop of the application, handling events and actions
    pub async fn run(&mut self, mut events: Events) -> Result<()> {
        loop {
            if let Some(e) = events.next().await {
                self.handle_event(e)?;
            }
            // while let Ok(action) = self.rx.try_recv() {
            //     self.handle_action(action.clone())?;
            //     if matches!(action, Action::ScrollDown | Action::Render) {
            //         self.draw(&mut term)?;
            //     }
            // }
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
                self.draw();
            }
            // Event::Crossterm(CrosstermEvent::Resize(x, y)) => Some(Action::Resize(x, y)),
            Event::Crossterm(CrosstermEvent::Key(key)) => {
                STATE.borrow_mut().tap_key(key)?;
            }
            Event::Command(command) => match command.as_str() {
                "quit" => {
                    self.quitting = true;
                }
                _ => (),
            },
            _ => (),
        };
        Ok(())
    }

    // Render the `AppWidget` as a stateful widget using `self` as the `State`
    fn draw(&mut self) -> Result<()> {
        self.term.draw(|frame| {
            frame.render_widget(AppWidget, frame.area());
            // self.update_frame_count(frame);
        })?;
        Ok(())
    }

    // Sets the frame count
    fn update_frame_count(&mut self, frame: &mut Frame<'_>) {
        self.frame_count = frame.count();
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

struct AppWidget;

impl StatefulWidget for AppWidget {
    type State = App;

    fn render(self, area: Rect, buf: &mut Buffer) {
        // Background color
        Block::default()
            // .bg(config::get().color.base00)
            .render(area, buf);

        use Constraint::*;
        let [header, main, footer] = Layout::vertical([Length(3), Min(3), Length(1)]).areas(area);
        let [list, preview] = Layout::horizontal([Percentage(50), Fill(1)]).areas(main);
        state.render_header(header, buf);
        state.render_list(list, buf);
    }
}

impl App {
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        HeaderWidget::new().render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        ListWidget.render(area, buf, &mut self.list);
    }
}
