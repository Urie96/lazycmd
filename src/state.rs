use crossterm::event::KeyEvent;
use mlua::prelude::*;
use std::time::Instant;

use crate::{widgets::Renderable, Keymap, Mode, Page, PageEntry};

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
}

impl State {
    pub fn set_current_page_entries(&mut self, entries: Vec<PageEntry>) {
        if self.current_page.is_none() {
            self.current_page = Some(Default::default())
        }
        let page = self.current_page.as_mut().unwrap();
        page.list = entries;
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

    pub fn go_to(&mut self, path: Vec<String>) {
        self.current_path = path;
        self.current_page = None;
        self.current_preview.take();
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
                current.saturating_add(amount as usize).min(len - 1)
            } else {
                current.saturating_sub(amount.unsigned_abs() as usize)
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
}
