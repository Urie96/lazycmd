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
    load_preset!("copy_from_neovim")?;
    load_preset!("socket")?;
    load_preset!("component")?;
    load_preset!("api")?;
    load_preset!("style")?;
    load_preset!("interactive")?;
    load_preset!("string")?;
    load_preset!("inspect")?;
    load_preset!("json")?;
    load_preset!("promise")?;
    load_preset!("time")?;
    load_preset!("keymap")?;
    load_preset!("html")?;
    load_preset!("http")?;
    load_preset!("http_server")?;
    load_preset!("cache")?;
    load_preset!("fs")?;
    load_preset!("util")?;
    load_preset!("base64")?;
    load_preset!("url")?;
    load_preset!("clipboard")?;
    load_preset!("secrets")?;
    load_preset!("yaml")?;
    load_preset!("plugin_manager")?;
    load_preset!("manager")?;
    load_preset!("config")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn util_preset_provides_unpack_compatibility() -> mlua::Result<()> {
        let lua = Lua::new();
        let globals = lua.globals();
        globals.set("lc", lua.create_table()?)?;

        let raw_lc = lua.create_table()?;
        raw_lc.set(
            "osc52_copy",
            lua.create_function(|_, _text: String| Ok(()))?,
        )?;
        globals.set("_lc", raw_lc)?;

        lua.load(&include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/preset/lua/util.lua"))[..])
            .set_name("preset/lua/util.lua")
            .exec()?;

        let table_unpack_exists: bool =
            lua.load("return type(table.unpack) == 'function'").eval()?;
        let unpack_exists: bool = lua.load("return type(unpack) == 'function'").eval()?;
        let unpack_works: i64 = lua
            .load("return select(2, table.unpack({ 10, 20, 30 }))")
            .eval()?;

        assert!(table_unpack_exists);
        assert!(unpack_exists);
        assert_eq!(unpack_works, 20);

        Ok(())
    }

    #[test]
    fn string_preset_utf8_sub_works_without_utf8_global() -> mlua::Result<()> {
        let lua = Lua::new();
        let globals = lua.globals();

        globals.set("lc", lua.create_table()?)?;

        let raw_lc = lua.create_table()?;
        let style = lua.create_table()?;
        style.set("span", lua.create_function(|_, s: String| Ok(s))?)?;
        style.set("ansi", lua.create_function(|_, s: String| Ok(s))?)?;
        raw_lc.set("style", style)?;
        raw_lc.set(
            "split",
            lua.create_function(|lua, (s, sep): (String, String)| {
                let parts = lua.create_table()?;
                for (idx, part) in s.split(&sep).enumerate() {
                    parts.set(idx + 1, part)?;
                }
                Ok(parts)
            })?,
        )?;
        globals.set("_lc", raw_lc)?;
        globals.set("utf8", mlua::Value::Nil)?;

        lua.load(
            &include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/preset/lua/string.lua"
            ))[..],
        )
        .set_name("preset/lua/string.lua")
        .exec()?;

        let result: String = lua
            .load(r#"return string.utf8_sub("Hello 世界！🌍", 7, 8)"#)
            .eval()?;
        let tail: String = lua
            .load(r#"return string.utf8_sub("Hello 世界！🌍", -3, -1)"#)
            .eval()?;

        assert_eq!(result, "世界");
        assert_eq!(tail, "界！🌍");

        Ok(())
    }
}
