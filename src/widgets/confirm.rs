use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Clear, Paragraph},
};

use crate::{ConfirmButton, ConfirmDialog};

use Constraint::*;

/// Widget for rendering a confirm dialog
pub struct ConfirmWidget;

impl StatefulWidget for ConfirmWidget {
    type State = ConfirmDialog;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Clear the area first to prevent underlying content from showing through
        Clear.render(area, buf);

        // Draw dialog border with cyan color
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue));

        // Add title (always present, defaults to "Confirm") - title is centered by default
        if let Some(title) = &state.title {
            let block = block
                .title(title.as_str())
                .title_alignment(ratatui::layout::Alignment::Center)
                .title_style(Style::default().fg(Color::Blue));
            let inner = block.inner(area);
            block.render(area, buf);

            // Split into: prompt area, then buttons with divider
            // Buttons area takes 3 rows (divider + 2 rows for buttons)
            let buttons_area_height = 3u16;
            let prompt_area_height = inner.height.saturating_sub(buttons_area_height);

            let [prompt_area, buttons_area] =
                Layout::vertical([Length(prompt_area_height), Length(buttons_area_height)])
                    .areas(inner);

            // Render prompt text with white color, centered and wrapped
            let prompt_paragraph = Paragraph::new(state.prompt.as_str())
                .wrap(ratatui::widgets::Wrap { trim: true })
                .alignment(ratatui::layout::Alignment::Center)
                .style(Style::default().fg(Color::White));
            prompt_paragraph.render(prompt_area, buf);

            // Draw divider line at the top of buttons area (button row - 1)
            let divider_y = buttons_area.bottom().saturating_sub(2);
            for x in buttons_area.left()..buttons_area.right() {
                buf[(x, divider_y)]
                    .set_symbol(symbols::line::HORIZONTAL)
                    .set_style(Style::default().fg(Color::Blue));
            }

            // Split buttons area into left half and right half
            let [left_half, right_half] =
                Layout::horizontal([Percentage(50), Percentage(50)]).areas(buttons_area);

            // Render buttons
            let yes_selected = state.selected_button == ConfirmButton::Yes;
            let no_selected = state.selected_button == ConfirmButton::No;

            // Button text
            let yes_text = "[Y]es";
            let no_text = "(N)o";

            // Button width is 9
            let button_width = 9u16;

            // Button style for selected: white background, black text, bold
            let selected_style = Style::default()
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD);

            // Button style for unselected: white text, no background
            let unselected_style = Style::default().fg(Color::White);

            let yes_style = if yes_selected {
                selected_style
            } else {
                unselected_style
            };

            let no_style = if no_selected {
                selected_style
            } else {
                unselected_style
            };

            // Center buttons in their respective halves
            let yes_x = left_half
                .x
                .saturating_add(left_half.width.saturating_sub(button_width) / 2);
            let no_x = right_half
                .x
                .saturating_add(right_half.width.saturating_sub(button_width) / 2);
            let button_y = buttons_area.bottom().saturating_sub(1);

            let yes_area = Rect {
                x: yes_x,
                y: button_y,
                width: button_width,
                height: 1,
            };
            let no_area = Rect {
                x: no_x,
                y: button_y,
                width: button_width,
                height: 1,
            };

            // Render Yes button
            Paragraph::new(yes_text)
                .style(yes_style)
                .alignment(ratatui::layout::Alignment::Center)
                .render(yes_area, buf);

            // Render No button
            Paragraph::new(no_text)
                .style(no_style)
                .alignment(ratatui::layout::Alignment::Center)
                .render(no_area, buf);
        }
    }
}
