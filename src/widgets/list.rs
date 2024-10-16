use ratatui::{prelude::*, widgets};
use symbols::border::ROUNDED;

#[derive(Debug, Default)]
pub struct List {
    pub items: Vec<String>,
    pub list_state: widgets::ListState,
}

impl List {
    pub fn scroll_next(&mut self) {
        let wrap_index = self.items.len().max(1);
        let next = self
            .list_state
            .selected()
            .map_or(0, |i| (i + 1) % wrap_index);
        self.scroll_to(next);
    }

    fn scroll_to(&mut self, index: usize) {
        if self.items.is_empty() {
            self.list_state.select(None)
        } else {
            self.list_state.select(Some(index));
            // self.scrollbar_state = self.scrollbar_state.position(index);
        }
    }
}

pub struct ListWidget;

impl StatefulWidget for ListWidget {
    type State = List;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = widgets::Block::bordered()
            .border_set(ROUNDED)
            .title("header");
        let list = widgets::List::new(state.items.clone())
            .block(block)
            .highlight_style(Style::default().fg(Color::Blue))
            .highlight_spacing(widgets::HighlightSpacing::Always);
        StatefulWidget::render(list, area, buf, &mut state.list_state);
    }
}
