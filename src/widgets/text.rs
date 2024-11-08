use mlua::prelude::*;

pub struct Text(pub ratatui::text::Text<'static>);

impl LuaUserData for Text {}

#[derive(Clone)]
pub struct Span(pub ratatui::text::Span<'static>);

impl LuaUserData for Span {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // methods.add_meta_method(LuaMetaMethod::Add, method)
    }
}
