use crate::tui::{AppState, AppMode};
use crate::testing::fixtures::*;
use crate::testing::mocks::tui_mocks::MockBackend;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
// use ratatui::layout::Rect;  // Unused import

/// TUI Test Helper for comprehensive UI testing
pub struct TUITestHelper {
    pub app: AppState,
    pub backend: MockBackend,
    pub terminal: Terminal<MockBackend>,
}

impl TUITestHelper {
    /// Create a new TUI test helper with mock data
    pub fn new() -> Self {
        let mut app = AppState::default();
        
        // Populate with test data
        app.processes = create_realistic_test_processes();
        app.filtered_processes = app.processes.clone();
        app.ports = create_realistic_test_ports();
        app.filtered_ports = app.ports.clone();
        app.connections = create_realistic_test_connections();
        app.filtered_connections = app.connections.clone();

        let backend = MockBackend::new(120, 40);
        let terminal = Terminal::new(backend).unwrap();

        Self {
            app,
            backend: MockBackend::new(120, 40),
            terminal,
        }
    }

    /// Create helper with custom terminal size
    pub fn with_size(width: u16, height: u16) -> Self {
        let mut helper = Self::new();
        helper.backend = MockBackend::new(width, height);
        helper.terminal = Terminal::new(MockBackend::new(width, height)).unwrap();
        helper
    }

    /// Simulate key press and handle the event
    pub async fn press_key(&mut self, key_code: KeyCode) -> anyhow::Result<()> {
        let key_event = KeyEvent::new(key_code, KeyModifiers::NONE);
        self.app.handle_key_event(key_event).await
    }

    /// Simulate key press with modifiers
    pub async fn press_key_with_modifiers(
        &mut self,
        key_code: KeyCode,
        modifiers: KeyModifiers,
    ) -> anyhow::Result<()> {
        let key_event = KeyEvent::new(key_code, modifiers);
        self.app.handle_key_event(key_event).await
    }

    /// Render the current app state and capture output
    pub fn render(&mut self) -> anyhow::Result<()> {
        self.terminal.draw(|f| {
            crate::tui::dashboard::render_dashboard(f, &mut self.app);
        })?;
        Ok(())
    }

    /// Check if rendered output contains specific text
    pub fn contains_text(&self, text: &str) -> bool {
        self.backend.contains_text(text)
    }

    /// Get the number of render calls
    pub fn render_count(&self) -> usize {
        self.backend.draw_call_count()
    }

    /// Switch to specific mode and verify
    pub fn switch_to_mode(&mut self, mode: AppMode) {
        self.app.switch_to_mode(mode.clone());
        assert_eq!(self.app.mode, mode);
    }

    /// Enter search mode and type query
    pub async fn search(&mut self, query: &str) -> anyhow::Result<()> {
        // Enter search mode
        self.press_key(KeyCode::Char('/')).await?;
        assert!(self.app.search_active);

        // Type the query
        for ch in query.chars() {
            self.press_key(KeyCode::Char(ch)).await?;
        }

        // Apply search
        self.press_key(KeyCode::Enter).await?;
        assert!(!self.app.search_active);
        
        Ok(())
    }

    /// Navigate through list items
    pub async fn navigate(&mut self, direction: KeyCode, steps: usize) -> anyhow::Result<()> {
        for _ in 0..steps {
            self.press_key(direction).await?;
        }
        Ok(())
    }

    /// Test complete workflow: navigate -> search -> select -> action
    pub async fn test_workflow(&mut self, mode: AppMode, search_query: &str) -> anyhow::Result<()> {
        // Switch mode
        self.switch_to_mode(mode);
        
        // Render initial state
        self.render()?;
        
        // Search
        if !search_query.is_empty() {
            self.search(search_query).await?;
        }
        
        // Navigate
        self.navigate(KeyCode::Down, 2).await?;
        
        // Render final state
        self.render()?;
        
        Ok(())
    }

    /// Simulate terminal resize
    pub fn resize(&mut self, width: u16, height: u16) {
        self.backend = MockBackend::new(width, height);
        // In real usage, this would trigger an AppEvent::Resize
    }

    /// Test multiple selection workflow
    pub async fn test_multi_selection(&mut self, indices: &[usize]) -> anyhow::Result<()> {
        for &index in indices {
            // Navigate to index
            self.app.selected_index = index;
            // Toggle selection
            self.press_key(KeyCode::Char(' ')).await?;
        }
        
        assert!(self.app.multi_select_mode);
        assert_eq!(self.app.selected_items.len(), indices.len());
        
        Ok(())
    }

    /// Test confirmation dialog workflow
    pub async fn test_confirmation_dialog(&mut self, accept: bool) -> anyhow::Result<()> {
        // Trigger an action that shows confirmation dialog
        self.switch_to_mode(AppMode::ProcessView);
        if !self.app.filtered_processes.is_empty() {
            self.press_key(KeyCode::Enter).await?;
            
            if self.app.confirmation_dialog.is_some() {
                if accept {
                    self.press_key(KeyCode::Char('y')).await?;
                } else {
                    self.press_key(KeyCode::Char('n')).await?;
                }
            }
        }
        
        Ok(())
    }

    /// Test error handling scenarios
    pub async fn test_error_scenarios(&mut self) -> anyhow::Result<()> {
        // Test invalid navigation bounds
        self.app.selected_index = 0;
        self.press_key(KeyCode::Up).await?;
        assert_eq!(self.app.selected_index, 0);

        // Test empty search
        self.search("").await?;
        
        // Test invalid search patterns
        self.search("invalid#$%^").await?;
        
        Ok(())
    }
}

/// Assertion helpers for TUI testing
pub struct TUIAssertions;

impl TUIAssertions {
    /// Assert that app state is valid
    pub fn assert_valid_state(app: &AppState) {
        // Check selection bounds
        match app.mode {
            AppMode::ProcessView => {
                assert!(app.selected_index < app.filtered_processes.len() || app.filtered_processes.is_empty());
            }
            AppMode::PortView => {
                assert!(app.selected_index < app.filtered_ports.len() || app.filtered_ports.is_empty());
            }
            AppMode::ConnectionView => {
                assert!(app.selected_index < app.filtered_connections.len() || app.filtered_connections.is_empty());
            }
            _ => {}
        }

        // Check data consistency
        assert!(app.filtered_processes.len() <= app.processes.len());
        assert!(app.filtered_ports.len() <= app.ports.len());
        assert!(app.filtered_connections.len() <= app.connections.len());

        // Check search state consistency
        if app.search_active {
            assert!(!app.search_query.is_empty() || app.show_help);
        }
    }

    /// Assert that navigation works correctly
    pub fn assert_navigation_bounds(app: &AppState, _original_index: usize, expected_index: usize) {
        assert_eq!(app.selected_index, expected_index);
        
        // Ensure we didn't go out of bounds
        match app.mode {
            AppMode::ProcessView => {
                if !app.filtered_processes.is_empty() {
                    assert!(app.selected_index < app.filtered_processes.len());
                }
            }
            AppMode::PortView => {
                if !app.filtered_ports.is_empty() {
                    assert!(app.selected_index < app.filtered_ports.len());
                }
            }
            _ => {}
        }
    }

    /// Assert that search results are valid
    pub fn assert_search_results(app: &AppState, query: &str) {
        if query.is_empty() {
            return;
        }

        match app.mode {
            AppMode::ProcessView => {
                for process in &app.filtered_processes {
                    assert!(process.matches_search(query));
                }
            }
            AppMode::PortView => {
                for port in &app.filtered_ports {
                    assert!(port.matches_search(query));
                }
            }
            AppMode::ConnectionView => {
                for connection in &app.filtered_connections {
                    assert!(connection.matches_search(query));
                }
            }
            _ => {}
        }
    }

    /// Assert that theme switching works
    pub fn assert_theme_switch(app: &AppState, expected_theme_index: usize) {
        assert_eq!(app.current_theme_index, expected_theme_index);
        assert!(expected_theme_index < app.themes.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tui_helper_creation() {
        let helper = TUITestHelper::new();
        assert_eq!(helper.app.mode, AppMode::Dashboard);
        assert!(!helper.app.processes.is_empty());
        assert!(!helper.app.ports.is_empty());
    }

    #[tokio::test]
    async fn test_key_press_simulation() {
        let mut helper = TUITestHelper::new();
        
        // Test navigation
        helper.switch_to_mode(AppMode::ProcessView);
        let initial_index = helper.app.selected_index;
        
        helper.press_key(KeyCode::Down).await.unwrap();
        assert!(helper.app.selected_index >= initial_index);
    }

    #[tokio::test]
    async fn test_search_workflow() {
        let mut helper = TUITestHelper::new();
        helper.switch_to_mode(AppMode::ProcessView);
        
        helper.search("node").await.unwrap();
        
        // Should have filtered results
        assert!(helper.app.filtered_processes.len() <= helper.app.processes.len());
        
        // All filtered results should match search
        TUIAssertions::assert_search_results(&helper.app, "node");
    }

    #[tokio::test]
    async fn test_multi_selection() {
        let mut helper = TUITestHelper::new();
        helper.switch_to_mode(AppMode::ProcessView);
        
        helper.test_multi_selection(&[0, 1, 2]).await.unwrap();
        
        assert!(helper.app.multi_select_mode);
        assert_eq!(helper.app.selected_items.len(), 3);
    }

    #[tokio::test]  
    async fn test_assertions() {
        let helper = TUITestHelper::new();
        
        // Test state validation
        TUIAssertions::assert_valid_state(&helper.app);
        
        // Test theme validation
        TUIAssertions::assert_theme_switch(&helper.app, helper.app.current_theme_index);
    }

    #[tokio::test]
    async fn test_error_scenarios() {
        let mut helper = TUITestHelper::new();
        
        // Should not panic on error scenarios
        helper.test_error_scenarios().await.unwrap();
        
        // State should remain valid
        TUIAssertions::assert_valid_state(&helper.app);
    }
}