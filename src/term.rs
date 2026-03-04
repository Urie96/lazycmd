use std::io::{stdout, Stdout};

use anyhow::Result;
use crossterm::{execute, terminal::*};
use ratatui::prelude::*;

pub type Term = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<Term> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    // Don't hide cursor globally - let app control it based on mode
    Ok(terminal)
}

pub fn restore() {
    disable_raw_mode().ok();
    execute!(stdout(), LeaveAlternateScreen).ok();
}
