use ratatui::{prelude::*, widgets::*};

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
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

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

        let [input_area, divider_area, list_area] = Layout::vertical([
            Length(input_height),
            Length(divider_height),
            Min(0),
        ])
        .areas(inner);

        // Render filter input (custom simple input)
        let filter_text = format!("> {}", state.filter_input);
        Paragraph::new(filter_text)
            .style(Style::default().fg(Color::Yellow))
            .render(input_area, buf);

        // Draw divider line
        for x in divider_area.left()..divider_area.right() {
            buf[(x, divider_area.top())]
                .set_symbol(symbols::line::HORIZONTAL)
                .set_style(Style::default().fg(Color::Cyan));
        }

        // Render options list
        let options = state.get_filtered_options();

        if options.is_empty() {
            // Show "No results" message
            Paragraph::new("No matching options")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray))
                .render(list_area, buf);
        } else {
            let list_items: Vec<ListItem> = options
                .iter()
                .enumerate()
                .map(|(idx, opt)| {
                    let is_selected = Some(idx) == state.selected_index;
                    let display_text = opt.display.as_str();
                    let style = if is_selected {
                        Style::default()
                            .bg(Color::Cyan)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(display_text).style(style)
                })
                .collect();

            let list = List::new(list_items).highlight_spacing(HighlightSpacing::Always);

            // Use tui_input state for scrolling
            StatefulWidget::render(list, list_area, buf, &mut state.list_state);
        }

        // Render help text at bottom (inside dialog)
        if inner.height >= 1 {
            let help_y = inner.bottom() - 1;
            let help_text = "↑↓: Move | Enter: Select | Esc: Cancel";
            let help_x = inner.x + (inner.width.saturating_sub(help_text.len() as u16)) / 2;

            if help_x >= inner.x && help_y >= inner.y {
                Paragraph::new(help_text)
                    .style(Style::default().fg(Color::DarkGray))
                    .render(
                        Rect {
                            x: help_x,
                            y: help_y,
                            width: help_text.len() as u16,
                            height: 1,
                        },
                        buf,
                    );
            }
        }
    }
}
