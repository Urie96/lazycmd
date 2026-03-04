use mlua::prelude::*;
use ratatui::style::Color;
use ratatui::text::{Line, Span, Text};
use std::str::FromStr;

type AnyUserData = LuaAnyUserData;

pub struct LuaText(pub Text<'static>);

pub struct LuaLine(pub Line<'static>);

pub struct LuaSpan(pub Span<'static>);

impl LuaUserData for LuaSpan {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function_mut("fg", |lua, (ud, color): (AnyUserData, String)| {
            let color = Color::from_str(&color).into_lua_err()?;
            ud.borrow_mut::<Self>()?.0.style.fg = Some(color);
            ud.into_lua(lua)
        });

        methods.add_function_mut("bg", |lua, (ud, color): (AnyUserData, String)| {
            let color = Color::from_str(&color).into_lua_err()?;
            ud.borrow_mut::<Self>()?.0.style.bg = Some(color);
            ud.into_lua(lua)
        });

        methods.add_meta_function_mut(
            mlua::MetaMethod::Concat,
            |lua, (this, rhs): (AnyUserData, LuaValue)| {
                let span_lhs = this.take::<LuaSpan>()?.0;

                match rhs {
                    LuaValue::String(s) => lua.create_userdata(LuaLine(Line::from(vec![
                        span_lhs,
                        Span::raw(s.to_str()?.to_string()),
                    ]))),
                    LuaValue::UserData(ud) => {
                        if let Ok(span_rhs) = ud.take::<LuaSpan>() {
                            lua.create_userdata(LuaLine(Line::from(vec![span_lhs, span_rhs.0])))
                        } else if let Ok(line_rhs) = ud.take::<LuaLine>() {
                            let mut spans = vec![span_lhs];
                            spans.extend(line_rhs.0.spans);
                            lua.create_userdata(LuaLine(Line::from(spans)))
                        } else {
                            Err(mlua::Error::runtime("cannot concat Span with this type"))
                        }
                    }
                    _ => Err(mlua::Error::runtime(
                        "cannot concat Span with non-string/non-UserData value",
                    )),
                }
            },
        );
    }
}

impl LuaUserData for LuaLine {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function_mut("fg", |lua, (ud, color): (AnyUserData, String)| {
            let color = Color::from_str(&color).into_lua_err()?;
            ud.borrow_mut::<Self>()?.0.style.fg = Some(color);
            ud.into_lua(lua)
        });

        methods.add_function_mut("bg", |lua, (ud, color): (AnyUserData, String)| {
            let color = Color::from_str(&color).into_lua_err()?;
            ud.borrow_mut::<Self>()?.0.style.bg = Some(color);
            ud.into_lua(lua)
        });

        methods.add_meta_function_mut(
            mlua::MetaMethod::Concat,
            |lua, (this, rhs): (AnyUserData, LuaValue)| {
                let mut line_lhs = this.borrow_mut::<Self>()?;

                match rhs {
                    LuaValue::String(s) => {
                        line_lhs.0.push_span(Span::raw(s.to_str()?.to_string()));
                        this.into_lua(lua)
                    }
                    LuaValue::UserData(ud) => {
                        // 尝试转换为 Span
                        if let Ok(span_rhs) = ud.take::<LuaSpan>() {
                            line_lhs.0.push_span(span_rhs.0);
                            this.into_lua(lua)
                        } else {
                            Err(mlua::Error::runtime("cannot concat Line with this type"))
                        }
                    }
                    _ => Err(mlua::Error::runtime(
                        "cannot concat Line with non-string/non-UserData value",
                    )),
                }
            },
        );
    }
}

impl LuaUserData for LuaText {}
