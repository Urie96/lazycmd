use anyhow::Result;
use crossterm::event::{KeyEvent, KeyEventKind, KeyCode, KeyModifiers};
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
        // Special navigation keys (must come before generic Char)
        KeyCode::Up | KeyCode::Char('k') => {
            dialog.move_selection(-1);
            Ok(true)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            dialog.move_selection(1);
            Ok(true)
        }
        // Character input: update filter
        KeyCode::Char(c) => {
            // For keys with modifiers (except SHIFT), let keymap handle them
            if key.modifiers.contains(KeyModifiers::CONTROL)
                || key.modifiers.contains(KeyModifiers::ALT)
            {
                return Ok(false);
            }
            dialog.filter_input.push(c);
            dialog.update_filtered_options();
            Ok(true)
        }
        // Backspace: remove last character from filter
        KeyCode::Backspace => {
            dialog.filter_input.pop();
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
        // Home: select first option
        KeyCode::Home => {
            dialog.select_first();
            Ok(true)
        }
        // End: select last option
        KeyCode::End => {
            dialog.select_last();
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
