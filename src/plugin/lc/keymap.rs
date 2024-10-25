use crate::{Event, Keymap, PluginRunner};
use mlua::prelude::*;
use std::rc::Rc;

pub(super) fn new_table(p: &PluginRunner) -> mlua::Result<LuaTable> {
    let lua = &p.lua;
    let lc = lua.create_table_with_capacity(0, 5)?;

    let sender = p.event_sender.clone();
    lc.raw_set(
        "set",
        lua.create_function(move |lua, (key, cb): (String, LuaValue)| {
            sender
                .send(Event::AddKeymap(Keymap {
                    mode: None,
                    key_sequence: key.as_str().into(),
                    callback: match cb {
                        LuaValue::Function(f) => Rc::new(move || {
                            let f = f.clone();
                            Box::pin(async move {
                                if let Err(e) = f.clone().call_async::<()>(()).await {
                                    println!("{e}"); // todo
                                }
                            })
                        }),
                        other => {
                            let cmd = String::from_lua(other, lua).unwrap();
                            let sender = sender.clone();
                            Rc::new(move || {
                                sender.send(Event::Command(cmd.clone())).unwrap();
                                Box::pin(async {})
                            })
                        }
                    },
                }))
                .unwrap();
            Ok(())
        })?,
    )?;
    Ok(lc)
}
