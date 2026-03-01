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

        // Build styled spans
        let text = if state.current_plugin.is_empty() {
            // Just path in cyan, plus filter if active
            let mut spans = vec![Span::styled(path_str, Style::default().fg(Color::Cyan))];
            if !state.filter_input.is_empty() {
                spans.push(Span::styled(
                    format!(" [filter: {}]", state.filter_input),
                    Style::default().fg(Color::Yellow),
                ));
            }
            Text::from(Line::from(spans))
        } else {
            // Plugin name with colon (green) + path (cyan) + filter (yellow)
            let mut spans = vec![
                Span::styled(
                    format!("{}:", state.current_plugin),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(path_str, Style::default().fg(Color::Cyan)),
            ];
            if !state.filter_input.is_empty() {
                spans.push(Span::styled(
                    format!(" [filter: {}]", state.filter_input),
                    Style::default().fg(Color::Yellow),
                ));
            }
            Text::from(Line::from(spans))
        };

        Paragraph::new(text).render(area, buf);
    }
}
