use anyhow::Result;

mod app;
mod tui;
mod ui;

fn main() -> Result<()> {
    tui::run()
}


