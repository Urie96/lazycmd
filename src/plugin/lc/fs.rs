use crate::PluginRunner;
use mlua::prelude::*;

pub(super) fn new_table(p: &PluginRunner) -> mlua::Result<LuaTable> {
    let lua = &p.lua;
    let lc = lua.create_table_with_capacity(0, 5)?;

    // lc.raw_set(
    //     "read_dir_sync",
    //     lua.create_async_function(|_, (path): String| async {
    //         let mut read_dir = tokio::fs::read_dir(path).await?;
    //
    //         while let Some(entry) = read_dir.next_entry().await? {
    //             println!("tokio: {}", entry.path().display());
    //         }
    //         Ok(())
    //     })?,
    // )?;

    lc.raw_set(
        "read_dir_sync",
        lua.create_function(|lua, path: String| {
            let entries = std::fs::read_dir(path)?
                .map(|v| {
                    v.into_lua_err().and_then(|e| {
                        let tbl = lua.create_table_with_capacity(0, 2)?;
                        tbl.raw_set("name", e.file_name())?;
                        tbl.raw_set("is_dir", e.path().is_dir())?;
                        Ok(tbl)
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            lua.create_sequence_from(entries)
        })?,
    )?;
    Ok(lc)
}
