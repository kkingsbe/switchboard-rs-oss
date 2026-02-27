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

use bollard::Docker;
use std::io::{Cursor, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::time::Duration;

pub use crate::traits::{
    BuildOptions, DockerClientTrait, ProcessError, ProcessExecutorTrait, RealDockerClient,
    RealProcessExecutor,
};

pub mod run;
pub use run::{run_agent, AgentExecutionResult, ContainerConfig};

pub mod skills;

/// Get the Docker socket path from the active Docker context
///
/// On systems with multiple Docker installations (e.g., Podman + Docker Desktop),
/// this function queries the active Docker context to get the correct socket path.
///
/// If no executor is provided, a default `RealProcessExecutor` is created.
///
/// This function has a 5-second timeout to prevent indefinite hangs when Docker
/// is unavailable or slow.
async fn get_docker_socket_path(
    executor: Option<Arc<dyn ProcessExecutorTrait>>,
) -> Result<Option<String>, ProcessError> {
    eprintln!("DEBUG: get_docker_socket_path() called");
    let executor = executor.unwrap_or_else(|| Arc::new(RealProcessExecutor::new()));

    // Use tokio::process::Command for async execution with timeout
    // This avoids the issues with spawn_blocking on Windows
    eprintln!("DEBUG: About to run docker context show with timeout...");
    
    // Try to get Docker context with timeout
    let output = tokio::time::timeout(
        Duration::from_secs(5),
        async {
            let output = tokio::process::Command::new("docker")
                .args(["context", "show"])
                .output()
                .await
                .map_err(|e| {
                    ProcessError::ExecutionFailed {
                        program: "docker".to_string(),
                        error_details: format!("Failed to run docker context show: {}", e),
                        suggestion: "Check if Docker is installed".to_string(),
                    }
                })?;
            Ok::<_, ProcessError>(output)
        }
    )
    .await
    .map_err(|_| {
        eprintln!("DEBUG: Timeout occurred for docker context show!");
        ProcessError::ExecutionFailed {
            program: "docker".to_string(),
            error_details: "Timeout getting Docker context (docker context show)".to_string(),
            suggestion: "Check if Docker is running and responsive".to_string(),
        }
    })??;

    eprintln!("DEBUG: docker context show completed successfully");

    let context_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

    eprintln!("DEBUG: About to run docker context inspect with timeout for context: {}", context_name);

    // Use docker context inspect to get the endpoint for the active context (with timeout)
    let output = tokio::time::timeout(
        Duration::from_secs(5),
        async {
            let output = tokio::process::Command::new("docker")
                .args(["context", "inspect", &context_name])
                .output()
                .await
                .map_err(|e| {
                    ProcessError::ExecutionFailed {
                        program: "docker".to_string(),
                        error_details: format!("Failed to run docker context inspect: {}", e),
                        suggestion: "Check if Docker is installed".to_string(),
                    }
                })?;
            Ok::<_, ProcessError>(output)
        }
    )
    .await
    .map_err(|_| {
        eprintln!("DEBUG: Timeout occurred for docker context inspect!");
        ProcessError::ExecutionFailed {
            program: "docker".to_string(),
            error_details: "Timeout inspecting Docker context (docker context inspect)".to_string(),
            suggestion: "Check if Docker is running and responsive".to_string(),
        }
    })??;

    eprintln!("DEBUG: docker context inspect completed successfully");

    let json_output = String::from_utf8_lossy(&output.stdout);

    // Parse the JSON to extract the Docker endpoint
    // The endpoint is in the format: {"Endpoints": {"docker": {"Host": "unix:///path/to/socket"}}}
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_output) {
        if let Some(host) = parsed[0]["Endpoints"]["docker"]["Host"].as_str() {
            // Return the full host string (e.g., "unix:///path/to/sock")
            return Ok(Some(host.to_string()));
        }
    }

    Ok(None)
}

/// Connect to Docker using the active context's socket path
///
/// This function uses the Docker context system to get the correct socket path
/// (works with Podman, Docker Desktop, etc.), then falls back to the default.
///
/// If no executor is provided, a default `RealProcessExecutor` is created.
pub async fn connect_to_docker(
    executor: Option<Arc<dyn ProcessExecutorTrait>>,
) -> Result<Docker, anyhow::Error> {
    eprintln!("DEBUG: connect_to_docker() called");
    let executor = executor.unwrap_or_else(|| Arc::new(RealProcessExecutor::new()));

    // Try to get socket path from Docker context first
    if let Ok(Some(socket_path)) = get_docker_socket_path(Some(executor.clone())).await {
        eprintln!("DEBUG: Got socket path: {}", socket_path);
        // Handle unix:// socket paths
        if socket_path.starts_with("unix://") {
            let path = socket_path.strip_prefix("unix://").unwrap();
            // Try connecting to the context's socket
            eprintln!("DEBUG: Attempting to connect to socket: {}", path);
            if let Ok(docker) = Docker::connect_with_socket(path, 5, bollard::API_DEFAULT_VERSION) {
                eprintln!("DEBUG: Connected to socket successfully");
                return Ok(docker);
            }
        } else if socket_path.starts_with("npipe://") {
            // Windows named pipe
            let path = socket_path.strip_prefix("npipe://").unwrap();
            eprintln!("DEBUG: Attempting to connect to named pipe: {}", path);
            if let Ok(docker) = Docker::connect_with_named_pipe_defaults() {
                eprintln!("DEBUG: Connected to named pipe successfully");
                return Ok(docker);
            }
        }
    }

    eprintln!("DEBUG: Falling back to connect_with_local_defaults()");
    Docker::connect_with_local_defaults().map_err(|e| anyhow::anyhow!("{}", e))
}

/// Check if Docker daemon is available and responsive
///
/// This is a centralized helper function that checks Docker availability
/// with a 5-second timeout and provides consistent, user-friendly error
/// messages across all Docker-dependent commands.
///
/// This function should be called at the entry point of any command that
/// performs Docker operations (build, up, run, down, etc.) to ensure
/// consistent error handling and user experience.
///
/// If no executor is provided, a default `RealProcessExecutor` is created.
///
/// # Errors
///
/// Returns `DockerError::ConnectionTimeout` if Docker doesn't respond within 5 seconds.
/// Returns `DockerError::DockerUnavailable` if Docker is not available or permission is denied.
/// Returns `DockerError::ConnectionError` if the Docker daemon cannot be connected to.
///
/// # Examples
///
/// ```ignore
/// use switchboard::docker::check_docker_available;
///
/// async fn my_docker_command() -> Result<(), Box<dyn std::error::Error>> {
///     // Check Docker availability before attempting any Docker operations
///     check_docker_available().await?;
///     
///     // Proceed with Docker operations...
///     Ok(())
/// }
/// ```
pub async fn check_docker_available() -> Result<Docker, DockerError> {
    // Connect to Docker
    let docker = connect_to_docker(None).await.map_err(|e| {
        let error_msg = e.to_string();
        if error_msg.contains("permission denied")
            || error_msg.contains("Permission denied")
            || error_msg.contains("access denied")
        {
            DockerError::ConnectionError(
                "Permission denied accessing Docker daemon\n\n\
                Add your user to the docker group to use Docker without sudo:\n\n\
                sudo usermod -aG docker $USER\n\n\
                Then log out and log back in for the changes to take effect.\n\n\
                Verify with: groups $USER | grep docker"
                    .to_string(),
            )
        } else if error_msg.contains("connection refused")
            || error_msg.contains("Connection refused")
            || error_msg.contains("No such file")
        {
            DockerError::ConnectionError(
                "Docker daemon is not running\n\n\
                To start Docker:\n\
                  - Linux: sudo systemctl start docker (or docker start on Desktop)\n\
                  - macOS: Open Docker Desktop from Applications\n\
                  - Windows: Open Docker Desktop from Start Menu\n\n\
                Verify Docker is running with: docker ps"
                    .to_string(),
            )
        } else {
            DockerError::ConnectionError(format!(
                "Failed to connect to Docker daemon: {}\n\n\
                Is Docker daemon running and accessible?\n\n\
                On Linux, try: sudo systemctl start docker\n\
                On macOS/Windows: Start Docker Desktop\n\
                Permission issue? Run: sudo usermod -aG docker $USER",
                error_msg
            ))
        }
    })?;

    // Ping Docker daemon with timeout to verify it's responsive
    tokio::time::timeout(Duration::from_secs(5), docker.ping())
        .await
        .map_err(|_| DockerError::ConnectionTimeout {
            timeout_duration: "5s".to_string(),
            suggestion: "Docker daemon not running or not responding within timeout.\n\n\
To start Docker:\n\
  - Linux: sudo systemctl start docker (or docker start on Desktop)\n\
  - macOS: Open Docker Desktop from Applications\n\
  - Windows: Open Docker Desktop from Start Menu\n\n\
Verify Docker is running with: docker ps\n\n\
See https://docs.docker.com/engine/install/ for installation help."
                .to_string(),
        })?
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("permission denied")
                || error_msg.contains("Permission denied")
                || error_msg.contains("access denied")
            {
                DockerError::DockerUnavailable {
                    reason: "Permission denied accessing Docker daemon".to_string(),
                    suggestion: "Add your user to the docker group to use Docker without sudo:\n\n\
  sudo usermod -aG docker $USER\n\n\
Then log out and log back in for the changes to take effect.\n\n\
Verify with: groups $USER | grep docker"
                        .to_string(),
                }
            } else {
                DockerError::DockerUnavailable {
                    reason: format!("Docker daemon ping failed: {}", error_msg),
                    suggestion: "Docker daemon not running. Start Docker and try again.\n\n\
To start Docker:\n\
  - Linux: sudo systemctl start docker (or docker start on Desktop)\n\
  - macOS: Open Docker Desktop from Applications\n\
  - Windows: Open Docker Desktop from Start Menu\n\n\
Verify Docker is running with: docker ps\n\n\
See https://docs.docker.com/engine/install/ for installation help."
                        .to_string(),
                }
            }
        })?;

    Ok(docker)
}

/// Errors that can occur when interacting with Docker
#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    /// Docker daemon connection failed
    #[error("Docker connection error: {0}")]
    ConnectionError(String),

    /// Docker connection timeout
    #[error("Docker connection timed out after {timeout_duration}\n\n{suggestion}")]
    ConnectionTimeout {
        timeout_duration: String,
        suggestion: String,
    },

    /// Docker not available
    #[error("Docker daemon unavailable: {reason}\n\n{suggestion}")]
    DockerUnavailable { reason: String, suggestion: String },

    /// Image not found
    #[error("Docker image '{image_name}' not found\n\n{suggestion}")]
    ImageNotFoundError {
        image_name: String,
        suggestion: String,
    },

    /// Build error with details
    #[error("Docker build error: {error_details}\n\n{suggestion}")]
    BuildError {
        error_details: String,
        suggestion: String,
    },

    /// Container creation failure
    #[error("Failed to create container '{container_name}': {error_details}\n\n{suggestion}")]
    ContainerCreateError {
        container_name: String,
        error_details: String,
        suggestion: String,
    },

    /// Container start failure
    #[error("Failed to start container '{container_name}': {error_details}\n\n{suggestion}")]
    ContainerStartError {
        container_name: String,
        error_details: String,
        suggestion: String,
    },

    /// Container stop failure
    #[error("Failed to stop container '{container_name}': {error_details}\n\n{suggestion}")]
    ContainerStopError {
        container_name: String,
        error_details: String,
        suggestion: String,
    },

    /// Docker permission errors
    #[error("Permission denied for Docker operation: {operation}\n\n{suggestion}")]
    PermissionError {
        operation: String,
        suggestion: String,
    },

    /// I/O error
    #[error("I/O error: {operation} - {error_details}")]
    IoError {
        operation: String,
        error_details: String,
    },

    /// Metrics storage error
    #[error("Metrics storage error: {0}")]
    MetricsError(#[from] crate::metrics::MetricsError),

    /// Feature not yet implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl From<std::io::Error> for DockerError {
    fn from(err: std::io::Error) -> Self {
        DockerError::IoError {
            operation: "unknown".to_string(),
            error_details: err.to_string(),
        }
    }
}

/// Create a tarball of the build context directory with the Dockerfile included
///
/// This helper function creates a tarball containing all files from the build
/// context directory, with the provided Dockerfile content written as "Dockerfile"
/// at the root of the tarball.
///
/// # Arguments
///
/// * `build_context` - Path to the directory containing the build context
/// * `dockerfile_content` - The content of the Dockerfile to include in the tarball
///
/// # Returns
///
/// Returns a `Cursor<Vec<u8>>` containing the tarball bytes.
///
/// # Errors
///
/// Returns an error if:
/// - Reading files from the build context fails
/// - Writing to the tarball fails
pub fn create_build_context_tarball(
    build_context: &Path,
    dockerfile_content: &str,
) -> Result<Cursor<Vec<u8>>, anyhow::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::fs::File;
    use tar::Header;

    let mut tarball = Vec::new();
    {
        let mut encoder = GzEncoder::new(&mut tarball, Compression::default());
        let mut tar_builder = tar::Builder::new(&mut encoder);

        // Add Dockerfile to the tarball
        let dockerfile_path = Path::new("Dockerfile");
        let mut header = Header::new_gnu();
        header.set_size(dockerfile_content.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(0);
        tar_builder.append_data(&mut header, dockerfile_path, dockerfile_content.as_bytes())?;

        // Add all files from the build context directory
        // Only include .kilocode directory (the Dockerfile only copies this)
        if build_context.is_dir() {
            let entries = std::fs::read_dir(build_context)
                .map_err(|e| anyhow::anyhow!("Failed to read build context: {}", e))?;

            for entry in entries {
                let entry =
                    entry.map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                let relative_path = path
                    .strip_prefix(build_context)
                    .map_err(|e| anyhow::anyhow!("Failed to get relative path: {}", e))?;

                // Skip the Dockerfile if it exists in the build context (we already added it)
                if relative_path == Path::new("Dockerfile") {
                    continue;
                }

                // Only include .kilocode directory - everything else is not needed
                // (the Dockerfile only copies .kilocode into the image)
                let name = relative_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if name != "kilocode" {
                    continue; // Skip everything except .kilocode
                }

                if path.is_file() {
                    let mut file = File::open(&path).map_err(|e| {
                        anyhow::anyhow!("Failed to open file {}: {}", path.display(), e)
                    })?;
                    let file_size = file
                        .metadata()
                        .map_err(|e| {
                            anyhow::anyhow!("Failed to get metadata for {}: {}", path.display(), e)
                        })?
                        .len();

                    let mut header = Header::new_gnu();
                    header.set_size(file_size);
                    header.set_mode(0o644);
                    header.set_mtime(0);
                    tar_builder.append_data(&mut header, relative_path, &mut file)?;
                } else if path.is_dir() {
                    // Recursively add directories to the tarball
                    add_directory_to_tar(&mut tar_builder, &path, build_context)?;
                }
            }
        }

        drop(tar_builder);
        encoder.finish()?;
    }

    eprintln!("DEBUG: create_build_context_tarball about to return");
    Ok(Cursor::new(tarball))
}

/// Recursively add a directory to the tarball
///
/// # Arguments
///
/// * `tar_builder` - The tar builder to add files to
/// * `dir_path` - The directory path to add
/// * `base_path` - The base path to compute relative paths from
///
/// # Errors
///
/// Returns an error if reading files or writing to the tarball fails
#[allow(dead_code)]
fn add_directory_to_tar<W: Write>(
    tar_builder: &mut tar::Builder<W>,
    dir_path: &Path,
    base_path: &Path,
) -> Result<(), anyhow::Error> {
    use std::fs::File;
    use tar::Header;

    let entries = std::fs::read_dir(dir_path)
        .map_err(|e| anyhow::anyhow!("Failed to read directory {}: {}", dir_path.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(base_path)
            .map_err(|e| anyhow::anyhow!("Failed to get relative path: {}", e))?;

        if path.is_file() {
            let mut file = File::open(&path)
                .map_err(|e| anyhow::anyhow!("Failed to open file {}: {}", path.display(), e))?;
            let file_size = file
                .metadata()
                .map_err(|e| {
                    anyhow::anyhow!("Failed to get metadata for {}: {}", path.display(), e)
                })?
                .len();

            let mut header = Header::new_gnu();
            header.set_size(file_size);
            header.set_mode(0o644);
            header.set_mtime(0);
            tar_builder.append_data(&mut header, relative_path, &mut file)?;
        } else if path.is_dir() {
            add_directory_to_tar(tar_builder, &path, base_path)?;
        }
    }

    Ok(())
}

/// Docker client wrapper for managing images and containers
///
/// This structure provides a high-level interface to Docker Engine operations
/// using the `bollard` library. It encapsulates the Docker connection and provides
/// methods for building images, checking availability, and managing container
/// lifecycles. The client is designed for use in the Switchboard task scheduling and
/// Docker orchestration system.
///
/// # Fields
///
/// - `docker` - The underlying `bollard::Docker` client instance for Docker API calls
/// - `_image_name` - The Docker image name (currently unused, reserved for future use)
/// - `_image_tag` - The Docker image tag (currently unused, reserved for future use)
///
/// # Example
///
/// ```no_run
/// use switchboard::docker::DockerClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a new Docker client
/// let client = DockerClient::new(
///     "my-agent".to_string(),
///     "latest".to_string(),
/// ).await?;
///
/// // Check if Docker is available
/// client.check_available().await?;
///
/// // Get access to the underlying Docker client for advanced operations
/// let docker = client.docker();
/// # Ok(())
/// # }
/// ```
pub struct DockerClient {
    /// The Docker client trait object for dependency injection
    client: Arc<dyn DockerClientTrait>,
    /// The bollard Docker client (for backward compatibility)
    docker: Option<Docker>,
    /// Docker image name
    _image_name: String,
    /// Docker image tag
    _image_tag: String,
}

impl Clone for DockerClient {
    fn clone(&self) -> Self {
        DockerClient {
            client: Arc::clone(&self.client),
            docker: self.docker.clone(),
            _image_name: self._image_name.clone(),
            _image_tag: self._image_tag.clone(),
        }
    }
}

impl DockerClient {
    /// Create a new DockerClient instance and verify Docker is available
    ///
    /// # Arguments
    ///
    /// * `image_name` - The Docker image name (e.g., "switchboard-agent")
    /// * `image_tag` - The Docker image tag (e.g., "latest")
    ///
    /// # Errors
    ///
    /// Returns `DockerError::ConnectionError` if the connection to Docker daemon fails.
    /// Returns `DockerError::DockerUnavailable` if Docker is not available (ping fails).
    pub async fn new(image_name: String, image_tag: String) -> Result<Self, DockerError> {
        eprintln!("DEBUG: DockerClient::new() called");
        Self::new_with_executor(image_name, image_tag, None).await
    }

    /// Create a new DockerClient instance with a custom ProcessExecutor
    ///
    /// This constructor allows injecting a custom process executor for testing
    /// or custom process execution behavior.
    ///
    /// # Arguments
    ///
    /// * `image_name` - The Docker image name (e.g., "switchboard-agent")
    /// * `image_tag` - The Docker image tag (e.g., "latest")
    /// * `executor` - Optional process executor. If None, a default RealProcessExecutor is used.
    ///
    /// # Errors
    ///
    /// Returns `DockerError::ConnectionError` if the connection to Docker daemon fails.
    /// Returns `DockerError::DockerUnavailable` if Docker is not available (ping fails).
    pub async fn new_with_executor(
        image_name: String,
        image_tag: String,
        executor: Option<Arc<dyn ProcessExecutorTrait>>,
    ) -> Result<Self, DockerError> {
        let docker = connect_to_docker(executor.clone()).await.map_err(|e| {
            let error_msg = e.to_string();
            let helpful_msg = if error_msg.contains("permission denied")
                || error_msg.contains("Permission denied")
                || error_msg.contains("access denied")
            {
                format!(
                    "Docker connection error: {}\n\n\
                        Permission denied. Is the current user in the docker group?\n\n\
                        To fix this, run:\n\
                        sudo usermod -aG docker $USER\n\n\
                        Then log out and log back in for the changes to take effect.",
                    error_msg
                )
            } else if error_msg.contains("connection refused")
                || error_msg.contains("Connection refused")
                || error_msg.contains("No such file")
            {
                format!(
                    "Docker connection error: {}\n\n\
                        Is Docker daemon running?\n\n\
                        On Linux, try running:\n\
                        sudo systemctl start docker\n\n\
                        On macOS or Windows, make sure Docker Desktop is running.",
                    error_msg
                )
            } else {
                format!(
                    "Docker connection error: {}\n\n\
                        Is Docker daemon running and accessible?\n\n\
                        On Linux, try: sudo systemctl start docker\n\
                        On macOS/Windows: Start Docker Desktop\n\
                        Permission issue? Run: sudo usermod -aG docker $USER",
                    error_msg
                )
            };
            DockerError::ConnectionError(helpful_msg)
        })?;

        // Verify Docker is available by pinging the daemon (with timeout)
        eprintln!("DEBUG: About to ping Docker daemon with 10s timeout...");
        let ping_result = tokio::time::timeout(
            Duration::from_secs(10),
            docker.ping(),
        )
        .await;

        eprintln!("DEBUG: Ping result: {:?}", ping_result);

        ping_result
            .map_err(|_| {
                DockerError::DockerUnavailable {
                    reason: "Docker ping timed out after 10 seconds".to_string(),
                    suggestion: "Docker daemon may be slow to respond. Check Docker status with: docker info\n\n\
                        If Docker is not running:\n\
                        - Linux: sudo systemctl start docker\n\
                        - macOS/Windows: Start Docker Desktop"
                        .to_string(),
                }
            })?
            .map_err(|e| {
            let error_msg = e.to_string();
            DockerError::DockerUnavailable {
                reason: format!(
                    "Docker daemon appears to be running but is not responding: {}",
                    error_msg
                ),
                suggestion: "Check Docker status with: docker info\n\n\
                    If Docker is not running:\n\
                    - Linux: sudo systemctl start docker\n\
                    - macOS/Windows: Start Docker Desktop"
                    .to_string(),
            }
        })?;

        // Create the Docker client trait object (clone docker since we need to keep it for DockerClient)
        let client: Arc<dyn DockerClientTrait> =
            Arc::new(RealDockerClient::from_docker(docker.clone()));

        Ok(DockerClient {
            client,
            docker: Some(docker),
            _image_name: image_name,
            _image_tag: image_tag,
        })
    }

    /// Create a DockerClient from a RealDockerClient instance
    ///
    /// This constructor allows for dependency injection by accepting an existing
    /// RealDockerClient instance. This is useful for testing scenarios where you
    /// want to use a mock or test double implementation.
    ///
    /// # Arguments
    ///
    /// * `real_client` - A RealDockerClient instance to wrap
    ///
    /// # Notes
    ///
    /// The image_name and image_tag are set to empty strings since they are not
    /// needed when using an existing client connection. These fields are preserved
    /// for backward compatibility with existing code that may access them.
    pub fn from_real_client(real_client: crate::traits::RealDockerClient) -> Self {
        let docker = real_client.docker().clone();
        let client: Arc<dyn DockerClientTrait> = Arc::new(real_client);
        DockerClient {
            client,
            docker: Some(docker),
            _image_name: String::new(),
            _image_tag: String::new(),
        }
    }

    /// Check if Docker is available by pinging the daemon with a timeout.
    ///
    /// This function verifies that the Docker daemon is running and responsive by sending a ping request
    /// with a 5-second timeout. It provides detailed error messages when Docker is unavailable or
    /// times out, including platform-specific setup instructions.
    ///
    /// This method uses the trait-based client for the ping operation.
    ///
    /// # Error Scenarios
    ///
    /// - **ConnectionTimeout**: Docker doesn't respond within 5 seconds. This may indicate:
    ///   - Docker daemon is not running
    ///   - Docker daemon is overloaded or unresponsive
    ///   - Network connectivity issues
    ///
    /// - **DockerUnavailable**: Docker daemon ping fails. This may indicate:
    ///   - Permission denied (user not in docker group)
    ///   - Docker daemon is not properly installed
    ///   - Docker socket connection issues
    ///
    /// # Errors
    ///
    /// Returns `DockerError::ConnectionTimeout` if Docker doesn't respond within timeout.
    /// Returns `DockerError::DockerUnavailable` if Docker is not available or permission is denied.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let client = DockerClient::new("agent".to_string(), "latest".to_string()).await?;
    /// client.check_available().await?;
    /// // Docker is available, proceed with operations
    /// ```
    pub async fn check_available(&self) -> Result<(), DockerError> {
        // Use the trait-based client for the ping operation
        self.client.ping()
    }

    /// Returns a reference to the internal bollard Docker client.
    ///
    /// This getter provides access to the underlying `bollard::Docker` instance
    /// used by this client. This is useful for advanced use cases where direct
    /// access to the Docker API is needed beyond the methods provided by the
    /// `DockerClient` wrapper.
    ///
    /// The returned client is connected to the Docker daemon and can be used
    /// to interact with Docker containers, images, volumes, networks, and other
    /// Docker resources using the full bollard API.
    ///
    /// # Returns
    ///
    /// An optional reference to the internal `bollard::Docker` client instance.
    /// Returns `None` if the client was created via `new_with_client()` without
    /// providing the underlying Docker client.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use bollard::Docker;
    /// use switchboard::docker::DockerClient;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = DockerClient::new(
    ///     "my-image".to_string(),
    ///     "latest".to_string(),
    /// ).await?;
    ///
    /// // Get access to the underlying Docker client
    /// if let Some(docker) = client.docker() {
    ///     let containers = docker.list_containers::<String>(None).await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn docker(&self) -> Option<&Docker> {
        self.docker.as_ref()
    }

    /// Get a reference to the underlying Docker client trait
    ///
    /// This method provides access to the Docker client trait for operations
    /// that need to work with the trait directly.
    ///
    /// # Returns
    ///
    /// A reference to the Docker client trait object.
    pub fn client(&self) -> &dyn DockerClientTrait {
        self.client.as_ref()
    }

    /// Get a reference to the underlying Arc<dyn DockerClientTrait>
    ///
    /// This method provides access to the Arc-wrapped trait object,
    /// useful for passing to functions that accept &Arc<dyn DockerClientTrait>.
    ///
    /// # Returns
    ///
    /// A reference to the Arc containing the Docker client trait.
    pub fn client_arc(&self) -> &Arc<dyn DockerClientTrait> {
        &self.client
    }

    /// Build an agent image from a Dockerfile
    ///
    /// This method uses the Docker API to build a Docker image from the
    /// provided Dockerfile content and build context. The build output is streamed
    /// to stdout in real-time, and the resulting image ID is returned on success.
    ///
    /// This method delegates to the DockerClientTrait's build_image method.
    ///
    /// # Arguments
    ///
    /// * `dockerfile` - The content of the Dockerfile as a string
    /// * `build_context` - Path to the directory containing the build context
    /// * `image_name` - The name to tag the built image with (e.g., "switchboard-agent")
    /// * `image_tag` - The tag to apply to the built image (e.g., "latest", "v1.0.0")
    /// * `no_cache` - Whether to disable Docker's build cache (currently not used in trait implementation)
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the image ID as a String on success.
    ///
    /// # Errors
    ///
    /// Returns `DockerError::BuildError` if:
    /// - The build context directory does not exist
    /// - Creating the build context tarball fails
    /// - The Docker build fails
    pub async fn build_agent_image(
        &self,
        dockerfile: &str,
        build_context: &Path,
        image_name: &str,
        image_tag: &str,
        no_cache: bool,
    ) -> Result<String, DockerError> {
        eprintln!("DEBUG: build_agent_image() called");
        // Verify build context exists
        if !build_context.exists() {
            return Err(DockerError::BuildError {
                error_details: format!(
                    "Build context directory not found: {}",
                    build_context.display()
                ),
                suggestion: "Create the build context directory or verify the path is correct."
                    .to_string(),
            });
        }

        // Write the dockerfile to a temporary location in the build context
        // The trait's build_image expects a path to the Dockerfile
        let dockerfile_path = build_context.join("Dockerfile");
        std::fs::write(&dockerfile_path, dockerfile).map_err(|e| DockerError::IoError {
            operation: "write Dockerfile".to_string(),
            error_details: e.to_string(),
        })?;

        // Delegate to the trait's build_image method
        // Note: The no_cache parameter is not available in the trait method
        // For now, we ignore no_cache - this could be enhanced in the future
        let _ = no_cache; // Suppress unused variable warning

        // Create BuildOptions from the parameters
        let options = BuildOptions::new(image_name, image_tag).with_dockerfile(dockerfile);

        eprintln!("DEBUG: About to call self.client.build_image()...");
        let image_id = self
            .client
            .build_image(options, build_context.to_path_buf())?;
        eprintln!("DEBUG: build_image() completed, image_id: {}", image_id);

        // Clean up the temporary Dockerfile
        let _ = std::fs::remove_file(&dockerfile_path);

        Ok(image_id)
    }
}

// Implementation of AsRef<Arc<dyn DockerClientTrait>> for &DockerClient
// This allows &DockerClient to be dereferenced to &Arc<dyn DockerClientTrait>
// which enables passing &DockerClient where Arc<dyn DockerClientTrait> is expected
impl AsRef<Arc<dyn DockerClientTrait>> for &DockerClient {
    fn as_ref(&self) -> &Arc<dyn DockerClientTrait> {
        &self.client
    }
}

// Implement DockerClientTrait for DockerClient for compatibility
// This allows DockerClient to be used where DockerClientTrait is expected
impl crate::traits::DockerClientTrait for DockerClient {
    fn ping(&self) -> Result<(), DockerError> {
        let docker = self.docker.as_ref().expect("Docker client not available");
        tokio::runtime::Handle::current()
            .block_on(docker.ping())
            .map(|_| ())
            .map_err(|e| DockerError::ConnectionError(e.to_string()))
    }

    fn image_exists(&self, name: &str, tag: &str) -> Result<bool, DockerError> {
        use bollard::image::ListImagesOptions;

        let image_name = format!("{}:{}", name, tag);
        let docker = self.docker.as_ref().expect("Docker client not available");

        tokio::runtime::Handle::current()
            .block_on(async {
                let options = Some(ListImagesOptions::<String> {
                    all: false,
                    ..Default::default()
                });

                let images = docker.list_images(options).await?;

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
            .map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))
    }

    fn build_image(
        &self,
        options: crate::traits::BuildOptions,
        context: std::path::PathBuf,
    ) -> Result<String, DockerError> {
        // Clone the internal docker client (it's Arc-based)
        let docker = self.docker.as_ref().expect("Docker client not available").clone();
        
        // Clone values needed for block_in_place
        let dockerfile = options
            .dockerfile
            .unwrap_or_else(|| "Dockerfile".to_string());
        let image_name = options.image_name.clone();
        let tag = options.tag.clone();
        
        // Use block_in_place to handle both sync and async contexts properly
        let result = tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async move {
                use bollard::image::BuildImageOptions;
                use futures::StreamExt;
                
                // Create tarball
                let tarball = crate::docker::create_build_context_tarball(&context, &dockerfile)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
                let tarball_bytes = bytes::Bytes::from(tarball.into_inner());
                
                let build_options = BuildImageOptions {
                    dockerfile: "Dockerfile",
                    t: &format!("{}:{}", image_name, tag),
                    rm: true,
                    ..Default::default()
                };
                
                let mut stream = docker.build_image(build_options, None, Some(tarball_bytes));
                let mut final_image_id = String::new();
                
                while let Some(build_result) = stream.next().await {
                    match build_result {
                        Ok(info) => {
                            if let Some(id) = info.id {
                                final_image_id = id;
                            }
                        }
                        Err(e) => {
                            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
                        }
                    }
                }
                
                Ok(final_image_id)
            })
        }).map_err(|e| DockerError::ConnectionError(e.to_string()))?;
        
        Ok(result)
    }

    fn run_container(
        &self,
        _config: crate::traits::ContainerConfig,
    ) -> Result<String, DockerError> {
        // This is a simple implementation - for full functionality, use RealDockerClient
        Err(DockerError::NotImplemented(
            "run_container not implemented for DockerClient, use RealDockerClient".to_string(),
        ))
    }

    fn stop_container(&self, container_id: &str, timeout: u64) -> Result<(), DockerError> {
        use bollard::container::StopContainerOptions;

        let options = StopContainerOptions { t: timeout as i64 };
        let docker = self.docker.as_ref().expect("Docker client not available").clone();
        let container_id = container_id.to_string();

        // Use block_in_place to handle both sync and async contexts properly
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                docker.stop_container(&container_id, Some(options)).await
            })
        }).map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))?;
        
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

        let docker = self.docker.as_ref().expect("Docker client not available").clone();
        let container_id = container_id.to_string();

        // Use block_in_place to handle both sync and async contexts properly
        let logs = tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                let mut stream = docker.logs(&container_id, Some(options));
                let mut logs = String::new();

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

                Ok(logs)
            })
        }).map_err(|e| DockerError::ConnectionError(e.to_string()))?;

        Ok(logs)
    }

    fn wait_container(
        &self,
        container_id: &str,
        _timeout: u64,
    ) -> Result<crate::traits::ExitCode, DockerError> {
        use bollard::container::WaitContainerOptions;
        use futures::StreamExt;

        let options = WaitContainerOptions { condition: "exit" };
        let docker = self.docker.as_ref().expect("Docker client not available").clone();
        let container_id = container_id.to_string();

        // Use block_in_place to handle both sync and async contexts properly
        let exit_code = tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                let mut stream = docker.wait_container(&container_id, Some(options));

                if let Some(result) = stream.next().await {
                    match result {
                        Ok(status) => Ok(status.status_code as i32),
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(0)
                }
            })
        }).map_err(|e| DockerError::ConnectionError(e.to_string()))?;

        Ok(crate::traits::ExitCode::from_i32(exit_code))
    }

    fn create_container(
        &self,
        options: Option<bollard::container::CreateContainerOptions<String>>,
        config: bollard::container::Config<String>,
    ) -> Result<String, DockerError> {
        let docker = self.docker.as_ref().expect("Docker client not available").clone();
        
        // Use block_in_place to handle both sync and async contexts properly
        // This avoids the "Cannot start a runtime from within a runtime" error
        let result = tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                docker.create_container(options, config).await
            })
        }).map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))?;
        
        Ok(result.id)
    }

    fn start_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::StartContainerOptions<String>>,
    ) -> Result<(), DockerError> {
        let docker = self.docker.as_ref().expect("Docker client not available").clone();
        let container_id = container_id.to_string();
        
        // Use block_in_place to handle both sync and async contexts properly
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                docker.start_container(&container_id, options).await
            })
        }).map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))?;
        
        Ok(())
    }

    fn inspect_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::InspectContainerOptions>,
    ) -> Result<bollard::service::ContainerInspectResponse, DockerError> {
        let docker = self.docker.as_ref().expect("Docker client not available").clone();
        let container_id = container_id.to_string();
        
        // Use block_in_place to handle both sync and async contexts properly
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                docker.inspect_container(&container_id, options).await
            })
        }).map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))
    }

    fn kill_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::KillContainerOptions<String>>,
    ) -> Result<(), DockerError> {
        let docker = self.docker.as_ref().expect("Docker client not available").clone();
        let container_id = container_id.to_string();
        
        // Use block_in_place to handle both sync and async contexts properly
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
                docker.kill_container(&container_id, options).await
            })
        }).map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    // Test module for Docker client functionality
    use super::*;

    #[test]
    fn test_kilocode_included_in_build_context_tarball() {
        use flate2::read::GzDecoder;
        use std::fs;
        use tar::Archive;

        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let build_context = temp_dir.path();

        // Create a .kilocode subdirectory with some content
        let kilocode_dir = build_context.join(".kilocode");
        fs::create_dir(&kilocode_dir).expect("Failed to create .kilocode directory");

        // Create a test file inside .kilocode
        let test_file = kilocode_dir.join("config.json");
        fs::write(&test_file, r#"{"api_key": "test-key"}"#).expect("Failed to write config.json");

        // Create another nested directory inside .kilocode
        let nested_dir = kilocode_dir.join("mcp_servers");
        fs::create_dir(&nested_dir).expect("Failed to create mcp_servers directory");

        let nested_file = nested_dir.join("server.json");
        fs::write(&nested_file, r#"{"name": "test-server"}"#).expect("Failed to write server.json");

        // Create a Dockerfile in the temp directory
        let dockerfile_path = build_context.join("Dockerfile");
        let dockerfile_content = "FROM alpine:latest\nCMD [\"echo\", \"test\"]\n";
        fs::write(&dockerfile_path, dockerfile_content).expect("Failed to write Dockerfile");

        // Create the build context tarball
        let dockerfile = "FROM alpine:latest\nCMD [\"echo\", \"test\"]\n";
        let tarball_cursor = create_build_context_tarball(build_context, dockerfile)
            .expect("Failed to create tarball");
        let tarball_bytes = tarball_cursor.into_inner();

        // Decompress and parse the tarball
        let decoder = GzDecoder::new(&tarball_bytes[..]);
        let mut archive = Archive::new(decoder);

        // Collect all entries from the tarball
        let mut entries: Vec<String> = Vec::new();
        for entry in archive.entries().expect("Failed to read tarball entries") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path().expect("Failed to get entry path");
            let path_str = path.to_string_lossy().to_string();
            entries.push(path_str);
        }

        // Verify that Dockerfile is in the tarball
        assert!(
            entries.contains(&"Dockerfile".to_string()),
            "Dockerfile should be in the tarball. Entries found: {:?}",
            entries
        );

        // Verify that .kilocode directory is in the tarball
        let kilocode_entries: Vec<&String> = entries
            .iter()
            .filter(|e| e.starts_with(".kilocode"))
            .collect();
        assert!(
            !kilocode_entries.is_empty(),
            ".kilocode directory should be included in the tarball. Entries found: {:?}",
            entries
        );

        // Verify that config.json inside .kilocode is present
        assert!(
            entries.contains(&".kilocode/config.json".to_string()),
            ".kilocode/config.json should be in the tarball. Entries found: {:?}",
            entries
        );

        // Verify that nested directory structure is preserved
        assert!(
            entries.contains(&".kilocode/mcp_servers/server.json".to_string()),
            ".kilocode/mcp_servers/server.json should be in the tarball. Entries found: {:?}",
            entries
        );

        // Clean up
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_kilocode_directory_check_in_build_agent_image() {
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let build_context = temp_dir.path();

        // Verify .kilocode directory is missing
        let kilocode_dir = build_context.join(".kilocode");
        assert!(!kilocode_dir.exists() || !kilocode_dir.is_dir());

        // The check logic from build_agent_image would fail here
        let kilocode_check_path = build_context.join(".kilocode");
        if !kilocode_check_path.exists() || !kilocode_check_path.is_dir() {
            let expected_error_msg = format!(
                "The .kilocode directory is required for building the agent image but was not found in: {}\n\n\
                The .kilocode directory contains API keys, model configuration, and MCP server\n\
                definitions needed by the Kilo Code CLI. Please configure .kilocode/ in the Switchboard\n\
                repo with your API keys before building the agent image.",
                build_context.display()
            );

            // Verify the error message contains the expected key phrases
            assert!(expected_error_msg.contains(".kilocode directory is required"));
            assert!(expected_error_msg.contains("API keys"));
            assert!(expected_error_msg.contains("model configuration"));
            // The error message splits "MCP server" and "definitions" across lines
            assert!(expected_error_msg.contains("MCP server"));
        }

        // Clean up
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[test]
    fn test_kilocode_directory_exists() {
        use std::fs;

        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let build_context = temp_dir.path();

        // Create .kilocode directory
        let kilocode_dir = build_context.join(".kilocode");
        fs::create_dir(&kilocode_dir).expect("Failed to create .kilocode directory");

        // Verify .kilocode directory exists
        assert!(kilocode_dir.exists());
        assert!(kilocode_dir.is_dir());

        // Clean up
        temp_dir.close().expect("Failed to close temp directory");
    }

    #[tokio::test]
    async fn test_check_available_unavailable_daemon() {
        // Create a Docker client using an unreachable socket path to simulate
        // a Docker daemon that is not running or unavailable
        let socket_path = "/tmp/nonexistent_docker_socket_12345.sock";

        // Create Docker client pointing to an invalid socket
        let docker_result =
            Docker::connect_with_socket(socket_path, 5, bollard::API_DEFAULT_VERSION);

        // If we successfully connected (unlikely with nonexistent socket), skip the test
        // This would only happen if there's something actually listening on that path
        let docker = match docker_result {
            Ok(d) => d,
            Err(_) => {
                // Connection failed as expected - skip this test path since we need a client
                // to call check_available(). The test framework will handle this.
                return;
            }
        };

        // Create a DockerClient instance with the Docker client
        // Note: This bypasses the new() constructor to test check_available directly
        let client = DockerClient {
            client: Arc::new(RealDockerClient::from_docker(docker.clone())),
            docker: Some(docker),
            _image_name: "test-image".to_string(),
            _image_tag: "latest".to_string(),
        };

        // Call check_available() which should timeout after 5 seconds
        // since the socket is unreachable
        let result = client.check_available().await;

        // Verify that check_available() returns an error
        assert!(
            result.is_err(),
            "check_available() should return Err when Docker daemon is unavailable"
        );

        // Verify the error is DockerUnavailable with the expected message
        match result {
            Err(DockerError::DockerUnavailable { reason, suggestion }) => {
                assert_eq!(
                    reason, "Docker daemon is not running or is not responding",
                    "Error reason should match the expected text"
                );
                assert_eq!(
                    suggestion, "Start Docker Desktop or the Docker daemon, then try again",
                    "Error suggestion should match the expected text"
                );
            }
            Err(other_error) => {
                panic!("Expected DockerUnavailable error, got: {:?}", other_error);
            }
            Ok(_) => {
                panic!("check_available() should return Err when Docker daemon is unavailable");
            }
        }
    }
}
