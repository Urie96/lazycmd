use crossterm::event::KeyEvent;
use mlua::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

use crate::{widgets::Renderable, Keymap, Mode, Page, PageEntry};

/// Represents which button is selected in the confirm dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmButton {
    Yes,
    No,
}

impl ConfirmButton {
    pub fn toggle(&self) -> Self {
        match self {
            ConfirmButton::Yes => ConfirmButton::No,
            ConfirmButton::No => ConfirmButton::Yes,
        }
    }
}

/// State for the confirm dialog
#[derive(Debug)]
pub struct ConfirmDialog {
    pub title: Option<String>,
    pub prompt: String,
    pub on_confirm: LuaFunction,
    pub on_cancel: LuaFunction,
    pub selected_button: ConfirmButton,
}

impl ConfirmDialog {
    pub fn new(
        title: Option<String>,
        prompt: String,
        on_confirm: LuaFunction,
        on_cancel: LuaFunction,
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
    pub notification: Option<(String, Instant)>,
    pub filter_input: String,
    pub input_cursor_position: usize,
    /// Cache for pages to preserve cursor position, entries and filter when navigating back
    page_cache: HashMap<Vec<String>, Page>,
    /// Hooks to call before reload command
    pub pre_reload_hooks: Vec<LuaFunction>,
    /// Current plugin name
    pub current_plugin: String,
    /// Confirm dialog state (shown on top of all UI)
    pub confirm_dialog: Option<ConfirmDialog>,
}

impl State {
    pub fn set_current_page_entries(&mut self, entries: Vec<PageEntry>) {
        if self.current_page.is_none() {
            self.current_page = Some(Default::default())
        }
        let page = self.current_page.as_mut().unwrap();
        page.list = entries;
        // Sync filter state from State to Page
        page.filter_input = self.filter_input.clone();
        page.input_cursor_position = self.input_cursor_position;
        // Apply current filter to new entries
        page.apply_filter(&self.filter_input);
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
        self.current_page.as_ref().and_then(|p| {
            p.list_state
                .selected()
                .and_then(|s| p.filtered_list.get(s))
        })
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
        if let Some(mut page) = self.current_page.take() {
            // Sync filter state from State to Page before caching
            page.filter_input = self.filter_input.clone();
            page.input_cursor_position = self.input_cursor_position;
            self.page_cache.insert(self.current_path.clone(), page);
        }

        self.current_path = path.clone();
        self.current_preview.take();

        // Try to restore page from cache
        if let Some(page) = self.page_cache.remove(&path) {
            // Sync filter state from Page to State after restoring
            self.filter_input = page.filter_input.clone();
            self.input_cursor_position = page.input_cursor_position;
            self.current_page = Some(page);
            true // Restored from cache
        } else {
            // Clear filter for new paths
            self.filter_input.clear();
            self.input_cursor_position = 0;
            false // Not in cache, needs to be loaded
        }
    }

    /// Clear cache for current path (used by reload command)
    pub fn clear_current_cache(&mut self) {
        self.page_cache.remove(&self.current_path);
    }

    pub fn scroll_by(&mut self, amount: i16) {
        let page = self.current_page.as_mut().unwrap();
        if !page.list.is_empty() && page.list_state.selected().is_none() {
            page.list_state.select(Some(0));
        }
        if let Some(page) = &mut self.current_page {
            let len = page.list.len();
            if len == 0 {
                return;
            }

            let current = page.list_state.selected().unwrap_or(0);
            let new = if amount > 0 {
                // Calculate the target position
                let target = current.saturating_add(amount as usize);
                // Only wrap if single-step scroll and at the last entry
                if amount == 1 && current == len - 1 {
                    0
                } else {
                    target.min(len - 1)
                }
            } else {
                // Calculate the target position
                let target = current.saturating_sub(amount.unsigned_abs() as usize);
                // Only wrap if single-step scroll and at the first entry
                if amount == -1 && current == 0 {
                    len - 1
                } else {
                    target
                }
            };

            page.list_state.select(Some(new));
        }
    }

    pub fn scroll_preview_by(&mut self, amount: i16) {
        if let Some(p) = &mut self.current_preview {
            p.scroll_by(amount);
        }
    }

    pub fn set_notification(&mut self, message: String) {
        self.notification = Some((message, Instant::now() + std::time::Duration::from_secs(3)));
    }

    pub fn check_notification_expiry(&mut self) -> bool {
        if let Some((_, expires_at)) = &self.notification {
            if Instant::now() > *expires_at {
                self.notification.take();
                return true;
            }
        }
        false
    }

    /// Show the confirm dialog
    pub fn show_confirm_dialog(
        &mut self,
        title: Option<String>,
        prompt: String,
        on_confirm: LuaFunction,
        on_cancel: LuaFunction,
    ) {
        self.confirm_dialog = Some(ConfirmDialog::new(title, prompt, on_confirm, on_cancel));
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
}
