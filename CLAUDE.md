# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
BossyRust is a lightweight Terminal User Interface (TUI) process manager for macOS built with Rust and ratatui. It combines system process management with network port monitoring, providing an intuitive alternative to Activity Monitor and command-line tools like `ps`, `lsof`, and `kill`.

## Build and Development Commands

### Basic Commands
```bash
# Build the project
cargo build

# Run the TUI application
cargo run

# Run with CLI commands
cargo run -- port 3000
cargo run -- kill-port 3000
cargo run -- ps --top-cpu

# Build optimized release
cargo build --release

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run linting
cargo clippy -- -D warnings

# Run all checks
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

### Development Workflow
```bash
# Quick development cycle
cargo check  # Fast syntax/type checking
cargo run    # Test TUI functionality
cargo test   # Run unit tests
```

## Architecture Overview

### Core Modules Structure
```
src/
├── main.rs              # Entry point with CLI parsing and TUI initialization
├── process/             # Process management functionality
│   ├── info.rs         # Process information gathering and filtering
│   ├── killer.rs       # Process termination logic (SIGTERM → SIGKILL)
│   └── monitor.rs      # Real-time process monitoring
├── network/            # Network port management
│   ├── ports.rs        # Port enumeration and mapping to processes
│   ├── connections.rs  # Network connection tracking
│   └── utils.rs        # Network utilities and port categorization
├── tui/                # Terminal User Interface
│   ├── app.rs          # Application state management and event handling
│   ├── dashboard.rs    # Main rendering logic for all views
│   ├── events.rs       # Event handling system
│   └── components/     # Reusable UI components
└── commands/           # CLI command implementations
    ├── cli.rs          # Command-line interface handlers
    └── shortcuts.rs    # Quick action implementations
```

### Key Design Patterns

**State Management**: The `AppState` struct in `tui/app.rs` centralizes all application state including current mode, search filters, selected items, and data caches. This follows a single-source-of-truth pattern.

**Process Monitoring**: The `ProcessMonitor` in `process/monitor.rs` provides cached access to system information with configurable refresh intervals to balance responsiveness with system load.

**Port-to-Process Mapping**: The network module uses both `netstat` and `lsof` commands to create comprehensive port-to-process mappings, handling both IPv4 and IPv6 connections.

**Multi-Modal UI**: The TUI supports multiple views (Dashboard, Process View, Port View, Connection View) with consistent navigation patterns using function keys (F1-F4).

## Key Features Implementation

### Smart Search Patterns
The search system supports multiple patterns:
- `:3000` - Port number search
- `#1234` - Process ID search  
- `>50%` - CPU usage threshold
- `>1GB` - Memory usage threshold
- `node` - Process name search

### Process Termination Strategy
1. Send SIGTERM for graceful shutdown
2. Wait up to 5 seconds with 100ms polling
3. Escalate to SIGKILL if process still running
4. Final verification with 2-second timeout

### Development Port Detection
Common development ports (3000, 4200, 5000, 8000, 8080, etc.) are automatically identified and can be filtered/killed in bulk operations.

## Configuration

### Port Presets
The `config/presets.toml` file contains predefined port mappings and process groups for quick actions. This can be extended for custom development environments.

### UI Customization
- Refresh interval: 2 seconds default
- Auto-refresh can be toggled
- Multi-selection mode for batch operations
- Confirmation dialogs for destructive actions

## macOS-Specific Considerations

### System Integration
- Uses macOS `ps`, `lsof`, and `netstat` commands
- Handles process permissions gracefully
- Respects system process protection
- Integrates with macOS process hierarchy

### Performance Optimizations
- Caches system information between refreshes
- Uses efficient regex parsing for command output
- Implements lazy loading for large process lists
- 60fps UI updates with minimal system impact

## Development Guidelines

### Adding New Views
1. Create new module in `src/tui/`
2. Add render function following existing patterns
3. Register in `dashboard.rs` render dispatch
4. Add navigation key in `app.rs` event handling

### Extending CLI Commands
1. Add command variant to `Commands` enum in `main.rs`
2. Implement handler in `commands/cli.rs`
3. Add command parsing in `handle_cli_command()`

### Process Detection Logic
When adding new process types, update:
- `process/info.rs` for process categorization
- `commands/shortcuts.rs` for bulk operations
- `config/presets.toml` for configuration

## Testing Strategy

### Unit Tests
- Process parsing and filtering logic
- Network port mapping accuracy
- Search pattern matching
- Port availability checking

### Integration Tests
- CLI command execution
- TUI state management
- System command integration
- Error handling paths

## Security Considerations

- Never runs privileged operations without user confirmation
- Validates all system command inputs
- Prevents killing critical system processes
- Handles permission errors gracefully
- No credential storage or network communication