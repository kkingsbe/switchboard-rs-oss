//! Project command module
//!
//! This module provides the project subcommand implementations for the Switchboard CLI.
//! It includes functionality for initializing new Switchboard projects.

pub mod init;
pub mod types;

pub use init::run_project_init;
pub use types::{ExitCode, ProjectCommand, ProjectInit, ProjectSubcommand};
