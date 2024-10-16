use ratatui::{prelude::*, widgets::*};
use symbols::border::ROUNDED;

pub struct HeaderWidget {}

impl Widget for HeaderWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().border_set(ROUNDED).title("header");
        Paragraph::new("content").block(block).render(area, buf);
    }
}

impl HeaderWidget {
    pub fn new() -> Self {
        Self {}
    }
}
