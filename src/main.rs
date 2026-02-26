//! Switchboard CLI - Main entry point for the Switchboard application
//!
//! This binary provides the command-line interface for scheduling
//! AI coding agent prompts via Docker containers.

use switchboard::cli;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
