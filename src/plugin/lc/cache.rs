use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Get the cache file path
fn get_cache_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/state/lazycmd/cache.json")
    } else {
        PathBuf::from("/tmp/lazycmd_cache.json")
    }
}

/// Cache entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    value: serde_json::Value,
    expires: Option<u64>, // Unix timestamp
}

/// Load cache from disk
fn load_cache() -> mlua::Result<HashMap<String, CacheEntry>> {
    let cache_path = get_cache_path();

    if !cache_path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(&cache_path)
        .into_lua_err()
        .map_err(|e| LuaError::RuntimeError(format!("Failed to read cache file: {}", e)))?;

    serde_json::from_str(&content)
        .into_lua_err()
        .map_err(|e| LuaError::RuntimeError(format!("Failed to parse cache file: {}", e)))
}

/// Save cache to disk
fn save_cache(cache: &HashMap<String, CacheEntry>) -> mlua::Result<()> {
    let cache_path = get_cache_path();

    // Ensure directory exists
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)
            .into_lua_err()
            .map_err(|e| LuaError::RuntimeError(format!("Failed to create cache directory: {}", e)))?;
    }

    // Remove expired entries before saving
    let now = chrono::Utc::now().timestamp() as u64;
    let cleaned: HashMap<String, CacheEntry> = cache
        .iter()
        .filter(|(_, entry)| {
            entry
                .expires
                .map(|exp| exp > now)
                .unwrap_or(true)
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let json = serde_json::to_string_pretty(&cleaned)
        .into_lua_err()
        .map_err(|e| LuaError::RuntimeError(format!("Failed to serialize cache: {}", e)))?;

    std::fs::write(&cache_path, json)
        .into_lua_err()
        .map_err(|e| LuaError::RuntimeError(format!("Failed to write cache file: {}", e)))?;

    Ok(())
}

/// Convert Lua value to JSON value
fn lua_to_json(value: LuaValue) -> mlua::Result<serde_json::Value> {
    match value {
        LuaValue::Nil => Ok(serde_json::Value::Null),
        LuaValue::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        LuaValue::Integer(n) => Ok(serde_json::Value::Number(n.into())),
        LuaValue::Number(n) => Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(n).unwrap_or(serde_json::Number::from(0)),
        )),
        LuaValue::String(s) => Ok(serde_json::Value::String(s.to_string_lossy().to_string())),
        LuaValue::Table(t) => {
            // Try to detect if it's an array or object
            let len = t.len()?;
            let is_array = (1..=len).all(|i| t.contains_key(i).unwrap_or(false));

            if is_array && len > 0 {
                // Array
                let mut arr = Vec::new();
                for i in 1..=len {
                    let val = t.get(i)?;
                    arr.push(lua_to_json(val)?);
                }
                Ok(serde_json::Value::Array(arr))
            } else {
                // Object
                let mut obj = serde_json::Map::new();
                for pair in t.pairs::<String, LuaValue>() {
                    let (k, v) = pair?;
                    obj.insert(k, lua_to_json(v)?);
                }
                Ok(serde_json::Value::Object(obj))
            }
        }
        _ => Err(LuaError::RuntimeError(format!(
            "Unsupported type for cache: {:?}",
            value.type_name()
        ))),
    }
}

/// Convert JSON value to Lua value
fn json_to_lua(lua: &Lua, value: serde_json::Value) -> mlua::Result<LuaValue> {
    match value {
        serde_json::Value::Null => Ok(LuaValue::Nil),
        serde_json::Value::Bool(b) => Ok(LuaValue::Boolean(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Ok(LuaValue::Number(0.0))
            }
        }
        serde_json::Value::String(s) => Ok(lua.create_string(&s)?.into_lua(lua)?),
        serde_json::Value::Array(arr) => {
            let tbl = lua.create_table_with_capacity(arr.len(), 0)?;
            for (i, v) in arr.into_iter().enumerate() {
                tbl.raw_set(i + 1, json_to_lua(lua, v)?)?;
            }
            Ok(LuaValue::Table(tbl))
        }
        serde_json::Value::Object(obj) => {
            let tbl = lua.create_table_with_capacity(0, obj.len())?;
            for (k, v) in obj {
                tbl.raw_set(k, json_to_lua(lua, v)?)?;
            }
            Ok(LuaValue::Table(tbl))
        }
    }
}

pub(super) fn new_table(lua: &Lua) -> mlua::Result<LuaTable> {
    let get = lua
        .create_function(|lua, key: String| -> mlua::Result<LuaValue> {
            let mut cache = load_cache()?;

            let now = chrono::Utc::now().timestamp() as u64;

            if let Some(entry) = cache.get(&key) {
                // Check if expired
                if let Some(expires) = entry.expires {
                    if expires <= now {
                        // Remove expired entry
                        cache.remove(&key);
                        save_cache(&cache)?;
                        return Ok(LuaValue::Nil);
                    }
                }
                return json_to_lua(lua, entry.value.clone());
            }

            Ok(LuaValue::Nil)
        })?
        .into_lua(lua)?;

    let set = lua
        .create_function(|_lua, (key, value, opts): (String, LuaValue, Option<LuaTable>)| {
            let mut cache = load_cache()?;

            let json_value = lua_to_json(value)?;

            let expires = if let Some(opts) = opts {
                let ttl: Option<u64> = opts.get("ttl").ok();
                ttl.map(|ttl| {
                    chrono::Utc::now().timestamp() as u64 + ttl
                })
            } else {
                None
            };

            cache.insert(
                key,
                CacheEntry {
                    value: json_value,
                    expires,
                },
            );

            save_cache(&cache)?;
            Ok(())
        })?
        .into_lua(lua)?;

    let delete = lua
        .create_function(|_, key: String| -> mlua::Result<()> {
            let mut cache = load_cache()?;
            cache.remove(&key);
            save_cache(&cache)?;
            Ok(())
        })?
        .into_lua(lua)?;

    let clear = lua
        .create_function(|_lua, ()| -> mlua::Result<()> {
            let cache_path = get_cache_path();

            // If cache file exists, delete it
            if cache_path.exists() {
                std::fs::remove_file(&cache_path)
                    .into_lua_err()
                    .map_err(|e| LuaError::RuntimeError(format!("Failed to delete cache file: {}", e)))?;
            }

            Ok(())
        })?
        .into_lua(lua)?;

    lua.create_table_from([
        ("get", get),
        ("set", set),
        ("delete", delete),
        ("clear", clear),
    ])
}
