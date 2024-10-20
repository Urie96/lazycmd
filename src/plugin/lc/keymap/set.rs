use std::rc::Rc;

use mlua::{prelude::*, Lua};

use crate::{Keymap, State, STATE};

pub fn new(lua: &Lua) -> mlua::Result<LuaFunction> {
    lua.create_function(move |_, (key, cb): (String, LuaFunction)| {
        STATE.borrow_mut().add_keymap(Keymap {
            mode: None,
            key_sequence: key.as_str().into(),
            callback: Box::new(move || {
                cb.call::<()>(())?;
                Ok(())
            }),
        });
        Ok(())
    })
}
