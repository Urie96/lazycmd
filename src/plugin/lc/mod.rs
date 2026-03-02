mod api;
mod cache;
mod fs;
mod http;
mod keymap;
mod path;
mod time;
mod ui;

use crate::{plugin, Event};
use base64::Engine;
use mlua::prelude::*;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;
use tokio::{process::Command, time::sleep};

/// Load a preset Lua file (handles both debug and release builds)
macro_rules! load_preset {
    ($lua:expr, $name:literal) => {{
        #[cfg(debug_assertions)]
        {
            let content = std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/preset/",
                $name,
                ".lua"
            ))
            .expect(concat!("Failed to read preset ", $name, ".lua"));
            $lua.load(&content)
                .set_name(concat!("preset/", $name, ".lua"))
                .eval::<LuaTable>()
        }
        #[cfg(not(debug_assertions))]
        {
            $lua.load(
                &include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/preset/",
                    $name,
                    ".lua"
                ))[..],
            )
            .set_name(concat!("preset/", $name, ".lua"))
            .eval::<LuaTable>()
        }
    }};
}

/// Get the log file path for Lua plugin logs
fn get_log_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/state/lazycmd/lua.log")
    } else {
        PathBuf::from("/tmp/lazycmd.log")
    }
}

/// Write a log entry to the log file
fn write_log(level: &str, message: &str) {
    let log_path = get_log_path();

    // Ensure the directory exists
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Format the log entry with timestamp
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_entry = format!("[{}][{}] {}\n", timestamp, level, message);

    // Append to log file
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .and_then(|mut file| file.write_all(log_entry.as_bytes()));
}

pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
    let keymap = keymap::new_table(lua)?.into_lua(lua)?;
    let api = api::new_table(lua)?.into_lua(lua)?;
    let cache = cache::new_table(lua)?.into_lua(lua)?;
    let fs = fs::new_table(lua)?.into_lua(lua)?;
    let http = http::new_table(lua)?.into_lua(lua)?;
    let path = path::new_table(lua)?.into_lua(lua)?;
    let time = time::new_table(lua)?.into_lua(lua)?;

    // Load json and inspect modules from preset files
    let json_mod = load_preset!(lua, "json")?.into_lua(lua)?;
    let inspect_mod = load_preset!(lua, "inspect")?.into_lua(lua)?;

    let defer_fn = lua
        .create_function(|lua, (f, ms): (LuaFunction, u64)| {
            let sender = plugin::clone_sender(lua)?;

            tokio::task::spawn_local(async move {
                sleep(Duration::from_millis(ms)).await;
                sender
                    .send(Event::LuaCallback(Box::new(move |_| f.call(()))))
                    .unwrap();
            });
            Ok(())
        })?
        .into_lua(lua)?;

    let cmd = lua
        .create_function(|lua, cmd: String| plugin::send_event(lua, Event::Command(cmd)))?
        .into_lua(lua)?;

    // lc.system.executable: check if a command is executable (synchronous)
    let executable_fn = lua.create_function(|_, cmd: String| {
        // Check if command exists and is executable
        Ok(which::which(&cmd).is_ok())
    })?;

    // Create system table
    let system_tbl = lua.create_table()?;

    // Add executable function
    system_tbl.set("executable", executable_fn)?;

    // Add __call metamethod to support lc.system(cmd, callback) syntax
    system_tbl.set_metatable(Some({
        let mt = lua.create_table()?;
        mt.set(
            "__call",
            lua.create_function(
                |lua, (_, cmd, on_exit): (LuaValue, Vec<String>, LuaFunction)| {
                    if cmd.is_empty() {
                        return Err(LuaError::RuntimeError(
                            "Command cannot be empty".to_string(),
                        ));
                    }
                    let sender = plugin::clone_sender(lua)?;

                    tokio::task::spawn_local(async move {
                        let mut it = cmd.into_iter();

                        let command = it.next().unwrap();

                        let output = Command::new(&command).args(it).output().await;

                        let _ = sender.send(Event::LuaCallback(Box::new(move |lua| {
                            let out = match output {
                                Ok(output) => lua.create_table_from([
                                    ("code", output.status.code().into_lua(lua)?),
                                    ("stdout", lua.create_string(output.stdout)?.into_lua(lua)?),
                                    ("stderr", lua.create_string(output.stderr)?.into_lua(lua)?),
                                ]),
                                Err(e) => {
                                    let (code, err) = if e.kind() == std::io::ErrorKind::NotFound {
                                        (127, format!("command not found: {}", command))
                                    } else {
                                        (1, e.to_string())
                                    };
                                    lua.create_table_from([
                                        ("code", code.into_lua(lua)?),
                                        ("stdout", "".into_lua(lua)?),
                                        ("stderr", err.into_lua(lua)?),
                                    ])
                                }
                            };
                            let out = out?;
                            on_exit.call(out)
                        })));
                    });

                    Ok(())
                },
            )?,
        )?;
        mt
    }));

    let interactive_fn = lua
        .create_function(
            |lua, (cmd, on_complete): (Vec<String>, Option<LuaFunction>)| {
                if cmd.is_empty() {
                    return Err(LuaError::RuntimeError(
                        "Command cannot be empty".to_string(),
                    ));
                }
                plugin::send_event(lua, Event::InteractiveCommand(cmd, on_complete))
            },
        )?
        .into_lua(lua)?;

    let split = lua
        .create_function(|lua, (s, sep): (String, String)| lua.create_sequence_from(s.split(&sep)))?
        .into_lua(lua)?;

    let log_fn = lua
        .create_function(
            |lua, (level, format, args): (String, LuaString, LuaMultiValue)| {
                // Convert all args to strings
                let mut arg_strings = Vec::new();
                for arg in args {
                    match String::from_lua(arg, lua) {
                        Ok(s) => arg_strings.push(s),
                        Err(_) => arg_strings.push("[unconvertible]".to_string()),
                    }
                }

                // Format the message using the format string and args
                let message = if arg_strings.is_empty() {
                    format.to_string_lossy().to_string()
                } else {
                    // Simple format: replace {} with args sequentially
                    let fmt_str = format.to_string_lossy().to_string();
                    let mut result = fmt_str.clone();
                    let mut arg_idx = 0;
                    while let Some(pos) = result.find("{}") {
                        if arg_idx < arg_strings.len() {
                            result.replace_range(pos..pos + 2, &arg_strings[arg_idx]);
                            arg_idx += 1;
                        } else {
                            break;
                        }
                    }
                    result
                };

                write_log(&level, &message);
                Ok(())
            },
        )?
        .into_lua(lua)?;

    let osc52_copy = lua
        .create_function(|_, text: String| {
            // Encode text as base64
            let encoded = base64::engine::general_purpose::STANDARD.encode(&text);

            // Build OSC 52 escape sequence: ESC ] 52 ; c ; <base64_data> BEL
            let osc_sequence = format!("\x1b]52;c;{}\x07", encoded);

            // Write to terminal stdout
            if let Err(e) = io::stdout().write_all(osc_sequence.as_bytes()) {
                return Err(LuaError::RuntimeError(format!(
                    "Failed to write OSC 52 sequence: {}",
                    e
                )));
            }

            // Flush to ensure the sequence is sent
            if let Err(e) = io::stdout().flush() {
                return Err(LuaError::RuntimeError(format!(
                    "Failed to flush stdout: {}",
                    e
                )));
            }

            Ok(())
        })?
        .into_lua(lua)?;

    let notify_fn = lua
        .create_function(|lua, message: String| plugin::send_event(lua, Event::Notify(message)))?
        .into_lua(lua)?;

    ui::inject_string_meta_method(lua)?;

    let ui_tbl = lua.create_table_from([("line", ui::line(lua)?), ("text", ui::text(lua)?)])?;

    let lc = lua.create_table_from([
        ("defer_fn", defer_fn),
        ("keymap", keymap),
        ("api", api),
        ("cache", cache),
        ("fs", fs),
        ("http", http),
        ("cmd", cmd),
        ("split", split),
        ("system", mlua::Value::Table(system_tbl)),
        ("interactive", interactive_fn),
        ("path", path),
        ("time", time),
        ("log", log_fn),
        ("osc52_copy", osc52_copy),
        ("notify", notify_fn),
        ("json", json_mod),
        ("inspect", inspect_mod),
        ("ui", mlua::Value::Table(ui_tbl)),
    ])?;
    lua.globals().raw_set("lc", lc)
}
