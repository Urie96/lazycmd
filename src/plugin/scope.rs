use mlua::Lua;

use crate::{events::EventSender, State};

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
