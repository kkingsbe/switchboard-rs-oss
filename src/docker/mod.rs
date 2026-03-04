//! Docker Client - Build images and manage container lifecycle via Docker Engine API
//!
//! This module handles:
//! - Docker connection and availability checking using bollard
//! - DockerClient struct with image_name and image_tag configuration
//! - Error handling for Docker connection failures
//! - Agent image building using bollard (fully implemented)
//! - Container creation and execution (implemented)
//!
//! **Current Status:** Docker connection, build scaffolding, and container execution implemented

pub use crate::traits::{
    BuildOptions, DockerClientTrait, ProcessError, ProcessExecutorTrait, RealDockerClient,
    RealProcessExecutor,
};

pub mod run;
pub use run::{find_preexisting_skills, run_agent, AgentExecutionResult, ContainerConfig};

pub mod skills;

/// Image build context utilities module
pub mod build;
/// Connection and client management module
pub mod client;

/// Docker connection trait module
pub mod connection;

// Re-export all public items from client module for backward compatibility
pub use client::{
    check_docker_available, connect_to_docker, get_docker_socket_path, DockerClient, DockerError,
};

// Re-export all public items from build module for backward compatibility
pub use build::{add_directory_to_tar, create_build_context_tarball};

// Re-export all public items from connection module for trait abstraction
pub use connection::{
    DockerCommand, DockerResponse, DockerConnectionTrait, MockDockerConnection,
    MockDockerConnectionBuilder, RealDockerConnection,
};
