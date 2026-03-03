use ratatui::{prelude::*, widgets};

use crate::Page;

/// List widget with scrolloff - keeps cursor away from edges (like vim's scrolloff)
pub struct ListWidget {
    pub scrolloff: usize,
}

impl Default for ListWidget {
    fn default() -> Self {
        Self { scrolloff: 5 }
    }
}

impl StatefulWidget for ListWidget {
    type State = Page;

    fn render(self, area: Rect, buf: &mut Buffer, page: &mut Self::State) {
        let list = widgets::List::new(page.filtered_list.iter().map(|entry| entry.display()))
            .block(widgets::Block::default())
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::Black))
            .highlight_spacing(widgets::HighlightSpacing::Always);

        // Adjust offset based on scrolloff before rendering
        if let Some(selected) = page.list_state.selected() {
            let height = area.height as usize;
            let scrolloff = self.scrolloff.min(height / 2);
            let offset = page.list_state.offset();
            let cursor_pos = selected.saturating_sub(offset);
            let len = page.filtered_list.len();

            // When cursor is in the top scrolloff zone, scroll up to keep cursor at scrolloff
            if cursor_pos < scrolloff && offset > 0 {
                // Keep cursor at scrolloff position
                let new_offset = selected.saturating_sub(scrolloff);
                *page.list_state.offset_mut() = new_offset;
            }
            // When cursor is in the bottom scrolloff zone, scroll down
            else if cursor_pos >= height.saturating_sub(scrolloff) {
                let desired_pos = height.saturating_sub(scrolloff).saturating_sub(1);
                if selected >= desired_pos {
                    let new_offset = selected.saturating_sub(desired_pos);
                    // Limit offset so the last item is at or near bottom
                    let max_offset = if len > height { len - height } else { 0 };
                    *page.list_state.offset_mut() = new_offset.min(max_offset);
                }
            }
        }

        StatefulWidget::render(list, area, buf, &mut page.list_state);
    }
}
