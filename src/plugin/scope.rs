use mlua::{prelude::*, Lua};

use crate::{events::EventSender, Event, State};

pub fn scope<R>(
    lua: &Lua,
    state: &mut State,
    sender: &EventSender,
    f: impl FnOnce() -> mlua::Result<R>,
) -> mlua::Result<R> {
    lua.scope(|scope| {
        lua.set_named_registry_value("state", scope.create_any_userdata_ref_mut(state)?)?;
        lua.set_named_registry_value("sender", scope.create_any_userdata_ref(sender)?)?;
        f()
    })
}

pub(super) fn send_event(lua: &Lua, e: Event) -> LuaResult<()> {
    lua.named_registry_value::<LuaAnyUserData>("sender")?
        .borrow_scoped::<EventSender, _>(|sender| sender.send(e).unwrap())
}

pub(super) fn clone_sender(lua: &Lua) -> LuaResult<EventSender> {
    lua.named_registry_value::<LuaAnyUserData>("sender")?
        .borrow_scoped::<EventSender, _>(|sender| sender.clone())
}

pub(super) fn borrow_scope_state<R>(
    lua: &Lua,
    f: impl FnOnce(&State) -> LuaResult<R>,
) -> LuaResult<R> {
    lua.named_registry_value::<LuaAnyUserData>("state")?
        .borrow_scoped::<State, _>(f)?
}

pub(super) fn mut_scope_state<R>(
    lua: &Lua,
    f: impl FnOnce(&mut State) -> LuaResult<R>,
) -> LuaResult<R> {
    lua.named_registry_value::<LuaAnyUserData>("state")?
        .borrow_mut_scoped::<State, _>(f)?
}
