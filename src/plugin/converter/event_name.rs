use mlua::{ExternalError, FromLua};

use crate::events::EventName;

impl FromLua for EventName {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        Ok(match String::from_lua(value, lua)?.as_str() {
            "enter" => EventName::Enter,
            other => Err(format!("Unable to cast string '{other}' into EventName").into_lua_err())?,
        })
    }
}
