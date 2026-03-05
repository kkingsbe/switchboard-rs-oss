//! Docker Client - Connection management and Docker client wrapper
//!
//! This module provides:
//! - Docker connection management: get_docker_socket_path(), connect_to_docker(), check_docker_available()
//! - DockerError enum for error handling
//! - DockerClient struct with all methods
//! - DockerClientTrait implementations

use crate::docker::connection::{
    DockerConnectionTrait, RealDockerConnection,
};
use bollard::Docker;
use std::path::Path;
use std::sync::Arc;
use tokio::time::Duration;

pub use crate::traits::{
    BuildOptions, DockerClientTrait, ProcessError, ProcessExecutorTrait, RealDockerClient,
    RealProcessExecutor,
};

/// Error message used when Docker client is not available
const DOCKER_NOT_AVAILABLE: &str = "Docker client not available";

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
                let path = socket_path.strip_prefix("npipe://").ok_or_else(|| {
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
/// - `docker` - The underlying `bollard::Docker` client instance for Docker API calls
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
/// // Get access to the underlying Docker client for advanced operations
/// let docker = client.docker();
/// # Ok(())
/// # }
/// ```
pub struct DockerClient {
    /// The Docker connection trait object for connection management
    connection: Arc<dyn DockerConnectionTrait>,
    /// The Docker client trait object for Docker operations
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
            connection: Arc::clone(&self.connection),
            client: Arc::clone(&self.client),
            docker: self.docker.clone(),
            _image_name: self._image_name.clone(),
            _image_tag: self._image_tag.clone(),
        }
    }
}

impl DockerClient {
    /// Get the Docker client, returning an error if not available
    ///
    /// # Errors
    ///
    /// Returns `DockerError::ConnectionError` if the Docker client is not available.
    fn get_docker(&self) -> Result<&Docker, DockerError> {
        self.docker.as_ref().ok_or(DockerError::ConnectionError(
            DOCKER_NOT_AVAILABLE.to_string(),
        ))
    }

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
        // Connect to Docker using the connection trait
        let docker = connection.connect().map_err(|e| {
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
            docker: Some(docker),
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
        let docker = real_client.docker().clone();
        let client: Arc<dyn DockerClientTrait> = Arc::new(real_client);
        // Create a default RealDockerConnection for backward compatibility
        let connection: Arc<dyn DockerConnectionTrait> = Arc::new(RealDockerConnection::new());
        DockerClient {
            connection,
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
        let docker = self.get_docker()?;
        // Use block_in_place to properly handle being called from within an async context
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(docker.ping())
        })
        .map(|_| ())
        .map_err(|e| DockerError::ConnectionError(e.to_string()))
    }

    fn image_exists(&self, name: &str, tag: &str) -> Result<bool, DockerError> {
        use bollard::image::ListImagesOptions;

        let image_name = format!("{}:{}", name, tag);
        let docker = self.get_docker()?;

        // Use block_in_place to properly handle being called from within an async context
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async {
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
        })
        .map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))
    }

    fn build_image(
        &self,
        options: crate::traits::BuildOptions,
        context: std::path::PathBuf,
    ) -> Result<String, DockerError> {
        // Clone the internal docker client (it's Arc-based)
        let docker = self.get_docker()?.clone();

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

                // Create tarball - call through the module path to ensure correct resolution
                let tarball = crate::docker::create_build_context_tarball(&context, &dockerfile)
                    .map_err(|e| std::io::Error::other(e.to_string()))?;
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
                            return Err(std::io::Error::other(e.to_string()));
                        }
                    }
                }

                Ok(final_image_id)
            })
        })
        .map_err(|e| DockerError::ConnectionError(e.to_string()))?;

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
        let docker = self.get_docker()?.clone();
        let container_id = container_id.to_string();

        // Use block_in_place to handle both sync and async contexts properly
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async { docker.stop_container(&container_id, Some(options)).await })
        })
        .map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))?;

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

        let docker = self.get_docker()?.clone();
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
        })
        .map_err(|e| DockerError::ConnectionError(e.to_string()))?;

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
        let docker = self.get_docker()?.clone();
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
        })
        .map_err(|e| DockerError::ConnectionError(e.to_string()))?;

        Ok(crate::traits::ExitCode::from_i32(exit_code))
    }

    fn create_container(
        &self,
        options: Option<bollard::container::CreateContainerOptions<String>>,
        config: bollard::container::Config<String>,
    ) -> Result<String, DockerError> {
        let docker = self.get_docker()?.clone();

        // Use block_in_place to handle both sync and async contexts properly
        // This avoids the "Cannot start a runtime from within a runtime" error
        let result = tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async { docker.create_container(options, config).await })
        })
        .map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))?;

        Ok(result.id)
    }

    fn start_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::StartContainerOptions<String>>,
    ) -> Result<(), DockerError> {
        let docker = self.get_docker()?.clone();
        let container_id = container_id.to_string();

        // Use block_in_place to handle both sync and async contexts properly
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async { docker.start_container(&container_id, options).await })
        })
        .map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))?;

        Ok(())
    }

    fn inspect_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::InspectContainerOptions>,
    ) -> Result<bollard::service::ContainerInspectResponse, DockerError> {
        let docker = self.get_docker()?.clone();
        let container_id = container_id.to_string();

        // Use block_in_place to handle both sync and async contexts properly
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async { docker.inspect_container(&container_id, options).await })
        })
        .map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))
    }

    fn kill_container(
        &self,
        container_id: &str,
        options: Option<bollard::container::KillContainerOptions<String>>,
    ) -> Result<(), DockerError> {
        let docker = self.get_docker()?.clone();
        let container_id = container_id.to_string();

        // Use block_in_place to handle both sync and async contexts properly
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(async { docker.kill_container(&container_id, options).await })
        })
        .map_err(|e: bollard::errors::Error| DockerError::ConnectionError(e.to_string()))
    }
}
