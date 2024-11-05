use mlua::Lua;

use crate::{events::EventSender, State};

use super::lc;

pub fn new_lua(state: &mut State, sender: &EventSender) -> mlua::Result<Lua> {
    let lua = Lua::new();
    lc::register(&lua)?;

    macro_rules! preset {
        ($name:literal) => {{
            #[cfg(debug_assertions)]
            {
                std::fs::read(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/preset/",
                    $name,
                    ".lua"
                ))
                .expect(concat!(
                    "Failed to read 'yazi-plugin/preset/",
                    $name,
                    ".lua'"
                ))
            }
            #[cfg(not(debug_assertions))]
            {
                &include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/preset/",
                    $name,
                    ".lua"
                ))[..]
            }
        }};
    }
    super::scope(&lua, state, sender, || {
        lua.load(preset!("init")).call::<()>(())
    })?;

    Ok(lua)
}
