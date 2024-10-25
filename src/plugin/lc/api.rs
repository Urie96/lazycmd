use crate::{Event, PageEntry, PluginRunner};
use mlua::prelude::*;

pub(super) fn new_table(p: &PluginRunner) -> mlua::Result<LuaTable> {
    let lua = &p.lua;
    let lc = lua.create_table_with_capacity(0, 20)?;

    let sender = p.event_sender.clone();
    lc.raw_set(
        "page_set_entries",
        lua.create_function(move |_, entries: Vec<PageEntry>| {
            sender.send(Event::PageSetEntries(entries)).unwrap();
            Ok(())
        })?,
    )?;

    let sender = p.event_sender.clone();
    lc.raw_set(
        "go_to",
        lua.create_function(move |_, path: Vec<String>| {
            sender.send(Event::Enter(path)).unwrap();
            Ok(())
        })?,
    )?;

    let state = p.state.clone();
    lc.raw_set(
        "get_current_path",
        lua.create_function(move |_, (): ()| Ok(state.borrow().current_path.clone()))?,
    )?;

    Ok(lc)
}
