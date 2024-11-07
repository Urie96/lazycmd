use ansi_to_tui::IntoText;
use std::str::FromStr;

use crate::widgets::{Span, Text};
use mlua::prelude::*;
use ratatui::style::{Color, Stylize};

pub(super) fn new_table(lua: &Lua) -> mlua::Result<LuaTable> {
    lua.create_table_from([("test", "a")])
}

pub(super) fn inject_string_meta_method(lua: &Lua) -> mlua::Result<()> {
    let string: LuaTable = lua.globals().get("string")?;
    string.raw_set(
        "fg",
        lua.create_function(|_, (str, color): (String, String)| {
            Ok(Span(
                ratatui::text::Span::raw(str).fg(Color::from_str(&color).into_lua_err()?),
            ))
        })?,
    )?;
    string.raw_set(
        "ansi",
        lua.create_function(|_, s: mlua::String| {
            Ok(Text(s.as_bytes().into_text().into_lua_err()?))
        })?,
    )?;
    Ok(())
}
