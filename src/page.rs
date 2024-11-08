use crate::widgets::Span;
use anyhow::bail;
use mlua::prelude::*;
use ratatui::widgets::{self, ListItem};

pub struct PageEntry {
    pub key: String,
    pub tbl: LuaTable,
}

impl FromLua for PageEntry {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        let tbl = LuaTable::from_lua(value, lua)?;
        let key: String = tbl.get("key")?;
        Ok(Self { key, tbl })
    }
}

impl PageEntry {
    pub fn display(&self) -> ListItem<'_> {
        let f = || match self.tbl.get::<LuaValue>("display")? {
            LuaValue::Nil => Ok(ListItem::new(self.key.as_str())),
            LuaValue::String(s) => Ok(ListItem::new(s.to_string_lossy())),
            LuaValue::UserData(ud) => {
                if let Ok(span) = ud.borrow::<Span>() {
                    Ok(ListItem::new(span.clone().0))
                } else {
                    bail!("Expected Span or nil")
                }
            }
            _ => bail!("Expected Span or nil"),
        };
        f().unwrap_or_else(|e| ListItem::new(e.to_string()))
    }
}

#[derive(Default)]
pub struct Page {
    pub list: Vec<PageEntry>,
    pub list_state: widgets::ListState,
}
