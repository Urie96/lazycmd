use crate::{plugin, widgets::Renderable, Event, PageEntry};
use mlua::prelude::*;

pub(super) fn new_table(lua: &Lua) -> mlua::Result<LuaTable> {
    let page_set_entries = lua
        .create_function(|lua, entries: Vec<PageEntry>| {
            plugin::mut_scope_state(lua, |state| {
                state.set_current_page_entries(entries);
                Ok(())
            })
        })?
        .into_lua(lua)?;

    let go_to = lua
        .create_function(|lua, path: Vec<String>| plugin::send_event(lua, Event::Enter(path)))?
        .into_lua(lua)?;

    let get_current_path = lua
        .create_function(|lua, ()| {
            plugin::borrow_scope_state(lua, |state| {
                lua.create_sequence_from(state.current_path.iter().map(|v| v.as_str()))
            })
        })?
        .into_lua(lua)?;

    let get_hovered_path = lua
        .create_function(|lua, ()| {
            plugin::borrow_scope_state(lua, |state| match state.hovered() {
                None => Ok(LuaNil),
                Some(hovered) => lua
                    .create_sequence_from(
                        state
                            .current_path
                            .iter()
                            .map(|v| v.as_str())
                            .chain([hovered.key.as_str()]),
                    )?
                    .into_lua(lua),
            })
        })?
        .into_lua(lua)?;

    let page_set_preview = lua
        .create_function(|lua, preview: Box<dyn Renderable>| {
            plugin::mut_scope_state(lua, |state| {
                state.current_preview = Some(preview);
                plugin::send_event(lua, Event::Render)?;
                Ok(())
            })
        })?
        .into_lua(lua)?;

    let page_get_hovered = lua
        .create_function(|lua, ()| {
            plugin::borrow_scope_state(lua, |state| Ok(state.hovered().map(|p| p.tbl.clone())))
        })?
        .into_lua(lua)?;

    lua.create_table_from([
        ("page_set_entries", page_set_entries),
        ("page_get_hovered", page_get_hovered),
        ("page_set_preview", page_set_preview),
        ("go_to", go_to),
        ("get_current_path", get_current_path),
        ("get_hovered_path", get_hovered_path),
    ])
}
