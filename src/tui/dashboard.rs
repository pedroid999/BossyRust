use crate::tui::themes::Theme;
use crate::tui::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub fn render_dashboard(f: &mut Frame, app: &mut AppState) {
    let theme = app.themes[app.current_theme_index].clone();
    let size = f.size();

    // Set the background color for the entire frame
    f.render_widget(
        Block::default().style(Style::default().bg(theme.background)),
        size,
    );

    match app.mode {
        crate::tui::AppMode::Dashboard => render_main_dashboard(f, app, &theme, size),
        crate::tui::AppMode::ProcessView => render_process_view(f, app, &theme, size),
        crate::tui::AppMode::PortView => render_port_view(f, app, &theme, size),
        crate::tui::AppMode::ConnectionView => render_connection_view(f, app, &theme, size),
        crate::tui::AppMode::ThemeSelector => render_theme_selector(f, app, &theme, size),
    }

    // Always render status bar
    render_status_bar(f, app, &theme, size);

    // Render overlays
    if app.show_help {
        render_help_dialog(f, &theme, size);
    } else if app.confirmation_dialog.is_some() {
        render_confirmation_dialog(f, app, &theme, size);
    }
}

fn render_main_dashboard(f: &mut Frame, app: &AppState, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(2), // Enhanced Status
        ])
        .split(area);

    // Header with improved navigation hints
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "BossyRust ",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("- Mac Process Manager").style(Style::default().fg(theme.foreground)),
        ]),
        Line::from("1: Dashboard | 2: Processes | 3: Ports | 4: Connections | 5: Themes | h: Help | q: Quit")
            .style(Style::default().fg(theme.text_secondary)),
    ])
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)));
    f.render_widget(header, chunks[0]);

    // Main content area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(main_chunks[0]);

    // Left panel - Top processes
    let top_processes: Vec<ListItem> = app
        .processes
        .iter()
        .take(10)
        .enumerate()
        .map(|(i, p)| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("{:2}. ", i + 1))
                    .style(Style::default().fg(theme.text_secondary)),
                Span::styled(
                    format!("{:15}", truncate_string(&p.name, 15)),
                    Style::default().fg(theme.foreground),
                ),
                Span::raw(format!(" {:>6.1}%", p.cpu_usage))
                    .style(Style::default().fg(theme.accent)),
                Span::raw(format!(" {:>8}", p.format_memory()))
                    .style(Style::default().fg(theme.secondary)),
            ]))
        })
        .collect();

    let processes_list = List::new(top_processes)
        .block(
            Block::default()
                .title("Top Processes (CPU)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.highlight)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_widget(processes_list, left_chunks[0]);

    // --- Enhanced CPU Usage Chart ---
    let current_cpu_usage = app.cpu_history.last().cloned().unwrap_or(0);
    let max_cpu_in_history = app.cpu_history.iter().max().cloned().unwrap_or(0).max(25);

    // Use a more reasonable scale that shows actual data well
    let y_max = if max_cpu_in_history <= 25 {
        25
    } else if max_cpu_in_history <= 50 {
        50
    } else if max_cpu_in_history <= 75 {
        75
    } else {
        100
    };

    let chart_title = format!("⚡ CPU Usage ({current_cpu_usage}%)");

    let chart_container = Block::default()
        .title(chart_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent));

    let chart_area = chart_container.inner(left_chunks[1]);
    f.render_widget(chart_container, left_chunks[1]);

    // Create a simple layout for the chart content
    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(4), // Y-axis labels
            Constraint::Min(0),    // Chart bars
        ])
        .split(chart_area);

    // Y-axis labels with regular intervals (5 labels total)
    let step = y_max / 4; // 4 equal steps for 5 labels
    let y_labels = [
        format!("{y_max:>3}%"),
        format!("{:>3}%", y_max - step),
        format!("{:>3}%", y_max - step * 2),
        format!("{:>3}%", y_max - step * 3),
        "  0%".to_string(),
    ];

    let y_axis_text = y_labels.join("\n\n\n"); // More spacing for better alignment
    let y_axis_labels =
        Paragraph::new(y_axis_text).style(Style::default().fg(theme.text_secondary));
    f.render_widget(y_axis_labels, inner_layout[0]);

    // Chart area with proper scaling - ensure bars are visible
    let chart_data: Vec<(&str, u64)> = app
        .cpu_history
        .iter()
        .enumerate()
        .map(|(i, &value)| {
            // Use index as label to help with spacing, ensure minimum height for visibility
            let visible_value = if value == 0 && current_cpu_usage > 0 {
                1
            } else {
                value.min(y_max)
            };
            (if i.is_multiple_of(10) { "│" } else { " " }, visible_value)
        })
        .collect();

    let barchart = BarChart::default()
        .data(&chart_data)
        .bar_width(2) // Wider bars for better visibility
        .bar_gap(0)
        .bar_style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .value_style(Style::default().fg(theme.background)) // Hide values for cleaner look
        .max(y_max);

    f.render_widget(barchart, inner_layout[1]);

    // Right panel - Port summary
    let listening_ports = app
        .ports
        .iter()
        .filter(|p| matches!(p.state, crate::network::ConnectionState::Listen))
        .count();
    let dev_ports = app.ports.iter().filter(|p| p.is_development_port()).count();

    let port_items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::raw("Total Ports: ").style(Style::default().fg(theme.text_secondary)),
            Span::raw(app.ports.len().to_string()).style(Style::default().fg(theme.foreground)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("Listening: ").style(Style::default().fg(theme.text_secondary)),
            Span::raw(listening_ports.to_string()).style(Style::default().fg(theme.secondary)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("Development: ").style(Style::default().fg(theme.text_secondary)),
            Span::raw(dev_ports.to_string()).style(Style::default().fg(theme.primary)),
        ])),
    ];

    let port_list = List::new(port_items).block(
        Block::default()
            .title("Port Summary")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );

    f.render_widget(port_list, main_chunks[1]);
}

fn render_process_view(f: &mut Frame, app: &mut AppState, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Process list
            Constraint::Length(2), // Enhanced Status
        ])
        .split(area);

    // Header
    let sort_indicator = match app.sort_order {
        crate::tui::SortOrder::Ascending => "↑",
        crate::tui::SortOrder::Descending => "↓",
    };

    let header_text = if app.search_active {
        format!(
            "Processes ({}) | Search: {} | Enter to confirm, Esc to cancel",
            app.filtered_processes.len(),
            app.search_query
        )
    } else {
        format!(
            "Processes ({}) - Sorted by {:?} {} | / search | x kill | space select | s sort | Esc back",
            app.filtered_processes.len(),
            app.sort_by,
            sort_indicator
        )
    };

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(theme.text_secondary))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Process Manager")
                .border_style(Style::default().fg(theme.border)),
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
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>8} ", p.pid), style),
                Span::styled(
                    format!("{:20} ", truncate_string(&p.name, 20)),
                    style.fg(theme.primary),
                ),
                Span::styled(format!("{:>6.1}% ", p.cpu_usage), style.fg(theme.accent)),
                Span::styled(
                    format!("{:>10} ", p.format_memory()),
                    style.fg(theme.secondary),
                ),
                Span::styled(
                    truncate_string(&p.status, 10),
                    style.fg(theme.text_secondary),
                ),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.highlight)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

fn render_port_view(f: &mut Frame, app: &mut AppState, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Port list
            Constraint::Length(2), // Enhanced Status
        ])
        .split(area);

    // Header
    let header_text = if app.search_active {
        format!(
            "Ports ({}) | Search: {} | Enter to confirm, Esc to cancel",
            app.filtered_ports.len(),
            app.search_query
        )
    } else {
        format!(
            "Ports ({}) | / search | x kill | :port pattern | s sort | Esc back",
            app.filtered_ports.len()
        )
    };

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(theme.text_secondary))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Port Manager")
                .border_style(Style::default().fg(theme.border)),
        );
    f.render_widget(header, chunks[0]);

    // Port list
    let items: Vec<ListItem> = app
        .filtered_ports
        .iter()
        .map(|p| {
            let protocol_color = match p.protocol {
                crate::network::Protocol::Tcp => theme.primary,
                crate::network::Protocol::Udp => theme.secondary,
            };

            let state_color = match p.state {
                crate::network::ConnectionState::Listen => theme.secondary,
                crate::network::ConnectionState::Established => theme.primary,
                _ => theme.text_secondary,
            };

            let service_info = p
                .get_service_suggestion()
                .unwrap_or_else(|| format!("{:?}", p.state));

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:>6} ", p.port),
                    Style::default().fg(theme.foreground),
                ),
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
                    Style::default().fg(theme.accent),
                ),
                Span::styled(
                    format!("{:20} ", p.process_name.as_deref().unwrap_or("-")),
                    Style::default().fg(theme.primary),
                ),
                Span::styled(
                    truncate_string(&service_info, 20),
                    Style::default().fg(theme.text_secondary),
                ),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.highlight)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

fn render_connection_view(f: &mut Frame, app: &mut AppState, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Connection list
            Constraint::Length(2), // Enhanced Status
        ])
        .split(area);

    let header_text = if app.search_active {
        format!(
            "Active Connections ({}) | Search: {} | Enter to confirm, Esc to cancel",
            app.filtered_connections.len(),
            app.search_query
        )
    } else {
        format!(
            "Active Connections ({}) | / search | s sort | Esc back",
            app.filtered_connections.len()
        )
    };

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(theme.text_secondary))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Network Connections")
                .border_style(Style::default().fg(theme.border)),
        );
    f.render_widget(header, chunks[0]);

    if app.filtered_connections.is_empty() {
        // Show message when no connections are available
        let message = Paragraph::new("No active connections found.\n\nThis could mean:\n• No established network connections\n• System permissions may be required\n• Try running with elevated privileges")
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)))
            .style(Style::default().fg(theme.text_secondary))
            .wrap(Wrap { trim: true });
        f.render_widget(message, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .filtered_connections
            .iter()
            .map(|c| {
                let protocol_color = match c.protocol {
                    crate::network::Protocol::Tcp => theme.primary,
                    crate::network::Protocol::Udp => theme.secondary,
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{:4} ", format!("{:?}", c.protocol)),
                        Style::default().fg(protocol_color),
                    ),
                    Span::styled(
                        format!("{:21} ", c.local_address),
                        Style::default().fg(theme.primary),
                    ),
                    Span::raw("-> ").style(Style::default().fg(theme.text_secondary)),
                    Span::styled(
                        format!("{:21} ", c.remote_address),
                        Style::default().fg(theme.secondary),
                    ),
                    Span::styled(
                        format!(
                            "{:>8} ",
                            c.pid.map_or("-".to_string(), |pid| pid.to_string())
                        ),
                        Style::default().fg(theme.accent),
                    ),
                    Span::styled(
                        c.process_name.as_deref().unwrap_or("-"),
                        Style::default().fg(theme.foreground),
                    ),
                ]))
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(app.selected_index));

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            )
            .highlight_style(
                Style::default()
                    .bg(theme.highlight)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(list, chunks[1], &mut list_state);
    }
}

fn render_theme_selector(f: &mut Frame, app: &mut AppState, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Theme list
        ])
        .split(area);

    let header = Paragraph::new("Select a Theme (Enter to confirm, Esc to cancel)")
        .style(Style::default().fg(theme.text_secondary))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        );
    f.render_widget(header, chunks[0]);

    let items: Vec<ListItem> = app
        .themes
        .iter()
        .map(|t| ListItem::new(t.name.clone()).style(Style::default().fg(theme.foreground)))
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.highlight)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

fn render_status_bar(f: &mut Frame, app: &AppState, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    // Enhanced status bar with loading indicators
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[1]);

    // Main status text
    let status_text = if let Some(message) = app.get_status_message() {
        message.to_string()
    } else if let Some(loading_msg) = app.get_loading_message() {
        loading_msg
    } else {
        format!(
            "Mode: {:?} | Items: {} | {} | Ctrl+R: Refresh | Ctrl+C: Quit",
            app.mode,
            match app.mode {
                crate::tui::AppMode::ProcessView => app.filtered_processes.len(),
                crate::tui::AppMode::PortView => app.filtered_ports.len(),
                crate::tui::AppMode::ConnectionView => app.filtered_connections.len(),
                _ => 0,
            },
            if app.auto_refresh {
                "Auto-refresh: ON"
            } else {
                "Auto-refresh: OFF"
            }
        )
    };

    // Status color based on app status
    let status_style = match &app.app_status {
        crate::tui::AppStatus::Ready => Style::default().fg(theme.foreground).bg(theme.primary),
        crate::tui::AppStatus::Loading(_) => Style::default().fg(theme.background).bg(theme.accent),
        crate::tui::AppStatus::Processing(_) => Style::default().fg(theme.background).bg(theme.secondary),
        crate::tui::AppStatus::Error(_) => Style::default().fg(theme.foreground).bg(Color::Red),
        crate::tui::AppStatus::Success(_) => Style::default().fg(theme.background).bg(Color::Green),
    };

    let status = Paragraph::new(status_text).style(status_style);
    f.render_widget(status, status_chunks[0]);

    // Loading indicator and progress
    if app.is_loading() {
        let loading_text = match &app.loading_state {
            crate::tui::LoadingState::RefreshingData => "⟳ Refreshing...",
            crate::tui::LoadingState::KillingProcess(_) => "⚡ Killing...",
            crate::tui::LoadingState::KillingPort(_) => "⚡ Killing...",
            crate::tui::LoadingState::SearchingData => "🔍 Searching...",
            _ => "⟳ Working...",
        };
        
        let loading_indicator = Paragraph::new(loading_text)
            .style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));
        f.render_widget(loading_indicator, status_chunks[1]);
    } else {
        // Show refresh timer or other info when not loading
        let last_refresh = app.last_refresh.elapsed().as_secs();
        let refresh_info = format!("Last refresh: {}s ago", last_refresh);
        let info = Paragraph::new(refresh_info)
            .style(Style::default().fg(theme.text_secondary));
        f.render_widget(info, status_chunks[1]);
    }
}

fn render_confirmation_dialog(f: &mut Frame, app: &AppState, theme: &Theme, area: Rect) {
    if let Some(ref dialog) = app.confirmation_dialog {
        let popup_area = centered_rect(60, 40, area);

        f.render_widget(Clear, popup_area);

        // Determine colors based on danger level
        let (border_color, title_color, accent_color) = match dialog.danger_level {
            crate::tui::DangerLevel::Low => (theme.secondary, theme.secondary, theme.secondary),
            crate::tui::DangerLevel::Medium => (theme.accent, theme.accent, theme.accent),
            crate::tui::DangerLevel::High => (Color::Yellow, Color::Yellow, Color::Yellow),
            crate::tui::DangerLevel::Critical => (Color::Red, Color::Red, Color::Red),
        };

        let danger_indicator = match dialog.danger_level {
            crate::tui::DangerLevel::Low => "ℹ️",
            crate::tui::DangerLevel::Medium => "⚠️",
            crate::tui::DangerLevel::High => "🔥",
            crate::tui::DangerLevel::Critical => "💀",
        };

        let mut dialog_lines = vec![
            Line::from(vec![
                Span::styled(danger_indicator, Style::default().fg(accent_color)),
                Span::raw(" "),
                Span::styled(dialog.title.clone(), Style::default().fg(title_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(dialog.message.clone()).style(Style::default().fg(theme.foreground)),
        ];

        // Add context information if available
        if let Some(ref context) = dialog.context_info {
            dialog_lines.push(Line::from(""));
            dialog_lines.push(Line::from(vec![
                Span::styled("Details: ", Style::default().fg(theme.text_secondary).add_modifier(Modifier::BOLD)),
                Span::raw(context.clone()).style(Style::default().fg(theme.text_secondary)),
            ]));
        }

        dialog_lines.push(Line::from(""));
        dialog_lines.push(Line::from(""));

        // Enhanced confirmation options
        match dialog.danger_level {
            crate::tui::DangerLevel::Critical => {
                dialog_lines.push(Line::from(vec![
                    Span::styled("Type 'YES' to confirm: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled(&app.critical_confirmation_buffer, Style::default().fg(Color::Red).bg(theme.highlight)),
                    Span::styled("_", Style::default().fg(Color::Red)),
                ]));
                dialog_lines.push(Line::from(vec![
                    Span::styled("n/Esc", Style::default().fg(theme.secondary)),
                    Span::raw(" - Cancel"),
                ]));
            },
            _ => {
                dialog_lines.push(Line::from(vec![
                    Span::styled("y/Enter", Style::default().fg(accent_color).add_modifier(Modifier::BOLD)),
                    Span::raw(" - Confirm  |  "),
                    Span::styled("n/Esc", Style::default().fg(theme.secondary)),
                    Span::raw(" - Cancel"),
                ]));
            }
        }

        let dialog_widget = Paragraph::new(dialog_lines)
            .block(
                Block::default()
                    .title(" Confirmation Required ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().fg(theme.foreground))
            .wrap(Wrap { trim: true });

        f.render_widget(dialog_widget, popup_area);
    }
}

fn render_help_dialog(f: &mut Frame, theme: &Theme, area: Rect) {
    let popup_area = centered_rect(70, 60, area);

    f.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            "BossyRust - Process Manager Help",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  1: Dashboard | 2: Processes | 3: Ports | 4: Connections | 5: Themes"),
        Line::from("  ↑/↓ or j/k - Navigate    u/d - Page up/down    g/G - Top/bottom"),
        Line::from("  Space - Multi-select    c - Clear selection    Esc - Smart back"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions:",
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Enter/x/Delete - Kill selected process/port"),
        Line::from("  / - Search mode    s - Cycle sort options"),
        Line::from("  r/Ctrl+R - Refresh data    q - Quit    h - Help"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Search Patterns:",
            Style::default()
                .fg(theme.secondary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  :3000 - Find processes using port 3000"),
        Line::from("  #1234 - Search by Process ID"),
        Line::from("  >50% - Processes using more than 50% CPU"),
        Line::from("  >1GB - Processes using more than 1GB memory"),
        Line::from("  node - Search by process name"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Tips:",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  • Multi-select with Space, then Delete to kill multiple"),
        Line::from("  • Use Ctrl+R to refresh if data seems stale"),
        Line::from("  • Connection view requires active network connections"),
        Line::from(""),
        Line::from("Press h, ?, or Esc to close this help"),
    ];

    let help_widget = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        )
        .style(Style::default().fg(theme.foreground))
        .wrap(Wrap { trim: true });

    f.render_widget(help_widget, popup_area);
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
