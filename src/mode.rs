use mlua::{ExternalError, FromLua};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Main,
    Input,
}

impl FromLua for Mode {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        Ok(match String::from_lua(value, lua)?.as_str() {
            "main" | "m" => Mode::Main,
            "input" | "i" => Mode::Input,
            other => Err(format!("Unable to cast string '{other}' into Mode").into_lua_err())?,
        })
    }
}
