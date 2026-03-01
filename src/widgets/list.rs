use ratatui::{prelude::*, widgets};

use crate::Page;

pub struct ListWidget;

impl StatefulWidget for ListWidget {
    type State = Page;

    fn render(self, area: Rect, buf: &mut Buffer, page: &mut Self::State) {
        let list = widgets::List::new(page.filtered_list.iter().map(|entry| entry.display()))
            .block(widgets::Block::default())
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black))
            .highlight_spacing(widgets::HighlightSpacing::Always);
        StatefulWidget::render(list, area, buf, &mut page.list_state);
    }
}
