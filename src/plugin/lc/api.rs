use crate::{events::EventSender, Event, PageEntry, State};
use mlua::prelude::*;

pub(super) fn new_table(lua: &Lua) -> mlua::Result<LuaTable> {
    let page_set_entries = lua
        .create_function(|lua, entries: Vec<PageEntry>| {
            lua.named_registry_value::<LuaAnyUserData>("state")?
                .borrow_mut_scoped::<State, _>(|state| state.set_current_page_entries(entries))
        })?
        .into_lua(lua)?;

    let go_to = lua
        .create_function(|lua, path: Vec<String>| {
            lua.named_registry_value::<LuaAnyUserData>("sender")?
                .borrow_scoped::<EventSender, _>(|sender| sender.send(Event::Enter(path)).unwrap())
        })?
        .into_lua(lua)?;

    let get_current_path = lua
        .create_function(|lua, ()| {
            lua.named_registry_value::<LuaAnyUserData>("state")?
                .borrow_scoped::<State, _>(|state| state.current_path.clone())
        })?
        .into_lua(lua)?;

    lua.create_table_from([
        ("page_set_entries", page_set_entries),
        ("go_to", go_to),
        ("get_current_path", get_current_path),
    ])
}
