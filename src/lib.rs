//! Switchboard - A Rust-based CLI tool for scheduling AI coding agent prompts via Docker containers
//!
//! This library provides the core functionality for the Switchboard CLI tool, including:
//! - CLI argument parsing and command handling
//! - Configuration management
//! - Docker container orchestration
//! - Logging utilities
//! - Task scheduling capabilities
//! - Metrics collection and storage
//!
//! **Current Implementation Status:**
//!
//! - `cli`: Fully implemented clap-based CLI framework with run_up() fully implemented (976 lines)
//! - `config`: Fully implemented with comprehensive test coverage (1206 lines, 30 tests), includes OverlapMode enum and overlap configuration
//! - `scheduler`: Fully implemented with tokio-cron-scheduler integration (658 lines), includes queue handling for overlap mode
//! - `docker`: Docker connection, Dockerfile parsing, and container execution implemented (mod.rs 524 lines, run/ sub-module ~749 lines total, 8 tests)
//! - `logger`: Fully implemented with file and terminal writers (mod.rs 136 lines, file.rs 406 lines, terminal.rs 386 lines, 13 tests)
//! - `metrics`: Fully implemented metrics data structures, storage, and collector (mod.rs 291 lines, store.rs 584 lines, collector.rs 298 lines, 30 total tests)
//! - `architect`: Core state management system for multi-agent coordination workflow (mod.rs ~80 lines, state.rs ~500 lines, session.rs ~350 lines, session protocol with idempotency support)
//! - `skills`: Skills module for skill discovery and installation via npx skills CLI (in development)
//!
//! **Re-exports:** All CLI command structs (Cli, Commands, UpCommand, RunCommand, etc.) are public.
//! The handler functions (run_up, run_run, etc.) are also public and can be accessed directly
//! via the cli module (e.g., switchboard::cli::run(), switchboard::cli::run_run(), etc.).
//!
//! # Example
//!
//! ```ignore
//! use switchboard::cli;
//! use switchboard::config::Settings;
//!
//! // Load configuration and run the scheduler
//! ```

// Module declarations
pub mod cli;
pub mod commands;
pub mod config;
#[cfg(feature = "discord")]
pub mod discord;

/// Re-export of the Discord listener startup function.
///
/// This function starts the Discord bot and begins listening for messages.
/// Requires the "discord" feature to be enabled.
#[cfg(feature = "discord")]
pub use discord::start_discord_listener;
pub mod docker;
pub mod gateway;
pub mod api;
pub mod logger;
pub mod logging;
pub mod metrics;
pub mod observability;
pub mod scheduler;
pub mod skills;
pub mod traits;
pub mod ui;
pub mod workflows;
