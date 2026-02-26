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
    dirty: bool,
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
            dirty: false,
            quitting: false,
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
            if self.dirty {
                self.term.draw(|frame| {
                    frame.render_stateful_widget(AppWidget, frame.area(), &mut self.state);
                })?;
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
                if self.state.check_notification_expiry() {
                    self.dirty = true;
                }
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
                self.dirty = true;
            }
            Event::LuaCallback(cb) => {
                plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                    cb(&self.lua)?;
                    Ok(())
                })?;
            }
            Event::InteractiveCommand(cmd, on_complete) => {
                // Execute the interactive command
                let result = self.execute_interactive_command(cmd);

                // Call the completion callback if provided
                if let Some(cb) = on_complete {
                    let exit_code = match result {
                        Ok(code) => code,
                        Err(e) => {
                            // Log the error and use -1 as exit code
                            eprintln!("Error executing interactive command: {}", e);
                            -1
                        }
                    };
                    plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                        cb.call::<()>(exit_code)?;
                        Ok(())
                    })?;
                }
            }
            Event::Notify(message) => {
                self.state.set_notification(message);
                self.dirty = true;
            }
        }
        Ok(())
    }

    fn execute_interactive_command(&mut self, cmd: Vec<String>) -> Result<i32> {
        if cmd.is_empty() {
            bail!("Interactive command cannot be empty");
        }

        let mut it = cmd.iter();
        let program = it.next().unwrap();
        let args: Vec<&String> = it.collect();

        // Temporarily restore the terminal to let the subprocess take control
        term::restore();

        // Execute the command and wait for it to complete
        let result = std::process::Command::new(program)
            .args(&args)
            .status()
            .context(format!("Failed to execute command: {}", program))?;

        // Re-initialize the terminal for TUI
        self.term = term::init()?;

        // Return the exit code
        Ok(result.code().unwrap_or(-1))
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
                self.dirty = true;
            }
            "scroll_preview_by" => {
                let num = match it.next() {
                    Some(num) => num
                        .parse::<i16>()
                        .context("wrong format for scroll_preview_by")?,
                    None => 1,
                };
                self.state.scroll_preview_by(num);
                self.dirty = true;
            }
            "reload" => {
                self.run_event_hooks(EventHook::EnterPost)?;
                self.dirty = true;
            }
            _ => bail!("Unsupported command {}", command),
        };
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

        // Draw notification in bottom-right corner
        if let Some((message, _)) = &state.notification {
            // Fixed size notification box
            let notification_width = 40u16;  // Fixed width
            let notification_height = 3u16;    // Fixed height (1 line + borders)

            // Calculate bottom-right position (fixed offset from bottom-right corner)
            let x = area.width.saturating_sub(notification_width + 2);
            let y = area.height.saturating_sub(notification_height + 1);

            let notification_area = Rect {
                x,
                y,
                width: notification_width.min(area.width),
                height: notification_height.min(area.height),
            };

            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow));
            let inner = block.inner(notification_area);
            block.render(notification_area, buf);
            Paragraph::new(message.as_str())
                .style(Style::default().fg(Color::Yellow))
                .render(inner, buf);
        }
    }
}
