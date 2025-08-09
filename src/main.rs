use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System};

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App state
    let mut app = App::new();

    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture).ok();
    terminal.show_cursor().ok();

    if let Err(err) = res {
        eprintln!("error: {err:?}");
    }
    Ok(())
}

struct App {
    sys: System,
    sort_by: SortBy,
    filter: String,
    is_filtering: bool,
    selected_index: usize,
    tick_rate: Duration,
}

#[derive(Copy, Clone)]
enum SortBy {
    Cpu,
    Mem,
    Pid,
}

impl App {
    fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
        );
        sys.refresh_all();
        Self {
            sys,
            sort_by: SortBy::Cpu,
            filter: String::new(),
            is_filtering: false,
            selected_index: 0,
            tick_rate: Duration::from_millis(500),
        }
    }

    fn refresh(&mut self) {
        self.sys.refresh_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::new())
                .with_processes(ProcessRefreshKind::everything()),
        );
    }
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        app.refresh();
        terminal.draw(|f| ui(f, app))?;

        // Input with timeout so we keep refreshing
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
                            KeyCode::Char('/') => {
                                app.filter.clear();
                                app.is_filtering = true;
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

fn ui(frame: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Length(3), // cpu/mem
            Constraint::Min(5),    // process list
            Constraint::Length(1), // footer
        ])
        .split(frame.size());

    render_header(frame, chunks[0]);
    render_system(frame, chunks[1], app);
    render_processes(frame, chunks[2], app);
    render_footer(frame, chunks[3]);
}

fn render_header(frame: &mut ratatui::Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("rust-top ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("- q: quit, j/k: nav, c/m/p: sort by cpu/mem/pid, /: filter"),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Overview"));
    frame.render_widget(title, area);
}

fn render_system(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let total_mem = app.sys.total_memory();
    let used_mem = app.sys.used_memory();
    let mem_percent = if total_mem == 0 { 0.0 } else { (used_mem as f64 / total_mem as f64) * 100.0 };

    let global_cpu = app
        .sys
        .global_cpu_info()
        .cpu_usage();

    let lines = vec![
        Line::from(vec![
            Span::styled("CPU: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{global_cpu:.1}%")),
        ]),
        Line::from(vec![
            Span::styled("Mem: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:.1}% ({} / {} MiB)", mem_percent, used_mem / 1024, total_mem / 1024)),
        ]),
    ];

    let p = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("System"))
        .wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn render_processes(frame: &mut ratatui::Frame, area: Rect, app: &mut App) {
    // Collect processes into a vec and sort
    let mut processes: Vec<_> = app
        .sys
        .processes()
        .iter()
        .map(|(pid, p)| (*pid, p))
        .collect();

    match app.sort_by {
        SortBy::Cpu => processes.sort_by(|a, b| b.1.cpu_usage().total_cmp(&a.1.cpu_usage())),
        SortBy::Mem => processes.sort_by(|a, b| b.1.memory().cmp(&a.1.memory())),
        SortBy::Pid => processes.sort_by(|a, b| b.0.cmp(&a.0)),
    }

    // Optional filtering by name substring
    let filter_lower = app.filter.to_lowercase();
    if !filter_lower.is_empty() {
        processes.retain(|(_, p)| p.name().to_string().to_lowercase().contains(&filter_lower));
    }

    // Clamp selection
    if app.selected_index >= processes.len() {
        app.selected_index = processes.len().saturating_sub(1);
    }

    // Render list items
    let header = ListItem::new(Line::from(vec![
        Span::styled(format!("{:>6}  {:>5}  {:>6}  {}", "PID", "%CPU", "MEM", "NAME"), Style::default().add_modifier(Modifier::BOLD)),
    ]));

    let mut items: Vec<ListItem> = Vec::with_capacity(processes.len() + 1);
    items.push(header);

    for (i, (pid, proc_)) in processes.iter().enumerate().take(200) { // limit list for performance
        let name = proc_.name().to_string();
        let cpu = proc_.cpu_usage();
        let mem_kib = proc_.memory();
        let style = if i == app.selected_index { Style::default().bg(Color::Blue).fg(Color::White) } else { Style::default() };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{:>6}  {:>5.1}  {:>6}  {}", format!("{pid}"), cpu, mem_kib / 1024, name), style),
        ])));
    }

    let title_text = if app.filter.is_empty() {
        "Processes".to_string()
    } else {
        format!("Processes | filter: {}{}", app.filter, if app.is_filtering { "_" } else { "" })
    };
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title_text));
    frame.render_widget(list, area);
}

fn render_footer(frame: &mut ratatui::Frame, area: Rect) {
    let parts = vec![
        Span::raw("Press q to quit. "),
        Span::styled("Arrows/jk", Style::default().fg(Color::Green)),
        Span::raw(" move, "),
        Span::styled("c/m/p", Style::default().fg(Color::Green)),
        Span::raw(" sort. "),
    ];
    // Show filter state when engaged
    // Safe to access global APP via closure? We'll pass needed info by updating signature if needed
    // Workaround: display a hint here; active filter string shown in process title.
    let text = Line::from(parts);
    let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(p, area);
}
