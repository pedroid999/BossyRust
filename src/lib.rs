pub mod commands;
pub mod config;
pub mod network;
pub mod process;
pub mod tui;

pub mod testing;

// Re-export key types from modules for easier testing access
pub use tui::*;
pub use process::*;
pub use network::*;