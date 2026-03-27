use crate::State;
use ratatui::{prelude::*, widgets::*};

pub struct HeaderWidget;

impl StatefulWidget for HeaderWidget {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Format path
        let path_str = if state.current_path.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", state.current_path.join("/"))
        };

        // Get filter from current page
        let filter = state
            .current_page
            .as_ref()
            .map(|p| p.list_filter.as_str())
            .unwrap_or("");

        let mut spans = vec![Span::styled(path_str, Style::default().fg(Color::Cyan))];
        if !filter.is_empty() {
            spans.push(Span::styled(
                format!(" [filter: {}]", filter),
                Style::default().fg(Color::Yellow),
            ));
        }
        let text = Text::from(Line::from(spans));

        Paragraph::new(text).render(area, buf);
    }
}
