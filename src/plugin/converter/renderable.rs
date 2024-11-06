use ansi_to_tui::IntoText;
use mlua::{prelude::*, FromLua};
use ratatui::widgets::Widget;

use crate::preview::Renderable;

impl Renderable for String {
    fn render(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::buffer::Buffer) {
        ratatui::text::Text::from(self.as_str()).render(area, buf);
    }
}

struct Text(ratatui::text::Text<'static>);

impl Renderable for Text {
    fn render(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::buffer::Buffer) {
        (&self.0).render(area, buf);
    }
}

impl FromLua for Box<dyn Renderable> {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        Ok(match value {
            LuaValue::String(s) => Box::new(Text(s.as_bytes().into_text().into_lua_err()?)),
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
