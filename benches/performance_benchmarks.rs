use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use bossy_rust::testing::*;
use bossy_rust::process::ProcessInfo;
use bossy_rust::network::PortInfo;
use bossy_rust::tui::AppState;

fn bench_process_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_search");
    
    // Test different dataset sizes
    for size in [10, 100, 1000].iter() {
        let processes: Vec<ProcessInfo> = (0..*size)
            .map(|i| create_test_process(i as u32, &format!("process_{}", i), 20.0, 1024))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("name_search", size),
            size,
            |b, _| {
                b.iter(|| {
                    processes.iter()
                        .filter(|p| p.matches_search("node"))
                        .count()
                })
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("pid_search", size),
            size,
            |b, _| {
                b.iter(|| {
                    processes.iter()
                        .filter(|p| p.matches_search("#1234"))
                        .count()
                })
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("cpu_search", size),
            size,
            |b, _| {
                b.iter(|| {
                    processes.iter()
                        .filter(|p| p.matches_search(">50%"))
                        .count()
                })
            }
        );
    }
    
    group.finish();
}

fn bench_port_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("port_operations");
    
    for size in [10, 100, 1000].iter() {
        let ports: Vec<PortInfo> = (3000..*size + 3000)
            .map(|port| create_test_port(port as u16, bossy_rust::network::Protocol::TCP, Some(1234)))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("port_search", size),
            size,
            |b, _| {
                b.iter(|| {
                    ports.iter()
                        .filter(|p| p.matches_search(":3000"))
                        .count()
                })
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("development_port_check", size),
            size,
            |b, _| {
                b.iter(|| {
                    ports.iter()
                        .filter(|p| p.is_development_port())
                        .count()
                })
            }
        );
    }
    
    group.finish();
}

fn bench_app_state_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("app_state_operations");
    
    let mut app = AppState::default();
    
    // Populate with test data
    for i in 0..1000 {
        app.processes.push(create_test_process(i, &format!("process_{}", i), 25.0, 2048));
        app.ports.push(create_test_port(3000 + i as u16, bossy_rust::network::Protocol::TCP, Some(i)));
    }
    
    group.bench_function("filter_processes", |b| {
        b.iter(|| {
            let _filtered: Vec<_> = app.processes.iter()
                .filter(|p| p.matches_search("test"))
                .collect();
        })
    });
    
    group.bench_function("sort_processes_by_cpu", |b| {
        let mut processes = app.processes.clone();
        b.iter(|| {
            processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
        })
    });
    
    group.bench_function("sort_processes_by_memory", |b| {
        let mut processes = app.processes.clone();
        b.iter(|| {
            processes.sort_by(|a, b| b.memory_mb.cmp(&a.memory_mb));
        })
    });
    
    group.finish();
}

fn bench_tui_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("tui_operations");
    
    // Mock TUI operations that might be performance-critical
    group.bench_function("theme_switching", |b| {
        b.iter(|| {
            let themes = bossy_rust::tui::themes::ThemeManager::get_themes();
            let _theme = &themes[0];
        })
    });
    
    group.bench_function("tui_helper_creation", |b| {
        b.iter(|| {
            let _helper = TUITestHelper::new();
        })
    });
    
    group.finish();
}

fn bench_memory_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_formatting");
    
    let memory_values = vec![512, 1024, 2048, 4096, 8192, 16384];
    
    for memory_mb in memory_values {
        group.bench_with_input(
            BenchmarkId::new("format_memory", memory_mb),
            &memory_mb,
            |b, &memory| {
                let mut process = create_test_process(1234, "test", 30.0, memory);
                process.memory = memory;
                b.iter(|| {
                    let _formatted = process.format_memory();
                })
            }
        );
    }
    
    group.finish();
}

fn bench_search_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_patterns");
    
    let process = create_test_process(1234, "node_server_application", 40.0, 4096);
    let patterns = vec![
        "node",
        "#1234", 
        ":3000",
        ">50%",
        ">1GB",
        "server",
        "application"
    ];
    
    for pattern in patterns {
        group.bench_with_input(
            BenchmarkId::new("pattern_match", pattern),
            pattern,
            |b, pattern| {
                b.iter(|| {
                    let _matches = process.matches_search(pattern);
                })
            }
        );
    }
    
    group.finish();
}

fn bench_realistic_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_workload");
    
    // Simulate a realistic system with mixed processes
    let realistic_processes: Vec<ProcessInfo> = vec![
        create_test_process(1, "launchd", 0.1, 64),
        create_test_process(100, "node server.js", 45.2, 2048),
        create_test_process(101, "python manage.py runserver", 23.1, 1536),
        create_test_process(102, "Google Chrome", 15.8, 8192),
        create_test_process(103, "Visual Studio Code", 8.4, 4096),
        create_test_process(104, "docker-proxy", 12.1, 3072),
        create_test_process(105, "rust-analyzer", 3.2, 1024),
        create_test_process(106, "cargo run", 25.5, 512),
        create_test_process(107, "npm start", 18.3, 768),
        create_test_process(108, "webpack-dev-server", 35.7, 2560),
    ];
    
    group.bench_function("full_search_workflow", |b| {
        b.iter(|| {
            // Simulate a typical user search workflow
            let _node_processes: Vec<_> = realistic_processes.iter()
                .filter(|p| p.matches_search("node"))
                .collect();
            
            let _high_cpu: Vec<_> = realistic_processes.iter()
                .filter(|p| p.matches_search(">10%"))
                .collect();
            
            let _development: Vec<_> = realistic_processes.iter()
                .filter(|p| p.name.contains("node") || p.name.contains("npm") || p.name.contains("cargo"))
                .collect();
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_process_search,
    bench_port_operations, 
    bench_app_state_operations,
    bench_tui_operations,
    bench_memory_formatting,
    bench_search_patterns,
    bench_realistic_workload
);

criterion_main!(benches);