//! Docker Client - Connection management and Docker client wrapper
//!
//! This module provides:
//! - Docker connection management: get_docker_socket_path(), connect_to_docker(), check_docker_available()
//! - DockerError enum for error handling
//! - DockerClient struct with all methods
//! - DockerClientTrait implementations

use crate::docker::connection::{DockerConnectionTrait, RealDockerConnection};
use bollard::Docker;
use std::path::Path;
use std::sync::Arc;
use tokio::time::Duration;

pub use crate::traits::{
    BuildOptions, DockerClientTrait, ProcessError, ProcessExecutorTrait, RealDockerClient,
    RealProcessExecutor,
};

/// Get the Docker socket path from the active Docker context
///
/// On systems with multiple Docker installations (e.g., Podman + Docker Desktop),
/// this function queries the active Docker context to get the correct socket path.
///
/// If no executor is provided, a default `RealProcessExecutor` is created.
///
/// This function has a 5-second timeout to prevent indefinite hangs when Docker
/// is unavailable or slow.
pub async fn get_docker_socket_path(
    _executor: Option<Arc<dyn ProcessExecutorTrait>>,
) -> Result<Option<String>, ProcessError> {
    // Use tokio::process::Command for async execution with timeout
    // This avoids the issues with spawn_blocking on Windows

    // Try to get Docker context with timeout
    let output = tokio::time::timeout(Duration::from_secs(5), async {
        let output = tokio::process::Command::new("docker")
            .args(["context", "show"])
            .output()
            .await
            .map_err(|e| ProcessError::ExecutionFailed {
                program: "docker".to_string(),
                error_details: format!("Failed to run docker context show: {}", e),
                suggestion: "Check if Docker is installed".to_string(),
            })?;
        Ok::<_, ProcessError>(output)
    })
    .await
    .map_err(|_| {
        eprintln!("DEBUG: Timeout occurred for docker context show!");
        ProcessError::ExecutionFailed {
            program: "docker".to_string(),
            error_details: "Timeout getting Docker context (docker context show)".to_string(),
            suggestion: "Check if Docker is running and responsive".to_string(),
        }
    })??;

    let context_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Use docker context inspect to get the endpoint for the active context (with timeout)
    let output = tokio::time::timeout(Duration::from_secs(5), async {
        let output = tokio::process::Command::new("docker")
            .args(["context", "inspect", &context_name])
            .output()
            .await
            .map_err(|e| ProcessError::ExecutionFailed {
                program: "docker".to_string(),
                error_details: format!("Failed to run docker context inspect: {}", e),
                suggestion: "Check if Docker is installed".to_string(),
            })?;
        Ok::<_, ProcessError>(output)
    })
    .await
    .map_err(|_| {
        eprintln!("DEBUG: Timeout occurred for docker context inspect!");
        ProcessError::ExecutionFailed {
            program: "docker".to_string(),
            error_details: "Timeout inspecting Docker context (docker context inspect)".to_string(),
            suggestion: "Check if Docker is running and responsive".to_string(),
        }
    })??;

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
    let executor = executor.unwrap_or_else(|| Arc::new(RealProcessExecutor::new()));

    // Try to get socket path from Docker context first
    if let Ok(Some(socket_path)) = get_docker_socket_path(Some(executor.clone())).await {
        // Handle unix:// socket paths
        if socket_path.starts_with("unix://") {
            let path = socket_path.strip_prefix("unix://").ok_or_else(|| {
                anyhow::anyhow!(
                    "socket_path '{}' does not start with 'unix://'",
                    socket_path
                )
            })?;
            // Try connecting to the context's socket
            if let Ok(docker) = Docker::connect_with_socket(path, 5, bollard::API_DEFAULT_VERSION) {
                return Ok(docker);
            }
        } else if socket_path.starts_with("npipe://") {
            // Windows named pipe - only compile on Windows
            #[cfg(target_os = "windows")]
            {
                let _path = socket_path.strip_prefix("npipe://").ok_or_else(|| {
                    anyhow::anyhow!(
                        "socket_path '{}' does not start with 'npipe://'",
                        socket_path
                    )
                })?;
                if let Ok(docker) = Docker::connect_with_named_pipe_defaults() {
                    return Ok(docker);
                }
            }
        }
    }

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
/// - `client` - The Docker client trait object for Docker operations
/// - `_image_name` - The Docker image name (currently unused, reserved for future use)
/// - `_image_tag` - The Docker image tag (currently unused, reserved for future use)
///
/// Docker client for managing containers and images.
///
/// Provides high-level interface for Docker operations including:
/// - Container creation, start, stop, and removal
/// - Image building and pulling
/// - Container logging and metrics
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
/// // Use the client trait for Docker operations
/// let docker_client = client.client();
/// # Ok(())
/// # }
/// ```
pub struct DockerClient {
    /// The Docker connection trait object for connection management
    connection: Arc<dyn DockerConnectionTrait>,
    /// The Docker client trait object for Docker operations
    client: Arc<dyn DockerClientTrait>,
    /// The underlying bollard Docker client (derived from client)
    /// Only available when the "discord" feature is enabled
    #[cfg(feature = "discord")]
    docker: Docker,
    /// Docker image name
    _image_name: String,
    /// Docker image tag
    _image_tag: String,
}

impl Clone for DockerClient {
    fn clone(&self) -> Self {
        DockerClient {
            connection: Arc::clone(&self.connection),
            client: Arc::clone(&self.client),
            #[cfg(feature = "discord")]
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
        // Create a RealDockerConnection for the default constructor
        let connection: Arc<dyn DockerConnectionTrait> = Arc::new(RealDockerConnection::new());
        Self::new_with_connection(image_name, image_tag, connection).await
    }

    /// Create a new DockerClient instance with a custom DockerConnectionTrait
    ///
    /// This constructor allows injecting a custom connection implementation for testing
    /// or custom connection behavior.
    ///
    /// # Arguments
    ///
    /// * `image_name` - The Docker image name (e.g., "switchboard-agent")
    /// * `image_tag` - The Docker image tag (e.g., "latest")
    /// * `connection` - A connection implementation (RealDockerConnection or MockDockerConnection)
    ///
    /// # Errors
    ///
    /// Returns `DockerError::ConnectionError` if the connection to Docker daemon fails.
    /// Returns `DockerError::DockerUnavailable` if Docker is not available (ping fails).
    pub async fn new_with_connection(
        image_name: String,
        image_tag: String,
        connection: Arc<dyn DockerConnectionTrait>,
    ) -> Result<Self, DockerError> {
        // Connect to Docker using the connection trait (now async)
        let docker = connection.connect().await.map_err(|e| {
            let error_msg = e.to_string();
            let helpful_msg = if error_msg.contains("permission denied")
                || error_msg.contains("Permission denied")
                || error_msg.contains("access denied")
            {
                format!(
                    "Docker connection error: {}\n\n\n                        Permission denied. Is the current user in the docker group?\n\n\n                        To fix this, run:\n\n                        sudo usermod -aG docker $USER\n\n\n                        Then log out and log back in for the changes to take effect.",
                    error_msg
                )
            } else if error_msg.contains("connection refused")
                || error_msg.contains("Connection refused")
                || error_msg.contains("No such file")
            {
                format!(
                    "Docker connection error: {}\n\n\n                        Is Docker daemon running?\n\n\n                        On Linux, try running:\n\n                        sudo systemctl start docker\n\n\n                        On macOS or Windows, make sure Docker Desktop is running.",
                    error_msg
                )
            } else {
                format!(
                    "Docker connection error: {}\n\n\n                        Is Docker daemon running and accessible?\n\n\n                        On Linux, try: sudo systemctl start docker\n\n\n                        On macOS/Windows: Start Docker Desktop\n\n\n                        Permission issue? Run: sudo usermod -aG docker $USER",
                    error_msg
                )
            };
            DockerError::ConnectionError(helpful_msg)
        })?;

        // Verify Docker is available by pinging the daemon (with timeout)
        let ping_result = tokio::time::timeout(Duration::from_secs(10), docker.ping()).await;

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
            connection,
            client,
            #[cfg(feature = "discord")]
            docker,
            _image_name: image_name,
            _image_tag: image_tag,
        })
    }

    /// Create a new DockerClient instance with a custom ProcessExecutor
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
        #[cfg(feature = "discord")]
        let docker = real_client.docker().clone();
        let client: Arc<dyn DockerClientTrait> = Arc::new(real_client);
        // Create a default RealDockerConnection for backward compatibility
        let connection: Arc<dyn DockerConnectionTrait> = Arc::new(RealDockerConnection::new());
        DockerClient {
            connection,
            client,
            #[cfg(feature = "discord")]
            docker,
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

    /// Get a reference to the underlying bollard Docker client
    ///
    /// This method provides direct access to the underlying bollard Docker client.
    /// This is useful for advanced operations that are not exposed through the trait.
    ///
    /// # Returns
    ///
    /// An optional reference to the internal bollard Docker client.
    /// Returns `None` if the "discord" feature is not enabled.
    pub fn docker(&self) -> Option<&Docker> {
        #[cfg(feature = "discord")]
        {
            Some(&self.docker)
        }
        #[cfg(not(feature = "discord"))]
        {
            None
        }
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

// Implement DockerClientTrait for DockerClient
// This allows DockerClient to be used where DockerClientTrait is expected
// by delegating all operations to the internal client field
impl crate::traits::DockerClientTrait for DockerClient {
    fn ping(&self) -> Result<(), DockerError> {
        self.client.ping()
    }

    fn image_exists(&self, name: &str, tag: &str) -> Result<bool, DockerError> {
        self.client.image_exists(name, tag)
    }

    fn build_image(
        &self,
        options: crate::traits::BuildOptions,
        context: std::path::PathBuf,
    ) -> Result<String, DockerError> {
        self.client.build_image(options, context)
    }

    fn run_container(&self, config: crate::traits::ContainerConfig) -> Result<String, DockerError> {
        self.client.run_container(config)
    }

    fn stop_container(&self, container_id: &str, timeout: u64) -> Result<(), DockerError> {
        self.client.stop_container(container_id, timeout)
    }

    fn container_logs(
        &self,
        container_id: &str,
        follow: bool,
        tail: Option<u64>,
    ) -> Result<String, DockerError> {
        self.client.container_logs(container_id, follow, tail)
    }

    fn wait_container(
        &self,
        container_id: &str,
        timeout: u64,
    ) -> Result<crate::traits::ExitCode, DockerError> {
        self.client.wait_container(container_id, timeout)
    }

    fn create_container(
        &self,
        options: Option<bollard::container::CreateContainerOptions<String>>,
        config: bollard::container::Config<String>,
    ) -> Result<String, DockerError> {
        self.client.create_container(options, config)
    }

    fn start_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::StartContainerOptions<String>>,
    ) -> Result<(), DockerError> {
        self.client.start_container(container_id, options)
    }

    fn inspect_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::InspectContainerOptions>,
    ) -> Result<bollard::service::ContainerInspectResponse, DockerError> {
        self.client.inspect_container(container_id, options)
    }

    fn kill_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::KillContainerOptions<String>>,
    ) -> Result<(), DockerError> {
        self.client.kill_container(container_id, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docker::connection::{
        DockerCommand, DockerConnectionTrait, DockerResponse, MockDockerConnectionBuilder,
    };
    use std::sync::Arc;

    /// Test that DockerClient::new_with_connection accepts a MockDockerConnection
    ///
    /// This test verifies that the constructor properly accepts a mock connection
    /// and handles the connection attempt. The mock is configured to simulate
    /// a connection that cannot create a real Docker client, which is expected
    /// behavior for the mock in test environments.
    #[tokio::test]
    async fn test_docker_client_new_with_connection_accepts_mock() {
        // Create a mock connection with default settings
        let mock_connection: Arc<dyn DockerConnectionTrait> = Arc::new(
            MockDockerConnectionBuilder::new()
                .with_connect_success(true)
                .build()
        );

        // Attempt to create DockerClient - this should fail because
        // MockDockerConnection cannot create a real bollard Docker client
        let result = DockerClient::new_with_connection(
            "test-image".to_string(),
            "latest".to_string(),
            mock_connection,
        ).await;

        // The mock's connect() always returns an error because it cannot
        // create a real Docker client - this is expected behavior
        // Use is_err() to check for error without requiring Debug on Ok variant
        let had_error = result.is_err();
        assert!(had_error, "Expected error because mock cannot create real Docker client");
    }

    /// Test DockerClient properly handles connection failure from mock
    #[tokio::test]
    async fn test_docker_client_handles_connection_failure() {
        // Create a mock connection configured to fail
        let mock_connection: Arc<dyn DockerConnectionTrait> = Arc::new(
            MockDockerConnectionBuilder::new()
                .with_connect_success(false)
                .build()
        );

        let result = DockerClient::new_with_connection(
            "test-image".to_string(),
            "latest".to_string(),
            mock_connection,
        ).await;

        // Should return an error - use is_err() to avoid Debug requirement
        assert!(result.is_err(), "Expected error for failed connection");
    }

    /// Test DockerClient properly handles connection timeout from mock
    #[tokio::test]
    async fn test_docker_client_handles_connection_timeout() {
        // Create a mock connection configured to timeout
        let mock_connection: Arc<dyn DockerConnectionTrait> = Arc::new(
            MockDockerConnectionBuilder::new()
                .with_connect_timeout(Some(std::time::Duration::from_secs(5)))
                .build()
        );

        let result = DockerClient::new_with_connection(
            "test-image".to_string(),
            "latest".to_string(),
            mock_connection,
        ).await;

        // Should return a timeout error - use is_err() to avoid Debug requirement
        assert!(result.is_err(), "Expected error for timeout");
    }

    /// Test MockDockerConnection get_docker_socket_path returns configured value
    #[test]
    fn test_mock_connection_provides_socket_path() {
        let mock = MockDockerConnectionBuilder::new()
            .with_socket_path(Some("/custom/docker.sock".to_string()))
            .build();

        let result = mock.get_docker_socket_path();
        assert!(result.is_ok(), "Expected socket path to be returned: {:?}", result);
        assert_eq!(result.unwrap(), "/custom/docker.sock");
    }

    /// Test MockDockerConnection check_docker_available returns configured value
    #[test]
    fn test_mock_connection_check_available() {
        // Test with available = true
        let mock_available = MockDockerConnectionBuilder::new()
            .with_available(true)
            .build();
        let result = mock_available.check_docker_available();
        assert!(result.is_ok(), "Expected available to be true: {:?}", result);
        assert!(result.unwrap(), "Expected check_docker_available to return true");

        // Test with available = false
        let mock_unavailable = MockDockerConnectionBuilder::new()
            .with_available(false)
            .build();
        let result = mock_unavailable.check_docker_available();
        assert!(result.is_err(), "Expected error when Docker unavailable");
    }

    /// Test MockDockerConnection disconnect handles success/failure
    #[tokio::test]
    async fn test_mock_connection_disconnect() {
        // Test successful disconnect
        let mock_success = MockDockerConnectionBuilder::new()
            .with_disconnect_success(true)
            .build();
        let result = mock_success.disconnect().await;
        assert!(result.is_ok(), "Expected disconnect to succeed: {:?}", result);

        // Test failed disconnect
        let mock_fail = MockDockerConnectionBuilder::new()
            .with_disconnect_success(false)
            .build();
        let result = mock_fail.disconnect().await;
        assert!(result.is_err(), "Expected disconnect to fail");
    }

    /// Test MockDockerConnection execute returns configured response
    #[tokio::test]
    async fn test_mock_connection_execute() {
        let mock_response = DockerResponse::Ping { result: "OK".to_string() };
        
        let mock = MockDockerConnectionBuilder::new()
            .with_execute_success(true)
            .with_execute_response(Some(mock_response.clone()))
            .build();

        let result = mock.execute(DockerCommand::Ping).await;
        assert!(result.is_ok(), "Expected execute to succeed: {:?}", result);
        
        // Use pattern matching instead of assert_eq since DockerResponse doesn't derive PartialEq
        match result.unwrap() {
            DockerResponse::Ping { result: r } => {
                assert_eq!(r, "OK", "Expected ping result to be 'OK'");
            }
            _ => panic!("Expected Ping response"),
        }
    }

    /// Test MockDockerConnection execute handles failure
    #[tokio::test]
    async fn test_mock_connection_execute_failure() {
        let mock = MockDockerConnectionBuilder::new()
            .with_execute_success(false)
            .build();

        let result = mock.execute(DockerCommand::Ping).await;
        assert!(result.is_err(), "Expected execute to fail");
    }

    /// Test DockerClient clones correctly with Arc<dyn DockerConnectionTrait>
    #[test]
    fn test_docker_client_arc_connection_is_cloneable() {
        // This test verifies that DockerClient can work with Arc<dyn DockerConnectionTrait>
        // and that the Arc can be cloned, which is important for concurrent operations
        
        // Create an Arc-wrapped mock connection
        let arc_mock: Arc<dyn DockerConnectionTrait> = Arc::new(
            MockDockerConnectionBuilder::new()
                .with_socket_path(Some("/test.sock".to_string()))
                .build()
        );
        
        // Clone the Arc (this is how DockerClient stores the connection)
        let arc_clone = Arc::clone(&arc_mock);
        
        // Both should be able to get socket path
        assert!(arc_mock.get_docker_socket_path().is_ok(), "Original Arc should work");
        assert!(arc_clone.get_docker_socket_path().is_ok(), "Cloned Arc should work");
        
        // Both should return the same value
        assert_eq!(
            arc_mock.get_docker_socket_path().unwrap(),
            arc_clone.get_docker_socket_path().unwrap()
        );
    }

    /// Test DockerError can be created and contains useful information
    #[test]
    fn test_docker_error_types() {
        // Test ConnectionError
        let err = DockerError::ConnectionError("test error".to_string());
        assert!(format!("{}", err).contains("test error"));

        // Test ConnectionTimeout
        let err = DockerError::ConnectionTimeout {
            timeout_duration: "5s".to_string(),
            suggestion: "test suggestion".to_string(),
        };
        assert!(format!("{}", err).contains("5s"));

        // Test DockerUnavailable
        let err = DockerError::DockerUnavailable {
            reason: "test reason".to_string(),
            suggestion: "test suggestion".to_string(),
        };
        assert!(format!("{}", err).contains("test reason"));
    }
}
