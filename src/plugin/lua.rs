use mlua::Lua;

use super::lc;

pub fn init_lua(lua: &Lua) -> mlua::Result<()> {
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
    lua.load(preset!("init")).call(())
}
