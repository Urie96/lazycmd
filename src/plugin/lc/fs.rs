use mlua::prelude::*;

/// Check if a path is readable
fn is_readable(path: &std::path::Path) -> bool {
    // Try to open with read-only mode
    match std::fs::OpenOptions::new().read(true).open(path) {
        Ok(file) => {
            // Successfully opened, check if we can actually read metadata
            file.metadata().is_ok()
        }
        Err(_) => false,
    }
}

/// Check if a path is writable
fn is_writable(path: &std::path::Path) -> bool {
    // Try to open with write mode (without truncating)
    match std::fs::OpenOptions::new().write(true).create(false).open(path) {
        Ok(file) => {
            // Successfully opened, can write
            drop(file);
            true
        }
        Err(e) => {
            // If the file doesn't exist, check if we can create it in the parent directory
            if e.kind() == std::io::ErrorKind::NotFound {
                if let Some(parent) = path.parent() {
                    return is_writable(parent);
                }
            }
            false
        }
    }
}

/// Check if a path is executable
fn is_executable(path: &std::path::Path) -> bool {
    // On Unix systems, check file permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(path) {
            Ok(metadata) => {
                let permissions = metadata.permissions();
                let mode = permissions.mode();
                // Check execute bits (owner, group, or others)
                (mode & 0o111) != 0
            }
            Err(_) => false,
        }
    }

    #[cfg(windows)]
    {
        // On Windows, check file extension or use other methods
        // This is a simplified check - Windows executable detection is more complex
        match path.extension() {
            Some(ext) => {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                matches!(
                    ext_lower.as_str(),
                    "exe" | "bat" | "cmd" | "ps1" | "com" | "msi" | "sh"
                )
            }
            None => false,
        }
    }
}

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

    let read_file_sync = lua
        .create_function(|_, path: String| -> mlua::Result<(String, Option<String>)> {
            match std::fs::read_to_string(&path) {
                Ok(content) => Ok((content, None)),
                Err(e) => Ok((String::new(), Some(e.to_string()))),
            }
        })?
        .into_lua(lua)?;

    let write_file_sync = lua
        .create_function(
            |_, (path, content): (String, String)| -> mlua::Result<(bool, Option<String>)> {
                match std::fs::write(&path, content) {
                    Ok(_) => Ok((true, None)),
                    Err(e) => Ok((false, Some(e.to_string()))),
                }
            },
        )?
        .into_lua(lua)?;

    let stat_sync = lua.create_function(|lua, path: String| -> mlua::Result<LuaTable> {
        let path_obj = std::path::Path::new(&path);
        let exists = path_obj.exists();
        let (is_file, is_dir) = if exists {
            let metadata = std::fs::metadata(&path).ok();
            (
                metadata.as_ref().map(|m| m.is_file()).unwrap_or(false),
                metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false),
            )
        } else {
            (false, false)
        };

        let is_readable = exists && is_readable(path_obj);
        let is_writable = is_writable(path_obj);
        let is_executable = exists && is_executable(path_obj);

        lua.create_table_from([
            ("exists", exists.into_lua(lua)?),
            ("is_file", is_file.into_lua(lua)?),
            ("is_dir", is_dir.into_lua(lua)?),
            ("is_readable", is_readable.into_lua(lua)?),
            ("is_writable", is_writable.into_lua(lua)?),
            ("is_executable", is_executable.into_lua(lua)?),
        ])
    })?
    .into_lua(lua)?;

    let mkdir_sync = lua
        .create_function(
            |_, path: String| -> mlua::Result<(bool, Option<String>)> {
                match std::fs::create_dir_all(&path) {
                    Ok(_) => Ok((true, None)),
                    Err(e) => Ok((false, Some(e.to_string()))),
                }
            },
        )?
        .into_lua(lua)?;

    lua.create_table_from([
        ("read_dir_sync", read_dir_sync),
        ("read_file_sync", read_file_sync),
        ("write_file_sync", write_file_sync),
        ("stat", stat_sync),
        ("mkdir", mkdir_sync),
    ])
}
