use anyhow::Result;
use crossterm::event::{KeyEvent, KeyEventKind, KeyCode, KeyModifiers};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::State;

/// Handle character input in Input mode
/// Returns true if the key was handled, false if it should be passed to keymap
pub fn handle_input_mode_key(state: &mut State, key: KeyEvent) -> Result<bool> {
    // Ignore release events
    if key.kind == KeyEventKind::Release {
        return Ok(false);
    }

    match key.code {
        // Ctrl-A: move cursor to start
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.input_cursor_position = 0;
            Ok(true)
        }
        // Ctrl-E: move cursor to end
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.input_cursor_position = state.filter_input.len();
            Ok(true)
        }
        // Ctrl-U: delete all characters before cursor
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if state.input_cursor_position > 0 {
                state.filter_input = state.filter_input[state.input_cursor_position..].to_string();
                state.input_cursor_position = 0;
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
            state
                .filter_input
                .insert(state.input_cursor_position, c);
            state.input_cursor_position += c.len_utf8();
            Ok(true)
        }
        KeyCode::Backspace => {
            if state.input_cursor_position > 0 {
                // Find the start of the character before cursor
                let char_start = state.filter_input[..state.input_cursor_position]
                    .char_indices()
                    .rev()
                    .nth(0)
                    .map(|(idx, _)| idx)
                    .unwrap_or(0);
                state
                    .filter_input
                    .remove(char_start);
                state.input_cursor_position = char_start;
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
            if state.input_cursor_position > 0 {
                // Find previous character boundary
                let prev_pos = state.filter_input[..state.input_cursor_position]
                    .char_indices()
                    .rev()
                    .nth(0)
                    .map(|(idx, _)| idx)
                    .unwrap_or(0);
                state.input_cursor_position = prev_pos;
            }
            Ok(true)
        }
        KeyCode::Right => {
            if state.input_cursor_position < state.filter_input.len() {
                // Find next character boundary (skip the character at current position)
                let next_pos = state.filter_input[state.input_cursor_position..]
                    .char_indices()
                    .nth(1)
                    .map(|(idx, _)| state.input_cursor_position + idx)
                    .unwrap_or(state.filter_input.len());
                state.input_cursor_position = next_pos;
            }
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

    // Initialize cursor position (will be updated during render)
    // Default position: after prompt, inside the bordered input box
    // Input box is 50 wide, horizontally centered, at y=5
    // Content area starts at x + 1, y + 1
    // For now, set a reasonable default that will be updated on first render
    const INPUT_WIDTH: u16 = 50;
    const PROMPT: &str = "/> ";
    let x = (80u16.saturating_sub(INPUT_WIDTH)) / 2; // Assuming typical terminal width 80
    let prompt_width = PROMPT.width() as u16;
    let cursor_char_width: u16 = state.filter_input
        .chars()
        .take(state.input_cursor_position)
        .map(|c| c.width().unwrap_or(0) as u16)
        .sum();
    state.filter_cursor_x = x + 1 + prompt_width + cursor_char_width; // x + border(1) + prompt_width + cursor_char_width
    state.filter_cursor_y = 5 + 1; // y + border(1)
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
        // Find the start of the character before cursor
        let char_start = state.filter_input[..state.input_cursor_position]
            .char_indices()
            .rev()
            .nth(0)
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        state
            .filter_input
            .remove(char_start);
        state.input_cursor_position = char_start;
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
