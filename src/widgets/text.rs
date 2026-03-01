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
    }
}

impl LuaUserData for Text {}
