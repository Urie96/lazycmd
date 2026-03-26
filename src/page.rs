use crate::widgets::{LuaLine, LuaSpan, LuaText};
use anyhow::bail;
use mlua::prelude::*;
use ratatui::{text::Line, widgets};

#[derive(Clone)]
pub struct PageEntry {
    pub key: String,
    pub tbl: LuaTable,
}

impl FromLua for PageEntry {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        let tbl = LuaTable::from_lua(value, lua)?;
        let key: String = tbl.get("key")?;
        Ok(Self { key, tbl })
    }
}

impl PageEntry {
    pub fn keymap_table(&self) -> LuaResult<Option<LuaTable>> {
        match self.tbl.get::<LuaValue>("keymap")? {
            LuaValue::Nil => Ok(None),
            LuaValue::Table(tbl) => Ok(Some(tbl)),
            other => Err(LuaError::RuntimeError(format!(
                "entry.keymap must be a table, got {}",
                other.type_name()
            ))),
        }
    }

    /// Extract the Text content from the display field
    pub fn display(&self) -> Line<'_> {
        let f = || match self.tbl.get::<LuaValue>("display")? {
            LuaValue::Nil => Ok(Line::from(self.key.as_str())),
            LuaValue::String(s) => Ok(Line::from(s.to_string_lossy())),
            LuaValue::UserData(ud) => {
                if let Ok(span) = ud.borrow::<LuaSpan>() {
                    Ok(Line::from(span.0.clone()))
                } else if let Ok(line) = ud.borrow::<LuaLine>() {
                    Ok(Line::from(line.0.clone()))
                } else {
                    bail!("Expected Span, Line, Text, or nil")
                }
            }
            _ => bail!("Expected Span, Line, string, or nil"),
        };
        f().unwrap_or_else(|e| Line::from(e.to_string()))
    }
}

#[derive(Default)]
pub struct Page {
    pub list: Vec<PageEntry>,
    pub filtered_list: Vec<PageEntry>,
    pub list_state: widgets::ListState,
    /// List filter string for this page
    pub list_filter: String,
}

impl Page {
    /// Extract display text from a PageEntry
    fn extract_display_text(&self, entry: &PageEntry) -> String {
        match entry.tbl.get::<LuaValue>("display") {
            Ok(LuaValue::Nil) => entry.key.clone(),
            Ok(LuaValue::String(s)) => s.to_string_lossy().to_string(),
            Ok(LuaValue::UserData(ud)) => {
                if let Ok(span) = ud.borrow::<LuaSpan>() {
                    span.0.to_string()
                } else if let Ok(line) = ud.borrow::<LuaLine>() {
                    line.0.to_string()
                } else if let Ok(text) = ud.borrow::<LuaText>() {
                    // Text implements Display, to_string() returns lines joined by '\n'
                    text.0.to_string()
                } else {
                    entry.key.clone()
                }
            }
            _ => entry.key.clone(),
        }
    }

    /// Apply filter to the list, updating filtered_list
    pub fn apply_filter(&mut self) {
        self.filtered_list = if self.list_filter.is_empty() {
            self.list.clone()
        } else {
            let filter_lower = self.list_filter.to_lowercase();
            self.list
                .iter()
                .filter(|entry| {
                    let key_lower = entry.key.to_lowercase();
                    let display_lower = self.extract_display_text(entry).to_lowercase();
                    key_lower.contains(&filter_lower) || display_lower.contains(&filter_lower)
                })
                .cloned()
                .collect()
        };

        // Reset selection to first item or none if empty
        if self.filtered_list.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }
}
