mod api;
mod fs;
mod keymap;
mod ui;

use crate::{
    events::{EventName, EventSender},
    Event,
};
use mlua::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
    let keymap = keymap::new_table(lua)?.into_lua(lua)?;
    let api = api::new_table(lua)?.into_lua(lua)?;
    let fs = fs::new_table(lua)?.into_lua(lua)?;
    let defer_fn = lua
        .create_function(|lua, (f, ms): (LuaFunction, u64)| {
            let sender = lua
                .named_registry_value::<LuaAnyUserData>("sender")?
                .borrow_scoped::<EventSender, _>(|sender| sender.clone())?;

            tokio::task::spawn_local(async move {
                sleep(Duration::from_millis(ms)).await;
                sender
                    .send(Event::LuaCallback(Box::new(move || {
                        f.call::<()>(()).unwrap();
                    })))
                    .unwrap();
            });
            Ok(())
        })?
        .into_lua(lua)?;

    let cmd = lua
        .create_function(move |lua, cmd: String| {
            lua.named_registry_value::<LuaAnyUserData>("sender")?
                .borrow_scoped::<EventSender, _>(|sender| sender.send(Event::Command(cmd)).unwrap())
        })?
        .into_lua(lua)?;

    let on_event = lua
        .create_function(move |lua, (event_name, cb): (EventName, LuaFunction)| {
            lua.named_registry_value::<LuaAnyUserData>("sender")?
                .borrow_scoped::<EventSender, _>(|sender| {
                    sender.send(Event::AddEventHook(event_name, cb)).unwrap()
                })
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
    ])?;
    lua.globals().raw_set("lc", lc)
}
