mod commands;
mod config;
mod network;
mod process;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use tokio::time::sleep;
use tui::{AppEvent, AppState, EventHandler};

#[derive(Parser)]
#[command(name = "bossy-rust")]
#[command(about = "A lightweight Terminal User Interface (TUI) process manager for macOS")]
#[command(version = "0.1.0")]
#[command(author = "Pedro Nieto")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show what's using a specific port
    Port { port: u16 },
    /// Kill process using a specific port
    KillPort { port: u16 },
    /// Show all ports with optional filtering
    Ports {
        /// Show only common development ports
        #[arg(long)]
        common: bool,
        /// Show only listening ports
        #[arg(long)]
        listening: bool,
    },
    /// Kill processes by name
    KillProcess {
        name: String,
        /// Force kill (SIGKILL instead of SIGTERM)
        #[arg(short, long)]
        force: bool,
    },
    /// Show processes with optional filtering
    Ps {
        /// Show top CPU consumers
        #[arg(long)]
        top_cpu: bool,
        /// Show top memory consumers
        #[arg(long)]
        top_memory: bool,
        /// Limit number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Cleanup common development processes
    Cleanup {
        /// Target development processes
        #[arg(long)]
        dev: bool,
    },
    /// Find available port in range
    FindPort {
        /// Start port (default: 3000)
        #[arg(default_value = "3000")]
        start: u16,
        /// End port (default: start + 100)
        end: Option<u16>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(command) => {
            // Handle CLI commands
            handle_cli_command(command).await?;
        }
        None => {
            // Launch interactive TUI
            run_tui().await?;
        }
    }

    Ok(())
}

async fn handle_cli_command(command: Commands) -> Result<()> {
    use commands::CliHandler;

    match command {
        Commands::Port { port } => {
            CliHandler::show_port_info(port).await?;
        }
        Commands::KillPort { port } => {
            CliHandler::kill_port(port).await?;
        }
        Commands::Ports { common, listening } => {
            CliHandler::show_ports(common, listening).await?;
        }
        Commands::KillProcess { name, force } => {
            CliHandler::kill_process(&name, force).await?;
        }
        Commands::Ps {
            top_cpu,
            top_memory,
            limit,
        } => {
            CliHandler::show_processes(top_cpu, top_memory, limit).await?;
        }
        Commands::Cleanup { dev } => {
            CliHandler::cleanup_processes(dev).await?;
        }
        Commands::FindPort { start, end } => {
            let end = end.unwrap_or(start + 100);
            CliHandler::find_available_port(start, end).await?;
        }
    }

    Ok(())
}

async fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = AppState::new()?;
    let event_handler = EventHandler::default();

    // Main event loop
    loop {
        // Render UI
        terminal.draw(|f| {
            tui::dashboard::render_dashboard(f, &mut app);
        })?;

        // Handle events
        match event_handler.next().await? {
            AppEvent::Key(key_event) => {
                app.handle_key_event(key_event).await?;
            }
            AppEvent::Resize(width, height) => {
                terminal.resize(ratatui::layout::Rect::new(0, 0, width, height))?;
            }
            AppEvent::Refresh => {
                if app.should_refresh() {
                    app.refresh_data()?;
                }
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }

        // Small delay to prevent excessive CPU usage
        sleep(Duration::from_millis(16)).await; // ~60fps
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
