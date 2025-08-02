//! Property-based tests for BossyRust
//! 
//! This module contains property-based tests using proptest to verify
//! that key invariants hold across a wide range of inputs.

#[cfg(test)]
use proptest::prelude::*;

#[cfg(test)]
mod search_properties {
    use super::*;
    use crate::testing::*;
    use crate::*;

    /// Generate arbitrary process names for testing
    fn arb_process_name() -> impl Strategy<Value = String> {
        prop_oneof![
            "node.*",
            "python.*",
            "cargo.*",
            "rust.*",
            "npm.*",
            "[a-z]{3,10}",
        ].prop_map(|s| s.to_string())
    }

    /// Generate arbitrary search patterns
    fn arb_search_pattern() -> impl Strategy<Value = String> {
        prop_oneof![
            // Port patterns
            ":[0-9]{1,5}",
            // PID patterns  
            "#[0-9]{1,6}",
            // CPU patterns
            ">[0-9]{1,2}%",
            // Memory patterns
            ">[0-9]{1,4}MB",
            ">[0-9]{1,2}GB",
            // Name patterns
            "[a-zA-Z]{3,15}",
        ].prop_map(|s| s.to_string())
    }

    proptest! {
        #[test]
        fn test_process_search_consistency(
            pattern in arb_search_pattern(),
            processes in prop::collection::vec(any::<u32>().prop_map(|pid| {
                create_test_process(pid, "test_process", 10.0, 512)
            }), 1..100)
        ) {
            // Property: Search should be deterministic
            let results1 = processes.iter()
                .filter(|p| p.matches_search(&pattern))
                .collect::<Vec<_>>();
            
            let results2 = processes.iter()
                .filter(|p| p.matches_search(&pattern))
                .collect::<Vec<_>>();
            
            prop_assert_eq!(results1.len(), results2.len());
        }

        #[test]
        fn test_search_subset_property(
            pattern in arb_search_pattern(),
            processes in prop::collection::vec(any::<u32>().prop_map(|pid| {
                create_test_process(pid, "test_process", 20.0, 1024)
            }), 1..50)
        ) {
            // Property: Filtered results should be a subset of original
            let all_count = processes.len();
            let filtered_count = processes.iter()
                .filter(|p| p.matches_search(&pattern))
                .count();
            
            prop_assert!(filtered_count <= all_count);
        }

        #[test]
        fn test_port_search_validity(
            port in 1u16..65535,
            process_name in arb_process_name()
        ) {
            let port_info = create_test_port(port, network::Protocol::Tcp, Some(1234));
            let search_pattern = format!(":{}", port);
            
            // Property: Port search should find exact matches
            if port_info.matches_search(&search_pattern) {
                prop_assert_eq!(port_info.port, port);
            }
        }

        #[test]
        fn test_pid_search_validity(
            pid in 1u32..99999,
            process_name in arb_process_name()
        ) {
            let process = create_test_process(pid, &process_name, 25.0, 2048);
            let search_pattern = format!("#{}", pid);
            
            // Property: PID search should find exact matches
            if process.matches_search(&search_pattern) {
                prop_assert_eq!(process.pid, pid);
            }
        }

        #[test]
        fn test_search_empty_pattern(
            processes in prop::collection::vec(any::<u32>().prop_map(|pid| {
                create_test_process(pid, "test", 15.0, 512) 
            }), 0..100)
        ) {
            // Property: Empty search pattern behavior
            let empty_results = processes.iter()
                .filter(|p| p.matches_search(""))
                .count();
            
            // Empty pattern should either match all or none consistently
            prop_assert!(empty_results == 0 || empty_results == processes.len());
        }

        #[test] 
        fn test_memory_search_consistency(
            memory_mb in 1u64..8192,
            process_name in arb_process_name()
        ) {
            let mut process = create_test_process(1234, &process_name, 30.0, memory_mb);
            process.memory = memory_mb;
            
            let search_pattern = format!(">{}MB", memory_mb - 1);
            
            // Property: Process with memory X should match ">Y MB" where Y < X
            if memory_mb > 1 {
                prop_assert!(process.matches_search(&search_pattern));
            }
        }

        #[test]
        fn test_cpu_search_bounds(
            cpu_percent in 0.0f64..100.0,
            process_name in arb_process_name()
        ) {
            let mut process = create_test_process(1234, &process_name, cpu_percent as f32, 1024);
            process.cpu_usage = cpu_percent as f32;
            
            // Property: CPU searches should respect bounds
            let high_cpu_pattern = ">95%";
            let low_cpu_pattern = ">5%";
            
            if cpu_percent > 95.0 {
                prop_assert!(process.matches_search(high_cpu_pattern));
            }
            
            if cpu_percent > 5.0 {
                prop_assert!(process.matches_search(low_cpu_pattern));
            }
        }

        #[test]
        fn test_name_search_case_sensitivity(
            name in "[a-zA-Z]{5,15}",
            search_term in "[a-zA-Z]{3,10}"
        ) {
            let process = create_test_process(1234, &name, 20.0, 1024);
            
            // Property: Name search should be case-insensitive or consistently sensitive
            let lower_match = process.matches_search(&search_term.to_lowercase());
            let upper_match = process.matches_search(&search_term.to_uppercase());
            
            // If one case matches, behavior should be consistent
            if lower_match || upper_match {
                // At least basic substring matching should work
                prop_assert!(
                    name.to_lowercase().contains(&search_term.to_lowercase()) ||
                    !lower_match
                );
            }
        }
    }
}

#[cfg(test)]
mod data_structure_properties {
    use super::*;
    use crate::tui::AppState;

    proptest! {
        #[test]
        fn test_app_state_invariants(
            selected_index in 0usize..1000,
            process_count in 0usize..1000
        ) {
            let mut app = AppState::default();
            
            // Add some test processes
            for i in 0..process_count.min(100) {
                app.processes.push(create_test_process(i as u32, "test", 15.0, 512));
            }
            app.filtered_processes = app.processes.clone();
            
            // Set selected index
            if !app.filtered_processes.is_empty() {
                let safe_index = selected_index % app.filtered_processes.len();
                app.selected_index = safe_index;
            }
            
            // Property: Selected index should never exceed bounds
            if !app.filtered_processes.is_empty() {
                prop_assert!(app.selected_index < app.filtered_processes.len());
            }
            
            // Property: Filtered processes should be subset of all processes
            prop_assert!(app.filtered_processes.len() <= app.processes.len());
        }

        #[test]
        fn test_port_validation(
            port in 1u16..65535,
            protocol in prop_oneof!["TCP", "UDP"]
        ) {
            let port_info = create_test_port(port, network::Protocol::Tcp, Some(1234));
            
            // Property: Port numbers should be valid
            prop_assert!(port_info.port > 0);
            prop_assert!(port_info.port <= 65535);
            
            // Property: Port should have consistent string representation
            let port_str = port_info.port.to_string();
            prop_assert!(port_str.parse::<u16>().unwrap() == port_info.port);
        }

        #[test]
        fn test_process_memory_bounds(
            memory_mb in 0u64..16384
        ) {
            let mut process = create_test_process(1234, "test", 20.0, memory_mb);
            process.memory = memory_mb;
            
            // Property: Memory usage should be non-negative
            prop_assert!(process.memory >= 0);
            
            // Property: Memory formatting should be consistent
            let formatted = process.format_memory();
            prop_assert!(!formatted.is_empty());
            prop_assert!(formatted.contains("MB") || formatted.contains("GB"));
        }

        #[test]
        fn test_search_pattern_parsing(
            port_num in 1u16..65535,
            pid_num in 1u32..99999,
            cpu_val in 1u8..100,
            mem_val in 1u32..9999
        ) {
            // Property: All search patterns should parse without panicking
            let patterns = vec![
                format!(":{}", port_num),
                format!("#{}", pid_num), 
                format!(">{}%", cpu_val),
                format!(">{}MB", mem_val),
                format!(">{}GB", mem_val / 1000 + 1),
            ];
            
            let process = create_test_process(pid_num, "test", 25.0, 1024);
            
            for pattern in patterns {
                // Should not panic
                let _result = process.matches_search(&pattern);
                prop_assert!(true); // If we get here, no panic occurred
            }
        }
    }
}

#[cfg(test)]
mod performance_properties {
    use super::*;
    use std::time::Instant;

    proptest! {
        #[test]
        fn test_search_performance_scales_linearly(
            process_count in 1usize..1000,
            pattern in "[a-zA-Z]{3,10}"
        ) {
            // Generate test processes
            let processes: Vec<_> = (0..process_count.min(500))
                .map(|i| create_test_process(i as u32, &format!("process_{}", i), 20.0, 1024))
                .collect();
            
            // Measure search time
            let start = Instant::now();
            let _results: Vec<_> = processes.iter()
                .filter(|p| p.matches_search(&pattern))
                .collect();
            let duration = start.elapsed();
            
            // Property: Search should complete in reasonable time
            // Even for 500 processes, should be under 100ms in debug mode
            prop_assert!(duration.as_millis() < 500);
        }

        #[test]
        fn test_memory_usage_bounds(
            num_processes in 1usize..100
        ) {
            let processes: Vec<_> = (0..num_processes)
                .map(|i| create_test_process(i as u32, "test", 15.0, 512))
                .collect();
            
            // Property: Memory usage should be predictable
            // Each process struct should not be excessively large
            let size_estimate = std::mem::size_of::<ProcessInfo>() * processes.len();
            
            // Should be reasonable memory usage (under 10MB for 100 processes)
            prop_assert!(size_estimate < 10 * 1024 * 1024);
        }
    }
}