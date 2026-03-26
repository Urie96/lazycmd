use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use mlua::prelude::*;

use crate::{plugin, State};

/// Handle keyboard input for select dialog
/// Returns true if the key was handled, false otherwise
pub fn handle_select_dialog_key(
    lua: &Lua,
    state: &mut State,
    event_sender: &tokio::sync::mpsc::UnboundedSender<crate::events::Event>,
    key: KeyEvent,
) -> Result<bool> {
    // Ignore release events
    if key.kind == KeyEventKind::Release {
        return Ok(false);
    }

    let dialog = match &mut state.select_dialog {
        Some(d) => d,
        None => return Ok(false),
    };

    match key.code {
        // Special navigation keys
        KeyCode::Up => {
            dialog.move_selection(-1);
            Ok(true)
        }
        KeyCode::Down => {
            dialog.move_selection(1);
            Ok(true)
        }
        // Home: move cursor to start
        KeyCode::Home => {
            dialog.cursor_to_start();
            Ok(true)
        }
        // End: move cursor to end
        KeyCode::End => {
            dialog.cursor_to_end();
            Ok(true)
        }
        // Left: move cursor left
        KeyCode::Left => {
            dialog.cursor_left();
            Ok(true)
        }
        // Right: move cursor right
        KeyCode::Right => {
            dialog.cursor_right();
            Ok(true)
        }
        // Delete: delete character at cursor
        KeyCode::Delete => {
            dialog.delete_at_cursor();
            dialog.update_filtered_options();
            Ok(true)
        }
        // Ctrl-A: move cursor to start
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            dialog.cursor_to_start();
            Ok(true)
        }
        // Ctrl-E: move cursor to end
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            dialog.cursor_to_end();
            Ok(true)
        }
        // Ctrl-U: delete all characters before cursor
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            dialog.delete_before_cursor_all();
            Ok(true)
        }
        // Character input: insert at cursor position
        KeyCode::Char(c) => {
            // For keys with modifiers (except SHIFT), let keymap handle them
            if key.modifiers.contains(KeyModifiers::CONTROL)
                || key.modifiers.contains(KeyModifiers::ALT)
            {
                return Ok(false);
            }
            dialog.insert_char(c);
            dialog.update_filtered_options();
            Ok(true)
        }
        // Backspace: delete character before cursor
        KeyCode::Backspace => {
            dialog.delete_before_cursor();
            dialog.update_filtered_options();
            Ok(true)
        }
        // Page Up: move selection up by page
        KeyCode::PageUp => {
            dialog.move_selection(-5);
            Ok(true)
        }
        // Page Down: move selection down by page
        KeyCode::PageDown => {
            dialog.move_selection(5);
            Ok(true)
        }
        // Enter: select current option and call callback
        KeyCode::Enter => {
            if let Some(dialog) = state.select_dialog.take() {
                let filtered = dialog.get_filtered_options();
                if let Some(idx) = dialog.selected_index {
                    if let Some(opt) = filtered.get(idx) {
                        plugin::scope(lua, state, event_sender, || {
                            dialog.on_selection.call::<()>(opt.value.clone())
                        })?;
                    } else {
                        // Callback with nil if no selection
                        plugin::scope(lua, state, event_sender, || {
                            dialog.on_selection.call::<()>(LuaValue::Nil)
                        })?;
                    }
                } else {
                    // Callback with nil if no selection
                    plugin::scope(lua, state, event_sender, || {
                        dialog.on_selection.call::<()>(LuaValue::Nil)
                    })?;
                }
            }
            Ok(true)
        }
        // Esc: cancel selection and call callback with nil
        KeyCode::Esc => {
            if let Some(dialog) = state.select_dialog.take() {
                plugin::scope(lua, state, event_sender, || {
                    dialog.on_selection.call::<()>(LuaValue::Nil)
                })?;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}
