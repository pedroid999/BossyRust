# BossyRust: Building a Modern Terminal Process Manager for macOS with Rust and ratatui

In the world of software development, managing processes and network ports is a daily task. Whether you're a frontend developer juggling multiple React development servers, a backend engineer managing microservices, or a full-stack developer with countless Node.js processes running, keeping track of what's using which port can be challenging. Traditional tools like Activity Monitor are great for system overview, but they lack the developer-focused features we need for our daily workflow.

That's why I built **BossyRust** — a lightweight Terminal User Interface (TUI) process manager specifically designed for macOS developers. It combines the power of system process management with intelligent network port monitoring, all wrapped in an intuitive terminal interface built with Rust and ratatui.

## The Problem: Developer Process Management Pain Points

As developers, we've all been there:
- That mysterious process using port 3000 when you're trying to start your React app
- Multiple Node.js processes cluttering your system from previous development sessions
- Struggling to identify which webpack dev server belongs to which project
- Using a combination of `ps`, `lsof`, `kill`, and `netstat` commands to manage processes manually

Traditional system tools work, but they're not optimized for modern development workflows. Activity Monitor is powerful but lacks developer-specific features like port-to-process mapping and development server detection. Command-line tools are precise but require remembering complex syntax and chaining multiple commands.

## Enter BossyRust: A Developer-First Approach

BossyRust bridges this gap by providing:

### 🎯 **Smart Process Detection**
The application understands common development processes and can identify Node.js servers, webpack dev servers, Python Flask applications, and other development tools automatically.

### 🔍 **Intelligent Search Patterns**
Instead of memorizing command-line syntax, BossyRust uses intuitive search patterns:
- `:3000` — Find what's using port 3000
- `#1234` — Search by Process ID
- `>50%` — Processes using more than 50% CPU
- `>1GB` — Memory-hungry processes
- `node` — Simple name-based search

### 🚀 **Graceful Process Management**
Unlike brutal `kill -9` commands, BossyRust implements a smart termination strategy:
1. Send SIGTERM for graceful shutdown
2. Wait up to 5 seconds with polling
3. Escalate to SIGKILL only if necessary
4. Provide user confirmation for safety

## Technical Architecture: Building with Rust and ratatui

### Why Rust?
Rust was the perfect choice for this project because:
- **Performance**: Near-zero overhead for real-time process monitoring
- **Safety**: Memory safety prevents crashes when dealing with system processes
- **Cross-compilation**: Easy distribution without runtime dependencies
- **Rich Ecosystem**: Excellent crates for system information and TUI development

### The ratatui Framework
[ratatui](https://github.com/ratatui-org/ratatui) is a modern TUI library that provides:
- **Immediate Mode Rendering**: 60fps updates with minimal resource usage
- **Flexible Layouts**: Responsive UI that adapts to terminal size
- **Rich Widgets**: Built-in components for tables, gauges, and charts
- **Event Handling**: Comprehensive keyboard and event management

### Core Architecture

The project follows a modular architecture:

```rust
src/
├── main.rs              # Entry point with CLI parsing
├── process/             # Process management core
│   ├── info.rs         # Process information gathering
│   ├── killer.rs       # Smart termination logic
│   └── monitor.rs      # Real-time monitoring with caching
├── network/            # Network port management
│   ├── ports.rs        # Port enumeration and mapping
│   └── utils.rs        # Network utilities and categorization
├── tui/                # Terminal User Interface
│   ├── app.rs          # Application state management
│   ├── dashboard.rs    # Multi-view rendering system
│   └── events.rs       # Event handling and navigation
└── commands/           # CLI command implementations
    └── cli.rs          # Command-line interface handlers
```

### State Management Pattern

The application uses a centralized state management pattern with the `AppState` struct:

```rust
pub struct AppState {
    pub mode: AppMode,
    pub should_quit: bool,
    pub search_query: String,
    pub selected_index: usize,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    
    // Data caching for performance
    pub processes: Vec<ProcessInfo>,
    pub filtered_processes: Vec<ProcessInfo>,
    pub ports: Vec<PortInfo>,
    
    // Monitoring with intelligent refresh
    pub process_monitor: ProcessMonitor,
    pub last_refresh: Instant,
}
```

This single-source-of-truth approach ensures consistent UI updates and makes the application easier to reason about.

## Key Features in Action

### 1. Multi-Modal Interface

BossyRust provides four distinct views accessible via function keys:

- **F1 - Process View**: Complete process listing with CPU/memory usage
- **F2 - Port View**: Network ports mapped to their processes
- **F3 - Connection View**: Active network connections
- **F4 - Help**: Comprehensive keyboard shortcuts guide

### 2. Smart Search and Filtering

The search system goes beyond simple string matching:

```rust
// Smart pattern matching in action
match search_pattern {
    pattern if pattern.starts_with(':') => {
        // Port search: ":3000"
        let port = parse_port(&pattern[1..])?;
        filter_by_port(processes, port)
    },
    pattern if pattern.starts_with('#') => {
        // PID search: "#1234"
        let pid = parse_pid(&pattern[1..])?;
        filter_by_pid(processes, pid)
    },
    pattern if pattern.starts_with('>') => {
        // Threshold search: ">50%" or ">1GB"
        parse_and_filter_threshold(processes, pattern)
    },
    _ => {
        // Standard name search
        filter_by_name(processes, pattern)
    }
}
```

### 3. Developer-Focused Port Management

The application recognizes common development ports and provides shortcuts for bulk operations:

```rust
// Common development ports detection
const DEV_PORTS: &[u16] = &[3000, 3001, 4200, 5000, 8000, 8080, 8081, 9000];

pub fn identify_dev_processes(ports: &[PortInfo]) -> Vec<DevProcess> {
    ports.iter()
        .filter(|port| DEV_PORTS.contains(&port.number))
        .map(|port| DevProcess::from_port_info(port))
        .collect()
}
```

### 4. Intelligent Process Termination

Instead of immediately killing processes, BossyRust implements a graceful shutdown strategy:

```rust
pub async fn terminate_process(pid: u32) -> Result<()> {
    // Step 1: Send SIGTERM for graceful shutdown
    send_signal(pid, Signal::SIGTERM)?;
    
    // Step 2: Wait with polling
    for _ in 0..50 { // 5 seconds with 100ms intervals
        if !process_exists(pid) {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // Step 3: Escalate to SIGKILL if necessary
    println!("Process {} didn't respond to SIGTERM, using SIGKILL", pid);
    send_signal(pid, Signal::SIGKILL)?;
    
    // Step 4: Final verification
    tokio::time::sleep(Duration::from_secs(2)).await;
    if process_exists(pid) {
        return Err(anyhow!("Failed to terminate process {}", pid));
    }
    
    Ok(())
}
```

## Performance Optimizations

### Intelligent Caching
The application implements smart caching to balance responsiveness with system load:

```rust
impl ProcessMonitor {
    pub fn get_processes(&mut self) -> &[ProcessInfo] {
        if self.should_refresh() {
            self.refresh_cache();
        }
        &self.cached_processes
    }
    
    fn should_refresh(&self) -> bool {
        self.last_refresh.elapsed() > self.refresh_interval
    }
}
```

### Efficient Rendering
Using ratatui's immediate mode rendering, the UI updates at 60fps while using minimal CPU:

```rust
// Selective rendering based on dirty state
pub fn draw(&mut self, f: &mut Frame) {
    if self.needs_redraw {
        match self.current_mode {
            AppMode::Dashboard => self.draw_dashboard(f),
            AppMode::ProcessView => self.draw_process_view(f),
            AppMode::PortView => self.draw_port_view(f),
        }
        self.needs_redraw = false;
    }
}
```

## CLI Integration

Beyond the TUI interface, BossyRust provides a comprehensive CLI for automation and scripting:

```bash
# Quick port management
bossy-rust port 3000                    # Show what's using port 3000
bossy-rust kill-port 3000               # Kill process using port 3000

# Process management
bossy-rust ps --top-cpu                 # Show top CPU consumers
bossy-rust kill-process node            # Kill Node.js processes

# Developer utilities
bossy-rust cleanup --dev                # Clean up development processes
bossy-rust find-port 3000 3100          # Find available port in range
```

## macOS Integration

The application leverages macOS-specific features for accuracy and performance:

### Native System Commands
Instead of cross-platform libraries that might miss details, BossyRust uses native macOS commands:

```rust
// Using native netstat for accurate port information
pub fn get_port_info() -> Result<Vec<PortInfo>> {
    let output = Command::new("netstat")
        .args(&["-anv", "-p", "tcp"])
        .output()?;
    
    parse_netstat_output(&output.stdout)
}

// Combining lsof for process-to-port mapping
pub fn map_ports_to_processes() -> Result<HashMap<u16, ProcessInfo>> {
    let lsof_output = Command::new("lsof")
        .args(&["-i", "-P"])
        .output()?;
    
    parse_lsof_output(&lsof_output.stdout)
}
```

### System Process Protection
The application includes safeguards against accidentally killing critical system processes:

```rust
const PROTECTED_PROCESSES: &[&str] = &[
    "kernel_task", "launchd", "WindowServer", "SystemUIServer"
];

pub fn is_system_critical(process: &ProcessInfo) -> bool {
    PROTECTED_PROCESSES.contains(&process.name.as_str()) ||
    process.pid < 100 || // Low PID system processes
    process.is_kernel_thread()
}
```

## Real-World Usage Scenarios

### Scenario 1: Port Conflict Resolution
```bash
# Developer tries to start React app
npm start
# Error: Port 3000 is already in use

# With BossyRust
bossy-rust port 3000
# Shows: webpack-dev-server (PID: 1234) using port 3000

bossy-rust kill-port 3000
# Gracefully terminates the conflicting process
```

### Scenario 2: Development Environment Cleanup
```bash
# End of workday cleanup
bossy-rust cleanup --dev
# Finds and terminates:
# - Node.js development servers
# - webpack processes
# - Python Flask apps
# - Other common development tools
```

### Scenario 3: Resource Investigation
Using the TUI's search patterns:
- `>80%` — Find CPU-intensive processes
- `>2GB` — Identify memory leaks
- `node` — See all Node.js processes
- `:8080` — Check what's using the HTTP alt port

## Installation and Getting Started

### Prerequisites
- macOS 14+ (tested extensively on recent versions)
- Rust 1.70+ for building from source

### Installation
```bash
# Clone and build
git clone https://github.com/your-username/bossy-rust
cd bossy-rust
cargo build --release

# The binary will be at target/release/bossy-rust
```

### Quick Start
```bash
# Launch the TUI
./bossy-rust

# Or use CLI commands
./bossy-rust port 3000
./bossy-rust ps --top-cpu
```

## Configuration and Customization

BossyRust supports configuration via `config/presets.toml`:

```toml
[common_ports]
react = 3000
angular = 4200
flask = 5000
django = 8000

[ui_settings]
refresh_interval_ms = 2000
confirm_destructive_actions = true
max_process_display = 100

[keyboard_shortcuts]
quick_kill = "K"
emergency_cleanup = "Ctrl+E"
```

## Future Enhancements

The roadmap includes several exciting features:

### 1. Process Groups and Tagging
```rust
// Tag processes for easier management
pub struct ProcessTag {
    name: String,
    color: Color,
    processes: Vec<u32>,
}
```

### 2. Historical Monitoring
```rust
// Track process lifecycle and resource usage over time
pub struct ProcessHistory {
    pid: u32,
    cpu_history: Vec<f32>,
    memory_history: Vec<u64>,
    created_at: SystemTime,
}
```

### 3. Custom Actions and Automation
```rust
// User-defined actions for common workflows
pub struct CustomAction {
    name: String,
    trigger: KeyBinding,
    command: String,
    confirmation_required: bool,
}
```

## Security Considerations

BossyRust prioritizes safety and security:

- **User Confirmation**: All destructive actions require explicit confirmation
- **System Process Protection**: Built-in safeguards prevent killing critical processes
- **Privilege Handling**: Graceful degradation when lacking permissions
- **No Network Communication**: All operations are local-only
- **No Credential Storage**: No sensitive information is stored or transmitted

## Contributing and Community

The project is open source and welcomes contributions:

**GitHub Repository**: [https://github.com/your-username/bossy-rust](https://github.com/your-username/bossy-rust)

### How to Contribute
1. Fork the repository
2. Create feature branches for your changes
3. Write tests for new functionality
4. Submit pull requests with clear descriptions

## Conclusion: A Tool Built by Developers, for Developers

BossyRust represents more than just another process manager — it's a tool designed specifically for modern development workflows. By combining the power of Rust's performance and safety with ratatui's excellent TUI capabilities, it provides a native macOS experience that understands the unique needs of developers.

Whether you're debugging port conflicts, cleaning up development environments, or monitoring resource usage, BossyRust offers an intuitive, powerful, and safe way to manage your system processes.

The project demonstrates how modern systems programming languages like Rust can create tools that are both powerful and user-friendly, bridging the gap between command-line efficiency and graphical interface usability.

**Try BossyRust today** and experience process management designed for the modern developer workflow. Your terminal — and your productivity — will thank you.

---

*BossyRust is open source and available on GitHub. Contributions, feedback, and feature requests are welcome from the community.*