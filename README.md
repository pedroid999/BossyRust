# BossyRust - Mac Process Manager

A lightweight Terminal User Interface (TUI) process manager for macOS that combines system process management with network port monitoring. Built with Rust and ratatui.

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)
![Language](https://img.shields.io/badge/language-Rust-orange)

## 🚀 Features

### Core Functionality
- **Real-time Process Monitoring**: List and monitor all running processes with CPU and memory usage
- **Intelligent Process Filtering**: Search by name, PID, CPU/memory thresholds, or custom patterns
- **Graceful Process Termination**: SIGTERM → SIGKILL escalation with user confirmation
- **Network Port Management**: Map ports to processes and kill by port number
- **Development Tools Integration**: Quick cleanup of common development processes

### Advanced Capabilities
- **Smart Search Patterns**:
  - `:3000` - Find processes using port 3000
  - `#1234` - Search by Process ID
  - `>50%` - Processes using more than 50% CPU
  - `>1GB` - Processes using more than 1GB memory
  - `node` - Search by process name

- **Developer-Focused Features**:
  - Detect common development ports (3000, 4200, 5000, 8080, etc.)
  - Bulk cleanup of dev processes (Node.js, webpack, etc.)
  - Port availability checking with suggestions
  - Service identification for well-known ports

### Terminal User Interface
- **Multi-Panel Dashboard**: Overview with top processes and port summary
- **Dedicated Views**: Separate process and port management interfaces
- **Intuitive Navigation**: Function keys (F1-F4) for view switching
- **Real-time Updates**: Configurable auto-refresh (default: 2 seconds)
- **Multi-selection**: Select and kill multiple processes at once

## 🛠️ Installation

### Prerequisites
- macOS (tested on macOS 14+)
- Rust 1.70+ (for building from source)

### Build from Source
```bash
git clone https://github.com/your-username/bossy-rust
cd bossy-rust
cargo build --release
```

The binary will be available at `target/release/bossy-rust`.

## 📖 Usage

### Interactive TUI Mode
Launch the full Terminal User Interface:
```bash
bossy-rust
```

### CLI Commands

#### Port Management
```bash
# Show what's using a specific port
bossy-rust port 3000

# Kill process using a specific port
bossy-rust kill-port 3000

# Show all listening ports
bossy-rust ports --listening

# Show common development ports
bossy-rust ports --common
```

#### Process Management
```bash
# Show top CPU consumers
bossy-rust ps --top-cpu

# Show top memory consumers
bossy-rust ps --top-memory

# Kill processes by name
bossy-rust kill-process node

# Force kill (SIGKILL immediately)
bossy-rust kill-process node --force
```

#### Development Utilities
```bash
# Clean up common development processes
bossy-rust cleanup --dev

# Find available port in range
bossy-rust find-port 3000 3100
```

### TUI Navigation

| Key | Action |
|-----|--------|
| `F1` | Switch to Process View |
| `F2` | Switch to Port View |
| `F3` | Switch to Connection View |
| `F4` / `?` | Show Help |
| `/` | Enter search mode |
| `k` / `Delete` | Kill selected item |
| `Space` | Multi-select |
| `Enter` | Primary action |
| `s` | Cycle sort options |
| `r` / `Ctrl+R` | Refresh data |
| `q` / `Esc` | Quit/Back |
| `Ctrl+C` | Force quit |

## 🏗️ Architecture

The project is organized into several key modules:

### Core Modules
- **Process Management** (`src/process/`): Process enumeration, monitoring, and termination
- **Network Management** (`src/network/`): Port scanning, connection tracking, and utilities
- **TUI Interface** (`src/tui/`): Ratatui-based user interface with multiple views
- **CLI Commands** (`src/commands/`): Command-line interface and quick actions

### Key Features
- **Asynchronous Design**: Non-blocking UI with real-time updates
- **Resource Efficient**: Minimal system impact with smart caching
- **macOS Native**: Uses system commands (`ps`, `lsof`, `netstat`) for accuracy
- **Developer Experience**: Intuitive shortcuts and smart defaults

## 🔧 Configuration

Configuration is available via the `config/presets.toml` file:

```toml
[common_ports]
react = 3000
angular = 4200
flask = 5000

[ui_settings]
refresh_interval_ms = 2000
confirm_destructive_actions = true

[keyboard_shortcuts]
quick_kill = "K"
emergency_cleanup = "E"
```

## 🧪 Development

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run linting
cargo clippy -- -D warnings
```

### Project Structure
```
src/
├── main.rs              # Entry point and CLI parsing
├── process/             # Process management
│   ├── info.rs         # Process information gathering
│   ├── killer.rs       # Process termination logic
│   └── monitor.rs      # Real-time monitoring
├── network/            # Network port management
│   ├── ports.rs        # Port enumeration and mapping
│   ├── connections.rs  # Connection tracking
│   └── utils.rs        # Network utilities
├── tui/                # Terminal User Interface
│   ├── app.rs          # Application state management
│   ├── dashboard.rs    # Main rendering logic
│   └── events.rs       # Event handling
└── commands/           # CLI command implementations
    ├── cli.rs          # Command handlers
    └── shortcuts.rs    # Quick actions
```

## 🚨 Security & Safety

- **User Confirmation**: Destructive actions require explicit confirmation
- **System Process Protection**: Prevents killing critical system processes
- **Privilege Handling**: Graceful handling of permission errors
- **No Credentials**: No storage or transmission of sensitive information

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [ratatui](https://github.com/ratatui-org/ratatui) for the excellent TUI framework
- Uses [sysinfo](https://github.com/GuillaumeGomez/sysinfo) for cross-platform system information
- Inspired by Activity Monitor and htop but designed specifically for macOS developers

---

**Note**: This tool is designed for development and system administration purposes. Always exercise caution when killing processes, especially system processes.