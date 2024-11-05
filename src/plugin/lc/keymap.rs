use crate::{Keymap, Mode, State};
use mlua::prelude::*;

pub(super) fn new_table(lua: &Lua) -> mlua::Result<LuaTable> {
    let set = lua
        .create_function(|lua, (mode, key, cb): (Mode, String, LuaFunction)| {
            lua.named_registry_value::<LuaAnyUserData>("state")?
                .borrow_mut_scoped::<State, _>(|state| {
                    state.add_keymap(Keymap {
                        mode,
                        key_sequence: key.as_str().into(),
                        callback: cb,
                    })
                })
        })?
        .into_lua(lua)?;
    lua.create_table_from([("set", set)])
}
