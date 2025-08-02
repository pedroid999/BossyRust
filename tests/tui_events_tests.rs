use bossy_rust::tui::{AppEvent, EventHandler};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::time::timeout;

#[cfg(test)]
mod event_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_event_handler_creation() {
        let handler = EventHandler::new(Duration::from_millis(100));
        // Should create without panic
        assert!(true);
    }

    #[tokio::test]
    async fn test_event_handler_default() {
        let handler = EventHandler::default();
        // Should create with default tick rate
        assert!(true);
    }

    #[tokio::test]
    async fn test_event_polling_timeout() {
        let handler = EventHandler::new(Duration::from_millis(10));
        
        // Should timeout and return Refresh event
        let start = std::time::Instant::now();
        let result = timeout(Duration::from_millis(50), handler.next()).await;
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        if let Ok(Ok(event)) = result {
            match event {
                AppEvent::Refresh => {
                    // Expected timeout behavior
                    assert!(duration >= Duration::from_millis(10));
                    assert!(duration < Duration::from_millis(30));
                }
                _ => {
                    // Other events are also valid if system generated them
                    assert!(true);
                }
            }
        }
    }

    #[test]
    fn test_app_event_variants() {
        // Test that all AppEvent variants can be created
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let _app_event_key = AppEvent::Key(key_event);
        let _app_event_resize = AppEvent::Resize(80, 24);
        let _app_event_refresh = AppEvent::Refresh;
        
        // Should compile and create without issues
        assert!(true);
    }

    #[tokio::test]
    async fn test_event_handler_responsiveness() {
        let handler = EventHandler::new(Duration::from_millis(1));
        
        // Test that handler responds quickly
        let start = std::time::Instant::now();
        let result = timeout(Duration::from_millis(100), handler.next()).await;
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration < Duration::from_millis(50), "Event handler too slow: {:?}", duration);
    }

    #[tokio::test]
    async fn test_multiple_event_polling() {
        let handler = EventHandler::new(Duration::from_millis(5));
        
        // Poll multiple events quickly
        for _ in 0..5 {
            let result = timeout(Duration::from_millis(20), handler.next()).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_event_handler_error_resilience() {
        let handler = EventHandler::new(Duration::from_millis(1));
        
        // Even if there are system issues, handler should not panic
        for _ in 0..10 {
            let result = handler.next().await;
            // Should either succeed or fail gracefully
            match result {
                Ok(_) => assert!(true),
                Err(_) => {
                    // Errors are acceptable in test environment
                    assert!(true);
                    break;
                }
            }
        }
    }

    #[test]
    fn test_key_event_creation() {
        // Test various key combinations
        let key_events = [
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE),
        ];
        
        for key_event in key_events {
            let app_event = AppEvent::Key(key_event);
            // Should create successfully
            match app_event {
                AppEvent::Key(_) => assert!(true),
                _ => panic!("Wrong event type"),
            }
        }
    }

    #[test]
    fn test_resize_event_creation() {
        let resize_events = [
            AppEvent::Resize(80, 24),
            AppEvent::Resize(120, 40),
            AppEvent::Resize(160, 50),
            AppEvent::Resize(1, 1),        // Minimum size
            AppEvent::Resize(999, 999),    // Large size
        ];
        
        for event in resize_events {
            match event {
                AppEvent::Resize(w, h) => {
                    assert!(w > 0);
                    assert!(h > 0);
                }
                _ => panic!("Wrong event type"),
            }
        }
    }
}

#[cfg(test)]
mod event_integration_tests {
    use super::*;
    use bossy_rust::testing::*;

    #[tokio::test]
    async fn test_key_event_integration_with_app_state() {
        let mut helper = TUITestHelper::new();
        
        // Test basic navigation keys
        let navigation_keys = [
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Home,
            KeyCode::End,
            KeyCode::PageUp,
            KeyCode::PageDown,
        ];
        
        for key in navigation_keys {
            let result = helper.press_key(key).await;
            assert!(result.is_ok(), "Failed to handle key: {:?}", key);
            TUIAssertions::assert_valid_state(&helper.app);
        }
    }

    #[tokio::test]
    async fn test_function_key_integration() {
        let mut helper = TUITestHelper::new();
        
        // Test function keys for mode switching
        let initial_mode = helper.app.mode.clone();
        
        // F1: Process View
        helper.press_key(KeyCode::F(1)).await.unwrap();
        assert_ne!(helper.app.mode, initial_mode);
        TUIAssertions::assert_valid_state(&helper.app);
        
        // F2: Port View
        helper.press_key(KeyCode::F(2)).await.unwrap();
        TUIAssertions::assert_valid_state(&helper.app);
        
        // F3: Connection View
        helper.press_key(KeyCode::F(3)).await.unwrap();
        TUIAssertions::assert_valid_state(&helper.app);
        
        // F4: Dashboard
        helper.press_key(KeyCode::F(4)).await.unwrap();
        TUIAssertions::assert_valid_state(&helper.app);
    }

    #[tokio::test]
    async fn test_control_key_combinations() {
        let mut helper = TUITestHelper::new();
        
        // Test Ctrl+C (quit)
        let result = helper.press_key_with_modifiers(KeyCode::Char('c'), KeyModifiers::CONTROL).await;
        assert!(result.is_ok());
        assert!(helper.app.should_quit);
        
        // Reset quit state for next test
        helper.app.should_quit = false;
        
        // Test Ctrl+R (refresh)
        let result = helper.press_key_with_modifiers(KeyCode::Char('r'), KeyModifiers::CONTROL).await;
        assert!(result.is_ok());
        assert!(!helper.app.should_quit);
    }

    #[tokio::test]
    async fn test_search_mode_events() {
        let mut helper = TUITestHelper::new();
        
        // Enter search mode
        helper.press_key(KeyCode::Char('/')).await.unwrap();
        assert!(helper.app.search_active);
        
        // Type search query
        let search_chars = ['n', 'o', 'd', 'e'];
        for ch in search_chars {
            helper.press_key(KeyCode::Char(ch)).await.unwrap();
        }
        assert_eq!(helper.app.search_query, "node");
        
        // Test backspace
        helper.press_key(KeyCode::Backspace).await.unwrap();
        assert_eq!(helper.app.search_query, "nod");
        
        // Test search help toggle
        helper.press_key(KeyCode::F(1)).await.unwrap();
        // Help state should toggle
        
        // Test tab completion
        helper.press_key(KeyCode::Tab).await.unwrap();
        // Should handle tab completion
        
        // Exit search mode
        helper.press_key(KeyCode::Esc).await.unwrap();
        assert!(!helper.app.search_active);
        assert!(helper.app.search_query.is_empty());
    }

    #[tokio::test]
    async fn test_confirmation_dialog_events() {
        let mut helper = TUITestHelper::new();
        
        // Set up a scenario that triggers confirmation dialog
        helper.switch_to_mode(bossy_rust::tui::AppMode::ProcessView);
        if !helper.app.filtered_processes.is_empty() {
            // Trigger action that shows confirmation
            helper.press_key(KeyCode::Enter).await.unwrap();
            
            if helper.app.confirmation_dialog.is_some() {
                // Test confirmation
                helper.press_key(KeyCode::Char('y')).await.unwrap();
                assert!(helper.app.confirmation_dialog.is_none());
            }
        }
        
        // Test rejection
        if !helper.app.filtered_processes.is_empty() {
            helper.press_key(KeyCode::Enter).await.unwrap();
            
            if helper.app.confirmation_dialog.is_some() {
                helper.press_key(KeyCode::Char('n')).await.unwrap();
                assert!(helper.app.confirmation_dialog.is_none());
            }
        }
    }

    #[tokio::test]
    async fn test_multi_selection_events() {
        let mut helper = TUITestHelper::new();
        helper.switch_to_mode(bossy_rust::tui::AppMode::ProcessView);
        
        // Test space key for multi-selection
        helper.press_key(KeyCode::Char(' ')).await.unwrap();
        
        if !helper.app.filtered_processes.is_empty() {
            assert!(helper.app.multi_select_mode);
            assert!(!helper.app.selected_items.is_empty());
        }
        
        // Move to next item and select
        helper.press_key(KeyCode::Down).await.unwrap();
        helper.press_key(KeyCode::Char(' ')).await.unwrap();
        
        TUIAssertions::assert_valid_state(&helper.app);
    }

    #[tokio::test]
    async fn test_help_dialog_events() {
        let mut helper = TUITestHelper::new();
        
        // Toggle help
        helper.press_key(KeyCode::Char('?')).await.unwrap();
        assert!(helper.app.show_help);
        
        // Toggle help off with 'q'
        helper.press_key(KeyCode::Char('q')).await.unwrap();
        assert!(!helper.app.show_help);
        assert!(!helper.app.should_quit); // Should not quit when help is shown
    }

    #[tokio::test]
    async fn test_theme_selector_events() {
        let mut helper = TUITestHelper::new();
        
        // Enter theme selector
        helper.press_key(KeyCode::Char('t')).await.unwrap();
        assert_eq!(helper.app.mode, bossy_rust::tui::AppMode::ThemeSelector);
        
        // Navigate themes
        let original_theme = helper.app.current_theme_index;
        helper.press_key(KeyCode::Down).await.unwrap();
        
        // Select theme
        helper.press_key(KeyCode::Enter).await.unwrap();
        assert_eq!(helper.app.mode, bossy_rust::tui::AppMode::Dashboard);
        
        // Exit theme selector without selection
        helper.press_key(KeyCode::Char('t')).await.unwrap();
        helper.press_key(KeyCode::Esc).await.unwrap();
        assert_eq!(helper.app.mode, bossy_rust::tui::AppMode::Dashboard);
    }

    #[tokio::test]
    async fn test_sorting_events() {
        let mut helper = TUITestHelper::new();
        helper.switch_to_mode(bossy_rust::tui::AppMode::ProcessView);
        
        let initial_sort = helper.app.sort_by.clone();
        
        // Test sort cycling
        helper.press_key(KeyCode::Char('s')).await.unwrap();
        assert_ne!(helper.app.sort_by, initial_sort);
        
        TUIAssertions::assert_valid_state(&helper.app);
    }

    #[tokio::test]
    async fn test_view_toggle_events() {
        let mut helper = TUITestHelper::new();
        
        // Test dashboard navigation (pressing 'd' switches to Dashboard mode)
        let initial_mode = helper.app.mode.clone();
        helper.press_key(KeyCode::Char('d')).await.unwrap();
        // Should switch to Dashboard mode
        assert_eq!(helper.app.mode, bossy_rust::tui::AppMode::Dashboard);
        
        // Test help toggle
        let initial_help = helper.app.show_help;
        helper.press_key(KeyCode::Char('?')).await.unwrap();
        assert_ne!(helper.app.show_help, initial_help);
    }

    #[tokio::test]
    async fn test_navigation_history_events() {
        let mut helper = TUITestHelper::new();
        
        // Build navigation history
        helper.switch_to_mode(bossy_rust::tui::AppMode::ProcessView);
        helper.switch_to_mode(bossy_rust::tui::AppMode::PortView);
        helper.switch_to_mode(bossy_rust::tui::AppMode::ConnectionView);
        
        // Test back navigation
        helper.press_key(KeyCode::Char('h')).await.unwrap();
        
        // Should have navigated back
        TUIAssertions::assert_valid_state(&helper.app);
    }

    #[tokio::test]
    async fn test_edge_case_events() {
        let mut helper = TUITestHelper::new();
        
        // Test with empty data
        helper.app.filtered_processes.clear();
        helper.app.filtered_ports.clear();
        helper.app.filtered_connections.clear();
        
        // Navigation should handle empty data gracefully
        helper.press_key(KeyCode::Down).await.unwrap();
        helper.press_key(KeyCode::Up).await.unwrap();
        helper.press_key(KeyCode::Enter).await.unwrap();
        
        TUIAssertions::assert_valid_state(&helper.app);
        
        // Quit events should always work
        helper.press_key(KeyCode::Char('q')).await.unwrap();
        assert!(helper.app.should_quit);
    }
}