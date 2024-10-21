use ratatui::{prelude::*, widgets};
use symbols::border::ROUNDED;

use crate::Page;

pub struct ListWidget;

impl StatefulWidget for ListWidget {
    type State = Page;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = widgets::Block::bordered()
            .border_set(ROUNDED)
            .title("header");
        let list = widgets::List::new(["1", "2", "3"])
            .block(block)
            .highlight_style(Style::default().fg(Color::Blue))
            .highlight_spacing(widgets::HighlightSpacing::Always);
        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }
}
