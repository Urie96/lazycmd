use ratatui::{prelude::*, widgets::*};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::SelectDialog;

/// Widget for rendering a select dialog
pub struct SelectWidget;

impl SelectWidget {
    fn char_width(c: char) -> u16 {
        c.width().unwrap_or(0) as u16
    }

    fn display_width(text: &str) -> u16 {
        text.chars().map(Self::char_width).sum()
    }

    fn visible_window(text: &str, cursor_position: usize, max_width: u16) -> (String, u16) {
        if text.is_empty() || max_width == 0 {
            return (String::new(), 0);
        }

        let cursor_position = cursor_position.min(text.len());
        let total_width_before_cursor = Self::display_width(&text[..cursor_position]);
        let max_width_before_cursor = if total_width_before_cursor >= max_width {
            max_width.saturating_sub(1)
        } else {
            max_width
        };

        let chars: Vec<(usize, char)> = text.char_indices().collect();
        let char_count = chars.len();
        let cursor_char_idx = chars
            .iter()
            .position(|(idx, _)| *idx >= cursor_position)
            .unwrap_or(char_count);

        let mut start_char_idx = 0usize;
        let mut width_before_cursor = 0u16;
        for idx in (0..cursor_char_idx).rev() {
            let next_width = width_before_cursor.saturating_add(Self::char_width(chars[idx].1));
            if next_width > max_width_before_cursor {
                start_char_idx = idx + 1;
                break;
            }
            width_before_cursor = next_width;
        }

        let start_byte = chars
            .get(start_char_idx)
            .map(|(idx, _)| *idx)
            .unwrap_or(text.len());

        let mut end_byte = text.len();
        let mut visible_width = 0u16;
        for idx in start_char_idx..char_count {
            let ch_width = Self::char_width(chars[idx].1);
            if visible_width.saturating_add(ch_width) > max_width {
                end_byte = chars[idx].0;
                break;
            }
            visible_width = visible_width.saturating_add(ch_width);
        }

        let visible_text = text[start_byte..end_byte].to_string();
        let cursor_offset = Self::display_width(&text[start_byte..cursor_position]);

        (visible_text, cursor_offset.min(max_width_before_cursor))
    }
}

impl StatefulWidget for SelectWidget {
    type State = SelectDialog;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        use Constraint::*;

        // Clear the area first to prevent underlying content from showing through
        Clear.render(area, buf);

        // The 'area' parameter is already the centered dialog position from AppWidget
        let dialog_area = area;

        // Draw dialog border with cyan color
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        // Add title
        let block = if let Some(title) = &state.prompt {
            block
                .title(title.as_str())
                .title_alignment(Alignment::Center)
        } else {
            block.title("Select").title_alignment(Alignment::Center)
        };

        let inner = block.inner(dialog_area);
        block.render(dialog_area, buf);

        // Split inner area into: filter input (1 row) + divider (1 row) + options list (remaining)
        let input_height = 1u16;
        let divider_height = 1u16;
        let _list_height = inner.height.saturating_sub(input_height + divider_height);

        let [input_area, divider_area, list_area] =
            Layout::vertical([Length(input_height), Length(divider_height), Min(0)]).areas(inner);

        let options = state.get_filtered_options();
        let prompt = " ";
        let prompt_width = prompt.width() as u16;
        let filtered_count = options.len();
        let total_count = state.options.len();
        let count_text = format!("{}/{}", filtered_count, total_count);
        let count_width = count_text.width() as u16;
        let gap_width = if inner.width > count_width { 1 } else { 0 };
        let left_width = input_area
            .width
            .saturating_sub(count_width.saturating_add(gap_width));

        let input_text_width = left_width.saturating_sub(prompt_width);
        let (visible_filter_input, cursor_offset) =
            Self::visible_window(&state.filter_input, state.cursor_position, input_text_width);

        // Render filter input on the left side
        let filter_text = Text::from(Line::from(vec![
            Span::styled(prompt, Style::default().fg(Color::Cyan)),
            Span::styled(visible_filter_input, Style::default().fg(Color::White)),
        ]));
        let left_input_area = Rect {
            x: input_area.x,
            y: input_area.y,
            width: left_width,
            height: input_area.height,
        };
        Paragraph::new(filter_text).render(left_input_area, buf);

        // Render count on the right side
        let count_area = Rect {
            x: input_area.right().saturating_sub(count_width),
            y: input_area.y,
            width: count_width,
            height: input_area.height,
        };
        Paragraph::new(count_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Right)
            .render(count_area, buf);

        // Calculate and store cursor position
        let cursor_x = input_area.x + prompt_width + cursor_offset;
        let cursor_y = input_area.y;
        state.cursor_x = cursor_x;
        state.cursor_y = cursor_y;

        // Draw divider line
        for x in divider_area.left()..divider_area.right() {
            buf[(x, divider_area.top())]
                .set_symbol(symbols::line::HORIZONTAL)
                .set_style(Style::default().fg(Color::Cyan));
        }

        // Adjust offset based on scrolloff (keep selected item away from edges)
        if let Some(selected) = state.selected_index {
            let height = list_area.height as usize;
            let scrolloff = 3.min(height / 2); // Keep 3 lines margin
            let offset = state.list_state.offset();
            let cursor_pos = selected.saturating_sub(offset);
            let len = options.len();

            // When cursor is in the top scrolloff zone, scroll up to keep cursor at scrolloff
            if cursor_pos < scrolloff && offset > 0 {
                let new_offset = selected.saturating_sub(scrolloff);
                *state.list_state.offset_mut() = new_offset;
            }
            // When cursor is in the bottom scrolloff zone, scroll down
            else if cursor_pos >= height.saturating_sub(scrolloff) {
                let desired_pos = height.saturating_sub(scrolloff).saturating_sub(1);
                if selected >= desired_pos {
                    let new_offset = selected.saturating_sub(desired_pos);
                    let max_offset = if len > height { len - height } else { 0 };
                    *state.list_state.offset_mut() = new_offset.min(max_offset);
                }
            }
        }

        if options.is_empty() {
            // Show "No results" message
            Paragraph::new("No matching options")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray))
                .render(list_area, buf);
        } else {
            // Custom rendering with padding and selection markers
            let offset = state.list_state.offset();
            let selected = state.selected_index;
            let height = list_area.height as usize;

            for (i, opt) in options.iter().enumerate().skip(offset).take(height) {
                let y = list_area.top() + (i - offset) as u16;
                let is_selected = Some(i) == selected;

                if is_selected {
                    let selected_color = Color::DarkGray;
                    // Selected: render with blue background and markers
                    let selected_style = Style::default().bg(selected_color);

                    // Left marker  with blue foreground only (no background)
                    buf[(list_area.left(), y)]
                        .set_char('')
                        .set_style(Style::default().fg(selected_color));

                    // Right marker  with blue foreground only (no background)
                    buf[(list_area.right() - 1, y)]
                        .set_char('')
                        .set_style(Style::default().fg(selected_color));

                    // Content area (with one space padding on each side)
                    let content_area = Rect {
                        x: list_area.left() + 1,
                        y,
                        width: list_area.width.saturating_sub(2),
                        height: 1,
                    };

                    // Clear and fill content area with blue background
                    for x in content_area.left()..content_area.right() {
                        buf[(x, y)].set_char(' ').set_style(selected_style);
                    }

                    // Render content with selection style
                    opt.display
                        .clone()
                        .patch_style(selected_style)
                        .render(content_area, buf);
                } else {
                    // Normal: render with padding on both sides
                    // Clear the entire line
                    for x in list_area.left()..list_area.right() {
                        buf[(x, y)].set_char(' ').set_style(Style::default());
                    }

                    // Content area (with one space padding on each side)
                    let content_area = Rect {
                        x: list_area.left() + 1,
                        y,
                        width: list_area.width.saturating_sub(2),
                        height: 1,
                    };

                    // Render content
                    (&opt.display).render(content_area, buf);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SelectWidget;
    use crate::{SelectDialog, SelectOption};
    use mlua::Lua;
    use ratatui::{buffer::Buffer, layout::Rect, text::Line, widgets::StatefulWidget};

    #[test]
    fn select_dialog_renders_filtered_count_in_input_row() {
        let lua = Lua::new();
        let on_selection = lua.create_function(|_, ()| Ok(())).unwrap().to_owned();
        let options = vec![
            SelectOption {
                value: mlua::Value::Nil,
                display: Line::from("alpha"),
            },
            SelectOption {
                value: mlua::Value::Nil,
                display: Line::from("beta"),
            },
            SelectOption {
                value: mlua::Value::Nil,
                display: Line::from("gamma"),
            },
        ];
        let mut dialog = SelectDialog::new(Some("Select".to_string()), options, on_selection);
        dialog.filter_input = "a".to_string();
        dialog.cursor_position = 1;
        dialog.update_filtered_options();

        let area = Rect::new(0, 0, 30, 8);
        let mut buf = Buffer::empty(area);
        SelectWidget.render(area, &mut buf, &mut dialog);
        let row = (0..area.width)
            .map(|x| buf[(x, 1)].symbol())
            .collect::<String>();

        assert!(row.contains("3/3"));
    }
}
