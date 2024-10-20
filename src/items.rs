use ratatui::widgets;

pub struct Entry {
    key: String,
}

pub struct PageItems {
    pub list: Vec<Entry>,
    pub state: widgets::ListState,
}
