use ratatui::{prelude::*, widgets::*};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Current input state
pub struct InputState {
    pub text: String,
    pub cursor_position: usize,
    pub cursor_x: u16,
    pub cursor_y: u16,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn from_str(s: &str) -> Self {
        Self {
            text: s.to_string(),
            cursor_position: s.len(),
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.text.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor_position < self.text.len() {
            self.text.remove(self.cursor_position);
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
    }

    pub fn cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    pub fn cursor_right(&mut self) {
        self.cursor_position = self.cursor_position.saturating_add(1).min(self.text.len());
    }

    pub fn cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn cursor_to_end(&mut self) {
        self.cursor_position = self.text.len();
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InputWidget;

impl StatefulWidget for InputWidget {
    type State = InputState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let prompt = " ";
        let display = format!("{}{}", prompt, state.text);

        // Create paragraph for the input
        let paragraph = Paragraph::new(display.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("Filter"),
            );

        paragraph.render(area, buf);

        // Calculate cursor position
        // Block has borders, so content starts at area.x + 1, area.y + 1
        // Prompt is rendered at area.x + 1
        // Cursor should appear after the character at cursor_position in the text
        // Use unicode width for proper cursor positioning with Unicode characters
        let prompt_width = prompt.width() as u16;
        let cursor_char_width: u16 = state.text
            .chars()
            .take(state.cursor_position)
            .map(|c| c.width().unwrap_or(0) as u16)
            .sum();
        let cursor_x = area.x + 1 + prompt_width + cursor_char_width;
        let cursor_y = area.y + 1; // Inside the bordered area

        // Store cursor position for use by app.rs
        state.cursor_x = cursor_x;
        state.cursor_y = cursor_y;

        // Note: Cursor positioning is done in app.rs after draw() completes
        // to avoid conflicts with ratatui's rendering
    }
}
