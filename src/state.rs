use crossterm::event::KeyEvent;
use mlua::prelude::*;
use ratatui::{text::Line, text::Text, widgets::ListState};
use std::collections::HashMap;
use std::time::Instant;

use crate::{widgets::Renderable, Keymap, Mode, Page, PageEntry};

/// Represents which button is selected in the confirm dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmButton {
    Yes,
    No,
}

/// Option for select dialog
#[derive(Debug, Clone)]
pub struct SelectOption {
    /// The value to return when this option is selected
    pub value: LuaValue,
    /// The text to display for this option
    pub display: Line<'static>,
}

/// State for the select dialog
#[derive(Debug)]
pub struct SelectDialog {
    /// Optional prompt/title text
    pub prompt: Option<String>,
    /// All available options (unfiltered)
    pub options: Vec<SelectOption>,
    /// Filtered options (subset of options that match the filter)
    pub filtered_options: Vec<usize>,
    /// Index of currently selected option in filtered_options
    pub selected_index: Option<usize>,
    /// Current filter input text
    pub filter_input: String,
    /// Cursor position in the filter input
    pub cursor_position: usize,
    /// Cursor x position (for terminal cursor display)
    pub cursor_x: u16,
    /// Cursor y position (for terminal cursor display)
    pub cursor_y: u16,
    /// List state for rendering and scrolling
    pub list_state: ListState,
    /// Callback function when selection is made (or canceled with nil)
    pub on_selection: LuaFunction,
}

impl SelectDialog {
    pub fn new(
        prompt: Option<String>,
        options: Vec<SelectOption>,
        on_selection: LuaFunction,
    ) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut dialog = Self {
            prompt,
            options,
            filtered_options: Vec::new(),
            selected_index: Some(0),
            filter_input: String::new(),
            cursor_position: 0,
            cursor_x: 0,
            cursor_y: 0,
            list_state,
            on_selection,
        };

        // Initialize filtered options
        dialog.update_filtered_options();

        dialog
    }

    /// Get the current filtered options
    pub fn get_filtered_options(&self) -> Vec<SelectOption> {
        self.filtered_options
            .iter()
            .filter_map(|&idx| self.options.get(idx).cloned())
            .collect()
    }

    /// Update filtered options based on current filter input
    pub fn update_filtered_options(&mut self) {
        if self.filter_input.is_empty() {
            // No filter: show all options
            self.filtered_options = (0..self.options.len()).collect();
        } else {
            // Filter options by display text (case-insensitive)
            let filter_lower = self.filter_input.to_lowercase();
            self.filtered_options = self
                .options
                .iter()
                .enumerate()
                .filter(|(_, opt)| {
                    opt.display
                        .to_string()
                        .to_lowercase()
                        .contains(&filter_lower)
                })
                .map(|(idx, _)| idx)
                .collect();
        }

        // Adjust selected index
        if self.filtered_options.is_empty() {
            self.selected_index = None;
            self.list_state.select(None);
        } else {
            // Keep current selection if valid, otherwise select first
            let new_idx = self.selected_index.and_then(|idx| {
                if idx < self.filtered_options.len() {
                    Some(idx)
                } else {
                    None
                }
            });

            self.selected_index = new_idx.or(Some(0));
            self.list_state.select(self.selected_index);
        }
    }

    /// Move selection by delta
    pub fn move_selection(&mut self, delta: i32) {
        let filtered_count = self.filtered_options.len();
        if filtered_count == 0 {
            return;
        }

        let current = self.selected_index.unwrap_or(0);
        let new = if delta > 0 {
            // Moving down: wrap around if at end
            let target = current + delta as usize;
            if target >= filtered_count && current == filtered_count - 1 {
                0 // Wrap to top
            } else {
                target.min(filtered_count - 1)
            }
        } else {
            // Moving up: wrap around if at top
            let abs_delta = delta.unsigned_abs() as usize;
            if abs_delta > current && current == 0 {
                filtered_count - 1 // Wrap to bottom
            } else {
                current.saturating_sub(abs_delta)
            }
        };

        self.selected_index = Some(new);
        self.list_state.select(Some(new));
    }

    /// Select first option
    pub fn select_first(&mut self) {
        if !self.filtered_options.is_empty() {
            self.selected_index = Some(0);
            self.list_state.select(Some(0));
        }
    }

    /// Select last option
    pub fn select_last(&mut self) {
        let count = self.filtered_options.len();
        if count > 0 {
            self.selected_index = Some(count - 1);
            self.list_state.select(Some(count - 1));
        }
    }

    /// Move cursor to start of input
    pub fn cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end of input
    pub fn cursor_to_end(&mut self) {
        self.cursor_position = self.filter_input.len();
    }

    /// Move cursor left by one character (not byte)
    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            // Find the previous character boundary
            let prev_pos = self.filter_input[..self.cursor_position]
                .char_indices()
                .rev()
                .nth(0)
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            self.cursor_position = prev_pos;
        }
    }

    /// Move cursor right by one character (not byte)
    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.filter_input.len() {
            // Find the next character boundary (skip the character at current position)
            let next_pos = self.filter_input[self.cursor_position..]
                .char_indices()
                .nth(1)
                .map(|(idx, _)| self.cursor_position + idx)
                .unwrap_or(self.filter_input.len());
            self.cursor_position = next_pos;
        }
    }

    /// Insert character at cursor position
    pub fn insert_char(&mut self, c: char) {
        self.filter_input.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();
    }

    /// Delete character before cursor (backspace)
    pub fn delete_before_cursor(&mut self) {
        if self.cursor_position > 0 {
            // Find the start of the character before cursor
            let char_start = self.filter_input[..self.cursor_position]
                .char_indices()
                .rev()
                .nth(0)
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            self.filter_input.remove(char_start);
            self.cursor_position = char_start;
        }
    }

    /// Delete character at cursor (delete)
    pub fn delete_at_cursor(&mut self) {
        if self.cursor_position < self.filter_input.len() {
            self.filter_input.remove(self.cursor_position);
        }
    }

    /// Delete all characters before cursor (ctrl-u)
    pub fn delete_before_cursor_all(&mut self) {
        if self.cursor_position > 0 {
            self.filter_input = self.filter_input[self.cursor_position..].to_string();
            self.cursor_position = 0;
            self.update_filtered_options();
        }
    }
}

impl ConfirmButton {
    pub fn toggle(&self) -> Self {
        match self {
            ConfirmButton::Yes => ConfirmButton::No,
            ConfirmButton::No => ConfirmButton::Yes,
        }
    }
}

/// State for the input dialog
#[derive(Debug)]
pub struct InputDialog {
    pub prompt: String,
    pub placeholder: String,
    pub text: String,
    pub cursor_position: usize,
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub on_submit: LuaFunction,
    pub on_cancel: LuaFunction,
    pub on_change: LuaFunction,
}

impl InputDialog {
    fn prev_char_boundary(text: &str, cursor_position: usize) -> usize {
        text[..cursor_position]
            .char_indices()
            .next_back()
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    fn next_char_boundary(text: &str, cursor_position: usize) -> usize {
        text[cursor_position..]
            .char_indices()
            .nth(1)
            .map(|(idx, _)| cursor_position + idx)
            .unwrap_or(text.len())
    }

    pub fn new(
        prompt: String,
        placeholder: String,
        value: String,
        on_submit: LuaFunction,
        on_cancel: LuaFunction,
        on_change: LuaFunction,
    ) -> Self {
        let cursor_position = value.len();
        Self {
            prompt,
            placeholder,
            text: value,
            cursor_position,
            cursor_x: 0,
            cursor_y: 0,
            on_submit,
            on_cancel,
            on_change,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            let prev_pos = Self::prev_char_boundary(&self.text, self.cursor_position);
            self.text.remove(prev_pos);
            self.cursor_position = prev_pos;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor_position < self.text.len() {
            self.text.remove(self.cursor_position);
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
    }

    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            let prev_pos = Self::prev_char_boundary(&self.text, self.cursor_position);
            self.cursor_position = prev_pos;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.text.len() {
            let next_pos = Self::next_char_boundary(&self.text, self.cursor_position);
            self.cursor_position = next_pos;
        }
    }

    pub fn cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn cursor_to_end(&mut self) {
        self.cursor_position = self.text.len();
    }
}

#[cfg(test)]
mod tests {
    use super::InputDialog;
    use mlua::Lua;

    fn make_dialog() -> InputDialog {
        let lua = Lua::new();
        let on_submit = lua.create_function(|_, ()| Ok(())).unwrap().to_owned();
        let on_cancel = lua.create_function(|_, ()| Ok(())).unwrap().to_owned();
        let on_change = lua.create_function(|_, ()| Ok(())).unwrap().to_owned();

        InputDialog::new(
            "Search".to_string(),
            "keyword".to_string(),
            String::new(),
            on_submit,
            on_cancel,
            on_change,
        )
    }

    #[test]
    fn input_dialog_backspace_handles_utf8() {
        let mut dialog = make_dialog();
        dialog.insert_char('搜');
        dialog.insert_char('索');

        dialog.backspace();
        assert_eq!(dialog.text, "搜");
        assert_eq!(dialog.cursor_position, '搜'.len_utf8());

        dialog.backspace();
        assert_eq!(dialog.text, "");
        assert_eq!(dialog.cursor_position, 0);
    }

    #[test]
    fn input_dialog_initial_value_places_cursor_at_end() {
        let lua = Lua::new();
        let on_submit = lua.create_function(|_, ()| Ok(())).unwrap().to_owned();
        let on_cancel = lua.create_function(|_, ()| Ok(())).unwrap().to_owned();
        let on_change = lua.create_function(|_, ()| Ok(())).unwrap().to_owned();

        let dialog = InputDialog::new(
            "Search".to_string(),
            "keyword".to_string(),
            "abc".to_string(),
            on_submit,
            on_cancel,
            on_change,
        );

        assert_eq!(dialog.text, "abc");
        assert_eq!(dialog.cursor_position, 3);
    }
}

/// State for the confirm dialog
#[derive(Debug)]
pub struct ConfirmDialog {
    pub title: Option<String>,
    pub prompt: String,
    pub on_confirm: LuaFunction,
    pub on_cancel: Option<LuaFunction>,
    pub selected_button: ConfirmButton,
}

impl ConfirmDialog {
    pub fn new(
        title: Option<String>,
        prompt: String,
        on_confirm: LuaFunction,
        on_cancel: Option<LuaFunction>,
    ) -> Self {
        Self {
            title,
            prompt,
            on_confirm,
            on_cancel,
            selected_button: ConfirmButton::Yes, // Default to Yes
        }
    }
}

#[derive(Default)]
pub struct State {
    pub current_mode: Mode,
    pub current_path: Vec<String>,
    pub current_page: Option<Page>,
    pub keymap_config: Vec<Keymap>,
    pub last_key_event_buffer: Vec<KeyEvent>,
    pub current_preview: Option<Box<dyn Renderable>>,
    pub notification: Option<(Text<'static>, Instant)>,

    /// Cache for pages to preserve cursor position, entries and filter when navigating back
    page_cache: HashMap<Vec<String>, Page>,
    /// Hooks to call before reload command
    pub pre_reload_hooks: Vec<LuaFunction>,
    /// Hooks to call before quit command
    pub pre_quit_hooks: Vec<LuaFunction>,
    /// Current plugin name
    pub current_plugin: String,
    /// Confirm dialog state (shown on top of all UI)
    pub confirm_dialog: Option<ConfirmDialog>,
    /// Select dialog state (shown on top of all UI)
    pub select_dialog: Option<SelectDialog>,
    /// Input dialog state (shown on top of all UI)
    pub input_dialog: Option<InputDialog>,
    /// Minimum lines to keep between cursor and edge (like vim's scrolloff)
    pub scrolloff: usize,
}

impl State {
    /// Create a new State with default values
    pub fn new() -> Self {
        Self {
            scrolloff: 5, // Keep 5 lines between cursor and edge
            ..Default::default()
        }
    }
}

impl State {
    pub fn set_current_page_entries(&mut self, entries: Vec<PageEntry>) {
        if self.current_page.is_none() {
            self.current_page = Some(Default::default())
        }
        let page = self.current_page.as_mut().unwrap();

        // Save current selected index before updating entries
        let old_selected = page.list_state.selected();

        page.list = entries;
        // Apply current filter to new entries
        page.apply_filter();

        // Restore selection if possible
        if let Some(old_idx) = old_selected {
            // Only restore if there was a previous selection
            if page.filtered_list.is_empty() {
                page.list_state.select(None);
            } else {
                // Keep the old selection if it's still valid
                if old_idx < page.filtered_list.len() {
                    page.list_state.select(Some(old_idx));
                } else {
                    // Old index is out of range, select the last item
                    page.list_state.select(Some(page.filtered_list.len() - 1));
                }
            }
        }
    }
    pub fn add_keymap(&mut self, keymap: Keymap) {
        self.keymap_config
            .retain(|v| !(v.mode == keymap.mode && v.key_sequence == keymap.key_sequence));
        self.keymap_config.push(keymap);
    }

    pub fn tap_key(&mut self, event: KeyEvent) -> anyhow::Result<Option<LuaFunction>> {
        self.last_key_event_buffer.push(event);
        let cands = self.keymap_candidates_iter().take(2).collect::<Vec<_>>();
        match cands.len() {
            0 => {
                self.last_key_event_buffer.clear();
                Ok(None)
            }
            1 => {
                let cand = cands.first().unwrap();
                if cand.key_sequence.all_match(&self.last_key_event_buffer) {
                    let cb = cands.first().unwrap().callback.clone(); // todo: remove clone
                    self.last_key_event_buffer.clear();
                    Ok(Some(cb))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    pub fn hovered(&self) -> Option<&PageEntry> {
        self.current_page
            .as_ref()
            .and_then(|p| p.list_state.selected().and_then(|s| p.filtered_list.get(s)))
    }

    fn keymap_candidates_iter(&self) -> impl Iterator<Item = &Keymap> {
        // todo: 加path
        self.keymap_config.iter().filter(|keymap| {
            keymap.mode == self.current_mode
                && keymap
                    .key_sequence
                    .prefix_match(&self.last_key_event_buffer)
        })
    }

    pub fn go_to(&mut self, path: Vec<String>) -> bool {
        // Cache current page before navigating away
        if let Some(page) = self.current_page.take() {
            self.page_cache.insert(self.current_path.clone(), page);
        }

        self.current_path = path.clone();
        self.current_preview.take();

        // Try to restore page from cache
        if let Some(page) = self.page_cache.remove(&path) {
            self.current_page = Some(page);
            true // Restored from cache
        } else {
            false // Not in cache, needs to be loaded
        }
    }

    /// Clear cache for current path (used by reload command)
    pub fn clear_current_cache(&mut self) {
        self.page_cache.remove(&self.current_path);
    }

    pub fn scroll_by(&mut self, amount: i16) {
        if let Some(page) = &mut self.current_page {
            if page.filtered_list.is_empty() {
                return;
            }

            let len = page.filtered_list.len();
            let current = page.list_state.selected().unwrap_or(0);

            // Calculate new selected index
            let new = if amount > 0 {
                let target = current.saturating_add(amount as usize);
                // Only wrap if single-step scroll and at the last entry
                if amount == 1 && current == len - 1 {
                    0
                } else {
                    target.min(len - 1)
                }
            } else {
                let target = current.saturating_sub(amount.unsigned_abs() as usize);
                // Only wrap if single-step scroll and at the first entry
                if amount == -1 && current == 0 {
                    len - 1
                } else {
                    target
                }
            };

            // Set the new selection
            // Offset will be adjusted by ListWidget::render based on scrolloff
            page.list_state.select(Some(new));
        }
    }

    pub fn scroll_preview_by(&mut self, amount: i16) {
        if let Some(p) = &mut self.current_preview {
            p.scroll_by(amount);
        }
    }

    pub fn set_notification(&mut self, message: Text<'static>) {
        self.notification = Some((message, Instant::now() + std::time::Duration::from_secs(3)));
    }

    /// Show the confirm dialog
    pub fn show_confirm_dialog(
        &mut self,
        title: Option<String>,
        prompt: String,
        on_confirm: LuaFunction,
        on_cancel: Option<LuaFunction>,
    ) {
        self.confirm_dialog = Some(ConfirmDialog::new(title, prompt, on_confirm, on_cancel));
    }

    /// Show the input dialog
    pub fn show_input_dialog(
        &mut self,
        prompt: String,
        placeholder: String,
        value: String,
        on_submit: LuaFunction,
        on_cancel: LuaFunction,
        on_change: LuaFunction,
    ) {
        self.input_dialog = Some(InputDialog::new(
            prompt,
            placeholder,
            value,
            on_submit,
            on_cancel,
            on_change,
        ));
    }

    /// Toggle selected button in confirm dialog
    pub fn toggle_confirm_button(&mut self) {
        if let Some(dialog) = &mut self.confirm_dialog {
            dialog.selected_button = dialog.selected_button.toggle();
        }
    }

    /// Get the current selected button
    pub fn get_selected_button(&self) -> Option<ConfirmButton> {
        self.confirm_dialog.as_ref().map(|d| d.selected_button)
    }

    /// Get the current scrolloff value
    pub fn scrolloff(&self) -> usize {
        self.scrolloff
    }
}
