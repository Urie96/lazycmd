use ratatui::{prelude::*, text::Text, widgets::Widget};

pub trait Renderable {
    fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer);
}

#[derive(Debug, Default)]
pub struct Preview {
    pub offset: usize,
    pub content: Option<Text<'static>>,
}

impl Preview {
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if let Some(content) = &self.content {
            content.render(area, buf)
        } else {
            Text::raw("Loading...").render(area, buf)
        };
    }
}
