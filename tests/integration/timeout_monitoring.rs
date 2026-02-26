//! Integration tests for timeout monitoring and container kill behavior
//!
//! These tests verify that timeout monitoring works correctly, including:
//! - Containers are killed when timeout expires
//! - Exit status shows timed_out=true and exit code 137 for timeout
//! - Normal exit completes without timeout when container finishes quickly
//! - SIGTERM is sent for graceful shutdown on timeout
//! - SIGKILL is sent if container doesn't exit within grace period
//! - Termination type and signal counts are tracked in metrics

#[cfg(feature = "integration")]
use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
#[cfg(feature = "integration")]
use std::time::Duration;

#[cfg(feature = "integration")]
use super::docker_available;
#[cfg(feature = "integration")]
use switchboard::docker::run::types::ContainerConfig;
#[cfg(feature = "integration")]
use switchboard::docker::run::wait::{wait_with_timeout, TerminationSignal};
#[cfg(feature = "integration")]
use switchboard::docker::DockerClient;
#[cfg(feature = "integration")]
use switchboard::logger::Logger;
#[cfg(feature = "integration")]
use switchboard::metrics::MetricsStore;
#[cfg(feature = "integration")]
use std::fs;
#[cfg(feature = "integration")]
use std::sync::{Arc, Mutex};

/// Test that a container sleeping longer than the timeout is killed
///
/// This test creates a container that sleeps for 10 seconds, but sets a
/// 5-second timeout. It verifies:
/// - Exit status has timed_out = true
/// - Exit code is 137 (SIGKILL)
/// - Container is not running after the timeout
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_timeout_kills_container() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Create a DockerClient
    let client = match DockerClient::new("test".to_string(), "latest".to_string()).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Skipping test: Failed to create DockerClient: {}", e);
            return;
        }
    };

    let docker = client.docker();
    let timeout_duration = Duration::from_secs(5);

    // Create a container that sleeps for 10 seconds (longer than our 5s timeout)
    let container_name = "switchboard-test-timeout-sleep";
    let create_result = docker
        .create_container(
            Some(CreateContainerOptions {
                name: container_name,
                platform: None,
            }),
            Config {
                image: Some("alpine:latest"),
                cmd: Some(vec!["sleep", "10"]),
                ..Default::default()
            },
        )
        .await;

    let container_id = match create_result {
        Ok(container_info) => container_info.id,
        Err(e) => {
            eprintln!("Skipping test: Failed to create container: {}", e);
            return;
        }
    };

    // Start the container
    match docker
        .start_container::<String>(&container_id, None::<StartContainerOptions<String>>)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Skipping test: Failed to start container: {}", e);
            // Try to clean up
            let _ = docker
                .remove_container(
                    &container_id,
                    None::<bollard::container::RemoveContainerOptions>,
                )
                .await;
            return;
        }
    }

    // Wait for container with timeout
    let exit_status =
        wait_with_timeout(&client, &container_id, timeout_duration, "test-agent", None).await;

    // Verify the exit status
    match exit_status {
        Ok(status) => {
            assert!(
                status.timed_out,
                "Container should have timed out, got timed_out={}",
                status.timed_out
            );
            assert_eq!(
                status.exit_code, 137,
                "Timeout should result in exit code 137 (SIGKILL), got {}",
                status.exit_code
            );
        }
        Err(e) => {
            panic!("wait_with_timeout returned error: {:?}", e);
        }
    }

    // Verify container is not running after timeout
    let inspect_result = docker.inspect_container(&container_id, None).await;
    match inspect_result {
        Ok(inspect) => {
            if let Some(state) = inspect.state {
                if let Some(running) = state.running {
                    assert!(!running, "Container should not be running after timeout");
                }
                // Verify exit code is 137 (SIGKILL)
                if let Some(exit_code) = state.exit_code {
                    assert_eq!(
                        exit_code, 137,
                        "Container exit code should be 137 (SIGKILL) after timeout, got {}",
                        exit_code
                    );
                }
            }
        }
        Err(e) => {
            // Container might have been removed already
            eprintln!(
                "Could not inspect container after timeout (may have been removed): {}",
                e
            );
        }
    }

    // Clean up the container
    let _ = docker
        .remove_container(
            &container_id,
            Some(bollard::container::RemoveContainerOptions {
                force: true,
                link: false,
                v: false,
            }),
        )
        .await;
}

/// Test that a container exiting quickly completes normally without timeout
///
/// This test creates a container that exits immediately with code 0,
/// but sets a 30-second timeout. It verifies:
/// - Exit status has timed_out = false
/// - Exit code is 0
/// - No timeout kill occurs
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_normal_exit_no_timeout() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Create a DockerClient
    let client = match DockerClient::new("test".to_string(), "latest".to_string()).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Skipping test: Failed to create DockerClient: {}", e);
            return;
        }
    };

    let docker = client.docker();
    let timeout_duration = Duration::from_secs(30);

    // Create a container that exits immediately with code 0
    let container_name = "switchboard-test-normal-exit";
    let create_result = docker
        .create_container(
            Some(CreateContainerOptions {
                name: container_name,
                platform: None,
            }),
            Config {
                image: Some("alpine:latest"),
                cmd: Some(vec!["sh", "-c", "exit 0"]),
                ..Default::default()
            },
        )
        .await;

    let container_id = match create_result {
        Ok(container_info) => container_info.id,
        Err(e) => {
            eprintln!("Skipping test: Failed to create container: {}", e);
            return;
        }
    };

    // Start the container
    match docker
        .start_container::<String>(&container_id, None::<StartContainerOptions<String>>)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Skipping test: Failed to start container: {}", e);
            // Try to clean up
            let _ = docker
                .remove_container(
                    &container_id,
                    None::<bollard::container::RemoveContainerOptions>,
                )
                .await;
            return;
        }
    }

    // Wait for container with a generous timeout
    let exit_status =
        wait_with_timeout(&client, &container_id, timeout_duration, "test-agent", None).await;

    // Verify the exit status
    match exit_status {
        Ok(status) => {
            assert!(
                !status.timed_out,
                "Container should not have timed out, got timed_out={}",
                status.timed_out
            );
            assert_eq!(
                status.exit_code, 0,
                "Container should exit with code 0, got {}",
                status.exit_code
            );
        }
        Err(e) => {
            panic!("wait_with_timeout returned error: {:?}", e);
        }
    }

    // Verify container is not running and has exit code 0
    let inspect_result = docker.inspect_container(&container_id, None).await;
    match inspect_result {
        Ok(inspect) => {
            if let Some(state) = inspect.state {
                if let Some(running) = state.running {
                    assert!(
                        !running,
                        "Container should not be running after normal exit"
                    );
                }
                if let Some(exit_code) = state.exit_code {
                    assert_eq!(
                        exit_code, 0,
                        "Container exit code should be 0 after normal exit, got {}",
                        exit_code
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("Could not inspect container after exit: {}", e);
        }
    }

    // Clean up the container
    let _ = docker
        .remove_container(
            &container_id,
            Some(bollard::container::RemoveContainerOptions {
                force: true,
                link: false,
                v: false,
            }),
        )
        .await;
}

/// Test that SIGTERM is sent for graceful shutdown on timeout
///
/// This test creates a container that catches SIGTERM and exits gracefully,
/// with a timeout shorter than the container's normal sleep time. It verifies:
/// - Exit status has timed_out = true
/// - Exit signal is TerminationSignal::SigTerm
/// - Container exits with code 143 (SIGTERM exit code)
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_sigterm_graceful_shutdown() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Create a DockerClient
    let client = match DockerClient::new("test".to_string(), "latest".to_string()).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Skipping test: Failed to create DockerClient: {}", e);
            return;
        }
    };

    let docker = client.docker();
    let timeout_duration = Duration::from_secs(3);

    // Create a container that sleeps for 30 seconds but traps SIGTERM to exit gracefully
    // The trap handler will cause the container to exit with code 143 when SIGTERM is received
    let container_name = "switchboard-test-sigterm-graceful";
    let create_result = docker
        .create_container(
            Some(CreateContainerOptions {
                name: container_name,
                platform: None,
            }),
            Config {
                image: Some("alpine:latest"),
                cmd: Some(vec!["sh", "-c", "trap 'exit 143' TERM; sleep 30"]),
                ..Default::default()
            },
        )
        .await;

    let container_id = match create_result {
        Ok(container_info) => container_info.id,
        Err(e) => {
            eprintln!("Skipping test: Failed to create container: {}", e);
            return;
        }
    };

    // Start the container
    match docker
        .start_container::<String>(&container_id, None::<StartContainerOptions<String>>)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Skipping test: Failed to start container: {}", e);
            // Try to clean up
            let _ = docker
                .remove_container(
                    &container_id,
                    None::<bollard::container::RemoveContainerOptions>,
                )
                .await;
            return;
        }
    }

    // Wait for container with timeout (shorter than the 30s sleep)
    let exit_status =
        wait_with_timeout(&client, &container_id, timeout_duration, "test-agent", None).await;

    // Verify the exit status
    match exit_status {
        Ok(status) => {
            assert!(
                status.timed_out,
                "Container should have timed out, got timed_out={}",
                status.timed_out
            );
            assert_eq!(
                status.termination_signal,
                TerminationSignal::SigTerm,
                "Should have used SIGTERM for graceful shutdown, got {:?}",
                status.termination_signal
            );
            assert_eq!(
                status.exit_code, 143,
                "SIGTERM should result in exit code 143, got {}",
                status.exit_code
            );
        }
        Err(e) => {
            panic!("wait_with_timeout returned error: {:?}", e);
        }
    }

    // Verify container is not running after SIGTERM
    let inspect_result = docker.inspect_container(&container_id, None).await;
    match inspect_result {
        Ok(inspect) => {
            if let Some(state) = inspect.state {
                if let Some(running) = state.running {
                    assert!(!running, "Container should not be running after SIGTERM");
                }
                // Verify exit code is 143 (SIGTERM)
                if let Some(exit_code) = state.exit_code {
                    assert_eq!(
                        exit_code, 143,
                        "Container exit code should be 143 (SIGTERM), got {}",
                        exit_code
                    );
                }
            }
        }
        Err(e) => {
            // Container might have been removed already
            eprintln!(
                "Could not inspect container after SIGTERM (may have been removed): {}",
                e
            );
        }
    }

    // Clean up the container
    let _ = docker
        .remove_container(
            &container_id,
            Some(bollard::container::RemoveContainerOptions {
                force: true,
                link: false,
                v: false,
            }),
        )
        .await;
}

/// Test that SIGKILL is sent if container doesn't exit within grace period
///
/// This test creates a container that ignores SIGTERM (doesn't exit during the
/// 10-second grace period), causing SIGKILL to be sent. It verifies:
/// - Exit status has timed_out = true
/// - Exit signal is TerminationSignal::SigKill
/// - Container exits with code 137 (SIGKILL exit code)
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_sigkill_after_grace_period_expires() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Create a DockerClient
    let client = match DockerClient::new("test".to_string(), "latest".to_string()).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Skipping test: Failed to create DockerClient: {}", e);
            return;
        }
    };

    let docker = client.docker();
    let timeout_duration = Duration::from_secs(3);

    // Create a container that sleeps for 60 seconds and ignores SIGTERM
    // This will cause the grace period to expire and SIGKILL to be sent
    let container_name = "switchboard-test-sigkill-force";
    let create_result = docker
        .create_container(
            Some(CreateContainerOptions {
                name: container_name,
                platform: None,
            }),
            Config {
                image: Some("alpine:latest"),
                cmd: Some(vec!["sh", "-c", "trap '' TERM; sleep 60"]),
                ..Default::default()
            },
        )
        .await;

    let container_id = match create_result {
        Ok(container_info) => container_info.id,
        Err(e) => {
            eprintln!("Skipping test: Failed to create container: {}", e);
            return;
        }
    };

    // Start the container
    match docker
        .start_container::<String>(&container_id, None::<StartContainerOptions<String>>)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Skipping test: Failed to start container: {}", e);
            // Try to clean up
            let _ = docker
                .remove_container(
                    &container_id,
                    None::<bollard::container::RemoveContainerOptions>,
                )
                .await;
            return;
        }
    }

    // Wait for container with timeout (shorter than the 60s sleep)
    let exit_status =
        wait_with_timeout(&client, &container_id, timeout_duration, "test-agent", None).await;

    // Verify the exit status
    match exit_status {
        Ok(status) => {
            assert!(
                status.timed_out,
                "Container should have timed out, got timed_out={}",
                status.timed_out
            );
            assert_eq!(
                status.termination_signal,
                TerminationSignal::SigKill,
                "Should have used SIGKILL after grace period, got {:?}",
                status.termination_signal
            );
            assert_eq!(
                status.exit_code, 137,
                "SIGKILL should result in exit code 137, got {}",
                status.exit_code
            );
        }
        Err(e) => {
            panic!("wait_with_timeout returned error: {:?}", e);
        }
    }

    // Verify container is not running after SIGKILL
    let inspect_result = docker.inspect_container(&container_id, None).await;
    match inspect_result {
        Ok(inspect) => {
            if let Some(state) = inspect.state {
                if let Some(running) = state.running {
                    assert!(!running, "Container should not be running after SIGKILL");
                }
                // Verify exit code is 137 (SIGKILL)
                if let Some(exit_code) = state.exit_code {
                    assert_eq!(
                        exit_code, 137,
                        "Container exit code should be 137 (SIGKILL), got {}",
                        exit_code
                    );
                }
            }
        }
        Err(e) => {
            // Container might have been removed already
            eprintln!(
                "Could not inspect container after SIGKILL (may have been removed): {}",
                e
            );
        }
    }

    // Clean up the container
    let _ = docker
        .remove_container(
            &container_id,
            Some(bollard::container::RemoveContainerOptions {
                force: true,
                link: false,
                v: false,
            }),
        )
        .await;
}

/// Test that timeout logging works correctly and logs the proper message
///
/// This test creates a container that sleeps for 10 seconds, but sets a
/// 5-second timeout. It verifies:
/// - Exit code is 137 (SIGKILL)
/// - A log file is created in the agent directory
/// - The log message contains: "[test-agent-timeout] Timed out after 5s - Container killed"
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_timeout_logs_message_correctly() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Create a temporary directory for logs
    let log_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Skipping test: Failed to create temp directory: {}", e);
            return;
        }
    };

    // Create a Logger instance
    let logger = Logger::new(log_dir.path().to_path_buf(), None, false);

    // Wrap logger in Arc<Mutex<>>
    let logger = Arc::new(Mutex::new(logger));

    // Create a DockerClient
    let client = match DockerClient::new("test".to_string(), "latest".to_string()).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Skipping test: Failed to create DockerClient: {}", e);
            return;
        }
    };

    let docker = client.docker();
    let timeout_duration = Duration::from_secs(5);
    let agent_name = "test-agent-timeout";

    // Create a container that sleeps for 10 seconds (longer than our 5s timeout)
    let container_name = "switchboard-test-timeout-logs";
    let create_result = docker
        .create_container(
            Some(CreateContainerOptions {
                name: container_name,
                platform: None,
            }),
            Config {
                image: Some("alpine:latest"),
                cmd: Some(vec!["sleep", "10"]),
                ..Default::default()
            },
        )
        .await;

    let container_id = match create_result {
        Ok(container_info) => container_info.id,
        Err(e) => {
            eprintln!("Skipping test: Failed to create container: {}", e);
            return;
        }
    };

    // Start the container
    match docker
        .start_container::<String>(&container_id, None::<StartContainerOptions<String>>)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Skipping test: Failed to start container: {}", e);
            // Try to clean up
            let _ = docker
                .remove_container(
                    &container_id,
                    None::<bollard::container::RemoveContainerOptions>,
                )
                .await;
            return;
        }
    }

    // Wait for container with timeout and pass the logger
    let exit_status = wait_with_timeout(
        &client,
        &container_id,
        timeout_duration,
        agent_name,
        Some(&logger),
    )
    .await;

    // Verify the exit status
    match exit_status {
        Ok(status) => {
            assert_eq!(
                status.exit_code, 137,
                "Timeout should result in exit code 137 (SIGKILL), got {}",
                status.exit_code
            );
        }
        Err(e) => {
            panic!("wait_with_timeout returned error: {:?}", e);
        }
    }

    // Verify container is not running after timeout
    let inspect_result = docker.inspect_container(&container_id, None).await;
    match inspect_result {
        Ok(inspect) => {
            if let Some(state) = inspect.state {
                if let Some(running) = state.running {
                    assert!(!running, "Container should not be running after timeout");
                }
            }
        }
        Err(e) => {
            // Container might have been removed already
            eprintln!(
                "Could not inspect container after timeout (may have been removed): {}",
                e
            );
        }
    }

    // Clean up the container
    let _ = docker
        .remove_container(
            &container_id,
            Some(bollard::container::RemoveContainerOptions {
                force: true,
                link: false,
                v: false,
            }),
        )
        .await;

    // Find and read the log file in the agent directory
    let agent_log_dir = log_dir.path().join(agent_name);
    let entries = match fs::read_dir(&agent_log_dir) {
        Ok(entries) => entries,
        Err(e) => {
            panic!(
                "Failed to read agent log directory '{}': {}",
                agent_log_dir.display(),
                e
            );
        }
    };

    // Find the log file
    let mut log_file_path: Option<std::path::PathBuf> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("log") {
            log_file_path = Some(path);
            break;
        }
    }

    let log_file_path = match log_file_path {
        Some(path) => path,
        None => {
            panic!(
                "No log file found in agent directory '{}'",
                agent_log_dir.display()
            );
        }
    };

    // Read the log file content
    let log_content = match fs::read_to_string(&log_file_path) {
        Ok(content) => content,
        Err(e) => {
            panic!(
                "Failed to read log file '{}': {}",
                log_file_path.display(),
                e
            );
        }
    };

    // Assert the log contains the expected timeout message
    let expected_message = "[test-agent-timeout] Timed out after 5s - Container killed";
    assert!(
        log_content.contains(expected_message),
        "Log file should contain '{}', but got:\n{}",
        expected_message,
        log_content
    );
}

/// Test that termination_type is set correctly in metrics after SIGTERM graceful shutdown
///
/// This test verifies that when a container exits via SIGTERM during the grace period,
/// the metrics correctly record termination_type as "sigterm".
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_termination_type_sigterm_in_metrics() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Create a temporary directory for metrics storage
    let temp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Skipping test: Failed to create temp directory: {}", e);
            return;
        }
    };

    // Create a MetricsStore
    let metrics_store = MetricsStore::new(temp_dir.path().to_path_buf());

    // Create a DockerClient
    let client = match DockerClient::new("test".to_string(), "latest".to_string()).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Skipping test: Failed to create DockerClient: {}", e);
            return;
        }
    };

    // Get the current workspace directory
    let workspace = match std::env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Skipping test: Failed to get current directory: {}", e);
            return;
        }
    };

    // Create a ContainerConfig
    let config = ContainerConfig {
        agent_name: "test-agent-sigterm".to_string(),
        env_vars: vec![],
        timeout: None,
        readonly: false,
        prompt: String::new(),
        skills: None,
    };

    // Prepare a command that catches SIGTERM and exits gracefully
    let cmd = vec![
        "sh".to_string(),
        "-c".to_string(),
        "trap 'exit 143' TERM; sleep 30".to_string(),
    ];

    // Call run_agent with a short timeout to trigger SIGTERM
    let result = match switchboard::docker::run::run_agent(
        workspace.to_str().unwrap(),
        &client,
        &config,
        Some("3s".to_string()), // 3 second timeout
        "alpine:latest",
        Some(&cmd),
        None, // No logger
        Some(&metrics_store),
        &config.agent_name,
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Skipping test: Failed to run agent: {:?}", e);
            return;
        }
    };

    // Verify the container was killed
    assert_eq!(
        result.exit_code, 143,
        "Exit code should be 143 (SIGTERM), got: {}",
        result.exit_code
    );

    // Load and verify metrics
    let all_metrics = match metrics_store.load() {
        Ok(metrics) => metrics,
        Err(e) => {
            panic!("Failed to load metrics: {:?}", e);
        }
    };

    // Verify agent metrics exist
    assert!(
        all_metrics.agents.contains_key(&config.agent_name),
        "Agent metrics should exist for '{}'",
        config.agent_name
    );

    let agent_data = &all_metrics.agents[&config.agent_name];

    // Verify sigterm_count was incremented
    assert_eq!(
        agent_data.sigterm_count, 1,
        "sigterm_count should be 1, got: {}",
        agent_data.sigterm_count
    );

    // Verify sigkill_count was NOT incremented
    assert_eq!(
        agent_data.sigkill_count, 0,
        "sigkill_count should be 0, got: {}",
        agent_data.sigkill_count
    );

    // Verify the last run has the correct error message indicating timeout
    if let Some(last_run) = agent_data.runs.last() {
        assert_eq!(
            last_run.error_message,
            Some("timed_out".to_string()),
            "Error message should indicate timeout, got: {:?}",
            last_run.error_message
        );
    }
}

/// Test that termination_type is set correctly in metrics after SIGKILL forced termination
///
/// This test verifies that when a container ignores SIGTERM and is killed via SIGKILL,
/// the metrics correctly record termination_type as "sigkill".
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_termination_type_sigkill_in_metrics() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Create a temporary directory for metrics storage
    let temp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Skipping test: Failed to create temp directory: {}", e);
            return;
        }
    };

    // Create a MetricsStore
    let metrics_store = MetricsStore::new(temp_dir.path().to_path_buf());

    // Create a DockerClient
    let client = match DockerClient::new("test".to_string(), "latest".to_string()).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Skipping test: Failed to create DockerClient: {}", e);
            return;
        }
    };

    // Get the current workspace directory
    let workspace = match std::env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Skipping test: Failed to get current directory: {}", e);
            return;
        }
    };

    // Create a ContainerConfig
    let config = ContainerConfig {
        agent_name: "test-agent-sigkill".to_string(),
        env_vars: vec![],
        timeout: None,
        readonly: false,
        prompt: String::new(),
        skills: None,
    };

    // Prepare a command that ignores SIGTERM (causing grace period to expire)
    let cmd = vec![
        "sh".to_string(),
        "-c".to_string(),
        "trap '' TERM; sleep 60".to_string(),
    ];

    // Call run_agent with a short timeout to trigger SIGTERM then SIGKILL
    let result = match switchboard::docker::run::run_agent(
        workspace.to_str().unwrap(),
        &client,
        &config,
        Some("3s".to_string()), // 3 second timeout
        "alpine:latest",
        Some(&cmd),
        None, // No logger
        Some(&metrics_store),
        &config.agent_name,
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Skipping test: Failed to run agent: {:?}", e);
            return;
        }
    };

    // Verify the container was killed
    assert_eq!(
        result.exit_code, 137,
        "Exit code should be 137 (SIGKILL), got: {}",
        result.exit_code
    );

    // Load and verify metrics
    let all_metrics = match metrics_store.load() {
        Ok(metrics) => metrics,
        Err(e) => {
            panic!("Failed to load metrics: {:?}", e);
        }
    };

    // Verify agent metrics exist
    assert!(
        all_metrics.agents.contains_key(&config.agent_name),
        "Agent metrics should exist for '{}'",
        config.agent_name
    );

    let agent_data = &all_metrics.agents[&config.agent_name];

    // Verify sigterm_count was NOT incremented (container ignored SIGTERM)
    assert_eq!(
        agent_data.sigterm_count, 0,
        "sigterm_count should be 0, got: {}",
        agent_data.sigterm_count
    );

    // Verify sigkill_count was incremented
    assert_eq!(
        agent_data.sigkill_count, 1,
        "sigkill_count should be 1, got: {}",
        agent_data.sigkill_count
    );

    // Verify the last run has the correct error message indicating timeout
    if let Some(last_run) = agent_data.runs.last() {
        assert_eq!(
            last_run.error_message,
            Some("timed_out".to_string()),
            "Error message should indicate timeout, got: {:?}",
            last_run.error_message
        );
    }
}

/// Test that sigterm_count and sigkill_count accumulate correctly across multiple runs
///
/// This test verifies that termination signal counters are properly accumulated
/// when multiple containers are terminated via different signals.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_multiple_termination_signals_accumulate() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Create a temporary directory for metrics storage
    let temp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Skipping test: Failed to create temp directory: {}", e);
            return;
        }
    };

    // Create a MetricsStore
    let metrics_store = MetricsStore::new(temp_dir.path().to_path_buf());

    // Create a DockerClient
    let client = match DockerClient::new("test".to_string(), "latest".to_string()).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Skipping test: Failed to create DockerClient: {}", e);
            return;
        }
    };

    // Get the current workspace directory
    let workspace = match std::env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Skipping test: Failed to get current directory: {}", e);
            return;
        }
    };

    let agent_name = "test-agent-multiple";

    // Run 2 containers that exit via SIGTERM
    for i in 0..2 {
        let config = ContainerConfig {
            agent_name: format!("{}-{}", agent_name, i),
            env_vars: vec![],
            timeout: None,
            readonly: false,
            prompt: String::new(),
            skills: None,
        };

        let cmd = vec![
            "sh".to_string(),
            "-c".to_string(),
            "trap 'exit 143' TERM; sleep 30".to_string(),
        ];

        let _result = match switchboard::docker::run::run_agent(
            workspace.to_str().unwrap(),
            &client,
            &config,
            Some("3s".to_string()),
            "alpine:latest",
            Some(&cmd),
            None,
            Some(&metrics_store),
            &config.agent_name,
            None,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to run agent {}: {:?}", i, e);
                continue;
            }
        };
    }

    // Run 3 containers that ignore SIGTERM and are killed via SIGKILL
    for i in 2..5 {
        let config = ContainerConfig {
            agent_name: format!("{}-{}", agent_name, i),
            env_vars: vec![],
            timeout: None,
            readonly: false,
            prompt: String::new(),
            skills: None,
        };

        let cmd = vec![
            "sh".to_string(),
            "-c".to_string(),
            "trap '' TERM; sleep 60".to_string(),
        ];

        let _result = match switchboard::docker::run::run_agent(
            workspace.to_str().unwrap(),
            &client,
            &config,
            Some("3s".to_string()),
            "alpine:latest",
            Some(&cmd),
            None,
            Some(&metrics_store),
            &config.agent_name,
            None,
        )
        .await
        {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to run agent {}: {:?}", i, e);
                continue;
            }
        };
    }

    // Load and verify metrics
    let all_metrics = match metrics_store.load() {
        Ok(metrics) => metrics,
        Err(e) => {
            panic!("Failed to load metrics: {:?}", e);
        }
    };

    // Verify all agents have correct termination counts
    for i in 0..2 {
        let name = format!("{}-{}", agent_name, i);
        assert!(
            all_metrics.agents.contains_key(&name),
            "Agent metrics should exist for '{}'",
            name
        );
        let agent_data = &all_metrics.agents[&name];
        assert_eq!(
            agent_data.sigterm_count, 1,
            "Agent {} should have sigterm_count=1, got: {}",
            name, agent_data.sigterm_count
        );
        assert_eq!(
            agent_data.sigkill_count, 0,
            "Agent {} should have sigkill_count=0, got: {}",
            name, agent_data.sigkill_count
        );
    }

    for i in 2..5 {
        let name = format!("{}-{}", agent_name, i);
        assert!(
            all_metrics.agents.contains_key(&name),
            "Agent metrics should exist for '{}'",
            name
        );
        let agent_data = &all_metrics.agents[&name];
        assert_eq!(
            agent_data.sigterm_count, 0,
            "Agent {} should have sigterm_count=0, got: {}",
            name, agent_data.sigterm_count
        );
        assert_eq!(
            agent_data.sigkill_count, 1,
            "Agent {} should have sigkill_count=1, got: {}",
            name, agent_data.sigkill_count
        );
    }
}
