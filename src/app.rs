use anyhow::{bail, Context, Result};
use crossterm::event::Event as CrosstermEvent;
use std::collections::HashMap;

use mlua::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Paragraph},
};
use tokio::sync::mpsc;

use crate::{
    events::{Event, EventHook, Events},
    plugin,
    term::{self, Term},
    widgets::{header::HeaderWidget, list::ListWidget},
    State,
};

pub struct App {
    event_sender: mpsc::UnboundedSender<Event>,
    state: State,
    term: Term,
    quitting: bool,
    event_hooks: HashMap<EventHook, Vec<LuaFunction>>,
    lua: Lua,
}

impl App {
    pub fn new(event_sender: mpsc::UnboundedSender<Event>) -> Self {
        let term = term::init().unwrap();
        let mut state = State::default();
        let lua = Lua::new();

        plugin::scope(&lua, &mut state, &event_sender, || plugin::init_lua(&lua)).unwrap();

        Self {
            lua,
            event_sender,
            state,
            term,
            quitting: Default::default(),
            event_hooks: Default::default(),
        }
    }

    /// Runs the main loop of the application, handling events and actions
    pub async fn run(&mut self, mut events: Events) -> Result<()> {
        self.event_sender.send(Event::Enter(Vec::new())).unwrap();
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

    fn run_event_hooks(&mut self, e: EventHook) -> Result<()> {
        if let Some(hooks) = self.event_hooks.get(&e) {
            plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                for hook in hooks {
                    hook.call::<()>(())?;
                }
                Ok(())
            })?;
        }
        Ok(())
    }

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
                let cb = { self.state.tap_key(key)? };
                if let Some(cb) = cb {
                    plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                        cb.call::<()>(())
                    })?;
                }
            }
            Event::Crossterm(_) => {}
            Event::Command(command) => self.handle_command(command.as_str())?,
            Event::AddKeymap(keymap) => self.state.add_keymap(keymap),
            Event::AddEventHook(name, cb) => self.event_hooks.entry(name).or_default().push(cb),
            Event::Enter(path) => {
                self.state.go_to(path);
                self.run_event_hooks(EventHook::EnterPost)?;
                self.event_sender.send(Event::Render).unwrap();
            }
            Event::LuaCallback(cb) => {
                plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                    cb(&self.lua)?;
                    Ok(())
                })?;
            }
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
            "scroll_by" => {
                let num = match it.next() {
                    Some(num) => num.parse::<i16>().context("wrong format for scroll_by")?,
                    None => 1,
                };
                self.state.scroll_by(num);
                self.state.current_preview.take();
                self.run_event_hooks(EventHook::HoverPost)?;
                self.event_sender.send(Event::Render).unwrap();
            }
            "scroll_preview_by" => {
                let num = match it.next() {
                    Some(num) => num
                        .parse::<i16>()
                        .context("wrong format for scroll_preview_by")?,
                    None => 1,
                };
                self.state.scroll_preview_by(num);
                self.event_sender.send(Event::Render).unwrap();
            }
            _ => bail!("Unsupported command {}", command),
        };
        Ok(())
    }

    // Render the `AppWidget` as a stateful widget using `self` as the `State`
    fn draw(&mut self) -> Result<()> {
        self.term.draw(|frame| {
            frame.render_stateful_widget(AppWidget, frame.area(), &mut self.state);
        })?;
        Ok(())
    }
}

struct AppWidget;

impl StatefulWidget for AppWidget {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut State) {
        use Constraint::*;
        let [header_area, main_area, _footer] =
            Layout::vertical([Length(3), Min(3), Length(1)]).areas(area);
        let [list_area, preview_area] =
            Layout::horizontal([Percentage(50), Fill(1)]).areas(main_area);

        HeaderWidget.render(header_area, buf);
        if let Some(page) = &mut state.current_page {
            ListWidget.render(list_area, buf, page);
        } else {
            Paragraph::new("loading...").render(list_area, buf);
        }

        {
            let preview_block = Block::bordered().border_type(BorderType::Rounded);
            let preview_inner = preview_block.inner(preview_area);
            preview_block.render(preview_area, buf);

            if let Some(p) = state.current_preview.as_mut() {
                p.render(preview_inner, buf);
            }
        }
    }
}
