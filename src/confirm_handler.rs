use anyhow::Result;
use crossterm::event::{KeyEvent, KeyEventKind, KeyCode};
use mlua::Lua;

use crate::{plugin, ConfirmButton, State};

/// Handle keyboard input for confirm dialog
/// Returns true if the key was handled, false otherwise
pub fn handle_confirm_dialog_key(
    lua: &Lua,
    state: &mut State,
    event_sender: &tokio::sync::mpsc::UnboundedSender<crate::events::Event>,
    key: KeyEvent,
) -> Result<bool> {
    // Ignore release events
    if key.kind == KeyEventKind::Release {
        return Ok(false);
    }

    match key.code {
        // Left arrow: select Yes
        KeyCode::Left => {
            state.confirm_dialog.as_mut().map(|d| {
                d.selected_button = ConfirmButton::Yes;
            });
            Ok(true)
        }
        // Right arrow: select No
        KeyCode::Right => {
            state.confirm_dialog.as_mut().map(|d| {
                d.selected_button = ConfirmButton::No;
            });
            Ok(true)
        }
        // Enter: execute selected button's callback
        KeyCode::Enter => {
            if let Some(dialog) = state.confirm_dialog.take() {
                match dialog.selected_button {
                    ConfirmButton::Yes => {
                        plugin::scope(lua, state, event_sender, || {
                            dialog.on_confirm.call::<()>(())
                        })?;
                    }
                    ConfirmButton::No => {
                        plugin::scope(lua, state, event_sender, || {
                            dialog.on_cancel.call::<()>(())
                        })?;
                    }
                }
            }
            Ok(true)
        }
        // Y key: confirm (execute on_confirm)
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some(dialog) = state.confirm_dialog.take() {
                plugin::scope(lua, state, event_sender, || {
                    dialog.on_confirm.call::<()>(())
                })?;
            }
            Ok(true)
        }
        // N key: cancel (execute on_cancel)
        KeyCode::Char('n') | KeyCode::Char('N') => {
            if let Some(dialog) = state.confirm_dialog.take() {
                plugin::scope(lua, state, event_sender, || {
                    dialog.on_cancel.call::<()>(())
                })?;
            }
            Ok(true)
        }
        // Esc: cancel (execute on_cancel)
        KeyCode::Esc => {
            if let Some(dialog) = state.confirm_dialog.take() {
                plugin::scope(lua, state, event_sender, || {
                    dialog.on_cancel.call::<()>(())
                })?;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}
