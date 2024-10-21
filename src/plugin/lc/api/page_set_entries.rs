use mlua::{prelude::*, Lua};

use crate::{events, Event, Keymap, STATE};

pub fn new(lua: &Lua) -> mlua::Result<LuaFunction> {
    lua.create_function(move |lua, (key, cb): (String, LuaValue)| {
        STATE.borrow_mut().add_keymap(Keymap {
            mode: None,
            key_sequence: key.as_str().into(),
            callback: match cb {
                LuaValue::Function(f) => Box::new(move || {
                    f.call::<()>(())?;
                    Ok(())
                }),
                other => {
                    let cmd = String::from_lua(other, lua)?;
                    Box::new(move || {
                        events::emit(Event::Command(cmd.clone()));
                        Ok(())
                    })
                }
            },
        });
        Ok(())
    })
}
