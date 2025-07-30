use crate::config::settings::{load_settings, save_settings, UserSettings};
use crate::network::{ConnectionInfo, PortInfo, PortManager};
use crate::process::{ProcessInfo, ProcessMonitor};
use crate::tui::themes::{Theme, ThemeManager};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Dashboard,
    ProcessView,
    PortView,
    ConnectionView,
    ThemeSelector,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortBy {
    Name,
    Pid,
    Cpu,
    Memory,
    Port,
    LocalAddress,
    RemoteAddress,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

pub struct AppState {
    pub mode: AppMode,
    pub should_quit: bool,
    pub search_query: String,
    pub search_active: bool,
    pub selected_index: usize,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub show_help: bool,
    pub status_message: Option<(String, Instant)>,
    pub confirmation_dialog: Option<ConfirmationDialog>,

    // Data
    pub processes: Vec<ProcessInfo>,
    pub filtered_processes: Vec<ProcessInfo>,
    pub ports: Vec<PortInfo>,
    pub filtered_ports: Vec<PortInfo>,
    pub connections: Vec<ConnectionInfo>,
    pub filtered_connections: Vec<ConnectionInfo>,

    // Monitoring
    pub process_monitor: ProcessMonitor,
    pub last_refresh: Instant,
    pub refresh_interval: Duration,
    pub auto_refresh: bool,

    // Multi-selection
    pub selected_items: Vec<usize>,
    pub multi_select_mode: bool,

    // CPU History for sparkline
    pub cpu_history: Vec<u64>,

    // Theming
    pub themes: Vec<Theme>,
    pub current_theme_index: usize,
}

#[derive(Debug, Clone)]
pub struct ConfirmationDialog {
    pub title: String,
    pub message: String,
    pub confirm_action: DialogAction,
}

#[derive(Debug, Clone)]
pub enum DialogAction {
    Process(u32),
    Processes(Vec<u32>),
    Port(u16),
}

impl AppState {
    pub fn new() -> Result<Self> {
        let mut process_monitor = ProcessMonitor::new();
        let processes = process_monitor.get_processes();
        let ports = PortManager::get_all_ports()?;
        let connections = PortManager::get_active_connections()?;
        let themes = ThemeManager::get_themes();
        let settings = load_settings().unwrap_or_default();
        let current_theme_index = themes
            .iter()
            .position(|t| t.name == settings.theme_name)
            .unwrap_or(0);

        Ok(Self {
            mode: AppMode::Dashboard,
            should_quit: false,
            search_query: String::new(),
            search_active: false,
            selected_index: 0,
            sort_by: SortBy::Cpu,
            sort_order: SortOrder::Descending,
            show_help: false,
            status_message: None,
            confirmation_dialog: None,

            filtered_processes: processes.clone(),
            processes,
            ports: ports.clone(),
            filtered_ports: ports.clone(),
            connections: connections.clone(),
            filtered_connections: connections,

            process_monitor,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(2),
            auto_refresh: true,

            selected_items: Vec::new(),
            multi_select_mode: false,

            cpu_history: vec![0; 100], // Store last 100 CPU usage points

            themes,
            current_theme_index,
        })
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // Handle global keys first
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => {
                    self.should_quit = true;
                    return Ok(());
                }
                KeyCode::Char('r') => {
                    self.refresh_data()?;
                    return Ok(());
                }
                _ => {}
            }
        }

        // Handle confirmation dialog
        if let Some(_dialog) = &self.confirmation_dialog {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    self.execute_dialog_action().await?;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.confirmation_dialog = None;
                }
                _ => {}
            }
            return Ok(());
        }

        // Handle search mode
        if self.search_active {
            match key.code {
                KeyCode::Enter => {
                    self.apply_search_filter();
                    self.search_active = false;
                }
                KeyCode::Esc => {
                    self.search_active = false;
                    self.search_query.clear();
                    self.reset_filters();
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.apply_search_filter();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.apply_search_filter();
                }
                _ => {}
            }
            return Ok(());
        }

        // Handle normal mode keys
        if self.mode == AppMode::ThemeSelector {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.selected_index = self.selected_index.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected_index < self.themes.len() - 1 {
                        self.selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    self.current_theme_index = self.selected_index;
                    let settings = UserSettings {
                        theme_name: self.themes[self.current_theme_index].name.clone(),
                    };
                    if let Err(e) = save_settings(&settings) {
                        self.set_status_message(format!("Error saving settings: {e}"));
                    }
                    self.mode = AppMode::Dashboard; // Go back to dashboard after selection
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.mode = AppMode::Dashboard;
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::PageUp => self.page_up(),
            KeyCode::PageDown => self.page_down(),
            KeyCode::Home => self.go_to_top(),
            KeyCode::End => self.go_to_bottom(),

            // Mode switching
            KeyCode::F(1) => self.mode = AppMode::ProcessView,
            KeyCode::F(2) => self.mode = AppMode::PortView,
            KeyCode::F(3) => self.mode = AppMode::ConnectionView,
            KeyCode::F(4) | KeyCode::Char('?') => self.toggle_help(),

            // Actions
            KeyCode::Char('d') => self.mode = AppMode::Dashboard, // Go to Dashboard
            KeyCode::Char('t') => self.mode = AppMode::ThemeSelector, // Go to Theme Selector
            KeyCode::Char('/') => self.enter_search_mode(),
            KeyCode::Char('r') => self.refresh_data()?,
            KeyCode::Char(' ') => self.toggle_selection(),
            KeyCode::Enter => self.primary_action().await?,
            KeyCode::Delete => self.kill_action(),

            // Sorting
            KeyCode::Char('s') => self.cycle_sort(),

            // Quit
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.show_help {
                    self.show_help = false;
                } else {
                    self.should_quit = true;
                }
            }

            _ => {}
        }

        Ok(())
    }

    pub fn should_refresh(&self) -> bool {
        self.auto_refresh && self.last_refresh.elapsed() >= self.refresh_interval
    }

    pub fn refresh_data(&mut self) -> Result<()> {
        self.processes = self.process_monitor.get_processes();
        self.ports = PortManager::get_all_ports()?;
        self.connections = PortManager::get_active_connections()?;

        // Update CPU history with actual system CPU usage (0-100%)
        let system_cpu_usage = self.process_monitor.get_system_cpu_usage() as u64;
        self.cpu_history.remove(0);
        self.cpu_history.push(system_cpu_usage);

        self.apply_current_filters();
        self.last_refresh = Instant::now();
        self.set_status_message("Data refreshed".to_string());
        Ok(())
    }

    pub fn apply_search_filter(&mut self) {
        if self.search_query.is_empty() {
            self.reset_filters();
            return;
        }

        match self.mode {
            AppMode::ProcessView => {
                self.filtered_processes = self
                    .processes
                    .iter()
                    .filter(|p| p.matches_search(&self.search_query))
                    .cloned()
                    .collect();
                self.sort_processes();
            }
            AppMode::PortView => {
                self.filtered_ports = self
                    .ports
                    .iter()
                    .filter(|p| p.matches_search(&self.search_query))
                    .cloned()
                    .collect();
                self.sort_ports();
            }
            AppMode::ConnectionView => {
                self.filtered_connections = self
                    .connections
                    .iter()
                    .filter(|c| c.matches_search(&self.search_query))
                    .cloned()
                    .collect();
                // TODO: Add sorting for connections if needed
            }
            _ => {}
        }

        self.selected_index = 0;
    }

    fn reset_filters(&mut self) {
        self.filtered_processes = self.processes.clone();
        self.filtered_ports = self.ports.clone();
        self.filtered_connections = self.connections.clone();
        self.apply_current_sorts();
        if self.mode != AppMode::ThemeSelector {
            self.selected_index = 0;
        }
    }

    fn apply_current_filters(&mut self) {
        if self.search_query.is_empty() {
            self.reset_filters();
        } else {
            self.apply_search_filter();
        }
    }

    fn apply_current_sorts(&mut self) {
        self.sort_processes();
        self.sort_ports();
        self.sort_connections();
    }

    fn sort_processes(&mut self) {
        match self.sort_by {
            SortBy::Name => {
                self.filtered_processes.sort_by(|a, b| {
                    let cmp = a.name.cmp(&b.name);
                    if self.sort_order == SortOrder::Ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            SortBy::Pid => {
                self.filtered_processes.sort_by(|a, b| {
                    let cmp = a.pid.cmp(&b.pid);
                    if self.sort_order == SortOrder::Ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            SortBy::Cpu => {
                self.filtered_processes.sort_by(|a, b| {
                    let cmp = a
                        .cpu_usage
                        .partial_cmp(&b.cpu_usage)
                        .unwrap_or(std::cmp::Ordering::Equal);
                    if self.sort_order == SortOrder::Ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            SortBy::Memory => {
                self.filtered_processes.sort_by(|a, b| {
                    let cmp = a.memory.cmp(&b.memory);
                    if self.sort_order == SortOrder::Ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            _ => {}
        }
    }

    fn sort_ports(&mut self) {
        if self.sort_by == SortBy::Port {
            self.filtered_ports.sort_by(|a, b| {
                let cmp = a.port.cmp(&b.port);
                if self.sort_order == SortOrder::Ascending {
                    cmp
                } else {
                    cmp.reverse()
                }
            });
        }
    }

    fn sort_connections(&mut self) {
        match self.sort_by {
            SortBy::LocalAddress => {
                self.filtered_connections.sort_by(|a, b| {
                    let cmp = a.local_address.cmp(&b.local_address);
                    if self.sort_order == SortOrder::Ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            SortBy::RemoteAddress => {
                self.filtered_connections.sort_by(|a, b| {
                    let cmp = a.remote_address.cmp(&b.remote_address);
                    if self.sort_order == SortOrder::Ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            SortBy::Pid => {
                self.filtered_connections.sort_by(|a, b| {
                    let cmp = a.pid.cmp(&b.pid);
                    if self.sort_order == SortOrder::Ascending {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            _ => {}
        }
    }

    // Navigation methods
    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    fn move_down(&mut self) {
        let max_index = match self.mode {
            AppMode::ProcessView => self.filtered_processes.len().saturating_sub(1),
            AppMode::PortView => self.filtered_ports.len().saturating_sub(1),
            AppMode::ConnectionView => self.filtered_connections.len().saturating_sub(1),
            _ => 0,
        };

        if self.selected_index < max_index {
            self.selected_index += 1;
        }
    }

    fn page_up(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(10);
    }

    fn page_down(&mut self) {
        let max_index = match self.mode {
            AppMode::ProcessView => self.filtered_processes.len().saturating_sub(1),
            AppMode::PortView => self.filtered_ports.len().saturating_sub(1),
            _ => 0,
        };

        self.selected_index = std::cmp::min(self.selected_index + 10, max_index);
    }

    fn go_to_top(&mut self) {
        self.selected_index = 0;
    }

    fn go_to_bottom(&mut self) {
        self.selected_index = match self.mode {
            AppMode::ProcessView => self.filtered_processes.len().saturating_sub(1),
            AppMode::PortView => self.filtered_ports.len().saturating_sub(1),
            _ => 0,
        };
    }

    fn enter_search_mode(&mut self) {
        self.search_active = true;
        self.search_query.clear();
    }

    fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    fn toggle_selection(&mut self) {
        if !self.multi_select_mode {
            self.multi_select_mode = true;
            self.selected_items.clear();
        }

        if self.selected_items.contains(&self.selected_index) {
            self.selected_items.retain(|&x| x != self.selected_index);
        } else {
            self.selected_items.push(self.selected_index);
        }

        if self.selected_items.is_empty() {
            self.multi_select_mode = false;
        }
    }

    fn cycle_sort(&mut self) {
        match self.mode {
            AppMode::ProcessView => {
                self.sort_by = match self.sort_by {
                    SortBy::Name => SortBy::Pid,
                    SortBy::Pid => SortBy::Cpu,
                    SortBy::Cpu => SortBy::Memory,
                    SortBy::Memory => SortBy::Name,
                    _ => SortBy::Name,
                };
            }
            AppMode::PortView => {
                self.sort_by = SortBy::Port;
                self.sort_order = match self.sort_order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                };
            }
            AppMode::ConnectionView => {
                self.sort_by = match self.sort_by {
                    SortBy::LocalAddress => SortBy::RemoteAddress,
                    SortBy::RemoteAddress => SortBy::Pid,
                    _ => SortBy::LocalAddress,
                };
            }
            _ => {}
        }

        self.apply_current_sorts();
        self.set_status_message(format!(
            "Sorted by {:?} ({})",
            self.sort_by,
            if self.sort_order == SortOrder::Ascending {
                "↑"
            } else {
                "↓"
            }
        ));
    }

    async fn primary_action(&mut self) -> Result<()> {
        match self.mode {
            AppMode::ProcessView => {
                if let Some(process) = self.filtered_processes.get(self.selected_index) {
                    self.show_kill_process_dialog(process.pid);
                }
            }
            AppMode::PortView => {
                if let Some(port) = self.filtered_ports.get(self.selected_index) {
                    self.show_kill_port_dialog(port.port);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn kill_action(&mut self) {
        if self.multi_select_mode && !self.selected_items.is_empty() {
            self.show_kill_multiple_dialog();
        } else {
            match self.mode {
                AppMode::ProcessView => {
                    if let Some(process) = self.filtered_processes.get(self.selected_index) {
                        self.show_kill_process_dialog(process.pid);
                    }
                }
                AppMode::PortView => {
                    if let Some(port) = self.filtered_ports.get(self.selected_index) {
                        self.show_kill_port_dialog(port.port);
                    }
                }
                _ => {}
            }
        }
    }

    fn show_kill_process_dialog(&mut self, pid: u32) {
        // Create dialog regardless of whether process exists (for testing)
        let process_name = self
            .processes
            .iter()
            .find(|p| p.pid == pid)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("PID {pid}"));

        self.confirmation_dialog = Some(ConfirmationDialog {
            title: "Kill Process".to_string(),
            message: format!(
                "Kill process '{process_name}' (PID: {pid})?\nPress 'y' to confirm, 'n' to cancel"
            ),
            confirm_action: DialogAction::Process(pid),
        });
    }

    fn show_kill_port_dialog(&mut self, port: u16) {
        self.confirmation_dialog = Some(ConfirmationDialog {
            title: "Kill Port".to_string(),
            message: format!(
                "Kill process using port {port}?\nPress 'y' to confirm, 'n' to cancel"
            ),
            confirm_action: DialogAction::Port(port),
        });
    }

    fn show_kill_multiple_dialog(&mut self) {
        let count = self.selected_items.len();
        self.confirmation_dialog = Some(ConfirmationDialog {
            title: "Kill Multiple".to_string(),
            message: format!("Kill {count} selected items?\nPress 'y' to confirm, 'n' to cancel"),
            confirm_action: DialogAction::Processes(
                self.selected_items
                    .iter()
                    .filter_map(|&index| match self.mode {
                        AppMode::ProcessView => self.filtered_processes.get(index).map(|p| p.pid),
                        _ => None,
                    })
                    .collect(),
            ),
        });
    }

    async fn execute_dialog_action(&mut self) -> Result<()> {
        if let Some(dialog) = self.confirmation_dialog.take() {
            match dialog.confirm_action {
                DialogAction::Process(pid) => {
                    match crate::process::ProcessKiller::kill_process_by_pid(pid, false).await {
                        Ok(()) => {
                            self.set_status_message(format!("Successfully killed process {pid}"));
                            self.refresh_data()?;
                        }
                        Err(e) => {
                            self.set_status_message(format!("Failed to kill process {pid}: {e}"));
                        }
                    }
                }
                DialogAction::Port(port) => {
                    match crate::process::ProcessKiller::kill_process_by_port(port).await {
                        Ok(pid) => {
                            self.set_status_message(format!(
                                "Successfully killed process {pid} using port {port}"
                            ));
                            self.refresh_data()?;
                        }
                        Err(e) => {
                            self.set_status_message(format!("Failed to kill port {port}: {e}"));
                        }
                    }
                }
                DialogAction::Processes(pids) => {
                    let mut success_count = 0;
                    for pid in pids {
                        if crate::process::ProcessKiller::kill_process_by_pid(pid, false)
                            .await
                            .is_ok()
                        {
                            success_count += 1;
                        }
                    }
                    self.set_status_message(format!("Killed {success_count} processes"));
                    self.multi_select_mode = false;
                    self.selected_items.clear();
                    self.refresh_data()?;
                }
            }
        }
        Ok(())
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some((message, Instant::now()));
    }

    pub fn get_status_message(&self) -> Option<&str> {
        if let Some((ref message, timestamp)) = self.status_message {
            if timestamp.elapsed() < Duration::from_secs(5) {
                Some(message.as_str())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new().expect("Failed to create default AppState")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn create_test_app_state() -> AppState {
        // Create a minimal test app state without system calls
        let themes = ThemeManager::get_themes();
        let settings = UserSettings::default();
        let current_theme_index = themes
            .iter()
            .position(|t| t.name == settings.theme_name)
            .unwrap_or(0);

        AppState {
            mode: AppMode::Dashboard,
            should_quit: false,
            search_query: String::new(),
            search_active: false,
            selected_index: 0,
            sort_by: SortBy::Cpu,
            sort_order: SortOrder::Descending,
            show_help: false,
            status_message: None,
            confirmation_dialog: None,

            processes: vec![],
            filtered_processes: vec![],
            ports: vec![],
            filtered_ports: vec![],
            connections: vec![],
            filtered_connections: vec![],

            process_monitor: ProcessMonitor::new(),
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(2),
            auto_refresh: true,

            selected_items: Vec::new(),
            multi_select_mode: false,

            cpu_history: vec![0; 100],
            themes,
            current_theme_index,
        }
    }

    #[test]
    fn test_app_state_creation() {
        let app = create_test_app_state();
        assert_eq!(app.mode, AppMode::Dashboard);
        assert!(!app.should_quit);
        assert!(!app.search_active);
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.sort_by, SortBy::Cpu);
        assert_eq!(app.sort_order, SortOrder::Descending);
    }

    #[test]
    fn test_mode_switching() {
        let mut app = create_test_app_state();

        // Test switching to different modes
        app.mode = AppMode::ProcessView;
        assert_eq!(app.mode, AppMode::ProcessView);

        app.mode = AppMode::PortView;
        assert_eq!(app.mode, AppMode::PortView);

        app.mode = AppMode::ConnectionView;
        assert_eq!(app.mode, AppMode::ConnectionView);
    }

    #[tokio::test]
    async fn test_key_event_handling_quit() {
        let mut app = create_test_app_state();

        // Test Ctrl+C to quit
        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        app.handle_key_event(key_event).await.unwrap();
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn test_key_event_handling_navigation() {
        let mut app = create_test_app_state();

        // Add some test data
        app.filtered_processes = vec![
            ProcessInfo {
                pid: 1,
                name: "test1".to_string(),
                cpu_usage: 10.0,
                memory: 1000,
                parent_pid: None,
                status: "Running".to_string(),
                start_time: 0,
                user_id: None,
                executable_path: None,
                command_line: vec![],
            },
            ProcessInfo {
                pid: 2,
                name: "test2".to_string(),
                cpu_usage: 20.0,
                memory: 2000,
                parent_pid: None,
                status: "Running".to_string(),
                start_time: 0,
                user_id: None,
                executable_path: None,
                command_line: vec![],
            },
        ];

        app.mode = AppMode::ProcessView;
        assert_eq!(app.selected_index, 0);

        // Test moving down
        let key_event = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        app.handle_key_event(key_event).await.unwrap();
        assert_eq!(app.selected_index, 1);

        // Test moving up
        let key_event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        app.handle_key_event(key_event).await.unwrap();
        assert_eq!(app.selected_index, 0);
    }

    #[tokio::test]
    async fn test_search_functionality() {
        let mut app = create_test_app_state();

        // Enter search mode
        let key_event = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        app.handle_key_event(key_event).await.unwrap();
        assert!(app.search_active);

        // Type search query
        let key_event = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE);
        app.handle_key_event(key_event).await.unwrap();
        assert_eq!(app.search_query, "t");

        let key_event = KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE);
        app.handle_key_event(key_event).await.unwrap();
        assert_eq!(app.search_query, "te");

        // Test backspace
        let key_event = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        app.handle_key_event(key_event).await.unwrap();
        assert_eq!(app.search_query, "t");

        // Exit search mode
        let key_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        app.handle_key_event(key_event).await.unwrap();
        assert!(!app.search_active);
        assert!(app.search_query.is_empty());
    }

    #[test]
    fn test_multi_selection() {
        let mut app = create_test_app_state();

        // Start multi-selection
        app.toggle_selection();
        assert!(app.multi_select_mode);
        assert!(app.selected_items.contains(&0));

        // Add another selection
        app.selected_index = 1;
        app.toggle_selection();
        assert!(app.selected_items.contains(&1));
        assert_eq!(app.selected_items.len(), 2);

        // Remove selection
        app.selected_index = 0;
        app.toggle_selection();
        assert!(!app.selected_items.contains(&0));
        assert_eq!(app.selected_items.len(), 1);

        // Clear all selections
        app.selected_index = 1;
        app.toggle_selection();
        assert!(!app.multi_select_mode);
        assert!(app.selected_items.is_empty());
    }

    #[test]
    fn test_sorting_functionality() {
        let mut app = create_test_app_state();

        // Test initial sort state
        assert_eq!(app.sort_by, SortBy::Cpu);
        assert_eq!(app.sort_order, SortOrder::Descending);

        // Switch to process view and cycle sort
        app.mode = AppMode::ProcessView;
        app.cycle_sort();
        assert_eq!(app.sort_by, SortBy::Memory); // Cpu -> Memory

        app.cycle_sort();
        assert_eq!(app.sort_by, SortBy::Name); // Memory -> Name

        app.cycle_sort();
        assert_eq!(app.sort_by, SortBy::Pid); // Name -> Pid

        app.cycle_sort();
        assert_eq!(app.sort_by, SortBy::Cpu); // Pid -> Cpu (back to start)
    }

    #[test]
    fn test_status_message() {
        let mut app = create_test_app_state();

        // Set status message
        app.set_status_message("Test message".to_string());
        assert_eq!(app.get_status_message(), Some("Test message"));

        // Message should expire after some time (tested by setting old timestamp)
        if let Some((_, ref mut timestamp)) = app.status_message {
            *timestamp = Instant::now() - Duration::from_secs(10);
        }
        assert_eq!(app.get_status_message(), None);
    }

    #[test]
    fn test_dialog_functionality() {
        let mut app = create_test_app_state();

        // Show kill process dialog
        app.show_kill_process_dialog(1234);
        assert!(app.confirmation_dialog.is_some());

        if let Some(ref dialog) = app.confirmation_dialog {
            assert_eq!(dialog.title, "Kill Process");
            assert!(dialog.message.contains("1234"));
        }

        // Show kill port dialog
        app.show_kill_port_dialog(8080);
        assert!(app.confirmation_dialog.is_some());

        if let Some(ref dialog) = app.confirmation_dialog {
            assert_eq!(dialog.title, "Kill Port");
            assert!(dialog.message.contains("8080"));
        }
    }

    #[test]
    fn test_refresh_timing() {
        let mut app = create_test_app_state();

        // Should not need refresh immediately after creation
        assert!(!app.should_refresh());

        // Set old refresh time to trigger refresh
        app.last_refresh = Instant::now() - Duration::from_secs(5);
        assert!(app.should_refresh());

        // Disable auto-refresh
        app.auto_refresh = false;
        assert!(!app.should_refresh());
    }

    #[test]
    fn test_navigation_bounds() {
        let mut app = create_test_app_state();

        // Add test processes
        app.filtered_processes = vec![
            ProcessInfo {
                pid: 1,
                name: "test1".to_string(),
                cpu_usage: 10.0,
                memory: 1000,
                parent_pid: None,
                status: "Running".to_string(),
                start_time: 0,
                user_id: None,
                executable_path: None,
                command_line: vec![],
            },
            ProcessInfo {
                pid: 2,
                name: "test2".to_string(),
                cpu_usage: 20.0,
                memory: 2000,
                parent_pid: None,
                status: "Running".to_string(),
                start_time: 0,
                user_id: None,
                executable_path: None,
                command_line: vec![],
            },
        ];

        app.mode = AppMode::ProcessView;

        // Test navigation bounds
        app.selected_index = 0;
        app.move_up(); // Should stay at 0
        assert_eq!(app.selected_index, 0);

        app.move_down();
        assert_eq!(app.selected_index, 1);

        app.move_down(); // Should stay at 1 (last item)
        assert_eq!(app.selected_index, 1);

        // Test go to top/bottom
        app.go_to_top();
        assert_eq!(app.selected_index, 0);

        app.go_to_bottom();
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_filter_application() {
        let mut app = create_test_app_state();

        // Add test data
        app.processes = vec![
            ProcessInfo {
                pid: 1,
                name: "node".to_string(),
                cpu_usage: 50.0,
                memory: 1024 * 1024 * 500, // 500MB
                parent_pid: None,
                status: "Running".to_string(),
                start_time: 0,
                user_id: None,
                executable_path: None,
                command_line: vec![],
            },
            ProcessInfo {
                pid: 2,
                name: "python".to_string(),
                cpu_usage: 10.0,
                memory: 1024 * 1024 * 100, // 100MB
                parent_pid: None,
                status: "Running".to_string(),
                start_time: 0,
                user_id: None,
                executable_path: None,
                command_line: vec![],
            },
        ];

        // Test search filtering
        app.mode = AppMode::ProcessView; // Set the mode first
        app.search_query = "node".to_string();
        app.apply_search_filter();
        assert_eq!(app.filtered_processes.len(), 1);
        assert_eq!(app.filtered_processes[0].name, "node");

        // Test CPU filtering
        app.search_query = ">30%".to_string();
        app.apply_search_filter();
        assert_eq!(app.filtered_processes.len(), 1);
        assert_eq!(app.filtered_processes[0].name, "node");

        // Reset filters
        app.reset_filters();
        assert_eq!(app.filtered_processes.len(), 2);
    }

    #[test]
    fn test_connection_view_filtering() {
        use crate::network::Protocol;

        let mut app = create_test_app_state();
        app.connections = vec![
            ConnectionInfo {
                protocol: Protocol::Tcp,
                local_address: "127.0.0.1:1234".parse().unwrap(),
                remote_address: "1.1.1.1:443".parse().unwrap(),
                pid: Some(100),
                process_name: Some("chrome".to_string()),
            },
            ConnectionInfo {
                protocol: Protocol::Tcp,
                local_address: "127.0.0.1:5678".parse().unwrap(),
                remote_address: "2.2.2.2:80".parse().unwrap(),
                pid: Some(200),
                process_name: Some("firefox".to_string()),
            },
        ];

        app.mode = AppMode::ConnectionView;
        app.reset_filters(); // Initialize filtered_connections

        // Test search by process name
        app.search_query = "chrome".to_string();
        app.apply_search_filter();
        assert_eq!(app.filtered_connections.len(), 1);
        assert_eq!(
            app.filtered_connections[0].process_name.as_deref(),
            Some("chrome")
        );

        // Test search by IP address
        app.search_query = "2.2.2.2".to_string();
        app.apply_search_filter();
        assert_eq!(app.filtered_connections.len(), 1);
        assert_eq!(app.filtered_connections[0].pid, Some(200));

        // Test search by PID
        app.search_query = "100".to_string();
        app.apply_search_filter();
        assert_eq!(app.filtered_connections.len(), 1);

        // Reset filters
        app.search_query = "".to_string();
        app.apply_search_filter();
        assert_eq!(app.filtered_connections.len(), 2);
    }
}
