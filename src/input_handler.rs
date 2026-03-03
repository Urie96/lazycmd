use anyhow::Result;
use crossterm::event::{KeyEvent, KeyEventKind, KeyCode, KeyModifiers};

use crate::State;

/// Handle character input in Input mode
/// Returns true if the key was handled, false if it should be passed to keymap
pub fn handle_input_mode_key(state: &mut State, key: KeyEvent) -> Result<bool> {
    // Ignore release events
    if key.kind == KeyEventKind::Release {
        return Ok(false);
    }

    // For keys with modifiers (except SHIFT), let keymap handle them
    if key.modifiers.contains(KeyModifiers::CONTROL)
        || key.modifiers.contains(KeyModifiers::ALT)
    {
        return Ok(false);
    }

    match key.code {
        KeyCode::Char(c) => {
            // Insert character at cursor position
            state
                .filter_input
                .insert(state.input_cursor_position, c);
            state.input_cursor_position += 1;
            Ok(true)
        }
        KeyCode::Backspace => {
            if state.input_cursor_position > 0 {
                state
                    .filter_input
                    .remove(state.input_cursor_position - 1);
                state.input_cursor_position -= 1;
            }
            Ok(true)
        }
        KeyCode::Delete => {
            if state.input_cursor_position < state.filter_input.len() {
                state
                    .filter_input
                    .remove(state.input_cursor_position);
            }
            Ok(true)
        }
        KeyCode::Left => {
            state.input_cursor_position =
                state.input_cursor_position.saturating_sub(1);
            Ok(true)
        }
        KeyCode::Right => {
            state.input_cursor_position = state
                .input_cursor_position
                .saturating_add(1)
                .min(state.filter_input.len());
            Ok(true)
        }
        KeyCode::Home => {
            state.input_cursor_position = 0;
            Ok(true)
        }
        KeyCode::End => {
            state.input_cursor_position = state.filter_input.len();
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// Enter filter mode, preserving current filter if already set
pub fn enter_filter_mode(state: &mut State) {
    state.current_mode = crate::Mode::Input;
    // Preserve current filter if already set, otherwise start fresh
    if state.filter_input.is_empty() {
        state.input_cursor_position = 0;
    } else {
        state.input_cursor_position = state.filter_input.len();
    }
    state.last_key_event_buffer.clear();
}

/// Exit filter mode
pub fn exit_filter_mode(state: &mut State, keep_filter: bool) {
    if !keep_filter {
        state.filter_input.clear();
        state.input_cursor_position = 0;
    }
    state.current_mode = crate::Mode::Main;
    state.last_key_event_buffer.clear();
}

/// Handle backspace in filter mode
pub fn handle_filter_backspace(state: &mut State) {
    if state.input_cursor_position > 0 {
        state
            .filter_input
            .remove(state.input_cursor_position - 1);
        state.input_cursor_position -= 1;
    }
}

/// Clear the filter
pub fn handle_filter_clear(state: &mut State) {
    state.filter_input.clear();
    state.input_cursor_position = 0;
}

/// Apply the current filter to the page
pub fn apply_filter(state: &mut State) -> Result<()> {
    if let Some(page) = &mut state.current_page {
        // Sync filter state from State to Page
        page.filter_input = state.filter_input.clone();
        page.input_cursor_position = state.input_cursor_position;
        page.apply_filter(&state.filter_input);
    }
    Ok(())
}
