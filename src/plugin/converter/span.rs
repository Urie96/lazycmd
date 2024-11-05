use mlua::prelude::*;
use mlua::{FromLua, UserData};

pub struct Span(pub ratatui::text::Span<'static>);

impl UserData for Span {}

impl FromLua for Span {
    fn from_lua(value: LuaValue, lua: &Lua) -> mlua::Result<Self> {
        Ok(match value {
            LuaValue::String(s) => Self(ratatui::text::Span::raw(s.to_string_lossy())),
            LuaValue::UserData(ud) => ud.take()?,
            _ => Err("expected string or Span".into_lua_err())?,
        })
    }
}
