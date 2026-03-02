//! Process Executor Trait for Child Process Management
//!
//! This module provides:
//! - ProcessExecutorTrait: A trait for spawning, waiting, and killing child processes
//! - RealProcessExecutor: A concrete implementation using std::process::Command
//! - MockProcessExecutor: A mock implementation for testing
//!
//! The trait allows for easier testing through dependency injection and
//! provides a clean abstraction over process lifecycle management.

use std::fmt::Debug;
use std::process::{Child, Command, ExitStatus, Stdio};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

#[cfg(unix)]
fn create_exit_status(code: i32) -> ExitStatus {
    ExitStatus::from_raw(code)
}

#[cfg(not(unix))]
fn create_exit_status(code: i32) -> ExitStatus {
    // On Windows, use cmd /c to exit with specific code
    // This is a workaround since from_raw is not available
    let output = std::process::Command::new("cmd")
        .args(["/c", &format!("exit {}", code)])
        .output()
        .unwrap();
    output.status
}

/// Errors that can occur during child process management
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
    /// Failed to kill the process
    KillFailed {
        program: String,
        error_details: String,
        suggestion: String,
    },
    /// Process already exited
    ProcessAlreadyExited {
        program: String,
        exit_status: std::process::ExitStatus,
    },
    /// Mock error for testing
    MockError { message: String },
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
            ProcessError::KillFailed {
                program,
                error_details,
                suggestion,
            } => {
                write!(
                    f,
                    "Failed to kill process '{}': {}. {}",
                    program, error_details, suggestion
                )
            }
            ProcessError::ProcessAlreadyExited {
                program,
                exit_status,
            } => {
                write!(
                    f,
                    "Process '{}' already exited with status: {:?}",
                    program, exit_status
                )
            }
            ProcessError::MockError { message } => {
                write!(f, "Mock error: {}", message)
            }
        }
    }
}

impl std::error::Error for ProcessError {}

/// Process Executor Trait - defines the interface for managing child processes
///
/// This trait provides an abstraction over child process lifecycle management,
/// allowing for:
/// - Spawning child processes
/// - Waiting for processes to complete
/// - Killing running processes
/// - Easier unit testing through mock implementations
///
/// # Example
///
/// ```ignore
/// use std::process::Command;
/// use crate::traits::process::{ProcessExecutorTrait, RealProcessExecutor};
///
/// let executor = RealProcessExecutor::new();
/// let mut cmd = Command::new("echo");
/// cmd.arg("hello");
///
/// let mut child = executor.spawn_child(cmd).expect("Failed to spawn child");
/// let status = executor.wait_child(&mut child).expect("Failed to wait for child");
/// assert!(status.success());
/// ```
pub trait ProcessExecutorTrait: Send + Sync + Debug {
    /// Spawn a child process from the given command
    ///
    /// # Arguments
    ///
    /// * `cmd` - The command to spawn
    ///
    /// # Errors
    ///
    /// Returns ProcessError if spawning fails
    fn spawn_child(&self, cmd: Command) -> Result<Child, ProcessError>;

    /// Wait for a child process to complete and return its exit status
    ///
    /// # Arguments
    ///
    /// * `child` - The child process to wait on (mutable reference)
    ///
    /// # Errors
    ///
    /// Returns ProcessError if waiting fails
    fn wait_child(&self, child: &mut Child) -> Result<ExitStatus, ProcessError>;

    /// Kill a running child process
    ///
    /// # Arguments
    ///
    /// * `child` - The child process to kill (mutable reference)
    ///
    /// # Errors
    ///
    /// Returns ProcessError if killing fails
    fn kill_child(&self, child: &mut Child) -> Result<(), ProcessError>;
}

/// Real process executor implementation that actually This executor provides manages child processes.
///
/// the concrete implementation for spawning, waiting,
/// and killing processes in the host system.
#[derive(Debug, Clone, Default)]
pub struct RealProcessExecutor;

impl RealProcessExecutor {
    /// Create a new RealProcessExecutor
    #[must_use]
    pub fn new() -> Self {
        RealProcessExecutor
    }
}

impl ProcessExecutorTrait for RealProcessExecutor {
    fn spawn_child(&self, mut cmd: Command) -> Result<Child, ProcessError> {
        // Configure the command to spawn with piped stdin/stdout/stderr
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        cmd.spawn().map_err(|e| {
            let program = cmd.get_program().to_string_lossy().to_string();
            if e.kind() == std::io::ErrorKind::NotFound {
                ProcessError::ProgramNotFound {
                    program,
                    suggestion: "Ensure the program is installed and available in PATH".to_string(),
                }
            } else {
                ProcessError::IoError {
                    error_details: e.to_string(),
                    suggestion: "Check file permissions and system configuration".to_string(),
                }
            }
        })
    }

    fn wait_child(&self, child: &mut Child) -> Result<ExitStatus, ProcessError> {
        child.wait().map_err(|e| ProcessError::IoError {
            error_details: e.to_string(),
            suggestion: "Failed to wait for process".to_string(),
        })
    }

    fn kill_child(&self, child: &mut Child) -> Result<(), ProcessError> {
        // First try to kill the process
        match child.kill() {
            Ok(()) => {
                // Wait for the process to actually terminate after being killed
                let _ = child.wait();
                Ok(())
            }
            Err(e) => {
                // Check if the process already exited
                if e.kind() == std::io::ErrorKind::InvalidInput {
                    // Process already exited, which is fine
                    return Ok(());
                }
                let program = "unknown".to_string(); // We don't track the program name here
                Err(ProcessError::KillFailed {
                    program,
                    error_details: e.to_string(),
                    suggestion: "Ensure you have permission to kill the process".to_string(),
                })
            }
        }
    }
}

/// Configuration for MockProcessExecutor
#[derive(Debug, Clone)]
pub struct MockConfig {
    /// If set, spawn will return this error
    pub spawn_error: Option<ProcessError>,
    /// If set, wait will return this exit status
    pub wait_status: Option<std::process::ExitStatus>,
    /// If set, wait will return this error
    pub wait_error: Option<ProcessError>,
    /// If set, kill will return this error
    pub kill_error: Option<ProcessError>,
}

impl Default for MockConfig {
    fn default() -> Self {
        MockConfig {
            spawn_error: None,
            wait_status: Some(create_exit_status(0)),
            wait_error: None,
            kill_error: None,
        }
    }
}

/// Mock process executor for testing.
///
/// This executor allows simulating successful exits, errors, and other
/// conditions without actually executing external processes.
///
/// # Example
///
/// ```ignore
/// use crate::traits::process::{ProcessExecutorTrait, MockProcessExecutor};
/// use std::process::Command;
///
/// // Create a mock that always succeeds
/// let mock = MockProcessExecutor::with_success();
///
/// // Or create a mock with failure exit code
/// let mock = MockProcessExecutor::with_failure(1);
/// ```
#[derive(Debug, Clone)]
pub struct MockProcessExecutor {
    config: MockConfig,
}

impl MockProcessExecutor {
    /// Create a new MockProcessExecutor with default configuration (success)
    #[must_use]
    pub fn new() -> Self {
        MockProcessExecutor::default()
    }

    /// Create a mock that simulates a successful exit
    #[must_use]
    pub fn with_success() -> Self {
        MockProcessExecutor {
            config: MockConfig::default(),
        }
    }

    /// Create a mock that simulates a failed exit
    #[must_use]
    pub fn with_failure(exit_code: i32) -> Self {
        MockProcessExecutor {
            config: MockConfig {
                wait_status: Some(create_exit_status(exit_code)),
                ..Default::default()
            },
        }
    }

    /// Create a mock that simulates a spawn error
    #[must_use]
    pub fn with_spawn_error(error: ProcessError) -> Self {
        MockProcessExecutor {
            config: MockConfig {
                spawn_error: Some(error),
                ..Default::default()
            },
        }
    }

    /// Create a mock that simulates a wait error
    #[must_use]
    pub fn with_wait_error(error: ProcessError) -> Self {
        MockProcessExecutor {
            config: MockConfig {
                wait_error: Some(error),
                ..Default::default()
            },
        }
    }

    /// Create a mock that simulates a kill error
    #[must_use]
    pub fn with_kill_error(error: ProcessError) -> Self {
        MockProcessExecutor {
            config: MockConfig {
                kill_error: Some(error),
                ..Default::default()
            },
        }
    }

    /// Create a mock with custom configuration
    #[must_use]
    pub fn with_config(config: MockConfig) -> Self {
        MockProcessExecutor { config }
    }

    /// Get a reference to the configuration
    #[must_use]
    pub fn config(&self) -> &MockConfig {
        &self.config
    }
}

impl Default for MockProcessExecutor {
    fn default() -> Self {
        Self::with_success()
    }
}

// Implement a mock Child type for testing
// This is a simple wrapper that simulates process behavior
#[derive(Debug)]
pub struct MockChild {
    pub pid: u32,
    exited: bool,
    exit_status: Option<ExitStatus>,
}

impl MockChild {
    /// Create a new MockChild
    #[must_use]
    pub fn new(pid: u32) -> Self {
        MockChild {
            pid,
            exited: false,
            exit_status: None,
        }
    }

    /// Create a new MockChild with a specific exit status
    #[must_use]
    pub fn with_exit_status(pid: u32, status: ExitStatus) -> Self {
        MockChild {
            pid,
            exited: true,
            exit_status: Some(status),
        }
    }

    /// Check if the child has exited
    pub fn exited(&self) -> bool {
        self.exited
    }

    /// Take the exit status (returns None if not exited)
    pub fn take_exit_status(&mut self) -> Option<ExitStatus> {
        self.exited = true;
        self.exit_status.take()
    }
}

impl ProcessExecutorTrait for MockProcessExecutor {
    fn spawn_child(&self, _cmd: Command) -> Result<Child, ProcessError> {
        // Check if we should return an error
        if let Some(ref error) = self.config.spawn_error {
            return Err(error.clone());
        }

        // For the mock, we can't return a real Child, so we need a different approach
        // We'll use a trick: spawn a dummy process that does nothing
        // Actually, let's use a different approach - use a no-op command
        let mut cmd = Command::new("true"); // true command does nothing and exits successfully
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        cmd.spawn().map_err(|e| ProcessError::IoError {
            error_details: e.to_string(),
            suggestion: "Mock spawn failed".to_string(),
        })
    }

    fn wait_child(&self, child: &mut Child) -> Result<ExitStatus, ProcessError> {
        // Check if we should return an error
        if let Some(ref error) = self.config.wait_error {
            return Err(error.clone());
        }

        // Check if we have a configured wait status
        if let Some(ref status) = self.config.wait_status {
            return Ok(*status);
        }

        // Use the real wait
        child.wait().map_err(|e| ProcessError::IoError {
            error_details: e.to_string(),
            suggestion: "Mock wait failed".to_string(),
        })
    }

    fn kill_child(&self, child: &mut Child) -> Result<(), ProcessError> {
        // Check if we should return an error
        if let Some(ref error) = self.config.kill_error {
            return Err(error.clone());
        }

        // Try to kill, but ignore errors (process might already be gone)
        let _ = child.kill();
        let _ = child.wait();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// Test that MockProcessExecutor can simulate a successful exit
    #[test]
    fn test_mock_process_executor_success() {
        let mock = MockProcessExecutor::with_success();

        // Spawn should succeed
        let result = mock.spawn_child(Command::new("true"));
        assert!(result.is_ok(), "Spawn should succeed: {:?}", result.err());

        let mut child = result.unwrap();

        // Wait should return success status
        let status = mock.wait_child(&mut child);
        assert!(status.is_ok(), "Wait should succeed: {:?}", status.err());
        assert!(status.unwrap().success(), "Exit status should be success");
    }

    /// Test that MockProcessExecutor can simulate a failed exit
    #[test]
    fn test_mock_process_executor_failure() {
        let mock = MockProcessExecutor::with_failure(1);

        // Spawn should succeed
        let result = mock.spawn_child(Command::new("true"));
        assert!(result.is_ok(), "Spawn should succeed: {:?}", result.err());

        let mut child = result.unwrap();

        // Wait should return failure status
        let status = mock.wait_child(&mut child);
        assert!(status.is_ok(), "Wait should succeed: {:?}", status.err());
        assert!(!status.unwrap().success(), "Exit status should be failure");
    }

    /// Test that MockProcessExecutor can simulate error conditions
    #[test]
    fn test_mock_process_executor_spawn_error() {
        let error = ProcessError::ProgramNotFound {
            program: "nonexistent".to_string(),
            suggestion: "Install the program".to_string(),
        };
        let mock = MockProcessExecutor::with_spawn_error(error.clone());

        // Spawn should fail with the configured error
        let result = mock.spawn_child(Command::new("nonexistent"));
        assert!(result.is_err(), "Spawn should fail");
        assert_eq!(result.unwrap_err(), error);
    }

    /// Test that MockProcessExecutor can simulate wait errors
    #[test]
    fn test_mock_process_executor_wait_error() {
        let error = ProcessError::IoError {
            error_details: "Test error".to_string(),
            suggestion: "Test suggestion".to_string(),
        };
        let mock = MockProcessExecutor::with_wait_error(error.clone());

        // Spawn should succeed (no spawn error configured)
        let result = mock.spawn_child(Command::new("true"));
        assert!(result.is_ok());

        let mut child = result.unwrap();

        // Wait should fail with the configured error
        let result = mock.wait_child(&mut child);
        assert!(result.is_err(), "Wait should fail");
        assert_eq!(result.unwrap_err(), error);
    }

    /// Test that waiting on an already-exited process returns ProcessAlreadyExited error
    #[tokio::test]
    async fn test_mock_process_executor_wait_already_exited() {
        let exit_status = create_exit_status(0);
        let error = ProcessError::ProcessAlreadyExited {
            program: "test-program".to_string(),
            exit_status,
        };
        let mock = MockProcessExecutor::with_wait_error(error.clone());

        // Spawn should succeed (no spawn error configured)
        let result = mock.spawn_child(Command::new("true"));
        assert!(result.is_ok(), "Spawn should succeed: {:?}", result.err());

        let mut child = result.unwrap();

        // Wait should fail with ProcessAlreadyExited error
        let result = mock.wait_child(&mut child);
        assert!(
            result.is_err(),
            "Wait should fail when process already exited"
        );
        let actual_error = result.unwrap_err();

        // Verify it's the correct error type
        match actual_error {
            ProcessError::ProcessAlreadyExited {
                program,
                exit_status: _,
            } => {
                assert_eq!(program, "test-program", "Program name should match");
            }
            _ => panic!(
                "Expected ProcessAlreadyExited error, got: {:?}",
                actual_error
            ),
        }
    }

    /// Test that MockProcessExecutor can simulate kill errors
    #[test]
    fn test_mock_process_executor_kill_error() {
        let error = ProcessError::KillFailed {
            program: "test".to_string(),
            error_details: "Permission denied".to_string(),
            suggestion: "Check permissions".to_string(),
        };
        let mock = MockProcessExecutor::with_kill_error(error.clone());

        // Spawn should succeed
        let mut cmd = Command::new("sleep");
        cmd.arg("10");
        let result = mock.spawn_child(cmd);
        assert!(result.is_ok());

        let mut child = result.unwrap();

        // Kill should fail with the configured error
        let result = mock.kill_child(&mut child);
        assert!(result.is_err(), "Kill should fail");
        assert_eq!(result.unwrap_err(), error);
    }

    /// Test RealProcessExecutor with a simple command
    #[test]
    fn test_real_process_executor() {
        let executor = RealProcessExecutor::new();

        let mut cmd = Command::new("echo");
        cmd.arg("hello");

        let result = executor.spawn_child(cmd);
        assert!(result.is_ok(), "Spawn should succeed: {:?}", result.err());

        let mut child = result.unwrap();

        let status = executor.wait_child(&mut child);
        assert!(status.is_ok(), "Wait should succeed: {:?}", status.err());
        // echo exits with 0
        assert!(status.unwrap().success());
    }

    /// Test that ProcessError Display implementation works
    #[test]
    fn test_process_error_display() {
        let error = ProcessError::ProgramNotFound {
            program: "test-program".to_string(),
            suggestion: "Install it".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("test-program"));
        assert!(display.contains("Install it"));
    }

    /// Test that ProcessError implements Error trait
    #[test]
    fn test_process_error_is_error() {
        fn check_error<T: std::error::Error>() {}
        check_error::<ProcessError>();
    }
}
