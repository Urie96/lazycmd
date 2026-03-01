use crate::widgets::Span;
use anyhow::bail;
use mlua::prelude::*;
use ratatui::{
    text::Line,
    widgets::{self, ListItem},
};

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
    pub fn display(&self) -> ListItem<'_> {
        let f = || match self.tbl.get::<LuaValue>("display")? {
            LuaValue::Nil => Ok(ListItem::new(Line::from(self.key.as_str()))),
            LuaValue::String(s) => Ok(ListItem::new(Line::from(s.to_string_lossy()))),
            LuaValue::UserData(ud) => {
                if let Ok(span) = ud.borrow::<Span>() {
                    Ok(ListItem::new(span.clone().0))
                } else {
                    bail!("Expected Span or nil")
                }
            }
            _ => bail!("Expected Span or nil"),
        };
        f().unwrap_or_else(|e| ListItem::new(e.to_string()))
    }
}

#[derive(Default)]
pub struct Page {
    pub list: Vec<PageEntry>,
    pub filtered_list: Vec<PageEntry>,
    pub list_state: widgets::ListState,
    pub filter_input: String,
    pub input_cursor_position: usize,
}

impl Page {
    /// Extract display text from a PageEntry
    fn extract_display_text(&self, entry: &PageEntry) -> String {
        match entry.tbl.get::<LuaValue>("display") {
            Ok(LuaValue::Nil) => entry.key.clone(),
            Ok(LuaValue::String(s)) => s.to_string_lossy().to_string(),
            Ok(LuaValue::UserData(ud)) => {
                if let Ok(span) = ud.borrow::<Span>() {
                    span.0.to_string()
                } else {
                    entry.key.clone()
                }
            }
            _ => entry.key.clone(),
        }
    }

    /// Apply filter to the list, updating filtered_list
    pub fn apply_filter(&mut self, filter: &str) {
        self.filtered_list = if filter.is_empty() {
            self.list.clone()
        } else {
            let filter_lower = filter.to_lowercase();
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
