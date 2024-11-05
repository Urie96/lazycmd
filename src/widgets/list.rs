use ratatui::{prelude::*, widgets};
use symbols::border::ROUNDED;

use crate::Page;

pub struct ListWidget;

impl StatefulWidget for ListWidget {
    type State = Page;

    fn render(self, area: Rect, buf: &mut Buffer, page: &mut Self::State) {
        let block = widgets::Block::bordered()
            .border_set(ROUNDED)
            .title("header");
        let list = widgets::List::new(page.list.iter().map(|entry| entry.display()))
            .block(block)
            .highlight_style(Style::default().fg(Color::Blue))
            .highlight_spacing(widgets::HighlightSpacing::Always);
        StatefulWidget::render(list, area, buf, &mut page.list_state);
    }
}
