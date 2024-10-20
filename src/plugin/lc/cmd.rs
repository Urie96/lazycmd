use mlua::{prelude::*, Lua};

use crate::{events, Event};

pub fn new(lua: &Lua) -> mlua::Result<LuaFunction> {
    lua.create_function(|_, (cmd): (String)| {
        events::emit(Event::Command(cmd));
        Ok(())
    })
}
