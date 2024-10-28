use std::rc::Rc;

use mlua::{prelude::*, FromLua, ObjectLike, Table};

use crate::PageEntry;

use super::list_item::ListItem;

impl FromLua for PageEntry {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        let tbl = Table::from_lua(value, lua)?;
        Ok(Self {
            key: tbl.get("key")?,
            displayer: Rc::new(move || {
                let tbl = tbl;
                Box::pin(async {
                    let tbl = tbl.clone();
                    if tbl.get::<LuaValue>("displayer")?.is_nil() {
                        Ok(ratatui::widgets::ListItem::new(tbl.get::<String>("key")?))
                    } else {
                        let display: ListItem = tbl.call_async_method("displayer", ()).await?;
                        Ok(display.0)
                    }
                })
            }),
        })
    }
}
