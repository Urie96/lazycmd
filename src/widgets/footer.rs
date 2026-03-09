use crate::State;
use ratatui::{prelude::*, widgets::*};

pub struct FooterWidget;

impl StatefulWidget for FooterWidget {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Get the current page and calculate counter text
        let counter_text = if let Some(page) = &state.current_page {
            let total = page.filtered_list.len();
            if total == 0 {
                "0/0".to_string()
            } else {
                let current = page.list_state.selected().map(|i| i + 1).unwrap_or(0);
                format!("{}/{}", current, total)
            }
        } else {
            "0/0".to_string()
        };

        // Calculate width needed for the counter
        // Format:  (1 char) + space + counter + space +  (1 char)
        let counter_width = counter_text.len() as u16;
        let total_width = 1 + 1 + counter_width + 1 + 1; //  + space + counter + space + 

        // Right-align: calculate starting x position
        let x = area.width.saturating_sub(total_width);

        // Render the counter on the right side
        let spans = vec![
            // Left powerline symbol with blue foreground
            Span::styled(
                "",
                Style::default().fg(Color::Blue),
            ),
            // Counter text with blue background and white foreground
            Span::styled(
                format!(" {} ", counter_text),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue),
            ),
            // Right powerline symbol with blue foreground
            Span::styled(
                "",
                Style::default().fg(Color::Blue),
            ),
        ];

        let text = Text::from(Line::from(spans));

        // Render at the calculated position
        let counter_area = Rect {
            x,
            y: area.y,
            width: total_width,
            height: area.height,
        };

        Paragraph::new(text).render(counter_area, buf);
    }
}
