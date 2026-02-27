//! Container execution - Create and run Docker containers for agent execution
//!
//! This module handles:
//! - Container configuration types
//! - Container creation and execution using bollard
//! - Container lifecycle management
//! - Container log streaming from stdout/stderr
//! - Container timeout handling and exit waiting

use crate::docker::DockerError;

#[allow(clippy::module_inception)]
pub mod run;
pub mod streams;
pub mod types;
pub mod wait;

pub use self::run::{run_agent, AgentExecutionResult, find_preexisting_skills};
pub use self::streams::attach_and_stream_logs;
pub use self::types::{ContainerConfig, ContainerError};
pub use self::wait::{parse_timeout, wait_for_exit, wait_with_timeout, ExitStatus};
