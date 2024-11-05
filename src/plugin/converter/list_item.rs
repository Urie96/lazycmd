use mlua::{prelude::*, FromLua, UserData};

use super::span::Span;

pub struct ListItem(pub ratatui::widgets::ListItem<'static>);

impl UserData for ListItem {}

impl FromLua for ListItem {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        Ok(match value {
            LuaValue::String(s) => ListItem(ratatui::widgets::ListItem::new(s.to_string_lossy())),
            LuaValue::UserData(ud) => {
                if let Ok(span) = ud.take::<Span>() {
                    ListItem(ratatui::widgets::ListItem::from(span.0))
                } else {
                    Err("expect string or userdata ListItem".into_lua_err())?
                }
            }
            _ => Err("expect string or userdata ListItem".into_lua_err())?,
        })
    }
}
