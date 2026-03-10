use anyhow::Result;
use crossterm::event::{KeyEvent, KeyEventKind, KeyCode, KeyModifiers};

use crate::{plugin, State};

/// Handle keyboard input for the input dialog
/// Returns true if the key was handled, false if it should be passed to keymap
pub fn handle_input_dialog_key(
    lua: &mlua::Lua,
    state: &mut State,
    event_sender: &tokio::sync::mpsc::UnboundedSender<crate::events::Event>,
    key: KeyEvent,
) -> Result<bool> {
    // Ignore release events
    if key.kind == KeyEventKind::Release {
        return Ok(false);
    }

    let dialog = match &mut state.input_dialog {
        Some(d) => d,
        None => return Ok(false),
    };

    match key.code {
        // Enter: submit the input
        KeyCode::Enter => {
            let text = dialog.text.clone();
            let on_submit = dialog.on_submit.clone();
            state.input_dialog.take();

            // Call on_submit callback using plugin::scope
            plugin::scope(lua, state, event_sender, || {
                on_submit.call::<()>(text)
            })?;

            Ok(true)
        }
        // Escape: cancel the input
        KeyCode::Esc => {
            let on_cancel = dialog.on_cancel.clone();
            state.input_dialog.take();

            // Call on_cancel callback using plugin::scope
            plugin::scope(lua, state, event_sender, || {
                on_cancel.call::<()>(())
            })?;

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
            if dialog.cursor_position > 0 {
                dialog.text = dialog.text[dialog.cursor_position..].to_string();
                dialog.cursor_position = 0;

                // Call on_change callback using plugin::scope
                let text = dialog.text.clone();
                let on_change = dialog.on_change.clone();
                if !text.is_empty() || dialog.cursor_position > 0 {
                    plugin::scope(lua, state, event_sender, || {
                        on_change.call::<()>(text)
                    })?;
                }
            }
            Ok(true)
        }
        // For keys with modifiers (except SHIFT), let keymap handle them
        _ if key.modifiers.contains(KeyModifiers::CONTROL)
            || key.modifiers.contains(KeyModifiers::ALT) => {
            Ok(false)
        }
        KeyCode::Char(c) => {
            // Insert character at cursor position
            dialog.insert_char(c);

            // Call on_change callback using plugin::scope
            let text = dialog.text.clone();
            let on_change = dialog.on_change.clone();
            plugin::scope(lua, state, event_sender, || {
                on_change.call::<()>(text)
            })?;

            Ok(true)
        }
        KeyCode::Backspace => {
            dialog.backspace();

            // Call on_change callback using plugin::scope
            let text = dialog.text.clone();
            let on_change = dialog.on_change.clone();
            plugin::scope(lua, state, event_sender, || {
                on_change.call::<()>(text)
            })?;

            Ok(true)
        }
        KeyCode::Delete => {
            dialog.delete();

            // Call on_change callback using plugin::scope
            let text = dialog.text.clone();
            let on_change = dialog.on_change.clone();
            plugin::scope(lua, state, event_sender, || {
                on_change.call::<()>(text)
            })?;

            Ok(true)
        }
        KeyCode::Left => {
            dialog.cursor_left();
            Ok(true)
        }
        KeyCode::Right => {
            dialog.cursor_right();
            Ok(true)
        }
        KeyCode::Home => {
            dialog.cursor_to_start();
            Ok(true)
        }
        KeyCode::End => {
            dialog.cursor_to_end();
            Ok(true)
        }
        _ => Ok(false),
    }
}
