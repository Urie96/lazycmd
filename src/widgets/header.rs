use ratatui::{prelude::*, widgets::*};
use symbols::border::ROUNDED;
use crate::State;

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

        // Format filter if active
        let content = if !state.filter_input.is_empty() {
            format!("{} [filter: {}]", path_str, state.filter_input)
        } else {
            path_str
        };

        let block = Block::bordered().border_set(ROUNDED).title("header");
        Paragraph::new(content).block(block).render(area, buf);
    }
}
