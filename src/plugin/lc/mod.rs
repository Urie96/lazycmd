mod cmd;
mod defer_fn;
mod keymap;

use mlua::Lua;

pub fn set_lua_global(lua: &Lua) -> mlua::Result<()> {
    let lc = lua.create_table()?;
    lc.raw_set("defer_fn", defer_fn::new(lua)?)?;
    lc.raw_set("cmd", cmd::new(lua)?)?;
    lc.raw_set("keymap", keymap::new(lua)?)?;

    lua.globals().set("lc", lc)?;
    Ok(())
}
