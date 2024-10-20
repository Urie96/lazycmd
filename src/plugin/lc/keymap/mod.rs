mod set;

use mlua::{prelude::*, Lua};

pub fn new(lua: &Lua) -> mlua::Result<LuaTable> {
    let tbl = lua.create_table()?;
    tbl.raw_set("set", set::new(lua)?)?;
    Ok(tbl)
}
