use crate::widgets::{Line, Span};
use mlua::prelude::*;

/// Create a Line from multiple Spans or Strings
pub fn line(lua: &Lua) -> mlua::Result<LuaFunction> {
    lua.create_function(|_lua, args: LuaMultiValue| {
        let mut spans = Vec::new();

        for arg in args {
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
                            "Expected Span or String".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err(LuaError::RuntimeError(
                        "Expected Span or String".to_string(),
                    ));
                }
            }
        }

        Ok(Line(ratatui::text::Line {
            spans,
            alignment: None,
            style: ratatui::style::Style::default(),
        }))
    })
}
