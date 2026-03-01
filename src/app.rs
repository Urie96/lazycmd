use anyhow::{bail, Context, Result};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};

use mlua::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Paragraph},
};
use tokio::sync::mpsc;

use crate::{
    events::{Event, Events},
    plugin,
    term::{self, Term},
    widgets::{header::HeaderWidget, input::InputWidget, list::ListWidget},
    State,
};

pub struct App {
    event_sender: mpsc::UnboundedSender<Event>,
    state: State,
    term: Term,
    quitting: bool,
    dirty: bool,
    lua: Lua,
}

impl App {
    pub fn new(event_sender: mpsc::UnboundedSender<Event>, term: Term) -> Self {
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

    fn call_list(&mut self) -> Result<()> {
        anyhow::Context::context(
            plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                let lc: LuaTable = self.lua.globals().get("lc")?;
                let list_fn: LuaFunction = lc.get("_list")?;
                list_fn.call::<()>(())
            }),
            "Failed to call lc._list",
        )
    }

    fn call_preview(&mut self) -> Result<()> {
        anyhow::Context::context(
            plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                let lc: LuaTable = self.lua.globals().get("lc")?;
                let preview_fn: LuaFunction = lc.get("_preview")?;
                preview_fn.call::<()>(())
            }),
            "Failed to call lc._preview",
        )
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
                // Handle character input in Input mode
                if self.state.current_mode == crate::Mode::Input {
                    if self.handle_input_mode_key(key)? {
                        self.dirty = true;
                    } else {
                        // Try keymap first
                        let cb = { self.state.tap_key(key)? };
                        if let Some(cb) = cb {
                            plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                                cb.call::<()>(())
                            })?;
                        }
                    }
                } else {
                    let cb = { self.state.tap_key(key)? };
                    if let Some(cb) = cb {
                        plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                            cb.call::<()>(())
                        })?;
                    }
                }
            }
            Event::Crossterm(_) => {}
            Event::Command(command) => self.handle_command(command.as_str())?,
            Event::AddKeymap(keymap) => self.state.add_keymap(keymap),
            Event::Enter(path) => {
                let from_cache = self.state.go_to(path);
                if !from_cache {
                    self.call_list()?;
                } else {
                    // Restore preview for cached page
                    self.call_preview()?;
                }
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
                self.call_preview()?;
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
                self.state.clear_current_cache();
                self.call_list()?;
                self.dirty = true;
            }
            "enter_filter_mode" => {
                self.enter_filter_mode();
            }
            "exit_filter_mode" => {
                self.exit_filter_mode(false);
            }
            "accept_filter" => {
                self.exit_filter_mode(true);
            }
            "filter_backspace" => {
                self.handle_filter_backspace();
            }
            "filter_clear" => {
                self.handle_filter_clear();
            }
            "enter" => {
                if let Some(hovered) = self.state.hovered() {
                    let mut path = self.state.current_path.clone();
                    path.push(hovered.key.clone());
                    let from_cache = self.state.go_to(path);
                    if !from_cache {
                        self.call_list()?;
                    } else {
                        // Restore preview for cached page
                        self.call_preview()?;
                    }
                    self.dirty = true;
                }
            }
            "back" => {
                let mut path = self.state.current_path.clone();
                if !path.is_empty() {
                    path.pop();
                    let from_cache = self.state.go_to(path);
                    if !from_cache {
                        self.call_list()?;
                    } else {
                        // Restore preview for cached page
                        self.call_preview()?;
                    }
                    self.dirty = true;
                }
            }
            _ => bail!("Unsupported command {}", command),
        };
        Ok(())
    }

    /// Handle character input in Input mode
    /// Returns true if the key was handled, false if it should be passed to keymap
    fn handle_input_mode_key(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        use crossterm::event::KeyModifiers;

        // Ignore release events
        if key.kind == KeyEventKind::Release {
            return Ok(false);
        }

        // For keys with modifiers (except SHIFT), let keymap handle them
        if key.modifiers.contains(KeyModifiers::CONTROL)
            || key.modifiers.contains(KeyModifiers::ALT)
        {
            return Ok(false);
        }

        match key.code {
            KeyCode::Char(c) => {
                // Insert character at cursor position
                self.state.filter_input.insert(self.state.input_cursor_position, c);
                self.state.input_cursor_position += 1;
                self.apply_filter()?;
                Ok(true)
            }
            KeyCode::Backspace => {
                if self.state.input_cursor_position > 0 {
                    self.state.filter_input.remove(self.state.input_cursor_position - 1);
                    self.state.input_cursor_position -= 1;
                    self.apply_filter()?;
                }
                Ok(true)
            }
            KeyCode::Delete => {
                if self.state.input_cursor_position < self.state.filter_input.len() {
                    self.state.filter_input.remove(self.state.input_cursor_position);
                    self.apply_filter()?;
                }
                Ok(true)
            }
            KeyCode::Left => {
                self.state.input_cursor_position =
                    self.state.input_cursor_position.saturating_sub(1);
                self.dirty = true;
                Ok(true)
            }
            KeyCode::Right => {
                self.state.input_cursor_position = self.state
                    .input_cursor_position
                    .saturating_add(1)
                    .min(self.state.filter_input.len());
                self.dirty = true;
                Ok(true)
            }
            KeyCode::Home => {
                self.state.input_cursor_position = 0;
                self.dirty = true;
                Ok(true)
            }
            KeyCode::End => {
                self.state.input_cursor_position = self.state.filter_input.len();
                self.dirty = true;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn enter_filter_mode(&mut self) {
        self.state.current_mode = crate::Mode::Input;
        // Preserve current filter if already set, otherwise start fresh
        if self.state.filter_input.is_empty() {
            self.state.input_cursor_position = 0;
        } else {
            self.state.input_cursor_position = self.state.filter_input.len();
        }
        self.state.last_key_event_buffer.clear();
        self.dirty = true;
    }

    fn exit_filter_mode(&mut self, keep_filter: bool) {
        if !keep_filter {
            self.state.filter_input.clear();
            self.state.input_cursor_position = 0;
            let _ = self.apply_filter();
        }
        self.state.current_mode = crate::Mode::Main;
        self.state.last_key_event_buffer.clear();
        let _ = self.call_preview();
        self.dirty = true;
    }

    fn handle_filter_backspace(&mut self) {
        if self.state.input_cursor_position > 0 {
            self.state.filter_input.remove(self.state.input_cursor_position - 1);
            self.state.input_cursor_position -= 1;
            let _ = self.apply_filter();
            self.dirty = true;
        }
    }

    fn handle_filter_clear(&mut self) {
        self.state.filter_input.clear();
        self.state.input_cursor_position = 0;
        let _ = self.apply_filter();
        self.dirty = true;
    }

    fn apply_filter(&mut self) -> Result<()> {
        if let Some(page) = &mut self.state.current_page {
            // Sync filter state from State to Page
            page.filter_input = self.state.filter_input.clone();
            page.input_cursor_position = self.state.input_cursor_position;
            page.apply_filter(&self.state.filter_input);
            // Update preview for newly selected item
            self.state.current_preview.take();
            self.call_preview()?;
        }
        Ok(())
    }
}

struct AppWidget;

impl StatefulWidget for AppWidget {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut State) {
        use Constraint::*;

        // Layout depends on mode
        let (header_area, input_area, main_area) = if state.current_mode == crate::Mode::Input {
            let [header_area, input_area, main_area, _footer] =
                Layout::vertical([Length(3), Length(3), Min(3), Length(1)]).areas(area);
            (header_area, Some(input_area), main_area)
        } else {
            let [header_area, main_area, _footer] =
                Layout::vertical([Length(3), Min(3), Length(1)]).areas(area);
            (header_area, None, main_area)
        };

        let [list_area, preview_area] =
            Layout::horizontal([Percentage(50), Fill(1)]).areas(main_area);

        HeaderWidget.render(header_area, buf, state);

        // Render input widget if in filter mode
        if let Some(input_area) = input_area {
            let mut input_state = crate::InputState::from_str(&state.filter_input);
            input_state.cursor_position = state.input_cursor_position;
            InputWidget.render(input_area, buf, &mut input_state);
        }

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
