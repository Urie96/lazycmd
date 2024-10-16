use std::borrow::Borrow;

use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    action::Action,
    events::{Event, Events},
    term::Term,
    widgets::{
        header::HeaderWidget,
        list::{List, ListWidget},
    },
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    #[default]
    Common,
    Quit,
}

#[derive(Debug)]
pub struct App {
    /// Receiver end of an asynchronous channel for actions that the app needs
    /// to process.
    rx: UnboundedReceiver<Action>,

    /// Sender end of an asynchronous channel for dispatching actions from
    /// various parts of the app to be handled by the event loop.
    tx: UnboundedSender<Action>,

    /// The active mode of the application, which could change how user inputs
    /// and commands are interpreted.
    mode: Mode,

    /// frame counter
    frame_count: usize,

    list: List,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut s = Self {
            rx,
            tx,
            mode: Default::default(),
            frame_count: Default::default(),
            list: Default::default(),
        };
        s.list.items = (0..100).map(|v| v.to_string()).collect();
        s
    }

    /// Runs the main loop of the application, handling events and actions
    pub async fn run(&mut self, mut term: Term, mut events: Events) -> Result<()> {
        // uncomment to test error handling
        // panic!("test panic");
        // Err(color_eyre::eyre::eyre!("Error"))?;
        self.tx.send(Action::Init)?;

        loop {
            if let Some(e) = events.next().await {
                self.handle_event(e)?.map(|action| self.tx.send(action));
            }
            while let Ok(action) = self.rx.try_recv() {
                self.handle_action(action.clone())?;
                if matches!(action, Action::ScrollDown | Action::Render) {
                    self.draw(&mut term)?;
                }
            }
            if self.mode == Mode::Quit {
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
    fn handle_event(&mut self, e: Event) -> Result<Option<Action>> {
        let maybe_action = match e {
            Event::Quit => Some(Action::Quit),
            Event::Tick => Some(Action::Tick),
            Event::Render => Some(Action::Render),
            Event::Crossterm(CrosstermEvent::Resize(x, y)) => Some(Action::Resize(x, y)),
            Event::Crossterm(CrosstermEvent::Key(key)) => match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Down => Some(Action::ScrollDown),
                _ => None,
            },
            _ => None,
        };
        Ok(maybe_action)
    }

    /// Performs the `Action` by calling on a respective app method.
    ///
    /// Upon receiving an action, this function updates the application state, performs necessary
    /// operations like drawing or resizing the view, or changing the mode. Actions that affect the
    /// navigation within the application, are also handled. Certain actions generate a follow-up
    /// action which will be to be processed in the next iteration of the main event loop.
    fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => self.quit(),
            Action::ScrollDown => self.list.scroll_next(),
            _ => {}
        }
        Ok(())
    }

    // Render the `AppWidget` as a stateful widget using `self` as the `State`
    fn draw(&mut self, term: &mut Term) -> Result<()> {
        term.draw(|frame| {
            frame.render_stateful_widget(AppWidget, frame.area(), self);
            self.update_frame_count(frame);
        })?;
        Ok(())
    }

    // Sets the frame count
    fn update_frame_count(&mut self, frame: &mut Frame<'_>) {
        self.frame_count = frame.count();
    }
}

struct AppWidget;

impl StatefulWidget for AppWidget {
    type State = App;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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

    fn quit(&mut self) {
        self.mode = Mode::Quit
    }
}
