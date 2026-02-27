//! Integration and unit tests for switchboard

mod build_command;
mod down_command;
mod metrics_command;
mod performance_common;
mod skill_install_error_handling;
mod skills_install_error_handling;
mod skills_install_performance;
mod skills_install_performance_command;
mod skills_log_prefix;
mod timeout_parsing;
mod workspace_path_validation;

#[cfg(feature = "integration")]
mod integration;
