use ratatui::{prelude::*, widgets::*};

/// Current input state
pub struct InputState {
    pub text: String,
    pub cursor_position: usize,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
        }
    }

    pub fn from_str(s: &str) -> Self {
        Self {
            text: s.to_string(),
            cursor_position: s.len(),
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
        let prompt = "/> ";
        let display = format!("{}{}", prompt, state.text);

        // Create paragraph for the input
        let paragraph = Paragraph::new(display.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::bordered().border_type(BorderType::Rounded).title("Filter"));

        paragraph.render(area, buf);

        // Calculate cursor position (cursor should appear after the character at cursor_position)
        let prompt_width = prompt.len() as u16;
        // Position cursor after the character (so it's on the right side of the character at cursor_position)
        let cursor_x = area.x + prompt_width + state.cursor_position as u16 + 1;
        let cursor_y = area.y + 1; // Inside the bordered area

        // Ensure cursor is within bounds and set cursor style
        if cursor_x >= area.x && cursor_x < area.right() && cursor_y >= area.y && cursor_y < area.bottom() {
            if let Some(cell) = buf.cell_mut((cursor_x, cursor_y)) {
                cell.set_style(Style::default().bg(Color::White));
            }
        }
    }
}
