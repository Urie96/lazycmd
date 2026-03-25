use anyhow::{bail, Context, Result};
use crossterm::cursor::{Hide, MoveTo, SetCursorStyle, Show};
use crossterm::event::Event as CrosstermEvent;
use crossterm::execute;

use mlua::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Paragraph},
};
use std::time::Instant;
use tokio::sync::mpsc;

use libc::{sigaction, sigemptyset, SIGINT, SIG_IGN};
use std::mem;

use crate::{
    confirm_handler,
    events::{Event, Events},
    input_handler, plugin, select_handler,
    term::{self, Term},
    widgets::{
        confirm::ConfirmWidget, footer::FooterWidget, header::HeaderWidget,
        input::InputDialogState, input::InputDialogWidget, list::ListWidget, select::SelectWidget,
    },
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
    pub fn new(
        event_sender: mpsc::UnboundedSender<Event>,
        term: Term,
        plugin_name: Option<String>,
    ) -> Self {
        let mut state = State::new();
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

    /// Get cursor information (position and style) based on current mode
    fn get_cursor_info(&self) -> Option<(u16, u16, SetCursorStyle)> {
        // Check if input dialog is open (takes priority over select dialog and filter mode)
        if let Some(dialog) = &self.state.input_dialog {
            return Some((dialog.cursor_x, dialog.cursor_y, SetCursorStyle::SteadyBar));
        }

        // Check if select dialog is open (takes priority)
        if let Some(dialog) = &self.state.select_dialog {
            return Some((dialog.cursor_x, dialog.cursor_y, SetCursorStyle::SteadyBar));
        }

        // No cursor in main mode
        None
    }

    /// Set cursor after rendering (called by scopeguard)
    fn routine<B: std::io::Write>(
        backend: &mut B,
        cursor_info: Option<(u16, u16, SetCursorStyle)>,
    ) {
        if let Some((x, y, style)) = cursor_info {
            let _ = execute!(backend, style, MoveTo(x, y), Show);
        } else {
            let _ = execute!(backend, Hide);
        }
        let _ = backend.flush();
    }

    /// Runs the main loop of the application, handling events and actions
    pub async fn run(&mut self, mut events: Events) -> Result<()> {
        self.event_sender.send(Event::Enter(Vec::new())).unwrap();

        // Initially hide cursor (Main mode)
        execute!(self.term.backend_mut(), Hide)?;
        std::io::Write::flush(self.term.backend_mut())?;

        loop {
            if let Some(e) = events.next().await {
                self.handle_event(e)?;
            }
            if self.quitting {
                break;
            }

            if self.dirty {
                // Hide cursor during rendering
                execute!(self.term.backend_mut(), Hide)?;

                // Render
                self.term.draw(|frame| {
                    frame.render_stateful_widget(AppWidget, frame.area(), &mut self.state);
                })?;

                // Get cursor info after rendering (uses updated filter_cursor_x/y)
                let cursor_info = self.get_cursor_info();

                // Restore cursor state after draw (like yazi)
                Self::routine(self.term.backend_mut(), cursor_info);

                self.dirty = false;
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
                self.dirty = true;
            }
            Event::RefreshPreview => {
                self.state.current_preview.take();
                self.call_preview()?;
                self.dirty = true;
            }
            Event::Crossterm(CrosstermEvent::Resize(_, _)) => {
                self.dirty = true;
            }
            Event::Crossterm(CrosstermEvent::Key(key)) => {
                // If confirm dialog is shown, handle its keyboard input first
                if self.state.confirm_dialog.is_some() {
                    if confirm_handler::handle_confirm_dialog_key(
                        &self.lua,
                        &mut self.state,
                        &self.event_sender,
                        key,
                    )? {
                        self.dirty = true;
                    }
                    return Ok(());
                }

                // If select dialog is shown, handle its keyboard input first
                if self.state.select_dialog.is_some() {
                    if select_handler::handle_select_dialog_key(
                        &self.lua,
                        &mut self.state,
                        &self.event_sender,
                        key,
                    )? {
                        self.dirty = true;
                    }
                    return Ok(());
                }

                // If input dialog is shown, handle its keyboard input
                if self.state.input_dialog.is_some() {
                    if input_handler::handle_input_dialog_key(
                        &self.lua,
                        &mut self.state,
                        &self.event_sender,
                        key,
                    )? {
                        self.dirty = true;
                    }
                    return Ok(());
                }

                // Handle key events in main mode
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
            Event::InteractiveCommand {
                cmd,
                on_complete,
                wait_confirm,
            } => {
                // Execute the interactive command
                let result = self.execute_interactive_command(cmd, wait_confirm);

                self.dirty = true;

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
            Event::ShowSelect {
                prompt,
                options,
                on_selection,
            } => {
                self.state.select_dialog =
                    Some(crate::SelectDialog::new(prompt, options, on_selection));
                self.dirty = true;
            }
            Event::ShowInput {
                prompt,
                placeholder,
                on_submit,
                on_cancel,
                on_change,
            } => {
                self.state.input_dialog = Some(crate::InputDialog::new(
                    prompt,
                    placeholder,
                    on_submit,
                    on_cancel,
                    on_change,
                ));
                self.dirty = true;
            }
        }
        Ok(())
    }

    fn execute_interactive_command(
        &mut self,
        cmd: Vec<String>,
        wait_confirm: Option<LuaFunction>,
    ) -> Result<i32> {
        if cmd.is_empty() {
            bail!("Interactive command cannot be empty");
        }

        let mut it = cmd.iter();
        let program = it.next().unwrap();
        let args: Vec<&String> = it.collect();

        // Temporarily ignore SIGINT during interactive command execution
        // This prevents Ctrl-C from terminating lazycmd itself
        let mut old_action: libc::sigaction = unsafe { mem::zeroed() };
        let mut new_action: libc::sigaction = unsafe { mem::zeroed() };

        unsafe {
            // Get the current SIGINT handler
            sigaction(SIGINT, std::ptr::null(), &mut old_action);

            // Set SIGINT to ignore (SIG_IGN)
            new_action.sa_sigaction = SIG_IGN;
            sigemptyset(&mut new_action.sa_mask);
            new_action.sa_flags = 0;
            sigaction(SIGINT, &new_action, std::ptr::null_mut());
        }

        // Temporarily restore the terminal to let the subprocess take control
        term::restore();

        // Execute the command and wait for it to complete
        let result = std::process::Command::new(program)
            .args(&args)
            .status()
            .context(format!("Failed to execute command: {}", program))?;

        let exit_code = result.code().unwrap_or(-1);

        // Restore the original SIGINT handler
        unsafe {
            sigaction(SIGINT, &old_action, std::ptr::null_mut());
        }

        // If wait_confirm function is provided, call it to decide whether to wait
        let should_wait = if let Some(ref wait_fn) = wait_confirm {
            plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                let result: bool = wait_fn.call::<bool>(exit_code)?;
                Ok(result)
            })
            .unwrap_or(false)
        } else {
            false
        };

        if should_wait {
            println!("\nPress Enter to return to lazycmd...");
            let _ = std::io::stdin().read_line(&mut String::new());
        }

        // Re-initialize the terminal for TUI
        self.term = term::init()?;

        // Clear any pending input events to prevent spurious key presses
        // This handles the case where the subprocess (e.g., vim) leaves
        // input in the terminal buffer that would otherwise be captured
        while crossterm::event::poll(std::time::Duration::from_millis(10)).unwrap_or(false) {
            let _ = crossterm::event::read();
        }

        // Return the exit code
        Ok(exit_code)
    }

    fn handle_command(&mut self, command: &str) -> Result<()> {
        let splits = shell_words::split(command)?;
        if splits.is_empty() {
            bail!("Empty command {}", command)
        }
        let mut it = splits.iter();
        match it.next().unwrap().as_str() {
            "quit" => {
                for hook in self.state.pre_quit_hooks.clone() {
                    plugin::scope(&self.lua, &mut self.state, &self.event_sender, || {
                        hook.call::<()>(())
                    })?;
                }
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
                // Save the selected entry key to restore later
                let selected_key = self.state.hovered().map(|e| e.key.clone());
                // Clear current page entries (but don't clear list_state selection)
                if let Some(page) = &mut self.state.current_page {
                    page.list.clear();
                    page.filtered_list.clear();
                }
                self.state.clear_current_cache();
                self.call_list()?;
                // Restore selection by finding the entry with the same key
                if let Some(key) = selected_key {
                    if let Some(page) = &mut self.state.current_page {
                        // Find the index of the entry with the same key
                        if let Some(idx) = page.filtered_list.iter().position(|e| e.key == key) {
                            page.list_state.select(Some(idx));
                        } else if !page.filtered_list.is_empty() {
                            // Entry not found, keep the current selection or select the first item
                            if page.list_state.selected().is_none() {
                                page.list_state.select(Some(0));
                            }
                        }
                    }
                }
                self.dirty = true;
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
}

struct AppWidget;

impl StatefulWidget for AppWidget {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut State) {
        use Constraint::*;

        // Layout: header (1), main (remaining), footer (1)
        let [header_area, main_area, footer_area] =
            Layout::vertical([Length(1), Min(3), Length(1)]).areas(area);

        HeaderWidget.render(header_area, buf, state);

        // Render footer with list counter
        FooterWidget.render(footer_area, buf, state);

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

        let scrolloff = state.scrolloff;

        if let Some(page) = &mut state.current_page {
            let list_widget = ListWidget { scrolloff };
            list_widget.render(list_area, buf, page);
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

        // Check and clear expired notification
        if let Some((_, expiry)) = &state.notification {
            if Instant::now() > *expiry {
                state.notification = None;
            }
        }

        // Draw notification in bottom-right corner
        if let Some((message, _)) = &state.notification {
            // Dynamic size notification box
            let min_width = 20u16;
            let min_height = 1u16;

            // Calculate required dimensions based on message content
            let line_count = message.lines.len().max(min_height as usize);
            let max_line_width = message
                .lines
                .iter()
                .map(|l| l.width() as u16)
                .max()
                .unwrap_or(0);

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
            Paragraph::new(message.clone())
                .style(Style::default().fg(Color::Yellow))
                .render(inner, buf);
        }

        // Render confirm dialog (render last to appear on top of everything)
        if let Some(dialog) = &mut state.confirm_dialog {
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

            ConfirmWidget.render(dialog_area, buf, dialog);
        }

        // Render select dialog (render last to appear on top of everything)
        if let Some(dialog) = &mut state.select_dialog {
            // Calculate dialog dimensions: fixed size or clamped to fit
            let dialog_width = 80.min(area.width).max(40);
            let dialog_height = 20.min(area.height).max(10);

            // Center the dialog
            let x = (area.width.saturating_sub(dialog_width)) / 2;
            let y = (area.height.saturating_sub(dialog_height)) / 2;

            let dialog_area = Rect {
                x,
                y,
                width: dialog_width,
                height: dialog_height,
            };

            SelectWidget.render(dialog_area, buf, dialog);
        }

        // Render input dialog (render last to appear on top of everything)
        if let Some(dialog) = &mut state.input_dialog {
            // Fixed size: width 50, height 3
            let dialog_width = 50u16;
            let dialog_height = 3u16;

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

            let mut input_state = InputDialogState::new(&dialog.prompt, &dialog.placeholder);
            input_state.text = dialog.text.clone();
            input_state.cursor_position = dialog.cursor_position;
            InputDialogWidget::new().render(dialog_area, buf, &mut input_state);

            // Store cursor position for use in app.rs after draw completes
            dialog.cursor_x = input_state.cursor_x;
            dialog.cursor_y = input_state.cursor_y;
        }
    }
}
