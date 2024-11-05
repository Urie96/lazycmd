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
        let f = || match self.tbl.get::<LuaValue>("displayer")? {
            // LuaValue::Function(f)=>f.call_async((self.tbl)).await,
            LuaValue::Nil => Ok(ListItem::new(self.key.as_str())),
            _ => bail!("displayer is expected a function"),
        };
        f().unwrap_or_else(|e| ListItem::new(e.to_string()))
    }

    pub fn preview(&self) -> String {
        let f = || match self.tbl.get::<LuaValue>("preview")? {
            LuaValue::Nil => Ok("nil".to_string()),
            LuaValue::String(s) => Ok(s.to_string_lossy()),
            _ => bail!("preview "),
        };
        f().unwrap_or_else(|e| e.to_string())
    }
}

#[derive(Default)]
pub struct Page {
    pub list: Vec<PageEntry>,
    pub state: widgets::ListState,
    pub list_state: widgets::ListState,
}
