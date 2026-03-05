//! Docker Connection Trait - Trait for Docker connection management
//!
//! This module provides:
//! - `DockerConnectionTrait` - A trait for Docker connection management
//! - `RealDockerConnection` - Production implementation delegating to client.rs
//! - `MockDockerConnection` - Mock implementation for testing
//! - `DockerCommand` - Enum for Docker commands
//! - `DockerResponse` - Enum for Docker command responses

use bollard::Docker;
use std::future::Future;
use std::pin::Pin;

use crate::docker::DockerError;

/// Enum representing Docker commands that can be executed
///
/// This enum defines the supported Docker operations that can be
/// performed through the `execute()` method on `DockerConnectionTrait`.
#[derive(Debug, Clone)]
pub enum DockerCommand {
    /// List all containers
    ///
    /// # Arguments
    /// * `all` - If true, include stopped containers
    ListContainers { all: bool },

    /// List all images
    ///
    /// # Arguments
    /// * `all` - If true, include intermediate images
    ListImages { all: bool },

    /// Pull an image from a registry
    ///
    /// # Arguments
    /// * `image` - The image name (e.g., "nginx:latest")
    /// * `tag` - The specific tag to pull
    PullImage { image: String, tag: String },

    /// Build an image from a Dockerfile
    ///
    /// # Arguments
    /// * `path` - Path to the build context
    /// * `dockerfile` - Path to the Dockerfile relative to context
    /// * `tag` - Image tag (e.g., "myapp:latest")
    BuildImage {
        path: String,
        dockerfile: String,
        tag: String,
    },

    /// Create and start a container
    ///
    /// # Arguments
    /// * `image` - The image to use
    /// * `name` - Name for the container
    /// * `command` - Command to run in the container
    RunContainer {
        image: String,
        name: String,
        command: Option<Vec<String>>,
    },

    /// Start an existing container
    ///
    /// # Arguments
    /// * `id` - Container ID or name
    StartContainer { id: String },

    /// Stop a running container
    ///
    /// # Arguments
    /// * `id` - Container ID or name
    /// * `timeout` - Seconds to wait before killing
    StopContainer { id: String, timeout: Option<u64> },

    /// Remove a container
    ///
    /// # Arguments
    /// * `id` - Container ID or name
    /// * `force` - Force removal even if running
    RemoveContainer { id: String, force: bool },

    /// Get container logs
    ///
    /// # Arguments
    /// * `id` - Container ID or name
    /// * `tail` - Number of lines to read from the end
    ContainerLogs { id: String, tail: Option<u64> },

    /// Inspect a container
    ///
    /// # Arguments
    /// * `id` - Container ID or name
    InspectContainer { id: String },

    /// Ping the Docker daemon
    Ping,
}

/// Enum representing responses from Docker commands
///
/// This enum provides typed responses for each command variant
/// in `DockerCommand`.
#[derive(Debug, Clone)]
pub enum DockerResponse {
    /// Response for ListContainers command
    ///
    /// Contains a vector of container info tuples: (id, name, image, status)
    ListContainers(Vec<(String, String, String, String)>),

    /// Response for ListImages command
    ///
    /// Contains a vector of image info tuples: (id, repository, tag, size)
    ListImages(Vec<(String, String, String, u64)>),

    /// Response for PullImage command
    ///
    /// Contains the pulled image reference
    PullImage { reference: String },

    /// Response for BuildImage command
    ///
    /// Contains the built image ID
    BuildImage { image_id: String },

    /// Response for RunContainer command
    ///
    /// Contains the created container ID
    RunContainer { container_id: String },

    /// Response for StartContainer command
    ///
    /// Contains the container ID
    StartContainer { container_id: String },

    /// Response for StopContainer command
    ///
    /// Contains the container ID
    StopContainer { container_id: String },

    /// Response for RemoveContainer command
    ///
    /// Contains the removed container ID
    RemoveContainer { container_id: String },

    /// Response for ContainerLogs command
    ///
    /// Contains the log output as a string
    ContainerLogs { logs: String },

    /// Response for InspectContainer command
    ///
    /// Contains the container details as JSON string
    InspectContainer { details: String },

    /// Response for Ping command
    ///
    /// Contains the ping result string
    Ping { result: String },
}

/// Trait for Docker connection management
///
/// This trait provides an abstraction for Docker connection operations,
/// allowing for both production and testing implementations.
///
/// # Requirements
/// - Object-safe with `Send + Sync` bounds for shared references across async tasks
pub trait DockerConnectionTrait: Send + Sync {
    /// Get the Docker socket path
    ///
    /// Returns the path to the Docker socket from the active Docker context.
    ///
    /// # Errors
    ///
    /// Returns `DockerError` if the socket path cannot be determined.
    fn get_docker_socket_path(&self) -> Pin<Box<dyn Future<Output = Result<String, DockerError>> + Send + '_>>;

    /// Connect to Docker daemon
    ///
    /// Establishes a connection to the Docker daemon using the active context.
    ///
    /// # Errors
    ///
    /// Returns `DockerError` if connection fails.
    fn connect(&self) -> Pin<Box<dyn Future<Output = Result<Docker, DockerError>> + Send + '_>>;

    /// Disconnect from Docker daemon
    ///
    /// Closes the connection to the Docker daemon. For bollard, this is
    /// a no-op since it uses connection pooling, but the method is provided
    /// for API completeness.
    ///
    /// # Errors
    ///
    /// Returns `DockerError` if disconnection fails.
    fn disconnect(&self) -> Pin<Box<dyn Future<Output = Result<(), DockerError>> + Send>>;

    /// Check if Docker is available
    ///
    /// Verifies that the Docker daemon is running and responsive.
    ///
    /// # Errors
    ///
    /// Returns `DockerError` if Docker is not available.
    fn check_docker_available(&self) -> Pin<Box<dyn Future<Output = Result<bool, DockerError>> + Send + '_>>;

    /// Execute a Docker command
    ///
    /// Executes the specified Docker command and returns the response.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The `DockerCommand` to execute
    ///
    /// # Errors
    ///
    /// Returns `DockerError` if the command execution fails.
    fn execute(
        &self,
        cmd: DockerCommand,
    ) -> Pin<Box<dyn Future<Output = Result<DockerResponse, DockerError>> + Send>>;
}

/// Production implementation of DockerConnectionTrait
///
/// This implementation delegates to the existing functions in `client.rs`.
pub struct RealDockerConnection {
    _private: (),
}

impl RealDockerConnection {
    /// Create a new RealDockerConnection instance
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for RealDockerConnection {
    fn default() -> Self {
        Self::new()
    }
}

impl DockerConnectionTrait for RealDockerConnection {
    fn get_docker_socket_path(&self) -> Pin<Box<dyn Future<Output = Result<String, DockerError>> + Send + '_>> {
        Box::pin(async move {
            crate::docker::get_docker_socket_path(None)
                .await
                .map_err(|e| DockerError::ConnectionError(format!("Failed to get socket path: {}", e)))?
                .ok_or_else(|| DockerError::ConnectionError("No socket path found".to_string()))
        })
    }

    fn connect(&self) -> Pin<Box<dyn Future<Output = Result<Docker, DockerError>> + Send + '_>> {
        Box::pin(async move {
            crate::docker::connect_to_docker(None).await.map_err(|e| {
                DockerError::ConnectionError(format!("Failed to connect to Docker: {}", e))
            })
        })
    }

    fn disconnect(&self) -> Pin<Box<dyn Future<Output = Result<(), DockerError>> + Send>> {
        Box::pin(async move {
            // Bollard uses connection pooling, so we don't need to explicitly disconnect
            // The connection will be closed when the Docker client is dropped
            tracing::debug!("Disconnect called on RealDockerConnection - no-op for bollard");
            Ok(())
        })
    }

    fn check_docker_available(&self) -> Pin<Box<dyn Future<Output = Result<bool, DockerError>> + Send + '_>> {
        Box::pin(async move {
            match crate::docker::check_docker_available().await {
                Ok(_) => Ok(true),
                Err(e) => Err(e),
            }
        })
    }

    fn execute(
        &self,
        _cmd: DockerCommand,
    ) -> Pin<Box<dyn Future<Output = Result<DockerResponse, DockerError>> + Send>> {
        Box::pin(async move {
            // For production, we delegate to the actual Docker client operations
            // This is a placeholder implementation - in production, you would
            // create a Docker client and execute the command
            tracing::debug!(
                "Execute called on RealDockerConnection - command would be processed here"
            );

            // Return a not-implemented error for now since this is a placeholder
            Err(DockerError::NotImplemented(
                "execute() method not yet fully implemented for RealDockerConnection".to_string(),
            ))
        })
    }
}

/// Mock implementation of DockerConnectionTrait for testing
///
/// This implementation allows configuring the behavior for testing purposes.
pub struct MockDockerConnection {
    socket_path: Option<String>,
    connect_success: bool,
    available: bool,
    connect_timeout: Option<std::time::Duration>,
    /// Whether disconnect should succeed
    disconnect_success: bool,
    /// Whether execute should succeed
    execute_success: bool,
    /// Response to return for execute
    execute_response: Option<DockerResponse>,
}

/// Builder for MockDockerConnection
pub struct MockDockerConnectionBuilder {
    socket_path: Option<String>,
    connect_success: bool,
    available: bool,
    connect_timeout: Option<std::time::Duration>,
    disconnect_success: bool,
    execute_success: bool,
    execute_response: Option<DockerResponse>,
}

impl MockDockerConnectionBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            socket_path: Some("/var/run/docker.sock".to_string()),
            connect_success: true,
            available: true,
            connect_timeout: None,
            disconnect_success: true,
            execute_success: true,
            execute_response: None,
        }
    }

    /// Set the socket path to return
    pub fn with_socket_path(mut self, path: Option<String>) -> Self {
        self.socket_path = path;
        self
    }

    /// Set whether connection should succeed
    pub fn with_connect_success(mut self, success: bool) -> Self {
        self.connect_success = success;
        self
    }

    /// Set whether Docker should be available
    pub fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    /// Set connection timeout duration
    pub fn with_connect_timeout(mut self, timeout: Option<std::time::Duration>) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set whether disconnect should succeed
    pub fn with_disconnect_success(mut self, success: bool) -> Self {
        self.disconnect_success = success;
        self
    }

    /// Set whether execute should succeed
    pub fn with_execute_success(mut self, success: bool) -> Self {
        self.execute_success = success;
        self
    }

    /// Set the response to return for execute
    pub fn with_execute_response(mut self, response: Option<DockerResponse>) -> Self {
        self.execute_response = response;
        self
    }

    /// Build the MockDockerConnection
    pub fn build(self) -> MockDockerConnection {
        MockDockerConnection {
            socket_path: self.socket_path,
            connect_success: self.connect_success,
            available: self.available,
            connect_timeout: self.connect_timeout,
            disconnect_success: self.disconnect_success,
            execute_success: self.execute_success,
            execute_response: self.execute_response,
        }
    }
}

impl Default for MockDockerConnectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DockerConnectionTrait for MockDockerConnection {
    fn get_docker_socket_path(&self) -> Pin<Box<dyn Future<Output = Result<String, DockerError>> + Send + '_>> {
        Box::pin(async move {
            self.socket_path.clone().ok_or_else(|| {
                DockerError::ConnectionError("Mock: No socket path configured".to_string())
            })
        })
    }

    fn connect(&self) -> Pin<Box<dyn Future<Output = Result<Docker, DockerError>> + Send + '_>> {
        Box::pin(async move {
            // Check if timeout is configured - return timeout error if so
            if let Some(timeout) = self.connect_timeout {
                return Err(DockerError::ConnectionTimeout {
                    timeout_duration: format!("{:?}", timeout),
                    suggestion:
                        "Consider increasing the connection timeout or checking Docker daemon status"
                            .to_string(),
                });
            }

            if self.connect_success {
                // For mock, we can't actually connect to Docker, so we return an error
                // indicating this is a mock implementation
                Err(DockerError::ConnectionError(
                    "Mock: Cannot create actual Docker connection in mock".to_string(),
                ))
            } else {
                Err(DockerError::ConnectionError(
                    "Mock: Connection failed".to_string(),
                ))
            }
        })
    }

    fn disconnect(&self) -> Pin<Box<dyn Future<Output = Result<(), DockerError>> + Send>> {
        // Clone the data we need from self before entering the async block
        let disconnect_success = self.disconnect_success;

        Box::pin(async move {
            if disconnect_success {
                tracing::debug!("MockDockerConnection: disconnect succeeded");
                Ok(())
            } else {
                tracing::debug!("MockDockerConnection: disconnect failed");
                Err(DockerError::ConnectionError(
                    "Mock: Disconnect failed".to_string(),
                ))
            }
        })
    }

    fn check_docker_available(&self) -> Pin<Box<dyn Future<Output = Result<bool, DockerError>> + Send + '_>> {
        Box::pin(async move {
            if self.available {
                Ok(true)
            } else {
                Err(DockerError::DockerUnavailable {
                    reason: "Mock: Docker not available".to_string(),
                    suggestion: "Configure mock to return available".to_string(),
                })
            }
        })
    }

    fn execute(
        &self,
        cmd: DockerCommand,
    ) -> Pin<Box<dyn Future<Output = Result<DockerResponse, DockerError>> + Send>> {
        // Clone the data we need from self before entering the async block
        let execute_success = self.execute_success;
        let execute_response = self.execute_response.clone();

        Box::pin(async move {
            if !execute_success {
                return Err(DockerError::ConnectionError(
                    "Mock: Execute failed".to_string(),
                ));
            }

            // Return configured response if available
            if let Some(response) = &execute_response {
                tracing::debug!(
                    "MockDockerConnection: execute returning configured response for {:?}",
                    cmd
                );
                // Clone the response to avoid borrowing issues
                return Ok(response.clone());
            }

            // Return a mock response based on the command type
            // We need to clone cmd to avoid borrowing issues
            let response = match cmd.clone() {
                DockerCommand::ListContainers { .. } => DockerResponse::ListContainers(vec![(
                    "abc123".to_string(),
                    "nginx".to_string(),
                    "nginx:latest".to_string(),
                    "running".to_string(),
                )]),
                DockerCommand::ListImages { .. } => DockerResponse::ListImages(vec![(
                    "sha256:abc123".to_string(),
                    "nginx".to_string(),
                    "latest".to_string(),
                    142000000,
                )]),
                DockerCommand::PullImage { image, tag } => DockerResponse::PullImage {
                    reference: format!("{}:{}", image, tag),
                },
                DockerCommand::BuildImage { tag, .. } => DockerResponse::BuildImage {
                    image_id: format!("sha256:{}", &tag[..8]),
                },
                DockerCommand::RunContainer { name, .. } => DockerResponse::RunContainer {
                    container_id: format!("mock-{}", name),
                },
                DockerCommand::StartContainer { id } => {
                    DockerResponse::StartContainer { container_id: id }
                }
                DockerCommand::StopContainer { id, .. } => {
                    DockerResponse::StopContainer { container_id: id }
                }
                DockerCommand::RemoveContainer { id, .. } => {
                    DockerResponse::RemoveContainer { container_id: id }
                }
                DockerCommand::ContainerLogs { .. } => DockerResponse::ContainerLogs {
                    logs: "Mock container logs".to_string(),
                },
                DockerCommand::InspectContainer { id } => DockerResponse::InspectContainer {
                    details: format!(r#"{{"Id": "{}", "Name": "/mock-container"}}"#, id),
                },
                DockerCommand::Ping => DockerResponse::Ping {
                    result: "OK".to_string(),
                },
            };

            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_docker_connection_new() {
        let connection = RealDockerConnection::new();
        // Just verify it can be created
        let _ = connection;
    }

    #[test]
    fn test_real_docker_connection_default() {
        let connection = RealDockerConnection::default();
        // Just verify it can be created
        let _ = connection;
    }

    #[test]
    fn test_mock_docker_connection_builder_default() {
        let mock = MockDockerConnectionBuilder::new().build();

        // Default mock should return available
        assert!(mock.check_docker_available().is_ok());
    }

    #[test]
    fn test_mock_docker_connection_builder_with_socket_path() {
        let mock = MockDockerConnectionBuilder::new()
            .with_socket_path(Some("/custom/socket.sock".to_string()))
            .build();

        assert_eq!(
            mock.get_docker_socket_path().unwrap(),
            "/custom/socket.sock"
        );
    }

    #[test]
    fn test_mock_docker_connection_builder_with_connect_failure() {
        let mock = MockDockerConnectionBuilder::new()
            .with_connect_success(false)
            .build();

        assert!(mock.connect().is_err());
    }

    #[test]
    fn test_mock_docker_connection_builder_with_unavailable() {
        let mock = MockDockerConnectionBuilder::new()
            .with_available(false)
            .build();

        assert!(mock.check_docker_available().is_err());
    }

    #[test]
    fn test_mock_docker_connection_as_trait() {
        // Test that MockDockerConnection works with the trait
        let mock = MockDockerConnectionBuilder::new()
            .with_available(true)
            .build();

        // Use the trait via the concrete type
        assert!(mock.check_docker_available().is_ok());
    }

    #[test]
    fn test_mock_docker_connection_send_sync() {
        // Verify that MockDockerConnection implements Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockDockerConnection>();
    }

    #[test]
    fn test_connection_timeout_configuration() {
        // Test that timeout can be configured via builder
        let mock = MockDockerConnectionBuilder::new()
            .with_connect_timeout(Some(std::time::Duration::from_secs(5)))
            .build();

        // Verify the mock was configured with timeout
        let result = mock.connect();
        assert!(
            result.is_err(),
            "Connection should fail when timeout is configured"
        );

        // Verify it's a timeout error
        match result.unwrap_err() {
            DockerError::ConnectionTimeout { .. } => {}
            other => panic!("Expected ConnectionTimeout error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_connection_timeout_error() {
        // Test that connection timeout produces appropriate error
        let mock = MockDockerConnectionBuilder::new()
            .with_connect_timeout(Some(std::time::Duration::from_secs(10)))
            .build();

        let result = mock.connect();
        assert!(result.is_err(), "Expected error when connection times out");

        // Verify error contains timeout information
        let error = result.unwrap_err();
        match error {
            DockerError::ConnectionTimeout {
                timeout_duration,
                suggestion,
            } => {
                assert!(
                    timeout_duration.contains("10s"),
                    "Timeout duration should be 10s"
                );
                assert!(!suggestion.is_empty(), "Suggestion should not be empty");
            }
            other => panic!("Expected ConnectionTimeout error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_disconnect_success() {
        let mock = MockDockerConnectionBuilder::new()
            .with_disconnect_success(true)
            .build();

        let result = mock.disconnect();
        assert!(result.await.is_ok(), "Disconnect should succeed");
    }

    #[tokio::test]
    async fn test_disconnect_failure() {
        let mock = MockDockerConnectionBuilder::new()
            .with_disconnect_success(false)
            .build();

        let result = mock.disconnect();
        assert!(result.await.is_err(), "Disconnect should fail");
    }

    #[tokio::test]
    async fn test_execute_list_containers() {
        let mock = MockDockerConnectionBuilder::new().build();

        let result = mock.execute(DockerCommand::ListContainers { all: true });
        let response = result.await.unwrap();

        match response {
            DockerResponse::ListContainers(containers) => {
                assert!(!containers.is_empty());
            }
            other => panic!("Expected ListContainers response, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_ping() {
        let mock = MockDockerConnectionBuilder::new().build();

        let result = mock.execute(DockerCommand::Ping);
        let response = result.await.unwrap();

        match response {
            DockerResponse::Ping { result } => {
                assert_eq!(result, "OK");
            }
            other => panic!("Expected Ping response, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_with_custom_response() {
        let custom_response = DockerResponse::BuildImage {
            image_id: "custom-image-id".to_string(),
        };

        let mock = MockDockerConnectionBuilder::new()
            .with_execute_response(Some(custom_response.clone()))
            .build();

        let result = mock.execute(DockerCommand::BuildImage {
            path: "/tmp".to_string(),
            dockerfile: "Dockerfile".to_string(),
            tag: "test:latest".to_string(),
        });

        let response = result.await.unwrap();
        match response {
            DockerResponse::BuildImage { image_id } => {
                assert_eq!(image_id, "custom-image-id");
            }
            other => panic!("Expected BuildImage response, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_failure() {
        let mock = MockDockerConnectionBuilder::new()
            .with_execute_success(false)
            .build();

        let result = mock.execute(DockerCommand::Ping);
        assert!(result.await.is_err(), "Execute should fail when configured");
    }

    #[tokio::test]
    async fn test_real_disconnect_is_noop() {
        let connection = RealDockerConnection::new();
        let result = connection.disconnect();
        // Should always succeed (no-op for bollard)
        assert!(result.await.is_ok());
    }

    #[tokio::test]
    async fn test_real_execute_returns_not_implemented() {
        let connection = RealDockerConnection::new();
        let result = connection.execute(DockerCommand::Ping);
        // Should return NotImplemented error
        let err = result.await.unwrap_err();
        match err {
            DockerError::NotImplemented(_) => {}
            other => panic!("Expected NotImplemented error, got: {:?}", other),
        }
    }
}
