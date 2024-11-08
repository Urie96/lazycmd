mod api;
mod fs;
mod keymap;
mod path;
mod ui;

use crate::{events::EventHook, plugin, Event};
use mlua::prelude::*;
use std::time::Duration;
use tokio::{process::Command, time::sleep};

pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
    let keymap = keymap::new_table(lua)?.into_lua(lua)?;
    let api = api::new_table(lua)?.into_lua(lua)?;
    let fs = fs::new_table(lua)?.into_lua(lua)?;
    let path = path::new_table(lua)?.into_lua(lua)?;
    let defer_fn = lua
        .create_function(|lua, (f, ms): (LuaFunction, u64)| {
            let sender = plugin::clone_sender(lua)?;

            tokio::task::spawn_local(async move {
                sleep(Duration::from_millis(ms)).await;
                sender
                    .send(Event::LuaCallback(Box::new(move |_| f.call(()))))
                    .unwrap();
            });
            Ok(())
        })?
        .into_lua(lua)?;

    let cmd = lua
        .create_function(|lua, cmd: String| plugin::send_event(lua, Event::Command(cmd)))?
        .into_lua(lua)?;

    let command_fn = lua
        .create_function(|lua, (cmd, on_exit): (Vec<String>, LuaFunction)| {
            let sender = plugin::clone_sender(lua)?;

            tokio::task::spawn_local(async move {
                let mut it = cmd.into_iter();
                let output = Command::new(it.next().unwrap()).args(it).output().await;
                sender
                    .send(Event::LuaCallback(Box::new(move |lua| {
                        let out = output.into_lua_err().and_then(|out| {
                            lua.create_table_from([
                                ("code", out.status.code().into_lua(lua)?),
                                ("stdout", lua.create_string(out.stdout)?.into_lua(lua)?),
                                ("stderr", lua.create_string(out.stderr)?.into_lua(lua)?),
                            ])
                        })?;
                        // let b = a;
                        on_exit.call(out)
                    })))
                    .unwrap();
            });

            Ok(())
        })?
        .into_lua(lua)?;

    let on_event = lua
        .create_function(|lua, (event_name, cb): (EventHook, LuaFunction)| {
            plugin::send_event(lua, Event::AddEventHook(event_name, cb))
        })?
        .into_lua(lua)?;

    let split = lua
        .create_function(|lua, (s, sep): (String, String)| lua.create_sequence_from(s.split(&sep)))?
        .into_lua(lua)?;

    ui::inject_string_meta_method(lua)?;

    let lc = lua.create_table_from([
        ("defer_fn", defer_fn),
        ("keymap", keymap),
        ("api", api),
        ("fs", fs),
        ("cmd", cmd),
        ("on_event", on_event),
        ("split", split),
        ("system", command_fn),
        ("path", path),
    ])?;
    lua.globals().raw_set("lc", lc)
}
