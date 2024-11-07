use crate::{plugin, widgets::Renderable, Event, PageEntry};
use mlua::prelude::*;

pub(super) fn new_table(lua: &Lua) -> mlua::Result<LuaTable> {
    let page_set_entries = lua
        .create_function(|lua, (path, entries): (Vec<String>, Vec<PageEntry>)| {
            plugin::mut_scope_state(lua, |state| {
                if state.current_path == path {
                    state.set_current_page_entries(entries);
                }
                Ok(())
            })
        })?
        .into_lua(lua)?;

    let go_to = lua
        .create_function(|lua, path: Vec<String>| plugin::send_event(lua, Event::Enter(path)))?
        .into_lua(lua)?;

    let get_current_path = lua
        .create_function(|lua, ()| {
            plugin::borrow_scope_state(lua, |state| Ok(state.current_path.clone()))
        })?
        .into_lua(lua)?;

    let page_set_preview = lua
        .create_function(|lua, (path, preview): (Vec<String>, Box<dyn Renderable>)| {
            plugin::mut_scope_state(lua, |state| {
                if !path.is_empty()
                    && path[..path.len() - 1] == state.current_path
                    && state
                        .hovered()
                        .map(|h| &h.key == path.last().unwrap())
                        .unwrap_or(false)
                {
                    state.current_preview = Some(preview);
                    plugin::send_event(lua, Event::Render)?;
                }
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
    ])
}
