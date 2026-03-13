use mlua::Lua;

use super::lc;

/// Helper function to add plugin directories to package.path
fn add_plugin_paths(package: &mlua::Table, base_path: &str) -> mlua::Result<()> {
    let current_path: String = package.get("path")?;
    let mut new_paths = Vec::new();

    // Add base paths
    new_paths.push(format!("{}/?.lua", base_path));
    // Plugin entry points: examples/plugins/systemd.lazycmd/systemd/init.lua
    new_paths.push(format!("{}/plugins/?.lazycmd/?/init.lua", base_path));

    // Traverse plugin directory and add each plugin's internal directory
    // This enables: require('systemd.action') → systemd.lazycmd/systemd/action.lua
    let plugins_dir = format!("{}/plugins", base_path);
    if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(plugin_name) = path.file_name() {
                    if let Some(plugin_name) = plugin_name.to_str() {
                        if plugin_name.ends_with(".lazycmd") {
                            // Add plugin internal module path
                            // e.g., examples/plugins/systemd.lazycmd/systemd/?.lua
                            // This allows: require('systemd.action') → systemd.lazycmd/systemd/action.lua
                            new_paths.push(format!("{}/plugins/{}/?.lua", base_path, plugin_name));
                        }
                    }
                }
            }
        }
    }

    let new_path = format!("{};{};", current_path, new_paths.join(";"));
    package.set("path", new_path)
}

pub fn init_lua(lua: &Lua) -> mlua::Result<()> {
    // Set package.path based on build mode
    let package: mlua::Table = lua.globals().get("package")?;

    #[cfg(debug_assertions)]
    {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        add_plugin_paths(&package, &format!("{}/examples", manifest_dir))?;
    }

    #[cfg(not(debug_assertions))]
    {
        if let Ok(home) = std::env::var("HOME") {
            add_plugin_paths(&package, &format!("{}/.config/lazycmd", home))?;
        }
    }

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
    load_preset!("yaml")?;
    load_preset!("init")?;
    Ok(())
}
