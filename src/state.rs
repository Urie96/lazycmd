use crossterm::event::KeyEvent;
use mlua::prelude::*;

use crate::{widgets::Renderable, Keymap, Mode, Page, PageEntry};

#[derive(Default)]
pub struct State {
    pub current_mode: Mode,
    pub current_path: Vec<String>,
    pub current_page: Option<Page>,
    pub keymap_config: Vec<Keymap>,
    pub last_key_event_buffer: Vec<KeyEvent>,
    pub current_preview: Option<Box<dyn Renderable>>,
}

impl State {
    pub fn set_current_page_entries(&mut self, entries: Vec<PageEntry>) {
        if self.current_page.is_none() {
            self.current_page = Some(Default::default())
        }
        let page = self.current_page.as_mut().unwrap();
        page.list = entries;
        if !page.list.is_empty() && page.list_state.selected().is_none() {
            page.list_state.select(Some(0));
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
            .and_then(|p| p.list_state.selected().and_then(|s| p.list.get(s)))
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
        if let Some(page) = &mut self.current_page {
            if amount < 0 {
                page.list_state.scroll_up_by(amount.unsigned_abs())
            } else {
                page.list_state.scroll_down_by(amount.unsigned_abs())
            }
        }
    }

    pub fn scroll_preview_by(&mut self, amount: i16) {
        if let Some(p) = &mut self.current_preview {
            p.scroll_by(amount);
        }
    }
}
