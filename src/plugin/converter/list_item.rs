use mlua::{prelude::*, FromLua, UserData};

pub struct ListItem(pub ratatui::widgets::ListItem<'static>);

impl UserData for ListItem {}

impl FromLua for ListItem {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        Ok(match value {
            LuaValue::String(s) => ListItem(ratatui::widgets::ListItem::new(s.to_string_lossy())),
            LuaValue::UserData(ud) => ud.take::<ListItem>()?,
            _ => Err("expect string or userdata ListItem".into_lua_err())?,
        })
    }
}
