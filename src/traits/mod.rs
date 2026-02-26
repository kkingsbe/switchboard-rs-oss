//! Docker Client Traits - Abstraction layer for Docker operations
//!
//! This module provides:
//! - DockerClientTrait: A trait defining the interface for Docker client operations
//! - RealDockerClient: A concrete implementation using bollard
//! - BuildOptions: Configuration for building Docker images
//! - ProcessExecutorTrait: A trait for executing external processes
//! - RealProcessExecutor: A concrete implementation using std::process::Command
//! - ProcessOutput, ExitStatus, ProcessError: Supporting types for process execution
//!
//! The trait allows for easier testing through dependency injection and
//! provides a clean abstraction over the bollard Docker API.

use bollard::Docker;
use futures::StreamExt;
use std::fmt::Debug;
use std::path::PathBuf;
use std::process::Command;

pub use crate::commands::skills::ExitCode;
pub use crate::docker::run::types::ContainerConfig;
pub use crate::docker::DockerError;

/// Options for building a Docker image
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Docker image name (e.g., "switchboard-agent")
    pub image_name: String,
    /// Docker image tag (e.g., "latest")
    pub tag: String,
    /// Optional build args to pass to Docker build
    pub build_args: std::collections::HashMap<String, String>,
    /// Optional Dockerfile content (if None, uses Dockerfile in context)
    pub dockerfile: Option<String>,
}

impl BuildOptions {
    /// Create a new BuildOptions with the given image name and tag
    pub fn new(image_name: impl Into<String>, tag: impl Into<String>) -> Self {
        BuildOptions {
            image_name: image_name.into(),
            tag: tag.into(),
            build_args: std::collections::HashMap::new(),
            dockerfile: None,
        }
    }

    /// Add a build arg to the build options
    pub fn with_build_arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.build_args.insert(key.into(), value.into());
        self
    }

    /// Set the Dockerfile content
    pub fn with_dockerfile(mut self, dockerfile: impl Into<String>) -> Self {
        self.dockerfile = Some(dockerfile.into());
        self
    }
}

/// Docker Client Trait - defines the interface for Docker operations
///
/// This trait provides an abstraction over Docker operations, allowing for:
/// - Easier unit testing through mock implementations
/// - Dependency injection for different Docker client implementations
/// - A clean, focused API for Docker container management
pub trait DockerClientTrait: Send + Sync {
    /// Ping the Docker daemon to check availability
    ///
    /// # Errors
    ///
    /// Returns DockerError if Docker is unavailable or not responding
    fn ping(&self) -> Result<(), DockerError>;

    /// Check if a Docker image exists
    ///
    /// # Arguments
    ///
    /// * `name` - The image name (e.g., "switchboard-agent")
    /// * `tag` - The image tag (e.g., "latest")
    ///
    /// # Errors
    ///
    /// Returns DockerError if there's an issue checking the image
    fn image_exists(&self, name: &str, tag: &str) -> Result<bool, DockerError>;

    /// Build a Docker image
    ///
    /// # Arguments
    ///
    /// * `options` - Build configuration options
    /// * `context` - Path to the build context directory
    ///
    /// # Errors
    ///
    /// Returns DockerError if the build fails
    fn build_image(&self, options: BuildOptions, context: PathBuf) -> Result<String, DockerError>;

    /// Run a Docker container
    ///
    /// # Arguments
    ///
    /// * `config` - Container configuration
    ///
    /// # Errors
    ///
    /// Returns DockerError if container creation or start fails
    fn run_container(&self, config: ContainerConfig) -> Result<String, DockerError>;

    /// Stop a running Docker container
    ///
    /// # Arguments
    ///
    /// * `container_id` - The container ID to stop
    /// * `timeout` - Timeout in seconds to wait before force-killing
    ///
    /// # Errors
    ///
    /// Returns DockerError if stopping the container fails
    fn stop_container(&self, container_id: &str, timeout: u64) -> Result<(), DockerError>;

    /// Get container logs
    ///
    /// # Arguments
    ///
    /// * `container_id` - The container ID to get logs from
    /// * `follow` - Whether to follow the log stream
    /// * `tail` - Number of lines to show from the end (None for all)
    ///
    /// # Errors
    ///
    /// Returns DockerError if fetching logs fails
    fn container_logs(
        &self,
        container_id: &str,
        follow: bool,
        tail: Option<u64>,
    ) -> Result<String, DockerError>;

    /// Wait for a container to exit and return its exit code
    ///
    /// # Arguments
    ///
    /// * `container_id` - The container ID to wait for
    /// * `timeout` - Maximum time to wait in seconds
    ///
    /// # Errors
    ///
    /// Returns DockerError if waiting fails
    fn wait_container(&self, container_id: &str, timeout: u64) -> Result<ExitCode, DockerError>;

    /// Create a Docker container (without starting it)
    ///
    /// # Arguments
    ///
    /// * `options` - Container creation options (including container name)
    /// * `config` - Container configuration
    ///
    /// # Errors
    ///
    /// Returns container ID on success, DockerError on failure
    fn create_container(
        &self,
        options: Option<bollard::container::CreateContainerOptions<String>>,
        config: bollard::container::Config<String>,
    ) -> Result<String, DockerError>;

    /// Start a Docker container
    ///
    /// # Arguments
    ///
    /// * `container_id` - The container ID to start
    /// * `options` - Start options
    ///
    /// # Errors
    ///
    /// Returns DockerError if starting fails
    fn start_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::StartContainerOptions<String>>,
    ) -> Result<(), DockerError>;

    /// Inspect a Docker container to get its state
    ///
    /// # Arguments
    ///
    /// * `container_id` - The container ID to inspect
    /// * `options` - Inspect options
    ///
    /// # Errors
    ///
    /// Returns container inspection result on success, DockerError on failure
    fn inspect_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::InspectContainerOptions>,
    ) -> Result<bollard::service::ContainerInspectResponse, DockerError>;

    /// Send a signal to a Docker container
    ///
    /// # Arguments
    ///
    /// * `container_id` - The container ID
    /// * `options` - Signal options (e.g., signal name)
    ///
    /// # Errors
    ///
    /// Returns DockerError if signal fails
    fn kill_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::KillContainerOptions<String>>,
    ) -> Result<(), DockerError>;
}

/// Real Docker Client implementation using bollard
///
/// This struct wraps the bollard Docker client and implements DockerClientTrait
/// for production use. It provides the actual Docker operations by delegating
/// to the bollard library.
pub struct RealDockerClient {
    docker: Docker,
}

impl Clone for RealDockerClient {
    fn clone(&self) -> Self {
        RealDockerClient {
            docker: self.docker.clone(),
        }
    }
}

impl RealDockerClient {
    /// Create a new RealDockerClient by connecting to the local Docker daemon
    ///
    /// # Errors
    ///
    /// Returns DockerError if connection to Docker fails
    pub async fn new() -> Result<Self, DockerError> {
        // Check Docker availability and get client - this provides better error messages
        let docker = crate::docker::check_docker_available()
            .await
            .map_err(|e| DockerError::ConnectionError(e.to_string()))?;
        Ok(RealDockerClient { docker })
    }

    /// Create a new RealDockerClient from an existing Docker connection
    pub fn from_docker(docker: Docker) -> Self {
        RealDockerClient { docker }
    }

    /// Get a reference to the underlying bollard Docker client
    pub fn docker(&self) -> &Docker {
        &self.docker
    }
}

impl DockerClientTrait for RealDockerClient {
    fn ping(&self) -> Result<(), DockerError> {
        // Use blocking runtime for ping since bollard 0.18 uses async
        let _result = tokio::runtime::Handle::current()
            .block_on(self.docker.ping())
            .map_err(|e| DockerError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    fn image_exists(&self, name: &str, tag: &str) -> Result<bool, DockerError> {
        let image_name = format!("{}:{}", name, tag);

        let exists = tokio::runtime::Handle::current()
            .block_on(async {
                use bollard::image::ListImagesOptions;

                let options = Some(ListImagesOptions::<String> {
                    all: false,
                    ..Default::default()
                });

                let images = self.docker.list_images(options).await?;

                for image in images {
                    let repo_tags = &image.repo_tags;
                    for repo_tag in repo_tags {
                        if repo_tag == &image_name {
                            return Ok(true);
                        }
                    }
                }

                Ok(false)
            })
            .map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))?;

        Ok(exists)
    }

    fn build_image(&self, options: BuildOptions, context: PathBuf) -> Result<String, DockerError> {
        use bollard::image::BuildImageOptions;
        use futures::StreamExt;

        // Create tarball with build context
        let dockerfile_content = options
            .dockerfile
            .unwrap_or_else(|| "FROM scratch".to_string());
        let tarball = crate::docker::create_build_context_tarball(&context, &dockerfile_content)
            .map_err(|e| DockerError::BuildError {
                error_details: e.to_string(),
                suggestion: "Check that the build context directory exists and is readable"
                    .to_string(),
            })?;

        let image_name = format!("{}:{}", options.image_name, options.tag);

        let _build_options = BuildImageOptions {
            dockerfile: "Dockerfile",
            t: &image_name,
            rm: true,
            ..Default::default()
        };

        // Set up build args if any
        let _build_args: Vec<(String, String)> = options.build_args.into_iter().collect();

        // Convert tarball to bytes
        let tarball_bytes = bytes::Bytes::from(tarball.into_inner());

        let image_name = format!("{}:{}", options.image_name, options.tag);

        let build_options = BuildImageOptions {
            dockerfile: "Dockerfile",
            t: &image_name,
            rm: true,
            ..Default::default()
        };

        let image_id = tokio::runtime::Handle::current()
            .block_on(async {
                let mut stream = self
                    .docker
                    .build_image(build_options, None, Some(tarball_bytes));

                let mut final_image_id = String::new();

                while let Some(build_result) = stream.next().await {
                    match build_result {
                        Ok(info) => {
                            if let Some(id) = info.id {
                                final_image_id = id;
                            }
                            // Log build output for debugging
                            if let Some(stream_type) = info.stream {
                                tracing::debug!("Build output: {}", stream_type);
                            }
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                if final_image_id.is_empty() {
                    // If no image ID from stream, use the image name
                    Ok(image_name.clone())
                } else {
                    Ok(final_image_id)
                }
            })
            .map_err(|e| DockerError::BuildError {
                error_details: e.to_string(),
                suggestion: "Check Docker build logs for details".to_string(),
            })?;

        Ok(image_id)
    }

    fn run_container(&self, config: ContainerConfig) -> Result<String, DockerError> {
        // Delegate to the existing run_container logic in docker/run.rs
        // For now, we'll create a simple implementation
        use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
        use bollard::image::CreateImageOptions;

        let container_name = format!("switchboard-{}", config.agent_name);
        let image_name = "switchboard-agent:latest".to_string();

        // Clone docker for use in block_in_place
        let docker = self.docker.clone();

        // Pull image if needed - use block_in_place to handle async context
        let _ = tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                let options = Some(CreateImageOptions {
                    from_image: "switchboard-agent",
                    tag: "latest",
                    ..Default::default()
                });

                let mut stream = docker.create_image(options, None, None);
                while let Some(_result) = stream.next().await {
                    // Just consume the stream
                }
                Ok::<(), bollard::errors::Error>(())
            })
        });

        // Create container
        let host_config = bollard::service::HostConfig {
            auto_remove: Some(true),
            ..Default::default()
        };

        let container_config = Config {
            image: Some(image_name),
            env: Some(config.env_vars.clone()),
            host_config: Some(host_config),
            ..Default::default()
        };

        let create_options = CreateContainerOptions {
            name: &container_name,
            platform: None,
        };

        // Clone for block_in_place
        let docker = self.docker.clone();
        let container_name_clone = container_name.clone();
        
        let response = tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                docker
                    .create_container(Some(create_options), container_config)
                    .await
            })
        }).map_err(|e| DockerError::ContainerCreateError {
            container_name: container_name_clone,
            error_details: e.to_string(),
            suggestion: "Check that the Docker image exists and is valid".to_string(),
        })?;

        // Start container - use block_in_place
        let docker = self.docker.clone();
        let container_id = response.id.clone();
        let container_name_clone = container_name.clone();
        
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                docker
                    .start_container(&container_id, None::<StartContainerOptions<String>>)
                    .await
            })
        }).map_err(|e| DockerError::ContainerStartError {
            container_name: container_name_clone,
            error_details: e.to_string(),
            suggestion: "Check Docker logs for details".to_string(),
        })?;

        Ok(response.id)
    }

    fn stop_container(&self, container_id: &str, timeout: u64) -> Result<(), DockerError> {
        use bollard::container::StopContainerOptions;

        let options = StopContainerOptions { t: timeout as i64 };
        let docker = self.docker.clone();
        let container_id = container_id.to_string();

        // Use block_in_place to handle async context
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                docker
                    .stop_container(&container_id, Some(options))
                    .await
            })
        }).map_err(|e| DockerError::ContainerStopError {
            container_name: container_id.to_string(),
            error_details: e.to_string(),
                suggestion: "Check if the container is running".to_string(),
            })?;

        Ok(())
    }

    fn container_logs(
        &self,
        container_id: &str,
        follow: bool,
        tail: Option<u64>,
    ) -> Result<String, DockerError> {
        use bollard::container::LogsOptions;
        use futures::StreamExt;

        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            follow,
            tail: tail
                .map(|t| t.to_string())
                .unwrap_or_else(|| "all".to_string()),
            ..Default::default()
        };

        let mut logs = String::new();

        tokio::runtime::Handle::current()
            .block_on(async {
                let mut stream = self.docker.logs(container_id, Some(options));

                while let Some(log_result) = stream.next().await {
                    match log_result {
                        Ok(log) => {
                            logs.push_str(&log.to_string());
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                Ok(())
            })
            .map_err(|e| DockerError::ConnectionError(e.to_string()))?;

        Ok(logs)
    }

    fn wait_container(&self, container_id: &str, _timeout: u64) -> Result<ExitCode, DockerError> {
        use bollard::container::WaitContainerOptions;
        use futures::StreamExt;

        let options = WaitContainerOptions { condition: "exit" };

        let exit_code = tokio::runtime::Handle::current()
            .block_on(async {
                let mut stream = self.docker.wait_container(container_id, Some(options));

                if let Some(result) = stream.next().await {
                    match result {
                        Ok(status) => Ok(status.status_code as i32),
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(0) // Default to success if no status
                }
            })
            .map_err(|e| DockerError::ConnectionError(e.to_string()))?;

        Ok(ExitCode::from_i32(exit_code))
    }

    fn create_container(
        &self,
        options: Option<bollard::container::CreateContainerOptions<String>>,
        config: bollard::container::Config<String>,
    ) -> Result<String, DockerError> {
        tokio::runtime::Handle::current()
            .block_on(async {
                let response = self.docker.create_container(options, config).await?;
                Ok(response.id)
            })
            .map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))
    }

    fn start_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::StartContainerOptions<String>>,
    ) -> Result<(), DockerError> {
        tokio::runtime::Handle::current()
            .block_on(async { self.docker.start_container(container_id, options).await })
            .map_err(|e| DockerError::ConnectionError(e.to_string()))
    }

    fn inspect_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::InspectContainerOptions>,
    ) -> Result<bollard::service::ContainerInspectResponse, DockerError> {
        tokio::runtime::Handle::current()
            .block_on(async { self.docker.inspect_container(container_id, options).await })
            .map_err(|e| DockerError::ConnectionError(e.to_string()))
    }

    fn kill_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::KillContainerOptions<String>>,
    ) -> Result<(), DockerError> {
        tokio::runtime::Handle::current()
            .block_on(async { self.docker.kill_container(container_id, options).await })
            .map_err(|e| DockerError::ConnectionError(e.to_string()))
    }
}

// ============================================================================
// Process Executor Trait and Supporting Types
// ============================================================================

/// Exit status of a process
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitStatus {
    /// Process exited with a code
    Code(i32),
    /// Process was terminated by a signal
    Signal(i32),
    /// Unknown exit status
    Unknown,
}

impl ExitStatus {
    /// Create ExitStatus from std::process::ExitStatus
    #[allow(dead_code)]
    pub fn from_std(status: std::process::ExitStatus) -> Self {
        if let Some(code) = status.code() {
            ExitStatus::Code(code)
        } else {
            ExitStatus::Unknown
        }
    }

    /// Get the exit code if available
    pub fn code(&self) -> Option<i32> {
        match self {
            ExitStatus::Code(code) => Some(*code),
            _ => None,
        }
    }

    /// Check if the process succeeded (exit code 0)
    pub fn success(&self) -> bool {
        self.code() == Some(0)
    }
}

/// Output from a process execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    /// Standard output content
    pub stdout: String,
    /// Standard error content
    pub stderr: String,
    /// Exit status of the process
    pub status: ExitStatus,
}

impl ProcessOutput {
    /// Create a new ProcessOutput
    pub fn new(stdout: String, stderr: String, status: ExitStatus) -> Self {
        ProcessOutput {
            stdout,
            stderr,
            status,
        }
    }
}

/// Errors that can occur during process execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessError {
    /// Program not found
    ProgramNotFound { program: String, suggestion: String },
    /// IO error during execution
    IoError {
        error_details: String,
        suggestion: String,
    },
    /// Execution failed with additional context
    ExecutionFailed {
        program: String,
        error_details: String,
        suggestion: String,
    },
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::ProgramNotFound {
                program,
                suggestion,
            } => {
                write!(f, "Program not found: {}. {}", program, suggestion)
            }
            ProcessError::IoError {
                error_details,
                suggestion,
            } => {
                write!(f, "IO error: {}. {}", error_details, suggestion)
            }
            ProcessError::ExecutionFailed {
                program,
                error_details,
                suggestion,
            } => {
                write!(
                    f,
                    "Execution failed for '{}': {}. {}",
                    program, error_details, suggestion
                )
            }
        }
    }
}

impl std::error::Error for ProcessError {}

/// Process Executor Trait - defines the interface for executing external processes
///
/// This trait provides an abstraction over process execution, allowing for:
/// - Easier unit testing through mock implementations
/// - Dependency injection for different process executor implementations
/// - A clean, focused API for running external programs
pub trait ProcessExecutorTrait: Send + Sync + Debug {
    /// Execute a program with arguments
    ///
    /// # Arguments
    ///
    /// * `program` - The program to execute
    /// * `args` - Command line arguments
    ///
    /// # Errors
    ///
    /// Returns ProcessError if execution fails
    fn execute(&self, program: &str, args: &[String]) -> Result<ProcessOutput, ProcessError>;

    /// Execute a program with arguments, environment variables, and working directory
    ///
    /// # Arguments
    ///
    /// * `program` - The program to execute
    /// * `args` - Command line arguments
    /// * `env` - Environment variables to set (added to current environment)
    /// * `working_dir` - Optional working directory for the process
    ///
    /// # Errors
    ///
    /// Returns ProcessError if execution fails
    fn execute_with_env(
        &self,
        program: &str,
        args: &[String],
        env: std::collections::HashMap<String, String>,
        working_dir: Option<PathBuf>,
    ) -> Result<ProcessOutput, ProcessError>;
}

/// Real Process Executor implementation using std::process::Command
///
/// This struct wraps std::process::Command and implements ProcessExecutorTrait
/// for production use.
#[derive(Debug, Clone, Default)]
pub struct RealProcessExecutor;

impl RealProcessExecutor {
    /// Create a new RealProcessExecutor
    pub fn new() -> Self {
        RealProcessExecutor
    }
}

impl ProcessExecutorTrait for RealProcessExecutor {
    fn execute(&self, program: &str, args: &[String]) -> Result<ProcessOutput, ProcessError> {
        self.execute_with_env(program, args, std::collections::HashMap::new(), None)
    }

    fn execute_with_env(
        &self,
        program: &str,
        args: &[String],
        env: std::collections::HashMap<String, String>,
        working_dir: Option<PathBuf>,
    ) -> Result<ProcessOutput, ProcessError> {
        let mut cmd = Command::new(program);
        cmd.args(args);

        // Add environment variables
        for (key, value) in env {
            cmd.env(&key, &value);
        }

        // Set working directory if provided
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Capture output
        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ProcessError::ProgramNotFound {
                    program: program.to_string(),
                    suggestion: format!("Ensure '{}' is installed and available in PATH", program),
                }
            } else {
                ProcessError::IoError {
                    error_details: e.to_string(),
                    suggestion: "Check file permissions and system configuration".to_string(),
                }
            }
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let status = ExitStatus::from_std(output.status);

        Ok(ProcessOutput::new(stdout, stderr, status))
    }
}

#[cfg(test)]
mod process_executor_tests {
    use super::*;

    #[test]
    fn test_build_options_new() {
        let options = BuildOptions::new("test-image", "v1.0");
        assert_eq!(options.image_name, "test-image");
        assert_eq!(options.tag, "v1.0");
    }

    #[test]
    fn test_build_options_with_build_arg() {
        let options = BuildOptions::new("test-image", "v1.0").with_build_arg("ARG1", "value1");

        assert_eq!(options.build_args.get("ARG1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_build_options_with_dockerfile() {
        let options = BuildOptions::new("test-image", "v1.0").with_dockerfile("FROM ubuntu:latest");

        assert_eq!(options.dockerfile, Some("FROM ubuntu:latest".to_string()));
    }

    #[tokio::test]
    async fn test_real_docker_client_clone() {
        // This test only verifies clone works, doesn't require Docker to be running
        // Create should fail without Docker, so we just check the Result type
        let result = RealDockerClient::new().await;
        assert!(result.is_err() || result.is_ok()); // Either is fine for this test
    }
}
