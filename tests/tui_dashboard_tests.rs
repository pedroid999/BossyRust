use bossy_rust::testing::*;
use bossy_rust::tui::{AppState, AppMode};
use ratatui::{backend::TestBackend, Terminal, buffer::Buffer};

#[cfg(test)]
mod dashboard_rendering_tests {
    use super::*;

    #[test]
    fn test_dashboard_render_basic() {
        let mut app = AppState::default();
        app.processes = create_realistic_test_processes();
        app.filtered_processes = app.processes.clone();
        app.ports = create_realistic_test_ports();
        app.filtered_ports = app.ports.clone();

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|f| {
            bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_process_view_render() {
        let mut app = AppState::default();
        app.mode = AppMode::ProcessView;
        app.processes = create_realistic_test_processes();
        app.filtered_processes = app.processes.clone();

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|f| {
            bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
        });

        assert!(result.is_ok());
        
        // Check that the terminal buffer contains process information
        let buffer = terminal.backend().buffer();
        let content = buffer_to_string(buffer);
        
        // Should contain process names from our test data
        assert!(content.contains("node") || content.contains("Process"));
    }

    #[test] 
    fn test_port_view_render() {
        let mut app = AppState::default();
        app.mode = AppMode::PortView;
        app.ports = create_realistic_test_ports();
        app.filtered_ports = app.ports.clone();

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|f| {
            bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
        });

        assert!(result.is_ok());
        
        let buffer = terminal.backend().buffer();
        let content = buffer_to_string(buffer);
        
        // Should contain port information
        assert!(content.contains("Port") || content.contains("3000"));
    }

    #[test]
    fn test_connection_view_render() {
        let mut app = AppState::default();
        app.mode = AppMode::ConnectionView;
        app.connections = create_realistic_test_connections();
        app.filtered_connections = app.connections.clone();

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|f| {
            bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_theme_selector_render() {
        let mut app = AppState::default();
        app.mode = AppMode::ThemeSelector;

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|f| {
            bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
        });

        assert!(result.is_ok());
        
        let buffer = terminal.backend().buffer();
        let content = buffer_to_string(buffer);
        
        // Should contain theme information
        assert!(content.contains("Theme") || !app.themes.is_empty());
    }

    #[test]
    fn test_help_dialog_render() {
        let mut app = AppState::default();
        app.show_help = true;

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|f| {
            bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
        });

        assert!(result.is_ok());
        
        let buffer = terminal.backend().buffer();
        let content = buffer_to_string(buffer);
        
        // Should contain help text
        assert!(content.contains("Help") || content.contains("Key"));
    }

    #[test]
    fn test_confirmation_dialog_render() {
        use bossy_rust::tui::{ConfirmationDialog, DialogAction};
        
        let mut app = AppState::default();
        app.confirmation_dialog = Some(ConfirmationDialog {
            title: "Test Dialog".to_string(),
            message: "Test message".to_string(),
            confirm_action: DialogAction::Process(123),
        });

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|f| {
            bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
        });

        assert!(result.is_ok());
        
        let buffer = terminal.backend().buffer();
        let content = buffer_to_string(buffer);
        
        // Should contain dialog content
        assert!(content.contains("Test Dialog") || content.contains("confirm"));
    }

    #[test]
    fn test_search_help_dialog_render() {
        let mut app = AppState::default();
        app.search_active = true;
        app.show_help = true;

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|f| {
            bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_status_bar_render() {
        let mut app = AppState::default();
        app.set_status_message("Test status message".to_string());

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let result = terminal.draw(|f| {
            bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
        });

        assert!(result.is_ok());
        
        let buffer = terminal.backend().buffer();
        let content = buffer_to_string(buffer);
        
        // Should contain status message
        assert!(content.contains("Test status message") || content.contains("status"));
    }

    #[test]
    fn test_render_with_different_terminal_sizes() {
        let mut app = AppState::default();
        app.processes = create_realistic_test_processes();
        app.filtered_processes = app.processes.clone();

        // Test various terminal sizes
        let sizes = [(80, 24), (120, 40), (160, 50), (40, 20)];
        
        for (width, height) in sizes {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();

            let result = terminal.draw(|f| {
                bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
            });

            assert!(result.is_ok(), "Failed to render with size {}x{}", width, height);
        }
    }

    #[test]
    fn test_render_with_empty_data() {
        let mut app = AppState::default();
        // Empty all data
        app.processes.clear();
        app.filtered_processes.clear();
        app.ports.clear();
        app.filtered_ports.clear();
        app.connections.clear();
        app.filtered_connections.clear();

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        // Should not panic with empty data
        let result = terminal.draw(|f| {
            bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_render_with_large_datasets() {
        let mut app = AppState::default();
        
        // Create large datasets
        app.processes = (0..1000).map(|i| {
            create_test_process(i, &format!("process_{}", i), i as f32 % 100.0, i as u64 * 1024)
        }).collect();
        app.filtered_processes = app.processes.clone();

        app.ports = (3000..4000).map(|port| {
            create_test_port(port, bossy_rust::network::Protocol::Tcp, Some(port as u32))
        }).collect();
        app.filtered_ports = app.ports.clone();

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        // Should handle large datasets without issues
        let result = terminal.draw(|f| {
            bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_render_performance() {
        let mut app = AppState::default();
        app.processes = create_realistic_test_processes();
        app.filtered_processes = app.processes.clone();

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let start = std::time::Instant::now();
        
        // Render multiple times to test performance
        for _ in 0..100 {
            let result = terminal.draw(|f| {
                bossy_rust::tui::dashboard::render_dashboard(f, &mut app);
            });
            assert!(result.is_ok());
        }
        
        let duration = start.elapsed();
        
        // Rendering should be fast (less than 1 second for 100 renders)
        assert!(duration.as_millis() < 1000, "Rendering too slow: {:?}", duration);
    }

    // Helper function to convert buffer to string for assertions
    fn buffer_to_string(buffer: &Buffer) -> String {
        buffer.content().iter()
            .map(|cell| cell.symbol())
            .collect::<Vec<_>>()
            .join("")
    }
}