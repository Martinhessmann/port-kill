pub mod cli;
pub mod config;
pub mod console_app;
pub mod process_monitor;
pub mod types;

// macOS-specific modules (only compiled on macOS)
#[cfg(target_os = "macos")]
pub mod app;
#[cfg(target_os = "macos")]
pub mod tray_menu;
