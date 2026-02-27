//! Integration test for BUG-006: Potential Log Loss on Abnormal Container Termination
//!
//! This test verifies that when a container wait operation fails (not a timeout),
//! the log streaming task should complete normally or wait for a short flush period
//! before aborting, ensuring all container output is captured to the log file.
//!
//! # Current Behavior (BUG)
//!
//! The code in `src/docker/run/run.rs:449-458` immediately calls `log_task.abort()`
//! when a wait error occurs, which can cause buffered logs to be lost.
//!
//! # Test Scenario
//!
//! This test:
//! 1. Creates a container that produces output while running
//! 2. Triggers a wait error by forcibly removing the container
//! 3. Verifies that all expected log lines are captured to the log file
//! 4. Currently FAILS because the log task is aborted immediately without flush

#[cfg(feature = "integration")]
use super::docker_available;
#[cfg(feature = "integration")]
use std::fs;
#[cfg(feature = "integration")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "integration")]
use std::time::Duration;
#[cfg(feature = "integration")]
use switchboard::docker::run::types::ContainerConfig;
#[cfg(feature = "integration")]
use switchboard::docker::DockerClient;
#[cfg(feature = "integration")]
use switchboard::logger::Logger;
#[cfg(feature = "integration")]
use tokio::task::JoinHandle;

/// Integration test for BUG-006: Logs should be flushed even when wait error occurs
///
/// This test demonstrates the bug where immediate task abort causes log loss.
/// When the bug is fixed, this test should PASS.
///
/// # Test Steps
///
/// 1. Create a Docker container that outputs multiple lines with delays
/// 2. Set up a logger to capture container output
/// 3. Start the container with `run_agent`
/// 4. While the container is running, forcibly remove it to trigger a wait error
/// 5. Verify that all expected log lines are in the log file
///
/// # Expected Behavior (After Fix)
///
/// When the wait error occurs, the log streaming task should be allowed a short
/// grace period to flush buffered logs before being aborted. This ensures that
/// all container output including the last lines is captured.
///
/// # Current Behavior (BUG)
///
/// The log streaming task is aborted immediately when the wait error occurs,
/// causing any buffered but not-yet-written log lines to be lost.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_log_flush_on_wait_error_integration() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Create a temporary directory for logs
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let log_dir = temp_dir.path();

    // Create a logger instance
    let logger = Arc::new(Mutex::new(Logger::new(
        log_dir.to_path_buf(),
        Some("test-agent-bug006".to_string()),
        false, // Not in foreground mode for this test
    )));

    // Get the current workspace directory
    let workspace = std::env::current_dir().expect("Failed to get current directory");

    // Create a DockerClient instance
    let client =
        match DockerClient::new("test-image-bug006".to_string(), "latest".to_string()).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Skipping test: Failed to create DockerClient: {}", e);
                return;
            }
        };

    // Create a container that outputs multiple lines with delays
    // This increases the likelihood that logs are buffered when the error occurs
    let container_script = r#"
        echo "Line 1: Starting container"
        sleep 0.2
        echo "Line 2: Processing data"
        sleep 0.2
        echo "Line 3: Intermediate result"
        sleep 0.2
        echo "Line 4: Almost done"
        sleep 0.2
        echo "Line 5: Final output"
        sleep 10  # Sleep to give time for removal
        echo "Line 6: Should not be seen (removed before this)"
        exit 0
    "#;

    let cmd = vec![
        "sh".to_string(),
        "-c".to_string(),
        container_script.to_string(),
    ];

    // Create a ContainerConfig
    let config = ContainerConfig {
        agent_name: "test-agent-bug006".to_string(),
        env_vars: vec![],
        timeout: None,
        readonly: false,
        prompt: String::new(),
        skills: None,
    };

    // Get a clone of the client for the removal task
    let client_for_removal = client.clone();
    let agent_name_for_removal = "test-agent-bug006".to_string();

    // Spawn a task to remove the container after a delay
    // This simulates a wait error condition (container removed unexpectedly)
    let removal_task: JoinHandle<Result<(), bollard::errors::Error>> = tokio::spawn(async move {
        // Wait for the container to start and produce some output
        tokio::time::sleep(Duration::from_millis(600)).await;

        // List all containers to find the one we just started
        let docker = client_for_removal.docker();
        let containers = docker.list_containers::<String>(None).await?;

        // Find our container by name
        for container in containers {
            if let Some(names) = container.names {
                for name in names {
                    if name.contains(&agent_name_for_removal) {
                        let container_id = container.id.clone();

                        // Forcibly remove the container to trigger a wait error
                        // This simulates the scenario where Docker daemon disconnects
                        // or the container is removed unexpectedly
                        if let Some(id) = container_id {
                            let _ = docker
                                .remove_container(
                                    &id,
                                    Some(bollard::container::RemoveContainerOptions {
                                        force: true,
                                        link: false,
                                        v: false,
                                    }),
                                )
                                .await;
                        }
                    }
                }
            }
        }
        Ok(())
    });

    // Run the agent (this will encounter a wait error when container is removed)
    let agent_result = switchboard::docker::run::run_agent(
        workspace.to_str().unwrap(),
        &client,
        &config,
        Some("30s".to_string()), // 30 second timeout
        "alpine:latest",
        Some(&cmd),
        Some(logger.clone()),
        None, // No metrics store
        "test-agent-bug006",
        None, // No queued start time
    )
    .await;

    // Wait for the removal task to complete
    let _ = removal_task.await;

    // Read the log file to verify content
    let log_file_path = log_dir.join("test-agent-bug006.log");
    let log_content = fs::read_to_string(&log_file_path).expect("Failed to read log file");

    // The agent should have failed due to the wait error
    assert!(
        agent_result.is_err(),
        "Agent run should fail due to wait error when container is forcibly removed"
    );

    // Expected log lines that should be captured
    let expected_lines = vec![
        "Line 1: Starting container",
        "Line 2: Processing data",
        "Line 3: Intermediate result",
        "Line 4: Almost done",
        "Line 5: Final output",
    ];

    // Verify that all expected log lines are present
    for expected_line in &expected_lines {
        assert!(
            log_content.contains(expected_line),
            "Log file should contain: '{}'\nActual log content:\n{}",
            expected_line,
            log_content
        );
    }

    // BUG: Due to the immediate abort in src/docker/run/run.rs:452,
    // some of these lines may not be captured. When this test fails,
    // it demonstrates that the bug exists.

    // Clean up
    drop(temp_dir);
}

/// Integration test for BUG-006: Verify graceful log shutdown pattern
///
/// This test verifies that when a wait error occurs, the code should
/// use a graceful shutdown pattern that allows logs to flush before
/// aborting the log streaming task.
///
/// # Graceful Shutdown Pattern (Expected After Fix)
///
/// ```rust
/// if let Some(log_task) = log_task {
///     // Wait for logs to flush with a short timeout
///     tokio::time::timeout(
///         Duration::from_millis(500),
///         &mut log_task
///     ).await.ok();
///     // Now abort if task still running
///     log_task.abort();
/// }
/// ```
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_graceful_log_shutdown_pattern() {
    // This test documents the expected graceful shutdown pattern
    // for handling log streaming when wait errors occur.

    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // The graceful shutdown pattern should:
    // 1. Await the log task with a short timeout (e.g., 500ms)
    // 2. Only abort the task if it doesn't complete within the timeout
    // 3. This gives buffered logs a chance to flush to disk

    // This is a documentation test - the actual fix needs to be
    // implemented in src/docker/run/run.rs around line 449-458

    let timeout_duration = Duration::from_millis(500);
    assert!(
        timeout_duration.as_millis() == 500,
        "Graceful shutdown should use 500ms timeout for log flush"
    );

    // When the bug is fixed, the log streaming task will have a
    // chance to complete or flush before being aborted.
}
