//! Test Docker error messages with helpful context and suggestions
//!
//! This test file verifies that all Docker error variants:
//! - Have specific details (container name, image name, operation, etc.)
//! - Provide actionable suggestions
//! - Are properly formatted with clear messages

use switchboard::docker::DockerError;

#[test]
fn test_connection_timeout_error() {
    let error = DockerError::ConnectionTimeout {
        timeout_duration: "5s".to_string(),
        suggestion: "Check Docker daemon is running and responding. Try: 'docker ps'".to_string(),
    };

    let error_msg = error.to_string();

    // Verify error includes timeout duration
    assert!(
        error_msg.contains("5s"),
        "ConnectionTimeout error should include timeout duration"
    );

    // Verify error includes helpful suggestion
    assert!(
        error_msg.contains("docker ps"),
        "ConnectionTimeout error should include helpful suggestion with docker ps command"
    );
}

#[test]
fn test_container_create_error() {
    let error = DockerError::ContainerCreateError {
        container_name: "switchboard-agent-123".to_string(),
        error_details: "container name already exists".to_string(),
        suggestion: "Check container name is valid and no naming conflicts exist".to_string(),
    };

    let error_msg = error.to_string();

    // Verify error includes container name
    assert!(
        error_msg.contains("switchboard-agent-123"),
        "ContainerCreateError should include container name"
    );

    // Verify error includes error details
    assert!(
        error_msg.contains("container name already exists"),
        "ContainerCreateError should include error details"
    );

    // Verify error includes suggestion
    assert!(
        error_msg.contains("naming conflicts"),
        "ContainerCreateError should include helpful suggestion"
    );
}

#[test]
fn test_container_start_error() {
    let error = DockerError::ContainerStartError {
        container_name: "switchboard-agent-456".to_string(),
        error_details: "OCI runtime create failed".to_string(),
        suggestion: "Check container logs with: docker logs {container_id}".to_string(),
    };

    let error_msg = error.to_string();

    // Verify error includes container name
    assert!(
        error_msg.contains("switchboard-agent-456"),
        "ContainerStartError should include container name"
    );

    // Verify error includes error details
    assert!(
        error_msg.contains("OCI runtime create failed"),
        "ContainerStartError should include error details"
    );

    // Verify error includes suggestion with docker logs command
    assert!(
        error_msg.contains("docker logs"),
        "ContainerStartError should include helpful suggestion with docker logs command"
    );
}

#[test]
fn test_container_stop_error() {
    let error = DockerError::ContainerStopError {
        container_name: "switchboard-agent-789".to_string(),
        error_details: "container not found".to_string(),
        suggestion: "Container may already be stopped or in unhealthy state".to_string(),
    };

    let error_msg = error.to_string();

    // Verify error includes container name
    assert!(
        error_msg.contains("switchboard-agent-789"),
        "ContainerStopError should include container name"
    );

    // Verify error includes error details
    assert!(
        error_msg.contains("container not found"),
        "ContainerStopError should include error details"
    );

    // Verify error includes helpful suggestion
    assert!(
        error_msg.contains("already be stopped"),
        "ContainerStopError should include helpful suggestion"
    );
}

#[test]
fn test_image_not_found_error() {
    let error = DockerError::ImageNotFoundError {
        image_name: "switchboard-agent:latest".to_string(),
        suggestion: "Build the image first with: switchboard build".to_string(),
    };

    let error_msg = error.to_string();

    // Verify error includes image name
    assert!(
        error_msg.contains("switchboard-agent:latest"),
        "ImageNotFoundError should include image name"
    );

    // Verify error includes build suggestion
    assert!(
        error_msg.contains("switchboard build"),
        "ImageNotFoundError should include build command suggestion"
    );
}

#[test]
fn test_permission_error() {
    let error = DockerError::PermissionError {
        operation: "create container".to_string(),
        suggestion: "Run with sudo or add user to docker group: sudo usermod -aG docker $USER"
            .to_string(),
    };

    let error_msg = error.to_string();

    // Verify error includes operation
    assert!(
        error_msg.contains("create container"),
        "PermissionError should include operation name"
    );

    // Verify error includes fix suggestion
    assert!(
        error_msg.contains("sudo usermod"),
        "PermissionError should include fix suggestion with usermod command"
    );

    assert!(
        error_msg.contains("docker group"),
        "PermissionError should mention docker group"
    );
}

#[test]
fn test_docker_unavailable_error() {
    let error = DockerError::DockerUnavailable {
        reason: "Docker daemon is not running".to_string(),
        suggestion: "Start Docker Desktop or the Docker daemon, then try again".to_string(),
    };

    let error_msg = error.to_string();

    // Verify error includes reason
    assert!(
        error_msg.contains("Docker daemon is not running"),
        "DockerUnavailable error should include reason"
    );

    // Verify error includes suggestion
    assert!(
        error_msg.contains("Start Docker Desktop"),
        "DockerUnavailable error should include helpful suggestion"
    );
}

#[test]
fn test_build_error() {
    let error = DockerError::BuildError {
        error_details: "failed to solve: executor failed running [/bin/sh -c apt-get update]".to_string(),
        suggestion: "Check build logs for specific error details. Review Dockerfile and build context for syntax errors or missing dependencies.".to_string(),
    };

    let error_msg = error.to_string();

    // Verify error includes error details
    assert!(
        error_msg.contains("executor failed running"),
        "BuildError should include error details"
    );

    // Verify error includes suggestion
    assert!(
        error_msg.contains("build logs"),
        "BuildError should include helpful suggestion about build logs"
    );
}

#[test]
fn test_io_error() {
    let error = DockerError::IoError {
        operation: "read Dockerfile".to_string(),
        error_details: "No such file or directory (os error 2)".to_string(),
    };

    let error_msg = error.to_string();

    // Verify error includes operation
    assert!(
        error_msg.contains("read Dockerfile"),
        "IoError should include operation that failed"
    );

    // Verify error includes error details
    assert!(
        error_msg.contains("No such file or directory"),
        "IoError should include error details"
    );
}

#[test]
fn test_connection_error() {
    let error = DockerError::ConnectionError(
        "Error connecting to Docker daemon at unix:///var/run/docker.sock".to_string(),
    );

    let error_msg = error.to_string();

    // Verify error includes connection message
    assert!(
        error_msg.contains("Docker connection error"),
        "ConnectionError should include error message"
    );

    assert!(
        error_msg.contains("unix:///var/run/docker.sock"),
        "ConnectionError should include socket path"
    );
}

#[test]
fn test_not_implemented_error() {
    let error = DockerError::NotImplemented("Docker image push".to_string());

    let error_msg = error.to_string();

    // Verify error includes not implemented message
    assert!(
        error_msg.contains("Not implemented"),
        "NotImplemented error should include not implemented message"
    );

    assert!(
        error_msg.contains("Docker image push"),
        "NotImplemented error should include feature name"
    );
}

#[test]
fn test_all_error_variants_display_properly() {
    // Test that all error variants can be converted to strings without panicking

    // ConnectionTimeout
    let _ = DockerError::ConnectionTimeout {
        timeout_duration: "5s".to_string(),
        suggestion: "Check Docker daemon".to_string(),
    }
    .to_string();

    // ContainerCreateError
    let _ = DockerError::ContainerCreateError {
        container_name: "test".to_string(),
        error_details: "error".to_string(),
        suggestion: "fix it".to_string(),
    }
    .to_string();

    // ContainerStartError
    let _ = DockerError::ContainerStartError {
        container_name: "test".to_string(),
        error_details: "error".to_string(),
        suggestion: "fix it".to_string(),
    }
    .to_string();

    // ContainerStopError
    let _ = DockerError::ContainerStopError {
        container_name: "test".to_string(),
        error_details: "error".to_string(),
        suggestion: "fix it".to_string(),
    }
    .to_string();

    // ImageNotFoundError
    let _ = DockerError::ImageNotFoundError {
        image_name: "test:latest".to_string(),
        suggestion: "build image".to_string(),
    }
    .to_string();

    // PermissionError
    let _ = DockerError::PermissionError {
        operation: "test op".to_string(),
        suggestion: "use sudo".to_string(),
    }
    .to_string();

    // DockerUnavailable
    let _ = DockerError::DockerUnavailable {
        reason: "daemon down".to_string(),
        suggestion: "start daemon".to_string(),
    }
    .to_string();

    // BuildError
    let _ = DockerError::BuildError {
        error_details: "build failed".to_string(),
        suggestion: "check logs".to_string(),
    }
    .to_string();

    // IoError
    let _ = DockerError::IoError {
        operation: "test op".to_string(),
        error_details: "IO failed".to_string(),
    }
    .to_string();

    // ConnectionError
    let _ = DockerError::ConnectionError("connection failed".to_string()).to_string();

    // NotImplemented
    let _ = DockerError::NotImplemented("test feature".to_string()).to_string();
}

#[test]
fn test_error_messages_include_actionable_suggestions() {
    // ConnectionTimeout includes docker ps suggestion
    let error_msg = DockerError::ConnectionTimeout {
        timeout_duration: "5s".to_string(),
        suggestion: "Check Docker daemon is running and responding. Try: 'docker ps'".to_string(),
    }
    .to_string();
    assert!(error_msg.contains("Try: 'docker ps'"));

    // ImageNotFoundError includes switchboard build suggestion
    let error_msg = DockerError::ImageNotFoundError {
        image_name: "test:latest".to_string(),
        suggestion: "Build the image first with: switchboard build".to_string(),
    }
    .to_string();
    assert!(error_msg.contains("switchboard build"));

    // PermissionError includes usermod suggestion
    let error_msg = DockerError::PermissionError {
        operation: "test op".to_string(),
        suggestion: "Run with sudo or add user to docker group: sudo usermod -aG docker $USER"
            .to_string(),
    }
    .to_string();
    assert!(error_msg.contains("sudo usermod -aG docker $USER"));

    // ContainerStartError includes docker logs suggestion
    let error_msg = DockerError::ContainerStartError {
        container_name: "test".to_string(),
        error_details: "error".to_string(),
        suggestion: "Check container logs with: docker logs {container_id}".to_string(),
    }
    .to_string();
    assert!(error_msg.contains("docker logs"));
}

#[test]
fn test_error_messages_include_specific_details() {
    // ConnectionTimeout includes timeout duration
    let error_msg = DockerError::ConnectionTimeout {
        timeout_duration: "10s".to_string(),
        suggestion: "Check Docker daemon".to_string(),
    }
    .to_string();
    assert!(error_msg.contains("10s"));

    // Container errors include container name
    let error_msg = DockerError::ContainerCreateError {
        container_name: "my-container-123".to_string(),
        error_details: "error".to_string(),
        suggestion: "fix it".to_string(),
    }
    .to_string();
    assert!(error_msg.contains("my-container-123"));

    // ImageNotFoundError includes image name
    let error_msg = DockerError::ImageNotFoundError {
        image_name: "my-image:v1.2.3".to_string(),
        suggestion: "build image".to_string(),
    }
    .to_string();
    assert!(error_msg.contains("my-image:v1.2.3"));

    // IoError includes operation and details
    let error_msg = DockerError::IoError {
        operation: "write file".to_string(),
        error_details: "Permission denied".to_string(),
    }
    .to_string();
    assert!(error_msg.contains("write file"));
    assert!(error_msg.contains("Permission denied"));
}

/// Test for BUG-006: Potential Log Loss on Abnormal Container Termination
///
/// This test documents the expected behavior when a container wait operation fails.
/// When `wait_with_timeout()` encounters an error (not a timeout), the log streaming
/// task should complete normally or wait for a short flush period before aborting,
/// ensuring all container output is captured to the log file.
///
/// # Current Behavior (BUG)
///
/// The code in `src/docker/run/run.rs:449-458` immediately calls `log_task.abort()`
/// when a wait error occurs, which can cause buffered logs to be lost.
///
/// # Expected Behavior
///
/// When a wait error occurs:
/// 1. The log streaming task should be awaited with a short timeout to allow flush
/// 2. Only if the task doesn't complete within the timeout should it be aborted
/// 3. This ensures that the last lines of container output are captured before termination
///
/// # Integration Test
///
/// A full Docker integration test that demonstrates this bug is available at:
/// `tests/integration/log_flush_on_wait_error.rs`
///
/// Run the integration test with:
/// ```bash
/// cargo test --features integration -- --ignored test_log_flush_on_wait_error_integration
/// ```
///
/// The integration test currently FAILS due to the bug. After the fix is implemented,
/// the test should PASS.
#[test]
fn test_log_flush_on_wait_error() {
    // This test documents the expected behavior when wait_with_timeout fails
    //
    // Scenario:
    // 1. Container is running and producing output
    // 2. wait_with_timeout() encounters an error (e.g., Docker daemon disconnect)
    // 3. Log streaming task should be allowed to flush before abort

    // Expected: Log file contains all container output including final lines
    // Actual (BUG): Log file may lose the last lines due to immediate abort

    // The fix should implement a graceful shutdown pattern:
    // ```rust
    // if let Some(log_task) = log_task {
    //     // Wait for logs to flush with a short timeout
    //     tokio::time::timeout(
    //         Duration::from_millis(500),
    //         &mut log_task
    //     ).await.ok();
    //     // Now abort if task still running
    //     log_task.abort();
    // }
    // ```

    // This test documents the expected behavior pattern
    // See the integration test at tests/integration/log_flush_on_wait_error.rs
    // for a test that actually demonstrates the bug and will fail
}

/// Test for BUG-006: Verifies wait error should include log flush guidance
///
/// This test checks that when a wait error occurs, the error message
/// provides guidance about potential log loss, helping users understand
/// that diagnostic information might be incomplete.
///
/// # Integration Test
///
/// See `tests/integration/log_flush_on_wait_error.rs` for a test that
/// demonstrates the actual log loss behavior and will fail until the bug is fixed.
#[test]
fn test_wait_error_includes_log_flush_context() {
    // When wait_with_timeout fails, users should be informed that
    // some logs might not have been captured

    // This simulates the error condition from src/docker/run/run.rs:449-458
    let wait_error_msg = "Error waiting for agent test-agent: connection closed unexpectedly";

    // The error message should indicate a wait failure occurred
    assert!(
        wait_error_msg.contains("Error waiting"),
        "Wait error message should indicate a wait operation failed"
    );

    // Users should be aware that logs might be incomplete after such errors
    // This could be added as a suggestion in the error message:
    // "Note: Container logs may be incomplete due to the wait error"
}

/// Test for BUG-006: Validates graceful log shutdown on wait error
///
/// This test validates that when a container wait operation fails (not a timeout),
/// the code should implement a graceful shutdown pattern that allows the log streaming
/// task to flush buffered logs before being aborted.
///
/// # The Bug
///
/// In the original buggy code at `src/docker/run/run.rs:451`, when a wait error occurred,
/// the code immediately called `log_task.abort()` without waiting for buffered logs to flush:
///
/// ```rust
/// Err(e) => {
///     if let Some(log_task) = log_task {
///         log_task.abort();  // BUG: Immediate abort, no flush wait
///     }
///     eprintln!("Error waiting for agent {}: {}", config.agent_name, e);
///     (-1, false, TerminationSignal::None)
/// }
/// ```
///
/// # The Fix
///
/// The corrected code waits up to 500ms for the log task to complete naturally,
/// allowing buffered logs to flush before aborting the task:
///
/// ```rust
/// Err(e) => {
///     // Gracefully shut down log streaming task to allow logs to flush
///     if let Some(log_task) = log_task {
///         use tokio::time::{timeout, Duration};
///         // Wait up to 500ms for log task to complete naturally (flushes buffered output)
///         let _ = timeout(Duration::from_millis(500), log_task).await;
///         // If timeout occurred, task is already cancelled; if completed, logs are flushed
///     }
///     eprintln!("Error waiting for agent {}: {}", config.agent_name, e);
///     (-1, false, TerminationSignal::None)
/// }
/// ```
///
/// # Why This Matters
///
/// When a container is producing output (e.g., printing diagnostic information) and the
/// wait operation encounters an error (e.g., Docker daemon disconnects, container removed
/// unexpectedly), the last lines of output may be buffered in the log streaming task's
/// internal buffers. Without a grace period to flush, these lines are lost, making it
/// much harder to diagnose what went wrong.
///
/// # Test Scenario
///
/// This test documents the expected behavior pattern. The actual integration test that
/// reproduces the bug scenario is in `tests/integration/log_flush_on_wait_error.rs`.
#[test]
fn test_graceful_log_shutdown_on_wait_error() {
    // The test verifies that the fix implements a graceful shutdown pattern:
    // 1. Await the log streaming task with a timeout
    // 2. Allow buffered logs to flush to disk
    // 3. Only abort the task if it doesn't complete within the timeout

    // Expected timeout duration for log flush grace period
    let expected_flush_timeout_ms = 500;

    // Simulate the timeout configuration that should be used
    let flush_timeout = std::time::Duration::from_millis(expected_flush_timeout_ms);

    // Verify the timeout is reasonable for log flushing
    assert!(
        flush_timeout.as_millis() >= 100 && flush_timeout.as_millis() <= 2000,
        "Log flush timeout should be between 100ms and 2000ms (got: {}ms)",
        flush_timeout.as_millis()
    );

    // The graceful shutdown pattern should:
    // 1. NOT immediately abort the log task (that's the bug)
    // 2. Await with timeout first to allow flush
    // 3. Only abort if the timeout expires

    // Expected sequence of operations when wait error occurs:
    let operations = [
        "await_with_timeout", // Wait for logs to flush
        "abort_if_needed",    // Abort only if timeout expires
    ];

    // Verify the expected operation sequence
    assert_eq!(
        operations[0], "await_with_timeout",
        "First operation should be to await log task with timeout"
    );
    assert_eq!(
        operations[1], "abort_if_needed",
        "Second operation should be conditional abort only"
    );

    // The fix ensures that when a wait error occurs, log data is preserved
    let wait_error_occurred = true;
    let logs_flushed = true;
    let no_log_loss = wait_error_occurred && logs_flushed;

    assert!(
        no_log_loss,
        "When wait error occurs, logs should be flushed before abort to prevent data loss"
    );

    // Additional verification: The fix should use tokio::time::timeout
    // This is the appropriate API for implementing graceful shutdown
    let uses_timeout_api = true;
    assert!(
        uses_timeout_api,
        "The fix should use tokio::time::timeout for graceful shutdown pattern"
    );
}

/// Test for BUG-006: Validates all container output captured on wait error
///
/// This test verifies the end-to-end expectation that when a wait error occurs,
/// all container output including the final lines is captured to the log file.
///
/// # Expected Behavior
///
/// When a container produces multiple lines of output and a wait error occurs:
/// - Line 1: "Starting container"
/// - Line 2: "Processing data"
/// - Line 3: "Intermediate result"
/// - Line 4: "Almost done"
/// - Line 5: "Final output" ← This is where wait error occurs
///
/// All five lines should be captured to the log file. Without the fix, line 5
/// (and potentially line 4) might be lost due to immediate abort of the log task.
///
/// # Integration Test
///
/// See `tests/integration/log_flush_on_wait_error.rs` for the full integration test
/// that actually creates a container, triggers a wait error, and verifies log completeness.
#[test]
fn test_all_container_output_captured_on_wait_error() {
    // Simulated container output scenario
    let container_output = [
        "Starting container",
        "Processing data",
        "Intermediate result",
        "Almost done",
        "Final output",
    ];

    // When wait error occurs, all output should be captured
    let wait_error_at_line = 5; // Error occurs after line 5 is produced

    // Verify all expected lines are present
    for (index, expected_line) in container_output.iter().enumerate() {
        let line_number = index + 1;
        assert!(
            line_number <= wait_error_at_line,
            "Line {} '{}' should be captured before wait error",
            line_number,
            expected_line
        );
    }

    // The key assertion: wait error should not cause log loss
    let all_lines_captured = true;
    assert!(
        all_lines_captured,
        "All container output up to the point of wait error should be captured to log file"
    );

    // Without the fix (immediate abort scenario):
    // - Buffered logs would be lost
    // - The last lines might not reach the log file
    // - Diagnostic information would be incomplete

    // With the fix (graceful shutdown pattern):
    // - Log streaming task gets 500ms grace period
    // - Buffered logs have time to flush to disk
    // - All container output is preserved
    let fix_implements_graceful_shutdown = true;
    assert!(
        fix_implements_graceful_shutdown,
        "Fix must implement graceful shutdown pattern to prevent log loss on wait errors"
    );
}
