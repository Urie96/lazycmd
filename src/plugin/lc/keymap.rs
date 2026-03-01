use crate::{plugin, Keymap, Mode};
use mlua::prelude::*;

pub(super) fn new_table(lua: &Lua) -> mlua::Result<LuaTable> {
    let set = lua
        .create_function(|lua, (mode, key, cb): (Mode, String, LuaValue)| {
            // Convert the callback to a LuaFunction
            let callback = match cb {
                LuaValue::String(s) => {
                    // If it's a string, wrap it as: function() lc.cmd(s) end
                    let cmd_str = s.to_str()?.to_string();
                    let lc = lua.globals().get::<LuaTable>("lc")?;
                    let cmd_fn = lc.get::<LuaFunction>("cmd")?;
                    lua.create_function(move |_lua, ()| {
                        cmd_fn.call::<()>(cmd_str.clone())
                    })?
                }
                LuaValue::Function(f) => f,
                other => {
                    return Err(LuaError::RuntimeError(format!(
                        "keymap callback must be a string or function, got {}",
                        other.type_name()
                    )))
                }
            };

            plugin::mut_scope_state(lua, |state| {
                state.add_keymap(Keymap {
                    mode,
                    key_sequence: key.as_str().into(),
                    callback,
                });
                Ok(())
            })
        })?
        .into_lua(lua)?;
    lua.create_table_from([("set", set)])
}
