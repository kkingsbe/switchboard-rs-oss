//! Workflow init command module
//!
//! This module provides the workflow init subcommand implementation for the Switchboard CLI.
//! It includes functionality for initializing new Switchboard workflows.

pub mod init;
pub mod types;

pub use init::run_workflow_init;
pub use types::{ExitCode, WorkflowInit, WorkflowInitCommand, WorkflowInitSubcommand};
