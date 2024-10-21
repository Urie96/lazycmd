mod page_set_entries;

use mlua::{prelude::*, Lua};

pub fn new(lua: &Lua) -> mlua::Result<LuaTable> {
    let tbl = lua.create_table()?;
    Ok(tbl)
}
