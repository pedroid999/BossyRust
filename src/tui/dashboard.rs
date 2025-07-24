use crate::tui::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub fn render_dashboard(f: &mut Frame, app: &mut AppState) {
    let size = f.size();

    match app.mode {
        crate::tui::AppMode::Dashboard => render_main_dashboard(f, app, size),
        crate::tui::AppMode::ProcessView => render_process_view(f, app, size),
        crate::tui::AppMode::PortView => render_port_view(f, app, size),
        crate::tui::AppMode::ConnectionView => render_connection_view(f, app, size),
    }

    // Always render status bar
    render_status_bar(f, app, size);

    // Render overlays
    if app.confirmation_dialog.is_some() {
        render_confirmation_dialog(f, app, size);
    }
}

fn render_main_dashboard(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status
        ])
        .split(area);

    // Header
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "BossyRust ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("- Mac Process Manager"),
        ]),
        Line::from("F1: Processes | F2: Ports | F3: Connections | F4: Help | q: Quit"),
    ])
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Main content area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Left panel - Top processes
    let top_processes: Vec<ListItem> = app
        .processes
        .iter()
        .take(10)
        .enumerate()
        .map(|(i, p)| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("{:2}. ", i + 1)),
                Span::styled(
                    format!("{:15}", truncate_string(&p.name, 15)),
                    Style::default().fg(Color::White),
                ),
                Span::raw(format!(" {:>6.1}%", p.cpu_usage)),
                Span::raw(format!(" {:>8}", p.format_memory())),
            ]))
        })
        .collect();

    let processes_list = List::new(top_processes)
        .block(
            Block::default()
                .title("Top Processes (CPU)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_widget(processes_list, main_chunks[0]);

    // Right panel - Port summary
    let listening_ports = app
        .ports
        .iter()
        .filter(|p| matches!(p.state, crate::network::ConnectionState::Listen))
        .count();
    let dev_ports = app.ports.iter().filter(|p| p.is_development_port()).count();

    let port_items: Vec<ListItem> = vec![
        ListItem::new(format!("Total Ports: {}", app.ports.len())),
        ListItem::new(format!("Listening: {listening_ports}")),
        ListItem::new(format!("Development: {dev_ports}")),
        ListItem::new("".to_string()),
        ListItem::new("Recent Activity:"),
    ];

    let port_list = List::new(port_items).block(
        Block::default()
            .title("Port Summary")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );

    f.render_widget(port_list, main_chunks[1]);
}

fn render_process_view(f: &mut Frame, app: &mut AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Process list
            Constraint::Length(3), // Status
        ])
        .split(area);

    // Header
    let sort_indicator = match app.sort_order {
        crate::tui::SortOrder::Ascending => "↑",
        crate::tui::SortOrder::Descending => "↓",
    };

    let header_text = format!(
        "Processes ({}) - Sorted by {:?} {} | / to search | k to kill | space to select",
        app.filtered_processes.len(),
        app.sort_by,
        sort_indicator
    );

    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Process Manager"),
    );
    f.render_widget(header, chunks[0]);

    // Process list
    let items: Vec<ListItem> = app
        .filtered_processes
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let style = if app.selected_items.contains(&i) {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>8} ", p.pid), style),
                Span::styled(
                    format!("{:20} ", truncate_string(&p.name, 20)),
                    style.fg(Color::Cyan),
                ),
                Span::styled(format!("{:>6.1}% ", p.cpu_usage), style),
                Span::styled(format!("{:>10} ", p.format_memory()), style),
                Span::styled(truncate_string(&p.status, 10), style.fg(Color::Gray)),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

fn render_port_view(f: &mut Frame, app: &mut AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Port list
            Constraint::Length(3), // Status
        ])
        .split(area);

    // Header
    let header_text = format!(
        "Ports ({}) | / to search | k to kill | :port for quick search",
        app.filtered_ports.len()
    );

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Port Manager"));
    f.render_widget(header, chunks[0]);

    // Port list
    let items: Vec<ListItem> = app
        .filtered_ports
        .iter()
        .map(|p| {
            let protocol_color = match p.protocol {
                crate::network::Protocol::Tcp => Color::Blue,
                crate::network::Protocol::Udp => Color::Green,
            };

            let state_color = match p.state {
                crate::network::ConnectionState::Listen => Color::Green,
                crate::network::ConnectionState::Established => Color::Cyan,
                _ => Color::Gray,
            };

            let service_info = p
                .get_service_suggestion()
                .unwrap_or_else(|| format!("{:?}", p.state));

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>6} ", p.port), Style::default().fg(Color::White)),
                Span::styled(
                    format!("{:4} ", format!("{:?}", p.protocol)),
                    Style::default().fg(protocol_color),
                ),
                Span::styled(
                    format!("{:12} ", format!("{:?}", p.state)),
                    Style::default().fg(state_color),
                ),
                Span::styled(
                    format!(
                        "{:>8} ",
                        p.pid.map_or("-".to_string(), |pid| pid.to_string())
                    ),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!("{:20} ", p.process_name.as_deref().unwrap_or("-")),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    truncate_string(&service_info, 20),
                    Style::default().fg(Color::Gray),
                ),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

fn render_connection_view(f: &mut Frame, _app: &AppState, area: Rect) {
    let block = Block::default()
        .title("Network Connections")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new("Connection view - Coming soon!")
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_status_bar(f: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let status_text = if let Some(message) = app.get_status_message() {
        message.to_string()
    } else {
        format!(
            "Mode: {:?} | Items: {} | {} | Ctrl+R: Refresh | Ctrl+C: Quit",
            app.mode,
            match app.mode {
                crate::tui::AppMode::ProcessView => app.filtered_processes.len(),
                crate::tui::AppMode::PortView => app.filtered_ports.len(),
                _ => 0,
            },
            if app.auto_refresh {
                "Auto-refresh: ON"
            } else {
                "Auto-refresh: OFF"
            }
        )
    };

    let status =
        Paragraph::new(status_text).style(Style::default().fg(Color::White).bg(Color::Blue));

    f.render_widget(status, chunks[1]);
}

fn render_confirmation_dialog(f: &mut Frame, app: &AppState, area: Rect) {
    if let Some(ref dialog) = app.confirmation_dialog {
        let popup_area = centered_rect(50, 30, area);

        f.render_widget(Clear, popup_area);

        let dialog_text = vec![
            Line::from(dialog.message.clone()),
            Line::from(""),
            Line::from("y - Yes    n - No"),
        ];

        let dialog_widget = Paragraph::new(dialog_text)
            .block(
                Block::default()
                    .title(dialog.title.as_str())
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        f.render_widget(dialog_widget, popup_area);
    }
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

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len.saturating_sub(1)])
    }
}
