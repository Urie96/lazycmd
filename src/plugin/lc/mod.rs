mod api;
mod fs;
mod keymap;

use crate::{events::EventName, Event, PluginRunner};
use mlua::prelude::*;
use std::{rc::Rc, time::Duration};
use tokio::time::sleep;

pub(super) fn register(p: &PluginRunner) -> mlua::Result<()> {
    let lua = &p.lua;
    let lc = lua.create_table_with_capacity(0, 20)?;

    lc.raw_set("keymap", keymap::new_table(p)?)?;
    lc.raw_set("api", api::new_table(p)?)?;
    lc.raw_set("fs", fs::new_table(p)?)?;

    lc.raw_set(
        "defer_fn",
        lua.create_function(|_, (f, ms): (LuaFunction, u64)| {
            tokio::task::spawn_local(async move {
                sleep(Duration::from_millis(ms)).await;
                f.call_async::<()>(()).await.unwrap();
            });
            Ok(())
        })?,
    )?;

    let sender = p.event_sender.clone();
    lc.raw_set(
        "cmd",
        lua.create_function(move |_, cmd: String| {
            sender.send(Event::Command(cmd)).unwrap();
            Ok(())
        })?,
    )?;

    let sender = p.event_sender.clone();
    lc.raw_set(
        "on_event",
        lua.create_function(move |_, (event_name, cb): (EventName, LuaFunction)| {
            sender
                .send(Event::AddEventHook(
                    event_name,
                    Rc::new(move || {
                        let cb = cb.clone();
                        Box::pin(async move {
                            if let Err(e) = cb.call_async::<()>(()).await {
                                println!("{e}");
                            }
                        })
                    }),
                ))
                .unwrap();
            Ok(())
        })?,
    )?;

    p.lua.globals().set("lc", lc)?;
    Ok(())
}
