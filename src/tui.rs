use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::{App, SortBy};
use crate::ui::ui;

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    if let Err(err) = res {
        eprintln!("error: {err:?}");
    }
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        app.refresh();
        terminal.draw(|f| ui(f, app))?;

        if event::poll(app.tick_rate)? {
            match event::read()? {
                Event::Key(key) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        return Ok(());
                    }
                    if app.is_filtering {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.is_filtering = false;
                            }
                            KeyCode::Backspace => {
                                app.filter.pop();
                            }
                            KeyCode::Char(ch) => {
                                app.filter.push(ch);
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Up | KeyCode::Char('k') => app.selected_index = app.selected_index.saturating_sub(1),
                            KeyCode::Down | KeyCode::Char('j') => app.selected_index = app.selected_index.saturating_add(1),
                            KeyCode::PageUp => {
                                app.selected_index = app.selected_index.saturating_sub(10);
                            }
                            KeyCode::PageDown => {
                                app.selected_index = app.selected_index.saturating_add(10);
                            }
                            KeyCode::Home | KeyCode::Char('g') => {
                                app.selected_index = 0;
                            }
                            KeyCode::End | KeyCode::Char('G') => {
                                app.selected_index = usize::MAX;
                            }
                            KeyCode::Char('/') => {
                                app.filter.clear();
                                app.is_filtering = true;
                            }
                            KeyCode::Char('?') => {
                                app.show_help = !app.show_help;
                            }
                            KeyCode::Char('+') => {
                                let ms = (app.tick_rate.as_millis() as u64).saturating_sub(50).max(100);
                                app.tick_rate = Duration::from_millis(ms);
                            }
                            KeyCode::Char('-') => {
                                let ms = (app.tick_rate.as_millis() as u64).saturating_add(50).min(2000);
                                app.tick_rate = Duration::from_millis(ms);
                            }
                            KeyCode::Char('c') => app.sort_by = SortBy::Cpu,
                            KeyCode::Char('m') => app.sort_by = SortBy::Mem,
                            KeyCode::Char('p') => app.sort_by = SortBy::Pid,
                            _ => {}
                        }
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }
}


