use ansi_to_tui::IntoText;
use std::str::FromStr;

use crate::widgets::{Line, Span, Text};
use mlua::prelude::*;
use ratatui::style::{Color, Stylize};
use ratatui::text;

pub(super) fn inject_string_meta_method(lua: &Lua) -> mlua::Result<()> {
    // 设置 string 表中的函数
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
    string.raw_set(
        "split",
        lua.create_function(|_, (s, sep): (String, String)| {
            let parts: Vec<String> = s.split(&sep).map(|x| x.to_string()).collect();
            Ok(parts)
        })?,
    )?;
    Ok(())
}

/// Create a Line from a table of Spans or Strings
pub fn line(lua: &Lua) -> mlua::Result<LuaFunction> {
    lua.create_function(|_lua, args: LuaTable| {
        let len = args.raw_len();
        let mut spans = Vec::with_capacity(len);

        for pair in args.pairs::<LuaValue, LuaValue>() {
            let (_, arg) = pair?;
            match arg {
                LuaValue::String(s) => {
                    let content = s.to_str()?.to_string();
                    spans.push(ratatui::text::Span::raw(content));
                }
                LuaValue::UserData(ud) => {
                    if let Ok(span) = ud.take::<Span>() {
                        spans.push(span.0);
                    } else {
                        return Err(LuaError::RuntimeError(
                            "Expected Span or String in table".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err(LuaError::RuntimeError(
                        "Expected Span or String in table".to_string(),
                    ));
                }
            }
        }

        Ok(Line(ratatui::text::Line::from(spans)))
    })
}

/// Create a Text from a table of Lines, Spans, or Strings
pub fn text(lua: &Lua) -> mlua::Result<LuaFunction> {
    lua.create_function(|_lua, args: LuaTable| {
        let len = args.raw_len();
        let mut lines = Vec::with_capacity(len);

        for pair in args.pairs::<LuaValue, LuaValue>() {
            let (_, arg) = pair?;
            match arg {
                LuaValue::String(s) => {
                    let content = s.to_str()?;
                    // Split string by newlines into multiple lines
                    for line in content.lines() {
                        lines.push(ratatui::text::Line::raw(line.to_string()));
                    }
                }
                LuaValue::UserData(ud) => {
                    // Try Line first
                    if let Ok(line) = ud.take::<Line>() {
                        lines.push(line.0);
                    } else if let Ok(span) = ud.take::<Span>() {
                        lines.push(ratatui::text::Line::from(span.0));
                    } else {
                        return Err(LuaError::RuntimeError(
                            "Expected Line, Span, or String in table".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err(LuaError::RuntimeError(
                        "Expected Line, Span, or String in table".to_string(),
                    ));
                }
            }
        }
        Ok(Text(text::Text::from(lines)))
    })
}
