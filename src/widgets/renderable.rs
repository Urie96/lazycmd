use mlua::{prelude::*, FromLua};
use ratatui::{
    layout::Rect,
    prelude::*,
    widgets::{List, ListItem, ListState, Widget},
};

use super::Text;

pub trait Renderable {
    fn render(&mut self, area: Rect, buf: &mut ratatui::buffer::Buffer);
    #[allow(unused)]
    fn scroll_by(&mut self, offset: i16) {}
}

impl FromLua for Box<dyn Renderable> {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        Ok(match value {
            LuaValue::String(s) => Box::new(StatefulParagraph::from(s.to_string_lossy())),
            LuaValue::UserData(ud) => {
                if let Ok(text) = ud.take::<Text>() {
                    Box::new(StatefulParagraph::from(text.0))
                } else {
                    Err("expect string or userdata ListItem".into_lua_err())?
                }
            }
            // LuaValue::UserData(ud) => {
            //     if let Ok(span) = ud.take::<Text>() {
            //         Text(ratatui::widgets::Text::from(span.0))
            //     } else {
            //         Err("expect string or userdata ListItem".into_lua_err())?
            //     }
            // }
            _ => Err("expect string or userdata ListItem".into_lua_err())?,
        })
    }
}

#[derive(Default)]
pub struct StatefulParagraph {
    paragraph: ratatui::widgets::Paragraph<'static>,
    offset: u16,
    total_height: u16,
    scrollbar_state: ratatui::widgets::ScrollbarState,
}

impl<T> From<T> for StatefulParagraph
where
    T: Into<ratatui::text::Text<'static>>,
{
    fn from(value: T) -> Self {
        let text: ratatui::text::Text = value.into();
        let total_height = text.height().clamp(0, u16::MAX as usize) as u16;
        Self {
            total_height,
            paragraph: ratatui::widgets::Paragraph::new(text),
            scrollbar_state: ratatui::widgets::ScrollbarState::new(total_height as usize),
            ..Default::default()
        }
    }
}

impl LuaUserData for StatefulParagraph {}

impl Renderable for StatefulParagraph {
    fn render(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::buffer::Buffer) {
        self.offset = self
            .offset
            .clamp(0, self.total_height.saturating_sub(area.height));
        self.paragraph = std::mem::take(&mut self.paragraph).scroll((self.offset, 0));
        self.scrollbar_state = self
            .scrollbar_state
            .content_length(self.total_height.saturating_sub(area.height) as usize)
            .position(self.offset as usize);

        let [para_area, scrollbar_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        (&self.paragraph).render(para_area, buf);

        ratatui::widgets::Scrollbar::default()
            .track_symbol(Some(" "))
            .thumb_symbol("▐")
            .begin_symbol(None)
            .end_symbol(None)
            .render(scrollbar_area, buf, &mut self.scrollbar_state);
    }

    fn scroll_by(&mut self, offset: i16) {
        self.offset = self.offset.saturating_add_signed(offset);
    }
}

#[derive(Default)]
pub struct StatefulList {
    list: List<'static>,
    offset: u16,
    total_height: u16,
    scrollbar_state: ratatui::widgets::ScrollbarState,
}

impl<T> From<T> for StatefulList
where
    T: IntoIterator,
    T::Item: Into<ListItem<'static>>,
{
    fn from(value: T) -> Self {
        let list = List::new(value);
        let total_height = list.len().clamp(0, u16::MAX as usize) as u16;
        Self {
            list,
            total_height,
            ..Default::default()
        }
    }
}

impl Renderable for StatefulList {
    fn render(&mut self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        self.offset = self
            .offset
            .clamp(0, self.total_height.saturating_sub(area.height));

        let mut state = ListState::default().with_offset(self.offset as usize);
        StatefulWidget::render(&self.list, area, buf, &mut state);
    }

    fn scroll_by(&mut self, offset: i16) {
        self.offset = self.offset.saturating_add_signed(offset);
    }
}
