use mlua::prelude::*;
use ratatui::style::Color;
use std::str::FromStr;

type AnyUserData = LuaAnyUserData;

pub struct Text(pub ratatui::text::Text<'static>);

pub struct Line(pub ratatui::text::Line<'static>);

pub struct Span(pub ratatui::text::Span<'static>);

impl LuaUserData for Span {
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
                let span_lhs = this.take::<Span>()?.0;

                match rhs {
                    LuaValue::String(s) => {
                        lua.create_userdata(Line(ratatui::text::Line::from(vec![
                            span_lhs,
                            ratatui::text::Span::raw(s.to_str()?.to_string()),
                        ])))
                    }
                    LuaValue::UserData(ud) => {
                        if let Ok(span_rhs) = ud.take::<Span>() {
                            lua.create_userdata(Line(ratatui::text::Line::from(vec![
                                span_lhs, span_rhs.0,
                            ])))
                        } else if let Ok(line_rhs) = ud.take::<Line>() {
                            let mut spans = vec![span_lhs];
                            spans.extend(line_rhs.0.spans);
                            lua.create_userdata(Line(ratatui::text::Line::from(spans)))
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

impl LuaUserData for Line {
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
                        line_lhs
                            .0
                            .push_span(ratatui::text::Span::raw(s.to_str()?.to_string()));
                        this.into_lua(lua)
                    }
                    LuaValue::UserData(ud) => {
                        // 尝试转换为 Span
                        if let Ok(span_rhs) = ud.take::<Span>() {
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

impl LuaUserData for Text {}
