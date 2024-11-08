use mlua::{ExternalResult, FromLua, UserData};
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
            .highlight_style(Style::default().bg(Color::Red).fg(Color::White))
            .highlight_spacing(widgets::HighlightSpacing::Always);
        StatefulWidget::render(list, area, buf, &mut page.list_state);
    }
}

pub struct ListItem(ratatui::widgets::ListItem<'static>);

impl<T> From<T> for ListItem
where
    T: Into<ratatui::text::Text<'static>>,
{
    fn from(value: T) -> Self {
        Self(ratatui::widgets::ListItem::new(value))
    }
}

impl FromLua for ListItem {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        Ok(match value {
            mlua::Value::String(s) => s.to_string_lossy().into(),
            mlua::Value::UserData(ud) => {
                if let Ok(span) = ud.take::<Span>() {
                    span.into()
                } else {
                    Err("Expect ListItem").into_lua_err()?
                }
            }
            _ => Err("Expect ListItem").into_lua_err()?,
        })
    }
}
