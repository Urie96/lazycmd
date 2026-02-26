use mlua::prelude::*;
use ratatui::text::Line;

pub struct Text(pub ratatui::text::Text<'static>);

impl LuaUserData for Text {}

#[derive(Clone)]
pub struct Span(pub ratatui::text::Span<'static>);

impl LuaUserData for Span {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method_mut(LuaMetaMethod::Concat, |_, this, other: mlua::Value| {
            match other {
                mlua::Value::UserData(ud) => {
                    if let Ok(other_span) = ud.borrow::<Span>() {
                        // Span .. Span -> Text with both spans
                        let line = Line::from(vec![this.0.clone(), other_span.0.clone()]);
                        Ok(Text(ratatui::text::Text::from(line)))
                    } else if let Ok(other_text) = ud.borrow::<Text>() {
                        // Span .. Text -> Text (prepend span)
                        let mut lines = other_text.0.lines.clone();
                        if lines.is_empty() {
                            lines.push(Line::from(this.0.clone()));
                        } else {
                            let mut spans = lines[0].spans.clone();
                            spans.insert(0, this.0.clone());
                            lines[0] = Line::from(spans);
                        }
                        Ok(Text(ratatui::text::Text::from(lines)))
                    } else {
                        Err(LuaError::runtime("Cannot concat Span with this type"))
                    }
                }
                mlua::Value::String(s) => {
                    // Span .. String -> Text
                    let text = format!("{}{}", this.0.content, s.to_string_lossy());
                    Ok(Text(ratatui::text::Text::from(text)))
                }
                _ => Err(LuaError::runtime("Cannot concat Span with this type")),
            }
        });
    }
}
