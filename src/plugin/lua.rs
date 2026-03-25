use mlua::Lua;

use super::lc;

pub fn init_lua(lua: &Lua) -> mlua::Result<()> {
    lc::register(lua)?;

    macro_rules! preset {
        ($name:literal) => {{
            #[cfg(debug_assertions)]
            {
                std::fs::read(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/preset/lua/",
                    $name,
                    ".lua"
                ))
                .expect(concat!("Failed to read preset", $name, ".lua'"))
            }
            #[cfg(not(debug_assertions))]
            {
                &include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/preset/lua/",
                    $name,
                    ".lua"
                ))[..]
            }
        }};
    }

    // Load preset files
    macro_rules! load_preset {
        ($name:literal) => {{
            lua.load(preset!($name))
                .set_name(concat!("preset/lua/", $name, ".lua"))
                .call::<()>(())
        }};
    }

    load_preset!("system")?;
    load_preset!("component")?;
    load_preset!("api")?;
    load_preset!("style")?;
    load_preset!("interactive")?;
    load_preset!("string")?;
    load_preset!("inspect")?;
    load_preset!("json")?;
    load_preset!("time")?;
    load_preset!("keymap")?;
    load_preset!("http")?;
    load_preset!("cache")?;
    load_preset!("fs")?;
    load_preset!("util")?;
    load_preset!("base64")?;
    load_preset!("clipboard")?;
    load_preset!("yaml")?;
    load_preset!("plugin_manager")?;
    load_preset!("manager")?;
    load_preset!("default_keymap")?;
    load_preset!("init")?;
    Ok(())
}
