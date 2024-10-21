use std::{cell::RefCell, collections::HashMap};

use crossterm::event::KeyEvent;

use crate::{ro_cell::RoCell, Keymap, Mode, Page};

pub static STATE: RoCell<RefCell<State>> = RoCell::new();
pub fn init() {
    STATE.init(RefCell::new(State::new()));
}

pub struct State {
    pub current_mode: Mode,
    pub current_path: Vec<String>,
    pub pages: HashMap<Vec<String>, Page>,
    pub keymap_config: Vec<Keymap>,
    pub last_key_event_buffer: Vec<KeyEvent>,
}

impl State {
    fn new() -> Self {
        Self {
            current_path: Default::default(),
            current_mode: Mode::Main,
            pages: Default::default(),
            keymap_config: Default::default(),
            last_key_event_buffer: Default::default(),
        }
    }

    pub fn add_keymap(&mut self, keymap: Keymap) {
        self.keymap_config.push(keymap);
    }

    pub fn tap_key(&mut self, event: KeyEvent) -> anyhow::Result<()> {
        self.last_key_event_buffer.push(event);
        let cands = self.keymap_candidates_iter().take(2).collect::<Vec<_>>();
        match cands.len() {
            0 => {
                self.last_key_event_buffer.clear();
            }
            1 => {
                (cands.first().unwrap().callback)()?;
                self.last_key_event_buffer.clear();
            }
            _ => (),
        }
        Ok(())
    }

    fn keymap_candidates_iter(&self) -> impl Iterator<Item = &Keymap> {
        // todo: 加path
        self.keymap_config.iter().filter(|keymap| {
            keymap
                .mode
                .as_ref()
                .map_or(true, |v| v == &self.current_mode)
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
        if let Some(page) = self.pages.get_mut(&self.current_path) {
            page.list_state.scroll_down_by(amount)
        }
    }

    pub fn scroll_up_by(&mut self, amount: u16) {
        if let Some(page) = self.pages.get_mut(&self.current_path) {
            page.list_state.scroll_up_by(amount)
        }
    }
}
