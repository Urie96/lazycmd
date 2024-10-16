use anyhow::Result;
use mlua::{prelude::*, Lua};

use crate::ro_cell::RoCell;

pub static LUA: RoCell<Lua> = RoCell::new();

pub(super) fn init_lua() -> Result<()> {
    LUA.init(Lua::new());
    let lc = LUA.create_table_from([(
        "event",
        LUA.create_table_from([(
            "on",
            LuaFunction::wrap_raw(|| {
                println!("1");
            }),
        )])?,
    )])?;
    let globals = LUA.globals();
    Ok(())
}
