use mlua::prelude::*;

pub struct Text(pub ratatui::text::Text<'static>);

pub struct Line(pub ratatui::text::Line<'static>);

pub struct Span(pub ratatui::text::Span<'static>);

impl LuaUserData for Span {}

impl LuaUserData for Line {}

impl LuaUserData for Text {}
