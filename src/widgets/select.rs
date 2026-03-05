use ratatui::{prelude::*, widgets::*};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::SelectDialog;

/// Widget for rendering a select dialog
pub struct SelectWidget;

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

        // Render filter input
        let prompt = " ";
        let filter_text = Text::from(Line::from(vec![
            Span::styled(prompt, Style::default().fg(Color::Cyan)),
            Span::styled(
                state.filter_input.as_str(),
                Style::default().fg(Color::White),
            ),
        ]));
        Paragraph::new(filter_text).render(input_area, buf);

        // Calculate and store cursor position
        // Use unicode width for proper cursor positioning with Unicode characters
        let prompt_width = prompt.width() as u16;
        let cursor_char_width: u16 = state
            .filter_input
            .chars()
            .take(state.cursor_position)
            .map(|c| c.width().unwrap_or(0) as u16)
            .sum();
        let cursor_x = input_area.x + prompt_width + cursor_char_width;
        let cursor_y = input_area.y;
        state.cursor_x = cursor_x;
        state.cursor_y = cursor_y;

        // Draw divider line
        for x in divider_area.left()..divider_area.right() {
            buf[(x, divider_area.top())]
                .set_symbol(symbols::line::HORIZONTAL)
                .set_style(Style::default().fg(Color::Cyan));
        }

        // Render options list with custom styling
        let options = state.get_filtered_options();

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
                    // Selected: render with blue background and markers
                    let selected_style = Style::default().bg(Color::Blue).fg(Color::Black);

                    // Left marker  with blue foreground only (no background)
                    buf[(list_area.left(), y)]
                        .set_char('')
                        .set_style(Style::default().fg(Color::Blue));

                    // Right marker  with blue foreground only (no background)
                    buf[(list_area.right() - 1, y)]
                        .set_char('')
                        .set_style(Style::default().fg(Color::Blue));

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
                    let line = opt.display.clone();
                    let styled_spans: Vec<Span> = line
                        .spans
                        .iter()
                        .map(|span| Span::styled(span.content.as_ref(), selected_style))
                        .collect();
                    let styled_line = Line::from(styled_spans);
                    styled_line.render(content_area, buf);
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
                    opt.display.clone().render(content_area, buf);
                }
            }
        }
    }
}
