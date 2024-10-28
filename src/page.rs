use mlua::prelude::*;
use ratatui::widgets::{self, ListItem};

pub struct PageEntry {
    pub key: String,
    pub raw: LuaTable,
}

impl PageEntry {
    fn display(&self) -> ListItem {
        let f = || {
            let tbl = &self.raw;

            if tbl.get::<LuaValue>("displayer")?.is_nil() {
                Ok()
            }
        };
    }
}

#[derive(Default)]
pub struct Page {
    pub list: Vec<PageEntry>,
    pub state: widgets::ListState,
    pub list_state: widgets::ListState,
}
