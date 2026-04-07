use mlua::{prelude::*, FromLua};
use ratatui::{
    layout::Rect,
    prelude::*,
    text::{Line, Text},
    widgets::Widget,
};
use std::io::Write;

use super::{native_image, LuaImage, LuaLine, LuaSpan, LuaText};

pub trait Renderable {
    fn render(&mut self, area: Rect, buf: &mut ratatui::buffer::Buffer);
    #[allow(unused)]
    fn scroll_by(&mut self, offset: i16) {}
    #[allow(unused)]
    fn set_native_enabled(&mut self, enabled: bool) {
        let _ = enabled;
    }
    fn render_native(&mut self, backend: &mut dyn Write) -> anyhow::Result<bool> {
        let _ = backend;
        Ok(false)
    }
}

impl FromLua for Box<dyn Renderable> {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        Ok(match value {
            LuaValue::String(s) => Box::new(StatefulParagraph::from(s.to_string_lossy())),
            LuaValue::UserData(ud) => {
                if let Ok(text) = ud.take::<LuaText>() {
                    Box::new(StatefulParagraph::from(text.0))
                } else if let Ok(image) = ud.take::<LuaImage>() {
                    Box::new(MixedPreview::new(vec![PreviewChunk::Image(image)]))
                } else if let Ok(span) = ud.take::<LuaSpan>() {
                    Box::new(StatefulParagraph::from(Text::from(span.0)))
                } else if let Ok(line) = ud.take::<LuaLine>() {
                    Box::new(StatefulParagraph::from(Text::from(line.0)))
                } else {
                    Err("expected string, preview array, or preview userdata".into_lua_err())?
                }
            }
            LuaValue::Table(table) => Box::new(MixedPreview::new(table_to_chunks(lua, table)?)),
            _ => Err("expected string, preview array, or preview userdata".into_lua_err())?,
        })
    }
}

fn table_to_chunks(_lua: &Lua, table: LuaTable) -> mlua::Result<Vec<PreviewChunk>> {
    let mut chunks = Vec::with_capacity(table.raw_len());
    for value in table.sequence_values::<LuaValue>() {
        chunks.push(PreviewChunk::from_lua_value(value?)?);
    }
    Ok(chunks)
}

enum PreviewChunk {
    Text(Text<'static>),
    Image(LuaImage),
}

#[derive(Clone)]
struct NativePlacement {
    image: LuaImage,
    visual_start: u16,
    logical_start: usize,
    logical_len: usize,
    area: Rect,
}

impl PreviewChunk {
    fn from_lua_value(value: LuaValue) -> mlua::Result<Self> {
        match value {
            LuaValue::String(s) => Ok(Self::Text(Text::raw(s.to_str()?.to_string()))),
            LuaValue::UserData(ud) => {
                if let Ok(text) = ud.take::<LuaText>() {
                    Ok(Self::Text(text.0))
                } else if let Ok(image) = ud.take::<LuaImage>() {
                    Ok(Self::Image(image))
                } else if let Ok(line) = ud.take::<LuaLine>() {
                    Ok(Self::Text(Text::from(line.0)))
                } else if let Ok(span) = ud.take::<LuaSpan>() {
                    Ok(Self::Text(Text::from(span.0)))
                } else {
                    Err(
                        "expected Text, Image, Line, Span, or string in preview array"
                            .into_lua_err(),
                    )
                }
            }
            _ => Err("expected Text, Image, Line, Span, or string in preview array".into_lua_err()),
        }
    }
}

#[derive(Default)]
pub struct StatefulParagraph {
    paragraph: ratatui::widgets::Paragraph<'static>,
    offset: u16,
    scrollbar_state: ratatui::widgets::ScrollbarState,
}

impl<T> From<T> for StatefulParagraph
where
    T: Into<Text<'static>>,
{
    fn from(value: T) -> Self {
        let text: Text = value.into();
        let total_height = text.height().clamp(0, u16::MAX as usize) as u16;
        Self {
            paragraph: ratatui::widgets::Paragraph::new(text)
                .wrap(ratatui::widgets::Wrap { trim: false }),
            scrollbar_state: ratatui::widgets::ScrollbarState::new(total_height as usize),
            ..Default::default()
        }
    }
}

impl LuaUserData for StatefulParagraph {}

impl Renderable for StatefulParagraph {
    fn render(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::buffer::Buffer) {
        let [para_area, scrollbar_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let total_height = self.paragraph.line_count(para_area.width);
        self.scrollbar_state = self
            .scrollbar_state
            .content_length(para_area.width as usize);

        self.offset = self
            .offset
            .clamp(0, (total_height as u16).saturating_sub(area.height));
        self.paragraph = std::mem::take(&mut self.paragraph).scroll((self.offset, 0));
        self.scrollbar_state = self
            .scrollbar_state
            .content_length(total_height.saturating_sub(area.height as usize))
            .position(self.offset as usize);

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

pub struct MixedPreview {
    chunks: Vec<PreviewChunk>,
    offset: u16,
    native_enabled: bool,
    scrollbar_state: ratatui::widgets::ScrollbarState,
    cached_width: u16,
    cached_height: u16,
    cached_lines: Vec<Line<'static>>,
    native_layouts: Vec<NativePlacement>,
    visible_native: Vec<NativePlacement>,
}

impl MixedPreview {
    fn new(chunks: Vec<PreviewChunk>) -> Self {
        Self {
            chunks,
            offset: 0,
            native_enabled: true,
            scrollbar_state: ratatui::widgets::ScrollbarState::new(0),
            cached_width: 0,
            cached_height: 0,
            cached_lines: Vec::new(),
            native_layouts: Vec::new(),
            visible_native: Vec::new(),
        }
    }

    fn rebuild(&mut self, width: u16, viewport_height: u16) -> mlua::Result<()> {
        let mut lines = Vec::new();
        let mut native_layouts = Vec::new();
        let mut visual_offset = 0u16;
        let native_enabled = native_image::protocol().is_some();
        for chunk in &self.chunks {
            match chunk {
                PreviewChunk::Text(text) => {
                    visual_offset = visual_offset.saturating_add(
                        rendered_text_height(text, width).min(u16::MAX as usize) as u16,
                    );
                    lines.extend(text.lines.clone());
                }
                PreviewChunk::Image(image) => {
                    let native_size = if native_enabled && viewport_height > 0 {
                        native_image::measure_cell_area(
                            &image.path,
                            Rect::new(0, 0, width, viewport_height),
                        )
                        .ok()
                    } else {
                        None
                    };
                    let fallback_width = native_size
                        .as_ref()
                        .map(|rect| rect.width.max(1))
                        .unwrap_or(width);
                    let fallback_height = native_size
                        .as_ref()
                        .map(|rect| rect.height.max(1))
                        .or(image.max_height);
                    let rendered = image.render_block_preview(fallback_width, fallback_height)?;
                    if let Some(native_area) =
                        native_size.filter(|rect| rect.width > 0 && rect.height > 0)
                    {
                        native_layouts.push(NativePlacement {
                            image: image.clone(),
                            visual_start: visual_offset,
                            logical_start: lines.len(),
                            logical_len: rendered.lines.len(),
                            area: Rect::new(
                                0,
                                visual_offset,
                                native_area.width.min(rendered.width.max(1)),
                                rendered.height.max(1),
                            ),
                        });
                    }
                    visual_offset = visual_offset.saturating_add(rendered.height);
                    lines.extend(rendered.lines);
                }
            }
        }

        if lines.is_empty() {
            lines.push(Line::raw(""));
        }

        self.cached_lines = lines;
        self.cached_width = width;
        self.cached_height = viewport_height;
        self.native_layouts = native_layouts;
        Ok(())
    }
}

impl Renderable for MixedPreview {
    fn render(&mut self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let [para_area, scrollbar_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        if para_area.width == 0 || para_area.height == 0 {
            return;
        }

        if self.cached_width != para_area.width || self.cached_height != para_area.height {
            if let Err(err) = self.rebuild(para_area.width, para_area.height) {
                self.cached_lines = vec![Line::raw(err.to_string())];
                self.cached_width = para_area.width;
                self.cached_height = para_area.height;
            }
        }

        let paragraph = ratatui::widgets::Paragraph::new(Text::from(self.cached_lines.clone()))
            .wrap(ratatui::widgets::Wrap { trim: false });
        let total_height = paragraph.line_count(para_area.width);

        self.offset = self
            .offset
            .clamp(0, (total_height as u16).saturating_sub(area.height));
        self.scrollbar_state = self
            .scrollbar_state
            .content_length(total_height.saturating_sub(area.height as usize))
            .position(self.offset as usize);

        self.visible_native.clear();
        if self.native_enabled {
            for placement in &self.native_layouts {
                let start = placement.area.y;
                let end = placement.area.y.saturating_add(placement.area.height);
                let view_start = self.offset;
                let view_end = self.offset.saturating_add(para_area.height);

                if start < view_start || end > view_end {
                    continue;
                }

                self.visible_native.push(NativePlacement {
                    image: placement.image.clone(),
                    visual_start: placement.visual_start,
                    logical_start: placement.logical_start,
                    logical_len: placement.logical_len,
                    area: Rect::new(
                        para_area.x,
                        para_area.y + start.saturating_sub(self.offset),
                        placement.area.width.min(para_area.width),
                        placement.area.height.min(para_area.height),
                    ),
                });
            }
        }

        let mut display_lines = self.cached_lines.clone();
        let blank_line = Line::raw(" ".repeat(para_area.width as usize));
        for placement in &self.visible_native {
            let start = placement.logical_start;
            let end = start.saturating_add(placement.logical_len);
            for idx in start..end.min(display_lines.len()) {
                display_lines[idx] = blank_line.clone();
            }
        }

        ratatui::widgets::Paragraph::new(Text::from(display_lines))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((self.offset, 0))
            .render(para_area, buf);

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

    fn set_native_enabled(&mut self, enabled: bool) {
        self.native_enabled = enabled;
    }

    fn render_native(&mut self, backend: &mut dyn Write) -> anyhow::Result<bool> {
        let mut rendered = false;
        for placement in &self.visible_native {
            rendered |= native_image::render(backend, &placement.image.path, placement.area)?;
        }
        Ok(rendered)
    }
}

fn rendered_text_height(text: &Text<'_>, width: u16) -> usize {
    let width = width.max(1) as usize;
    text.lines
        .iter()
        .map(|line| {
            let line_width = line.width();
            if line_width == 0 {
                1
            } else {
                line_width.div_ceil(width)
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    use ratatui::buffer::Buffer;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn preview_array_is_converted_to_mixed_preview() {
        let lua = Lua::new();
        let preview: Box<dyn Renderable> = lua
            .load(r#"return { "head", "tail" }"#)
            .eval()
            .expect("preview array should convert");

        let mut preview = preview;
        let area = Rect::new(0, 0, 20, 4);
        let mut buf = Buffer::empty(area);
        preview.render(area, &mut buf);

        assert_eq!(buf[(0, 0)].symbol(), "h");
        assert_eq!(buf[(0, 1)].symbol(), "t");
    }

    #[test]
    fn mixed_preview_keeps_fallback_when_native_is_disabled() {
        let mut image = RgbaImage::new(2, 2);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        image.put_pixel(1, 0, Rgba([0, 255, 0, 255]));
        image.put_pixel(0, 1, Rgba([0, 0, 255, 255]));
        image.put_pixel(1, 1, Rgba([255, 255, 0, 255]));

        let path = std::env::temp_dir().join(format!(
            "lazycmd-renderable-native-disabled-{}.png",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_nanos()
        ));
        image.save(&path).expect("save temp image");

        let mut preview = MixedPreview::new(vec![PreviewChunk::Image(LuaImage::new(
            path.clone(),
            None,
            None,
        ))]);
        preview.set_native_enabled(false);

        let area = Rect::new(0, 0, 10, 4);
        let mut buf = Buffer::empty(area);
        preview.render(area, &mut buf);

        assert_eq!(buf[(0, 0)].symbol(), "▀");

        std::fs::remove_file(path).ok();
    }
}
