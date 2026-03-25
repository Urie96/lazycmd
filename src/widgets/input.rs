use ratatui::{prelude::*, widgets::*};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Alias for backward compatibility (used by filter mode)
pub type InputState = InputDialogState;

#[derive(Debug)]
pub struct InputDialogState {
    pub text: String,
    pub cursor_position: usize,
    pub cursor_x: u16,
    pub cursor_y: u16,
    /// Custom prompt to display
    pub prompt: String,
    /// Placeholder text (shown when text is empty)
    pub placeholder: String,
}

impl InputDialogState {
    fn prev_char_boundary(text: &str, cursor_position: usize) -> usize {
        text[..cursor_position]
            .char_indices()
            .next_back()
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    fn next_char_boundary(text: &str, cursor_position: usize) -> usize {
        text[cursor_position..]
            .char_indices()
            .nth(1)
            .map(|(idx, _)| cursor_position + idx)
            .unwrap_or(text.len())
    }

    pub fn new(prompt: &str, placeholder: &str) -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            cursor_x: 0,
            cursor_y: 0,
            prompt: prompt.to_string(),
            placeholder: placeholder.to_string(),
        }
    }

    /// Create from filter input (for filter mode compatibility)
    pub fn from_filter_input(s: &str) -> Self {
        Self {
            text: s.to_string(),
            cursor_position: s.len(),
            cursor_x: 0,
            cursor_y: 0,
            prompt: " ".to_string(),
            placeholder: String::new(),
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();
    }

    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            let prev_pos = Self::prev_char_boundary(&self.text, self.cursor_position);
            self.text.remove(prev_pos);
            self.cursor_position = prev_pos;
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
        if self.cursor_position > 0 {
            self.cursor_position = Self::prev_char_boundary(&self.text, self.cursor_position);
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.text.len() {
            self.cursor_position = Self::next_char_boundary(&self.text, self.cursor_position);
        }
    }

    pub fn cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    pub fn cursor_to_end(&mut self) {
        self.cursor_position = self.text.len();
    }
}

#[cfg(test)]
mod tests {
    use super::InputDialogState;

    #[test]
    fn input_dialog_state_backspace_handles_utf8() {
        let mut state = InputDialogState::new("Search", "keyword");
        state.insert_char('搜');
        state.insert_char('索');

        state.backspace();
        assert_eq!(state.text, "搜");
        assert_eq!(state.cursor_position, '搜'.len_utf8());
    }
}

/// Widget for rendering an input dialog with customizable prompt and title
pub struct InputDialogWidget;

impl InputDialogWidget {
    pub fn new() -> Self {
        Self
    }
}

impl StatefulWidget for InputDialogWidget {
    type State = InputDialogState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Clear the area first to prevent previous content from leaking through
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(' ');
                }
            }
        }

        // Arrow prefix " " for inside the input box (like select component)
        let arrow = " ";
        
        // Determine the text to display: actual text or placeholder
        let display_text = if state.text.is_empty() {
            &state.placeholder
        } else {
            state.text.as_str()
        };

        // Determine text color: gray for placeholder, white for actual input
        let text_color = if state.text.is_empty() {
            Color::DarkGray
        } else {
            Color::White
        };

        // Create text with arrow + user input (prompt is shown in title bar only)
        let text = Text::from(Line::from(vec![
            Span::styled(arrow, Style::default().fg(Color::Cyan)),
            Span::styled(display_text, Style::default().fg(text_color)),
        ]));

        // Title shows the prompt (like select component)
        // If prompt is empty, show "Input" as default
        let title_text = if state.prompt.is_empty() {
            "Input".to_string()
        } else {
            state.prompt.clone()
        };
        
        let paragraph = Paragraph::new(text)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .title_alignment(Alignment::Center)
                    .title(title_text),
            );

        paragraph.render(area, buf);

        // Calculate cursor position (arrow + cursor position in text)
        let arrow_width = arrow.width() as u16;
        let cursor_char_width: u16 = state
            .text
            .chars()
            .take(state.cursor_position)
            .map(|c| c.width().unwrap_or(0) as u16)
            .sum();
        let cursor_x = area.x + 1 + arrow_width + cursor_char_width;
        let cursor_y = area.y + 1; // Inside the bordered area

        // Store cursor position for use by app.rs
        state.cursor_x = cursor_x;
        state.cursor_y = cursor_y;
    }
}
