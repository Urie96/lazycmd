use crossterm::event::KeyEvent;
use mlua::prelude::*;

use crate::{preview::Preview, Keymap, Mode, Page, PageEntry};

#[derive(Default)]
pub struct State {
    pub current_mode: Mode,
    pub current_path: Vec<String>,
    pub current_page: Option<Page>,
    pub keymap_config: Vec<Keymap>,
    pub last_key_event_buffer: Vec<KeyEvent>,
    pub current_preview: Preview,
}

impl State {
    pub fn set_current_page_entries(&mut self, entries: Vec<PageEntry>) {
        self.current_page = Some(Page {
            list: entries,
            ..Default::default()
        })
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

    fn keymap_candidates_iter(&self) -> impl Iterator<Item = &Keymap> {
        // todo: 加path
        self.keymap_config.iter().filter(|keymap| {
            keymap.mode == self.current_mode
                && keymap
                    .key_sequence
                    .prefix_match(&self.last_key_event_buffer)
        })
    }

    pub fn go_to_parent(&mut self) {
        if self.current_path.is_empty() {
            return;
        }
        self.current_path.pop();
    }

    pub fn scroll_down_by(&mut self, amount: u16) {
        if let Some(page) = &mut self.current_page {
            page.list_state.scroll_down_by(amount)
        }
    }

    pub fn scroll_up_by(&mut self, amount: u16) {
        if let Some(page) = &mut self.current_page {
            page.list_state.scroll_up_by(amount)
        }
    }
}
