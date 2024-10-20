use std::{cell::RefCell, collections::HashMap};

use crossterm::event::KeyEvent;

use crate::{ro_cell::RoCell, Keymap, Mode, PageItems};

pub static STATE: RoCell<RefCell<State>> = RoCell::new();
pub fn init() {
    STATE.init(RefCell::new(State::new()));
}

pub struct State {
    current_mode: Mode,
    current_path: Vec<String>,
    items: HashMap<Vec<String>, PageItems>,
    keymap_config: Vec<Keymap>,
    last_key_event_buffer: Vec<KeyEvent>,
}

impl State {
    fn new() -> Self {
        Self {
            current_path: Default::default(),
            current_mode: Mode::Main,
            items: Default::default(),
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
}
