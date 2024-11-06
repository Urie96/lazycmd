use ratatui::{prelude::*, widgets::Widget};

pub trait Renderable {
    fn render(&self, area: Rect, buf: &mut ratatui::buffer::Buffer);
}

#[derive(Default)]
pub struct Preview {
    pub offset: usize,
    pub content: Option<Box<dyn Renderable>>,
}

impl Preview {
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if let Some(content) = &self.content {
            content.render(area, buf)
        } else {
            Text::raw("Loading...").render(area, buf)
        };
    }

    pub fn clear(&mut self) {
        self.offset = 0;
        self.content = None;
    }
}
