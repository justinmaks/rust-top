use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};
// sysinfo re-exports used via fully-qualified paths below; no trait imports needed

use crate::app::{App, SortBy};

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(7),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(frame.size());

    render_header(frame, chunks[0]);
    render_system(frame, chunks[1], app);
    render_processes(frame, chunks[2], app);
    render_footer(frame, chunks[3]);

    if app.show_help {
        render_help_popup(frame);
    }
}

fn render_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("rust-top ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("- q: quit, j/k: nav, c/m/p: sort by cpu/mem/pid, /: filter"),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Overview"));
    frame.render_widget(title, area);
}

fn render_system(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title("System");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let total_mem = app.sys.total_memory();
    let used_mem = app.sys.used_memory();
    let mem_percent = if total_mem == 0 { 0.0 } else { (used_mem as f64 / total_mem as f64) * 100.0 };
    let global_cpu = app.sys.global_cpu_info().cpu_usage();

    let sys_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(inner);

    let summary = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("CPU: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{global_cpu:.1}%")),
        ]),
        Line::from(vec![
            Span::styled("Mem: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:.1}% ({} / {} MiB)", mem_percent, used_mem / 1024, total_mem / 1024)),
        ]),
    ])
    .wrap(Wrap { trim: true });
    frame.render_widget(summary, sys_chunks[0]);

    let per_core = &app.sys.cpus();
    if !per_core.is_empty() {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(sys_chunks[1]);
        let half = (per_core.len() + 1) / 2;
        let left = &per_core[..half];
        let right = &per_core[half..];
        render_cpu_column(frame, cols[0], 0, left);
        render_cpu_column(frame, cols[1], half, right);
    }
}

fn render_cpu_column(frame: &mut Frame, area: Rect, offset: usize, cpus: &[sysinfo::Cpu]) {
    let rows = cpus.len() as u16;
    if rows == 0 || area.height == 0 {
        return;
    }
    let mut constraints = Vec::new();
    for _ in 0..rows.min(area.height) {
        constraints.push(Constraint::Length(1));
    }
    let chunks = Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);
    for (i, cpu) in cpus.iter().enumerate().take(chunks.len()) {
        let usage = cpu.cpu_usage();
        let label = format!("CPU {:>2}: {:>4.1}%", offset + i, usage);
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Green))
            .ratio((usage as f64 / 100.0).clamp(0.0, 1.0))
            .label(Span::raw(label));
        frame.render_widget(gauge, chunks[i]);
    }
}

fn render_processes(frame: &mut Frame, area: Rect, app: &mut App) {
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

    let filter_lower = app.filter.to_lowercase();
    if !filter_lower.is_empty() {
        processes.retain(|(_, p)| p.name().to_string().to_lowercase().contains(&filter_lower));
    }

    if let Some(sel_pid) = app.selected_pid {
        if let Some(pos) = processes.iter().position(|(pid, _)| *pid == sel_pid) {
            app.selected_index = pos;
        }
    }
    if app.selected_index >= processes.len() {
        app.selected_index = processes.len().saturating_sub(1);
    }
    if let Some((pid, _)) = processes.get(app.selected_index) {
        app.selected_pid = Some(*pid);
    }

    let header = ListItem::new(Line::from(vec![
        Span::styled(format!("{:>6}  {:>5}  {:>6}  {}", "PID", "%CPU", "MEM", "NAME"), Style::default().add_modifier(Modifier::BOLD)),
    ]));

    let mut items: Vec<ListItem> = Vec::with_capacity(processes.len() + 1);
    items.push(header);

    for (i, (pid, proc_)) in processes.iter().enumerate().take(200) {
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

fn render_footer(frame: &mut Frame, area: Rect) {
    let parts = vec![
        Span::raw("Press q to quit. "),
        Span::styled("Arrows/jk", Style::default().fg(Color::Green)),
        Span::raw(" move, "),
        Span::styled("PgUp/PgDn, g/G", Style::default().fg(Color::Green)),
        Span::raw(" jump, "),
        Span::styled("c/m/p", Style::default().fg(Color::Green)),
        Span::raw(" sort, "),
        Span::styled("/", Style::default().fg(Color::Green)),
        Span::raw(" filter, "),
        Span::styled("?", Style::default().fg(Color::Green)),
        Span::raw(" help, "),
        Span::styled("+/-", Style::default().fg(Color::Green)),
        Span::raw(" tick. "),
    ];
    let text = Line::from(parts);
    let p = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(p, area);
}

fn render_help_popup(frame: &mut Frame) {
    let area = centered_rect(70, 70, frame.size());
    frame.render_widget(Clear, area);
    let help = Paragraph::new(vec![
        Line::from("rust-top - keys:"),
        Line::from("  q/Esc/Ctrl-C: quit"),
        Line::from("  j/k or arrows: move selection"),
        Line::from("  PgUp/PgDn, g/G: page/top/bottom"),
        Line::from("  c/m/p: sort by CPU/mem/PID"),
        Line::from("  / then type: filter by name; Enter/Esc to finish"),
        Line::from("  +/-: adjust refresh rate"),
        Line::from("  ?: toggle this help"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);
    horizontal[1]
}


