use ansi_to_tui::IntoText;

use crate::plugin::lc::highlighter;
use crate::widgets::{LuaLine, LuaSpan, LuaText};
use mlua::prelude::*;
use ratatui::text::{Line, Span, Text};

pub fn span(lua: &Lua) -> mlua::Result<LuaFunction> {
    lua.create_function(|_lua, s: String| Ok(LuaSpan(Span::raw(s))))
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
                    spans.push(Span::raw(content));
                }
                LuaValue::UserData(ud) => {
                    if let Ok(span) = ud.take::<LuaSpan>() {
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

        Ok(LuaLine(Line::from(spans)))
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
                        lines.push(Line::raw(line.to_string()));
                    }
                }
                LuaValue::UserData(ud) => {
                    // Try Line first
                    if let Ok(line) = ud.take::<LuaLine>() {
                        lines.push(line.0);
                    } else if let Ok(span) = ud.take::<LuaSpan>() {
                        lines.push(Line::from(span.0));
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
        Ok(LuaText(Text::from(lines)))
    })
}

/// Highlight code with syntax highlighting
pub fn highlight(lua: &Lua) -> mlua::Result<LuaFunction> {
    lua.create_function(|_lua, (code, language): (String, String)| {
        highlighter::highlight(&code, &language)
            .map(|text| LuaText(text))
            .map_err(|e| LuaError::RuntimeError(format!("Highlighting failed: {}", e)))
    })
}

pub fn ansi(lua: &Lua) -> mlua::Result<LuaFunction> {
    lua.create_function(|_lua, s: String| Ok(LuaText(s.as_bytes().into_text().into_lua_err()?)))
}
