use mlua::Lua;

use super::lc;

pub fn init_lua(lua: &Lua) -> mlua::Result<()> {
    // Set package.path based on build mode
    let package: mlua::Table = lua.globals().get("package")?;
    let current_path: String = package.get("path")?;

    #[cfg(debug_assertions)]
    {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let new_path = format!(
            "{};{}/examples/?.lua;{}/examples/plugins/?.lazycmd/?/init.lua;",
            current_path, manifest_dir, manifest_dir
        );
        package.set("path", new_path)?;
    }

    #[cfg(not(debug_assertions))]
    {
        if let Ok(home) = std::env::var("HOME") {
            let new_path = format!(
                "{};{}/.config/lazycmd/?.lua;{}/.config/lazycmd/plugins/?.lazycmd/?/init.lua;",
                current_path, home, home
            );
            package.set("path", new_path)?;
        }
    }

    lc::register(lua)?;

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
                .expect(concat!("Failed to read preset", $name, ".lua'"))
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

    lua.load(preset!("util"))
        .set_name("preset/util.lua")
        .call::<()>(())?;

    lua.load(preset!("init"))
        .set_name("preset/init.lua")
        .call::<()>(())
}
