use ratatui::widgets;

#[derive(Default)]
pub struct PageEntry {
    key: String,
}

#[derive(Default)]
pub struct Page {
    pub list: Vec<PageEntry>,
    pub state: widgets::ListState,
    pub list_state: widgets::ListState,
}
