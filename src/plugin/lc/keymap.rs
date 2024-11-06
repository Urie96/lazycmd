use crate::{plugin, Keymap, Mode};
use mlua::prelude::*;

pub(super) fn new_table(lua: &Lua) -> mlua::Result<LuaTable> {
    let set = lua
        .create_function(|lua, (mode, key, cb): (Mode, String, LuaFunction)| {
            plugin::mut_scope_state(lua, |state| {
                state.add_keymap(Keymap {
                    mode,
                    key_sequence: key.as_str().into(),
                    callback: cb,
                });
                Ok(())
            })
        })?
        .into_lua(lua)?;
    lua.create_table_from([("set", set)])
}
