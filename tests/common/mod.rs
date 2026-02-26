//! Common test utilities, fixtures, and assertions for the Switchboard test suite.
//!
//! This module provides shared functionality used across unit and integration tests.
//!
//! # Mock Implementation Patterns
//!
//! This module documents the mock setup patterns for key traits used in Switchboard.
//! These patterns enable comprehensive unit testing without requiring actual Docker
//! containers or external processes.
//!
//! ## Overview
//!
//! The following traits are designed for dependency injection and can be mocked
//! for testing:
//!
//! - [`DockerClientTrait`] - For Docker container operations
//! - [`ProcessExecutorTrait`] - For executing external processes
//!
//! Both traits use [`async_trait`] to enable async method implementations, which
//! is essential for idiomatic async Rust code.
//!
//! # Example: Using Mocks in Unit Tests
//!
//! ```ignore
//! use mockall::mock;
//!
//! // Define mock implementations
//! mock! {
//!     DockerClientTrait {}
//! }
//!
//! #[tokio::test]
//! async fn test_example() {
//!     // Create a mock instance
//!     let mut mock = MockDockerClientTrait::new();
//!
//!     // Configure mock expectations
//!     mock.expect_ping()
//!         .returning(|_| Ok(true));
//!
//!     // Use the mock in your code
//!     let result = mock.ping().await;
//!     assert!(result.unwrap());
//! }
//! ```
//!
//! ---
//!
//! ## DockerClientTrait Mock Pattern
//!
//! The [`DockerClientTrait`] provides Docker container management capabilities.
//! Below is the expected trait definition and mock setup pattern.
//!
//! ### Expected Trait Definition
//!
//! ```ignore
//! use async_trait::async_trait;
//!
//! #[async_trait]
//! pub trait DockerClientTrait: Send + Sync {
//!     /// Check if Docker daemon is running
//!     async fn ping(&self) -> Result<bool, DockerError>;
//!
//!     /// Check if a Docker image exists
//!     async fn image_exists(&self, image_name: &str) -> Result<bool, DockerError>;
//!
//!     /// Build a Docker image from a Dockerfile
//!     async fn build_image(&self, name: &str, path: &str) -> Result<String, DockerError>;
//!
//!     /// Run a Docker container
//!     async fn run_container(&self, config: ContainerConfig) -> Result<String, DockerError>;
//!
//!     /// Stop a running container
//!     async fn stop_container(&self, container_id: &str) -> Result<(), DockerError>;
//!
//!     /// Get container logs
//!     async fn container_logs(&self, container_id: &str) -> Result<String, DockerError>;
//!
//!     /// Wait for container to complete
//!     async fn wait_container(&self, container_id: &str) -> Result<ExitStatus, DockerError>;
//! }
//! ```
//!
//! ### Mock Setup Pattern
//!
//! ```ignore
//! use mockall::mock;
//!
//! // Define the mock - this would be auto-generated or manually implemented
//! mock! {
//!     DockerClientTrait {
//!         async fn ping(&self) -> Result<bool, DockerError>;
//!         async fn image_exists(&self, image_name: &str) -> Result<bool, DockerError>;
//!         async fn build_image(&self, name: &str, path: &str) -> Result<String, DockerError>;
//!         async fn run_container(&self, config: ContainerConfig) -> Result<String, DockerError>;
//!         async fn stop_container(&self, container_id: &str) -> Result<(), DockerError>;
//!         async fn container_logs(&self, container_id: &str) -> Result<String, DockerError>;
//!         async fn wait_container(&self, container_id: &str) -> Result<ExitStatus, DockerError>;
//!     }
//! }
//!
//! // Helper function to create a pre-configured mock
//! #[allow(dead_code)]
//! pub fn create_mock_docker_client() -> MockDockerClientTrait {
//!     let mut mock = MockDockerClientTrait::new();
//!
//!     // Default expectations - can be overridden in tests
//!     mock.expect_ping()
//!         .returning(|_| Ok(true));
//!
//!     mock.expect_image_exists()
//!         .returning(|_| Ok(true));
//!
//!     mock.expect_build_image()
//!         .returning(|name, _| Ok(format!("sha256:mock-{}", name)));
//!
//!     mock.expect_run_container()
//!         .returning(|_| Ok("mock-container-id".to_string()));
//!
//!     mock.expect_stop_container()
//!         .returning(|_| Ok(()));
//!
//!     mock.expect_container_logs()
//!         .returning(|_| Ok("Mock container log output".to_string()));
//!
//!     mock.expect_wait_container()
//!         .returning(|_| Ok(ExitStatus::from_raw(0)));
//!
//!     mock
//! }
//! ```
//!
//! ### Usage Example in Tests
//!
//! ```ignore
//! #[tokio::test]
//! async fn test_container_lifecycle() {
//!     // Arrange
//!     let mut mock = create_mock_docker_client();
//!
//!     // Configure specific expectations for this test
//!     mock.expect_image_exists()
//!         .with(mockall::predicate::eq("my-image:latest"))
//!         .returning(|_| Ok(false));
//!
//!     mock.expect_build_image()
//!         .withf(|name, path| name == "my-image:latest" && path == "./Dockerfile")
//!         .returning(|name, _| Ok(format!("sha256:mock-{}", name)));
//!
//!     mock.expect_run_container()
//!         .returning(|_| Ok("container-123".to_string()));
//!
//!     mock.expect_wait_container()
//!         .returning(|_| Ok(ExitStatus::from_raw(0)));
//!
//!     // Act
//!     let exists = mock.image_exists("my-image:latest").await.unwrap();
//!     assert!(!exists);
//!
//!     let image_id = mock.build_image("my-image:latest", "./Dockerfile").await.unwrap();
//!     assert!(image_id.starts_with("sha256:mock-"));
//!
//!     let container_id = mock.run_container(ContainerConfig::default()).await.unwrap();
//!     assert_eq!(container_id, "container-123");
//!
//!     let status = mock.wait_container(&container_id).await.unwrap();
//!     assert!(status.success());
//! }
//! ```
//!
//! ---
//!
//! ## ProcessExecutorTrait Mock Pattern
//!
//! The [`ProcessExecutorTrait`] provides process execution capabilities for running
//! external commands.
//!
//! ### Expected Trait Definition
//!
//! ```ignore
//! use async_trait::async_trait;
//!
//! #[async_trait]
//! pub trait ProcessExecutorTrait: Send + Sync {
//!     /// Execute a command without additional environment variables
//!     async fn execute(&self, command: Command) -> Result<ProcessOutput, ProcessError>;
//!
//!     /// Execute a command with additional environment variables
//!     async fn execute_with_env(
//!         &self,
//!         command: Command,
//!         env_vars: HashMap<String, String>,
//!     ) -> Result<ProcessOutput, ProcessError>;
//! }
//! ```
//!
//! ### Mock Setup Pattern
//!
//! ```ignore
//! use mockall::mock;
//!
//! // Define the mock for ProcessExecutorTrait
//! mock! {
//!     ProcessExecutorTrait {
//!         async fn execute(&self, command: Command) -> Result<ProcessOutput, ProcessError>;
//!         async fn execute_with_env(
//!             &self,
//!             command: Command,
//!             env_vars: HashMap<String, String>,
//!         ) -> Result<ProcessOutput, ProcessError>;
//!     }
//! }
//!
//! // Helper function to create a pre-configured mock
//! #[allow(dead_code)]
//! pub fn create_mock_process_executor() -> MockProcessExecutorTrait {
//!     let mut mock = MockProcessExecutorTrait::new();
//!
//!     // Default successful execution
//!     mock.expect_execute()
//!         .returning(|_| Ok(ProcessOutput {
//!             stdout: "Command succeeded".to_string(),
//!             stderr: String::new(),
//!             status: ExitStatus::from_raw(0),
//!         }));
//!
//!     mock.expect_execute_with_env()
//!         .returning(|_, _| Ok(ProcessOutput {
//!             stdout: "Command succeeded".to_string(),
//!             stderr: String::new(),
//!             status: ExitStatus::from_raw(0),
//!         }));
//!
//!     mock
//! }
//!
//! // Builder pattern for more complex scenarios
//! #[allow(dead_code)]
//! pub struct MockProcessExecutorBuilder {
//!     mock: MockProcessExecutorTrait,
//! }
//!
//! impl MockProcessExecutorBuilder {
//!     pub fn new() -> Self {
//!         Self {
//!             mock: MockProcessExecutorTrait::new(),
//!         }
//!     }
//!
//!     pub fn with_execute_result(mut self, output: ProcessOutput) -> Self {
//!         self.mock.expect_execute()
//!             .returning(move |_| Ok(output.clone()));
//!         self
//!     }
//!
//!     pub fn with_execute_error(self, error: ProcessError) -> Self {
//!         self.mock.expect_execute()
//!             .returning(move |_| Err(error.clone()));
//!         self
//!     }
//!
//!     pub fn build(self) -> MockProcessExecutorTrait {
//!         self.mock
//!     }
//! }
//!
//! impl Default for MockProcessExecutorBuilder {
//!     fn default() -> Self {
//!         Self::new()
//!     }
//! }
//! ```
//!
//! ### Usage Example in Tests
//!
//! ```ignore
//! #[tokio::test]
//! async fn test_skill_execution() {
//!     // Arrange - create mock with controlled responses
//!     let mut mock = MockProcessExecutorTrait::new();
//!
//!     // Configure specific command behavior
//!     mock.expect_execute()
//!         .with(mockall::predicate::function(|cmd: &Command| {
//!             cmd.program == "python" && cmd.args.contains(&"script.py".to_string())
//!         }))
//!         .returning(|_| Ok(ProcessOutput {
//!             stdout: "Execution output".to_string(),
//!             stderr: "".to_string(),
//!             status: ExitStatus::from_raw(0),
//!         }));
//!
//!     // Act - use in your skill runner
//!     let result = run_skill(&mock, "python", vec!["script.py"]).await;
//!
//!     // Assert
//!     assert!(result.is_ok());
//! }
//!
//! #[tokio::test]
//! async fn test_process_failure() {
//!     // Arrange - simulate command failure
//!     let mut mock = MockProcessExecutorTrait::new();
//!
//!     mock.expect_execute()
//!         .returning(|_| Ok(ProcessOutput {
//!             stdout: String::new(),
//!             stderr: "Command not found".to_string(),
//!             status: ExitStatus::from_raw(127),
//!         }));
//!
//!     // Act
//!     let result = mock.execute(Command::new("nonexistent")).await;
//!
//!     // Assert
//!     let output = result.unwrap();
//!     assert!(!output.status.success());
//! }
//! ```
//!
//! ---
//!
//! ## async_trait Usage Guide
//!
//! The [`async_trait`] crate is essential for defining async methods in traits.
//! Without it, async methods in traits would require dynamic dispatch through
//! boxed futures, which has performance and ergonomics costs.
//!
//! ### Adding async_trait to Your Test
//!
//! ```ignore
//! use async_trait::async_trait;
//!
//! // Import the trait you want to mock
//! use crate::docker::DockerClientTrait;
//!
//! // Define mock with async_trait
//! #[async_trait]
//! impl DockerClientTrait for MockDockerClient {
//!     async fn ping(&self) -> Result<bool, DockerError> {
//!         // Your mock implementation
//!         Ok(true)
//!     }
//!
//!     async fn image_exists(&self, image_name: &str) -> Result<bool, DockerError> {
//!         // Your mock implementation
//!         Ok(image_name.starts_with("test-"))
//!     }
//!
//!     // ... other methods
//! }
//! ```
//!
//! ### Key Points About async_trait
//!
//! 1. **Always use `#[async_trait]`** - Both on the trait definition and any implementations
//! 2. **Use `async fn` in traits** - This desugars to returning a boxed future without async_trait
//! 3. **Mockall compatibility** - The mockall crate can generate mocks for async_trait-based traits
//! 4. **Error handling** - Return `Result<T, SomeError>` from async methods
//!
//! ---
//!
//! ## Best Practices for Mocking
//!
//! 1. **Use `mockall::predicate`** - For matching specific arguments
//! ```ignore
//! use mockall::predicate::*;
//!
//! mock.expect_method()
//!     .with(eq("specific-value"))
//!     .returning(|_| Ok(()));
//! ```
//!
//! 2. **Use `times(n)` for call verification**
//! ```ignore
//! mock.expect_method()
//!     .times(1)
//!     .returning(|_| Ok(()));
//! ```
//!
//! 3. **Use `panic_on_unsatisfied_expectations`** for debugging
//! ```ignore
//! let mut mock = MockTrait::new();
//! mock.panic_on_unsatisfied_expectations();
//! ```
//!
//! 4. **Prefer builder patterns** for complex mock setup
//! ```ignore
//! let mock = MockBuilder::new()
//!     .with_default_responses()
//!     .with_specific_override()
//!     .build();
//! ```
//!
//! 5. **Document expected behavior** in test comments
//!
//! ---
//!
//! ## See Also
//!
//! - [`mockall` crate documentation](https://docs.rs/mockall/)
//! - [`async_trait` crate documentation](https://docs.rs/async-trait/)
//! - [`tests/common/fixtures`](fixtures/index.html) - Test fixtures module
//! - [`tests/common/assertions`](assertions/index.html) - Custom assertions

pub mod assertions;
pub mod fixtures;
