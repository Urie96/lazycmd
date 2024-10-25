use mlua::{FromLua, Table};

use crate::PageEntry;

impl FromLua for PageEntry {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        let tbl = Table::from_lua(value, lua)?;
        Ok(Self {
            key: tbl.get("key")?,
            display: tbl.get("display")?,
        })
    }
}
