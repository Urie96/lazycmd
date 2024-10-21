use mlua::{prelude::*, Lua};

use super::lc;
use crate::preset;
use crate::ro_cell::RoCell;

pub static LUA: RoCell<Lua> = RoCell::new();

pub fn init() -> mlua::Result<()> {
    let lua = Lua::new();
    lc::set_lua_global(&lua)?;

    let m: LuaTable = lua.load(preset!("init")).call(())?;

    LUA.init(lua);
    Ok(())
}
