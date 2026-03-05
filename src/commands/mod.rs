//! Commands module - Individual command implementations
//!
//! This module provides command handlers for the Switchboard CLI tool.
//! Each command is implemented in its own module and exported here for convenience.
//!
//! # Available Commands
//!
//! - [`BuildCommand`] - Build Docker images for agents defined in switchboard.toml
//! - [`list_agents`] - Display configured agents and their details
//! - [`logs::run`] - View logs from agent executions
//! - [`metrics()`] - Display agent execution statistics
//! - [`ValidateCommand`] - Validate switchboard.toml configuration file
//!
//! # Module Structure
//!
//! The commands are organized as follows:
//!
//! - [`build`] - Docker image building functionality
//! - [`list`] - Agent listing and display functionality
//! - [`logs`] - Log viewing and following functionality
//! - [`mod@metrics`] - Metrics collection and display functionality
//! - [`validate`] - Configuration validation functionality
//!
//! # Usage
//!
//! Commands are typically used through the CLI interface defined in the `cli` module.
//! Each command struct implements a `run()` method that executes the command logic.

pub mod build;
pub mod list;
pub mod logs;
pub mod metrics;
pub mod skills;
pub mod validate;
pub mod workflows;

pub use build::BuildCommand;
pub use list::list_agents;
pub use metrics::metrics;
pub use skills::{run_skills, SkillsCommand};
pub use validate::ValidateCommand;
pub use workflows::{run_workflows, WorkflowsCommand};
