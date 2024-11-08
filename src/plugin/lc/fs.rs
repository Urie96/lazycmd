use mlua::prelude::*;

pub(super) fn new_table(lua: &Lua) -> mlua::Result<LuaTable> {
    let read_dir_sync = lua
        .create_function(|lua, path: String| {
            let f = || {
                std::fs::read_dir(path)?
                    .map(|v| {
                        v.into_lua_err().and_then(|e| {
                            let tbl = lua.create_table_with_capacity(0, 2)?;
                            tbl.raw_set("name", e.file_name())?;
                            tbl.raw_set("is_dir", e.path().is_dir())?;
                            Ok(tbl)
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
            };
            match f() {
                Ok(entries) => (entries, LuaNil).into_lua_multi(lua),
                Err(e) => (LuaNil, e.to_string()).into_lua_multi(lua),
            }
        })?
        .into_lua(lua)?;

    lua.create_table_from([("read_dir_sync", read_dir_sync)])
}
