//! Switchboard CLI - Main entry point for the Switchboard application
//!
//! This binary provides the command-line interface for scheduling
//! AI coding agent prompts via Docker containers.

use switchboard::cli;
use switchboard::ui::colors::{color_error, should_use_colors, ColorMode};

#[tokio::main]
async fn main() {
    match cli::run().await {
        Ok(color_mode) => {
            // Command succeeded, color_mode is available if needed for future use
            let _ = color_mode;
        }
        Err(e) => {
            // On error, we need to determine color mode - use Auto as default
            // since we can't get the parsed CLI value on error path
            let color_mode = ColorMode::Auto;

            if should_use_colors(color_mode) {
                eprintln!("{}", color_error(&format!("Error: {}", e)));
            } else {
                eprintln!("Error: {}", e);
            }
            std::process::exit(1);
        }
    }
}
