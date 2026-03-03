use anyhow::{bail, Context, Result};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEventKind};

use mlua::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Clear, Paragraph},
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
    pub fn new(event_sender: mpsc::UnboundedSender<Event>, term: Term, plugin_name: Option<String>) -> Self {
        let mut state = State::default();
        if let Some(name) = plugin_name {
            state.current_plugin = name;
        }
        let lua = Lua::new();

        plugin::scope(&lua, &mut state, &event_sender, || plugin::init_lua(&lua))
            .expect("Failed to initialize Lua");

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
                // If confirm dialog is shown, handle its keyboard input first
                if self.state.confirm_dialog.is_some() {
                    if self.handle_confirm_dialog_key(key)? {
                        self.dirty = true;
                    }
                    return Ok(());
                }

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
            Event::ShowConfirm {
                title,
                prompt,
                on_confirm,
                on_cancel,
            } => {
                self.state
                    .show_confirm_dialog(title, prompt, on_confirm, on_cancel);
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

        // Clear any pending input events to prevent spurious key presses
        // This handles the case where the subprocess (e.g., vim) leaves
        // input in the terminal buffer that would otherwise be captured
        while crossterm::event::poll(std::time::Duration::from_millis(10)).unwrap_or(false) {
            let _ = crossterm::event::read();
        }

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
                // Call all pre_reload hooks before executing reload
                for hook in self.state.pre_reload_hooks.clone() {
                    plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                        hook.call::<()>(())
                    })?;
                }
                // Clear current page entries
                if let Some(page) = &mut self.state.current_page {
                    page.list.clear();
                    page.filtered_list.clear();
                    page.list_state.select(None);
                }
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
                self.state
                    .filter_input
                    .insert(self.state.input_cursor_position, c);
                self.state.input_cursor_position += 1;
                self.apply_filter()?;
                Ok(true)
            }
            KeyCode::Backspace => {
                if self.state.input_cursor_position > 0 {
                    self.state
                        .filter_input
                        .remove(self.state.input_cursor_position - 1);
                    self.state.input_cursor_position -= 1;
                    self.apply_filter()?;
                }
                Ok(true)
            }
            KeyCode::Delete => {
                if self.state.input_cursor_position < self.state.filter_input.len() {
                    self.state
                        .filter_input
                        .remove(self.state.input_cursor_position);
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
                self.state.input_cursor_position = self
                    .state
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
            self.state
                .filter_input
                .remove(self.state.input_cursor_position - 1);
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

    /// Handle keyboard input for confirm dialog
    /// Returns true if the key was handled, false otherwise
    fn handle_confirm_dialog_key(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        use crossterm::event::{KeyCode, KeyEventKind};

        // Ignore release events
        if key.kind == KeyEventKind::Release {
            return Ok(false);
        }

        match key.code {
            // Left arrow: select Yes
            KeyCode::Left => {
                self.state.confirm_dialog.as_mut().map(|d| {
                    d.selected_button = crate::ConfirmButton::Yes;
                });
                self.dirty = true;
                Ok(true)
            }
            // Right arrow: select No
            KeyCode::Right => {
                self.state.confirm_dialog.as_mut().map(|d| {
                    d.selected_button = crate::ConfirmButton::No;
                });
                self.dirty = true;
                Ok(true)
            }
            // Enter: execute selected button's callback
            KeyCode::Enter => {
                if let Some(dialog) = self.state.confirm_dialog.take() {
                    self.dirty = true;
                    match dialog.selected_button {
                        crate::ConfirmButton::Yes => {
                            plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                                dialog.on_confirm.call::<()>(())
                            })?;
                        }
                        crate::ConfirmButton::No => {
                            plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                                dialog.on_cancel.call::<()>(())
                            })?;
                        }
                    }
                }
                Ok(true)
            }
            // Y key: confirm (execute on_confirm)
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(dialog) = self.state.confirm_dialog.take() {
                    self.dirty = true;
                    plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                        dialog.on_confirm.call::<()>(())
                    })?;
                }
                Ok(true)
            }
            // N key: cancel (execute on_cancel)
            KeyCode::Char('n') | KeyCode::Char('N') => {
                if let Some(dialog) = self.state.confirm_dialog.take() {
                    self.dirty = true;
                    plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                        dialog.on_cancel.call::<()>(())
                    })?;
                }
                Ok(true)
            }
            // Esc: cancel (execute on_cancel)
            KeyCode::Esc => {
                if let Some(dialog) = self.state.confirm_dialog.take() {
                    self.dirty = true;
                    plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                        dialog.on_cancel.call::<()>(())
                    })?;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
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

        // Layout: header (1), main (remaining), footer (1)
        let [header_area, main_area, _footer] =
            Layout::vertical([Length(1), Min(3), Length(1)]).areas(area);

        HeaderWidget.render(header_area, buf, state);

        let block_color = Color::DarkGray;

        // Draw outer border and split into list/preview areas
        let main_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(block_color));
        let main_inner = main_block.inner(main_area);
        main_block.render(main_area, buf);

        // Split into list, divider(1), preview areas
        let [list_area, _divider_area, preview_area] =
            Layout::horizontal([Percentage(50), Length(1), Fill(1)]).areas(main_inner);

        if let Some(page) = &mut state.current_page {
            ListWidget.render(list_area, buf, page);
        } else {
            Paragraph::new("loading...").render(list_area, buf);
        }

        // Draw vertical divider line from top to bottom of the outer border
        for y in main_area.top()..main_area.bottom() {
            buf[(_divider_area.left(), y)]
                .set_symbol(symbols::line::VERTICAL)
                .set_style(Style::default().fg(block_color));
        }

        // Connect divider to top border - replace corner with ┬
        buf[(_divider_area.left(), main_area.top())]
            .set_symbol("┬")
            .set_style(Style::default().fg(block_color));

        // Connect divider to bottom border - replace corner with ┴
        buf[(_divider_area.left(), main_area.bottom() - 1)]
            .set_symbol("┴")
            .set_style(Style::default().fg(block_color));

        if let Some(p) = state.current_preview.as_mut() {
            p.render(preview_area, buf);
        }

        // Render floating input widget if in filter mode (render last to appear on top)
        if state.current_mode == crate::Mode::Input {
            // Fixed width of 50, height of 3 (top border + content + bottom border)
            let input_width = 50u16;
            let input_height = 3u16;

            // Horizontally centered: x = (area_width - input_width) / 2
            let x = (area.width.saturating_sub(input_width)) / 2;

            // Vertically at row 5 (0-indexed)
            let y = 5u16;

            // Ensure input box is within bounds
            let x = x.min(area.width.saturating_sub(input_width));
            let y = y.min(area.height.saturating_sub(input_height));

            let input_area = Rect {
                x,
                y,
                width: input_width,
                height: input_height,
            };

            // Clear the area first to prevent underlying content from showing through
            Clear.render(input_area, buf);

            let mut input_state = crate::InputState::from_str(&state.filter_input);
            input_state.cursor_position = state.input_cursor_position;
            InputWidget.render(input_area, buf, &mut input_state);
        }

        // Draw notification in bottom-right corner
        if let Some((message, _)) = &state.notification {
            // Dynamic size notification box
            let min_width = 20u16;
            let min_height = 1u16;

            // Calculate required dimensions based on message content
            let lines: Vec<&str> = message.lines().collect();
            let line_count = lines.len().max(min_height as usize);
            let max_line_width = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;

            // Width: max of min_width and (max_line_width + 2 for padding)
            let notification_width = (max_line_width + 2).max(min_width);

            // Height: min_height + 2 for top/bottom borders
            let notification_height = (line_count as u16).max(min_height) + 2;

            // Calculate bottom-right position
            let x = area.width.saturating_sub(notification_width + 1);
            let y = area.height.saturating_sub(notification_height + 1);

            let notification_area = Rect {
                x: x.saturating_sub(1), // Extra padding from right edge
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

        // Render confirm dialog (render last to appear on top of everything)
        if let Some(dialog) = &state.confirm_dialog {
            // Fixed size: width 40, height 10
            let dialog_width = 40u16;
            let dialog_height = 10u16;

            // Center the dialog
            let x = (area.width.saturating_sub(dialog_width)) / 2;
            let y = (area.height.saturating_sub(dialog_height)) / 2;

            // Ensure dialog is within bounds
            let x = x.min(area.width.saturating_sub(dialog_width));
            let y = y.min(area.height.saturating_sub(dialog_height));

            let dialog_area = Rect {
                x,
                y,
                width: dialog_width,
                height: dialog_height,
            };

            // Clear the area first to prevent underlying content from showing through
            Clear.render(dialog_area, buf);

            // Draw dialog border with cyan color
            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue));

            // Add title (always present, defaults to "Confirm") - title is centered by default
            if let Some(title) = &dialog.title {
                let block = block
                    .title(title.as_str())
                    .title_alignment(ratatui::layout::Alignment::Center)
                    .title_style(Style::default().fg(Color::Blue));
                let inner = block.inner(dialog_area);
                block.render(dialog_area, buf);

                // Split into: prompt area, then buttons with divider
                // Buttons area takes 3 rows (divider + 2 rows for buttons)
                let buttons_area_height = 3u16;
                let prompt_area_height = inner.height.saturating_sub(buttons_area_height);

                let [prompt_area, buttons_area] =
                    Layout::vertical([Length(prompt_area_height), Length(buttons_area_height)])
                        .areas(inner);

                // Render prompt text with white color, centered and wrapped
                let prompt_paragraph = Paragraph::new(dialog.prompt.as_str())
                    .wrap(ratatui::widgets::Wrap { trim: true })
                    .alignment(ratatui::layout::Alignment::Center)
                    .style(Style::default().fg(Color::White));
                prompt_paragraph.render(prompt_area, buf);

                // Draw divider line at the top of buttons area (button row - 1)
                let divider_y = buttons_area.bottom().saturating_sub(2);
                for x in buttons_area.left()..buttons_area.right() {
                    buf[(x, divider_y)]
                        .set_symbol(symbols::line::HORIZONTAL)
                        .set_style(Style::default().fg(Color::Blue));
                }

                // Split buttons area into left half and right half
                let [left_half, right_half] = Layout::horizontal([Percentage(50), Percentage(50)])
                    .areas(buttons_area);

                // Render buttons
                let yes_selected = dialog.selected_button == crate::ConfirmButton::Yes;
                let no_selected = dialog.selected_button == crate::ConfirmButton::No;

                // Button text
                let yes_text = "[Y]es";
                let no_text = "(N)o";

                // Button width is 9
                let button_width = 9u16;

                // Button style for selected: white background, black text, bold
                let selected_style = Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD);

                // Button style for unselected: white text, no background
                let unselected_style = Style::default().fg(Color::White);

                let yes_style = if yes_selected {
                    selected_style
                } else {
                    unselected_style
                };

                let no_style = if no_selected {
                    selected_style
                } else {
                    unselected_style
                };

                // Center buttons in their respective halves
                let yes_x = left_half
                    .x
                    .saturating_add(left_half.width.saturating_sub(button_width) / 2);
                let no_x = right_half
                    .x
                    .saturating_add(right_half.width.saturating_sub(button_width) / 2);
                let button_y = buttons_area.bottom().saturating_sub(1);

                let yes_area = Rect {
                    x: yes_x,
                    y: button_y,
                    width: button_width,
                    height: 1,
                };
                let no_area = Rect {
                    x: no_x,
                    y: button_y,
                    width: button_width,
                    height: 1,
                };

                // Render Yes button
                Paragraph::new(yes_text)
                    .style(yes_style)
                    .alignment(ratatui::layout::Alignment::Center)
                    .render(yes_area, buf);

                // Render No button
                Paragraph::new(no_text)
                    .style(no_style)
                    .alignment(ratatui::layout::Alignment::Center)
                    .render(no_area, buf);
            }
        }
    }
}
