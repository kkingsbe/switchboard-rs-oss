//! Docker Connection Trait - Trait for Docker connection management
//!
//! This module provides:
//! - `DockerConnectionTrait` - A trait for Docker connection management
//! - `RealDockerConnection` - Production implementation delegating to client.rs
//! - `MockDockerConnection` - Mock implementation for testing

use bollard::Docker;

use crate::docker::DockerError;

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
    fn get_docker_socket_path(&self) -> Result<String, DockerError>;

    /// Connect to Docker daemon
    ///
    /// Establishes a connection to the Docker daemon using the active context.
    ///
    /// # Errors
    ///
    /// Returns `DockerError` if connection fails.
    fn connect_to_docker(&self) -> Result<Docker, DockerError>;

    /// Check if Docker is available
    ///
    /// Verifies that the Docker daemon is running and responsive.
    ///
    /// # Errors
    ///
    /// Returns `DockerError` if Docker is not available.
    fn check_docker_available(&self) -> Result<bool, DockerError>;
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
    fn get_docker_socket_path(&self) -> Result<String, DockerError> {
        // Use tokio::runtime to run the async function synchronously
        // This is necessary because the trait method is synchronous
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            DockerError::ConnectionError(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            crate::docker::get_docker_socket_path(None)
                .await
                .map_err(|e| {
                    DockerError::ConnectionError(format!("Failed to get socket path: {}", e))
                })
                .and_then(|opt| {
                    opt.ok_or_else(|| {
                        DockerError::ConnectionError("No socket path found".to_string())
                    })
                })
        })
    }

    fn connect_to_docker(&self) -> Result<Docker, DockerError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            DockerError::ConnectionError(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            crate::docker::connect_to_docker(None).await.map_err(|e| {
                DockerError::ConnectionError(format!("Failed to connect to Docker: {}", e))
            })
        })
    }

    fn check_docker_available(&self) -> Result<bool, DockerError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            DockerError::ConnectionError(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            match crate::docker::check_docker_available().await {
                Ok(_) => Ok(true),
                Err(e) => Err(e),
            }
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
}

/// Builder for MockDockerConnection
pub struct MockDockerConnectionBuilder {
    socket_path: Option<String>,
    connect_success: bool,
    available: bool,
    connect_timeout: Option<std::time::Duration>,
}

impl MockDockerConnectionBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            socket_path: Some("/var/run/docker.sock".to_string()),
            connect_success: true,
            available: true,
            connect_timeout: None,
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

    /// Build the MockDockerConnection
    pub fn build(self) -> MockDockerConnection {
        MockDockerConnection {
            socket_path: self.socket_path,
            connect_success: self.connect_success,
            available: self.available,
            connect_timeout: self.connect_timeout,
        }
    }
}

impl Default for MockDockerConnectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DockerConnectionTrait for MockDockerConnection {
    fn get_docker_socket_path(&self) -> Result<String, DockerError> {
        self.socket_path.clone().ok_or_else(|| {
            DockerError::ConnectionError("Mock: No socket path configured".to_string())
        })
    }

    fn connect_to_docker(&self) -> Result<Docker, DockerError> {
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
    }

    fn check_docker_available(&self) -> Result<bool, DockerError> {
        if self.available {
            Ok(true)
        } else {
            Err(DockerError::DockerUnavailable {
                reason: "Mock: Docker not available".to_string(),
                suggestion: "Configure mock to return available".to_string(),
            })
        }
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

        assert!(mock.connect_to_docker().is_err());
    }

    #[test]
    fn test_mock_docker_connection_builder_with_unavailable() {
        let mock = MockDockerConnectionBuilder::new()
            .with_available(false)
            .build();

        assert!(mock.check_docker_available().is_err());
    }

    #[test]
    fn test_mock_docker_connection_trait_object() {
        // Test that MockDockerConnection can be used as a trait object
        let mock: Box<dyn DockerConnectionTrait> = Box::new(
            MockDockerConnectionBuilder::new()
                .with_available(true)
                .build(),
        );

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
        let result = mock.connect_to_docker();
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
        // Test that connection timeout produces appropriate error via trait object
        let mock: Box<dyn DockerConnectionTrait> = Box::new(
            MockDockerConnectionBuilder::new()
                .with_connect_timeout(Some(std::time::Duration::from_secs(10)))
                .build(),
        );

        let result = mock.connect_to_docker();
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
}
