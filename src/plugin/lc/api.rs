use crate::{plugin, widgets::Renderable, Event, PageEntry};
use mlua::prelude::*;

pub(super) fn new_table(lua: &Lua) -> mlua::Result<LuaTable> {
    let page_set_entries = lua
        .create_function(|lua, entries: Vec<PageEntry>| {
            plugin::mut_scope_state(lua, |state| {
                state.set_current_page_entries(entries);
                plugin::send_render_event(lua)?;
                plugin::send_event(lua, Event::Command("scroll_by 0".to_string()))?; // trigger scroll
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
                plugin::send_render_event(lua)?;
                plugin::send_event(lua, Event::Command("scroll_preview_by 0".to_string()))
            })
        })?
        .into_lua(lua)?;

    let page_get_hovered = lua
        .create_function(|lua, ()| {
            plugin::borrow_scope_state(lua, |state| Ok(state.hovered().map(|p| p.tbl.clone())))
        })?
        .into_lua(lua)?;

    let page_get_entries = lua
        .create_function(|lua, ()| {
            plugin::borrow_scope_state(lua, |state| {
                let entries = state
                    .current_page
                    .as_ref()
                    .map(|page| page.list.iter().map(|entry| entry.tbl.clone()))
                    .into_iter()
                    .flatten();
                lua.create_sequence_from(entries)
            })
        })?
        .into_lua(lua)?;

    let argv = lua
        .create_function(|lua, ()| lua.create_sequence_from(std::env::args()))?
        .into_lua(lua)?;

    let set_filter = lua
        .create_function(|lua, filter: String| {
            plugin::mut_scope_state(lua, |state| {
                // Apply filter to current page
                if let Some(page) = &mut state.current_page {
                    page.list_filter = filter;
                    page.apply_filter();
                }
                state.last_key_event_buffer.clear();
                // Clear preview so it will be refreshed based on new selection
                state.current_preview.take();
                plugin::send_render_event(lua)?;
                plugin::send_event(lua, Event::RefreshPreview)?;
                Ok(())
            })
        })?
        .into_lua(lua)?;

    let get_filter = lua
        .create_function(|lua, ()| {
            plugin::borrow_scope_state(lua, |state| {
                Ok(state
                    .current_page
                    .as_ref()
                    .map(|page| page.list_filter.clone())
                    .unwrap_or_default())
            })
        })?
        .into_lua(lua)?;

    let append_hook_pre_reload = lua
        .create_function(|lua, cb: LuaFunction| {
            plugin::mut_scope_state(lua, |state| {
                state.pre_reload_hooks.push(cb);
                Ok(())
            })
        })?
        .into_lua(lua)?;

    let clear_page_cache = lua
        .create_function(|lua, path: Vec<String>| {
            plugin::mut_scope_state(lua, |state| {
                state.clear_cache_for_path(&path);
                Ok(())
            })
        })?
        .into_lua(lua)?;

    let append_hook_pre_quit = lua
        .create_function(|lua, cb: LuaFunction| {
            plugin::mut_scope_state(lua, |state| {
                state.pre_quit_hooks.push(cb);
                Ok(())
            })
        })?
        .into_lua(lua)?;

    let append_hook_post_page_enter = lua
        .create_function(|lua, cb: LuaFunction| {
            plugin::mut_scope_state(lua, |state| {
                state.post_page_enter_hooks.push(cb);
                Ok(())
            })
        })?
        .into_lua(lua)?;

    let get_available_keymaps = lua
        .create_function(|lua, ()| {
            plugin::borrow_scope_state(lua, |state| {
                let mut keymaps = Vec::new();
                for item in state.available_keymaps().map_err(LuaError::external)? {
                    let tbl = lua.create_table()?;
                    tbl.set("key", item.key)?;
                    tbl.set("desc", item.desc)?;
                    tbl.set("callback", item.callback)?;
                    tbl.set("source", item.source)?;
                    keymaps.push(tbl);
                }
                lua.create_sequence_from(keymaps)
            })
        })?
        .into_lua(lua)?;

    lua.create_table_from([
        ("page_set_entries", page_set_entries),
        ("page_get_entries", page_get_entries),
        ("page_get_hovered", page_get_hovered),
        ("page_set_preview", page_set_preview),
        ("go_to", go_to),
        ("get_current_path", get_current_path),
        ("get_hovered_path", get_hovered_path),
        ("argv", argv),
        ("set_filter", set_filter),
        ("get_filter", get_filter),
        ("get_available_keymaps", get_available_keymaps),
        ("clear_page_cache", clear_page_cache),
        ("append_hook_pre_reload", append_hook_pre_reload),
        ("append_hook_pre_quit", append_hook_pre_quit),
        ("append_hook_post_page_enter", append_hook_post_page_enter),
    ])
}
