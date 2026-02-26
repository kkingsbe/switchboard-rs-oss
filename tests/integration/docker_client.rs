//! Integration tests for Docker client functionality
//!
//! These tests verify the Docker client's core functionality including:
//! - Client initialization and configuration
//! - Container lifecycle operations
//! - Image management
//! - Volume and network operations

#[cfg(feature = "integration")]
use super::docker_available;
#[cfg(feature = "streams")]
use switchboard::docker::run::attach_and_stream_logs;
#[cfg(feature = "integration")]
use switchboard::docker::run::types::ContainerConfig;
#[cfg(feature = "integration")]
use switchboard::docker::DockerClient;
#[cfg(feature = "integration")]
use switchboard::logger::Logger;
#[cfg(feature = "integration")]
use std::fs;
#[cfg(feature = "integration")]
use std::sync::{Arc, Mutex};

/// Test that DockerClient can connect to the Docker daemon successfully.
/// This is a basic smoke test to ensure the Docker client wrapper works.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_docker_client_connection_success() {
    if !docker_available().await {
        eprintln!("Docker daemon not available, skipping test");
        return;
    }

    // Test DockerClient::new() with default parameters
    let client = match DockerClient::new("test-image".to_string(), "latest".to_string()).await {
        Ok(c) => c,
        Err(e) => {
            panic!("Failed to create DockerClient: {:?}", e);
        }
    };

    // Verify the client has a valid internal Docker client
    assert!(
        client.docker().version().await.is_ok(),
        "Docker version check failed"
    );
}

/// Test that check_available() correctly identifies Docker daemon availability.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_docker_client_connection_ping() {
    if !docker_available().await {
        eprintln!("Docker daemon not available, skipping test");
        return;
    }

    // Create a client first
    let client = match DockerClient::new("test-image".to_string(), "latest".to_string()).await {
        Ok(c) => c,
        Err(e) => {
            panic!("Failed to create DockerClient: {:?}", e);
        }
    };

    // Test check_available() when Docker is available
    let result = client.check_available().await;
    assert!(
        result.is_ok(),
        "check_available() should return Ok when Docker is available: {:?}",
        result
    );
}

/// Test that build_agent_image can successfully build a Docker image
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_build_agent_image() {
    if !docker_available().await {
        eprintln!("Docker daemon not available, skipping test");
        return;
    }

    // Create a DockerClient instance
    let client = match DockerClient::new("test-image".to_string(), "latest".to_string()).await {
        Ok(c) => c,
        Err(e) => {
            panic!("Failed to create DockerClient: {:?}", e);
        }
    };

    // Create a temporary directory for build context
    let build_context = tempfile::tempdir().expect("Failed to create temp directory");
    let build_context_path = build_context.path();

    // Write a minimal Dockerfile content
    let dockerfile = r#"FROM alpine:latest
CMD echo "Hello from test"
"#;

    // Call build_agent_image
    let image_id = match client
        .build_agent_image(
            dockerfile,
            build_context_path,
            "switchboard-test-image",
            "test",
            true,
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            panic!("Failed to build image: {:?}", e);
        }
    };

    // Verify the image_id is non-empty
    assert!(!image_id.is_empty(), "Image ID should not be empty");

    // Verify the built image exists by calling inspect_image
    let full_image_name = "switchboard-test-image:test";
    match client.docker().inspect_image(full_image_name).await {
        Ok(_) => {
            // Image exists, test passes
        }
        Err(e) => {
            panic!(
                "Failed to inspect built image '{}': {:?}",
                full_image_name, e
            );
        }
    }

    // Clean up: Remove the test image
    let _ = client
        .docker()
        .remove_image(
            full_image_name,
            Some(bollard::image::RemoveImageOptions {
                force: true,
                ..Default::default()
            }),
            None,
        )
        .await;
}

/// Test that run_agent can successfully execute a simple command in a container
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_run_agent_simple() {
    if !docker_available().await {
        eprintln!("Docker daemon not available, skipping test");
        return;
    }

    // Create a DockerClient instance
    let client = match DockerClient::new("test-image".to_string(), "latest".to_string()).await {
        Ok(c) => c,
        Err(e) => {
            panic!("Failed to create DockerClient: {:?}", e);
        }
    };

    // Get the current workspace directory
    let workspace = match std::env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            panic!("Failed to get current directory: {:?}", e);
        }
    };

    // Create a ContainerConfig
    let config = ContainerConfig {
        agent_name: "test-agent".to_string(),
        env_vars: vec!["TEST_VAR=test_value".to_string()],
        timeout: None,
        readonly: false,
        prompt: String::new(),
        skills: None,
    };

    // Prepare the command to run
    let cmd = vec![
        "sh".to_string(),
        "-c".to_string(),
        "echo 'Hello from agent' && exit 0".to_string(),
    ];

    // Call run_agent
    let result = match switchboard::docker::run::run_agent(
        workspace.to_str().unwrap(),
        &client,
        &config,
        Some("30s".to_string()),
        "alpine:latest",
        Some(&cmd),
        None, // No logger for simple test
        None, // No metrics store
        "test_agent_simple",
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            panic!("Failed to run agent: {:?}", e);
        }
    };

    // Verify the result is Ok with non-empty container_id and exit_code 0
    assert!(
        !result.container_id.is_empty(),
        "Container ID should not be empty"
    );
    assert_eq!(
        result.exit_code, 0,
        "Exit code should be 0 for successful execution"
    );
}

/// Test that run_agent correctly handles timeouts by killing long-running containers
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_run_agent_timeout() {
    if !docker_available().await {
        eprintln!("Docker daemon not available, skipping test");
        return;
    }

    // Create a DockerClient instance
    let client = match DockerClient::new("test-image".to_string(), "latest".to_string()).await {
        Ok(c) => c,
        Err(e) => {
            panic!("Failed to create DockerClient: {:?}", e);
        }
    };

    // Get the current workspace directory
    let workspace = match std::env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            panic!("Failed to get current directory: {:?}", e);
        }
    };

    // Create a ContainerConfig
    let config = ContainerConfig {
        agent_name: "test-agent-timeout".to_string(),
        env_vars: vec![],
        timeout: None,
        readonly: false,
        prompt: String::new(),
        skills: None,
    };

    // Prepare a long-running command that will exceed the timeout
    let cmd = vec![
        "sh".to_string(),
        "-c".to_string(),
        "sleep 10 && echo 'Should not see this'".to_string(),
    ];

    // Call run_agent with a short timeout
    let result = match switchboard::docker::run::run_agent(
        workspace.to_str().unwrap(),
        &client,
        &config,
        Some("5s".to_string()), // 5 second timeout
        "alpine:latest",
        Some(&cmd),
        None, // No logger for simple test
        None, // No metrics store
        "test_agent_timeout",
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            panic!("Failed to run agent: {:?}", e);
        }
    };

    // Verify the container was killed (exit_code 137 for SIGKILL or -1 for error)
    // A SIGKILL (signal 9) results in exit code 128+9=137
    assert!(
        result.exit_code == 137 || result.exit_code == -1,
        "Exit code should be 137 (SIGKILL) or -1 for timeout, got: {}",
        result.exit_code
    );
    assert!(
        !result.container_id.is_empty(),
        "Container ID should not be empty"
    );
}

/// Test that attach_and_stream_logs can successfully capture container output
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
// TODO: Uncomment when streams module is implemented
#[cfg(feature = "streams")]
async fn test_log_streaming() {
    if !docker_available().await {
        eprintln!("Docker daemon not available, skipping test");
        return;
    }

    // Create a DockerClient instance
    let client = match DockerClient::new("test-image".to_string(), "latest".to_string()).await {
        Ok(c) => c,
        Err(e) => {
            panic!("Failed to create DockerClient: {:?}", e);
        }
    };

    // Create a temporary directory for log output
    let log_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            panic!("Failed to create temp directory: {:?}", e);
        }
    };

    // Create a Logger instance with the temp directory and foreground_mode: false
    let logger = Logger::new(log_dir.path().to_path_buf(), None, false);

    // Wrap logger in Arc<Mutex<Logger>>
    let logger = Arc::new(Mutex::new(logger));

    // Create a ContainerConfig
    let config = ContainerConfig {
        agent_name: "test-agent-logs".to_string(),
        env_vars: vec![],
        timeout: None,
        readonly: false,
        prompt: String::new(),
        skills: None,
    };

    // Create container options with a unique name
    let container_name = format!("switchboard-test-logs-{}", config.agent_name);
    let options = Some(bollard::container::CreateContainerOptions {
        name: &container_name,
        platform: None,
    });

    // Create host config with auto_remove enabled
    let host_config = bollard::models::HostConfig {
        auto_remove: Some(true),
        ..Default::default()
    };

    // Create container configuration that produces stdout/stderr output
    let container_config = bollard::container::Config {
        image: Some("alpine:latest".to_string()),
        cmd: Some(vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo 'stdout message' && echo 'stderr message' >&2 && sleep 0.5".to_string(),
        ]),
        host_config: Some(host_config),
        ..Default::default()
    };

    // Create the container
    let container_id = match client
        .docker()
        .create_container(options, container_config)
        .await
    {
        Ok(info) => info.id,
        Err(e) => {
            panic!("Failed to create container: {:?}", e);
        }
    };

    // Start the container
    if let Err(e) = client
        .docker()
        .start_container::<String>(&container_id, None)
        .await
    {
        panic!("Failed to start container: {:?}", e);
    }

    // Call attach_and_stream_logs with the running container
    let logger_clone = Arc::clone(&logger);
    let stream_result = attach_and_stream_logs(
        &client,
        &container_id,
        &config.agent_name,
        Some(logger_clone),
        true, // follow logs as they are generated
    )
    .await;

    // Wait for the container to complete
    use futures::StreamExt;
    let mut wait_stream = client.docker().wait_container::<String>(
        &container_id,
        None::<bollard::container::WaitContainerOptions<String>>,
    );

    let exit_code = match wait_stream.next().await {
        Some(Ok(exit_code_info)) => exit_code_info.status_code,
        Some(Err(e)) => {
            panic!("Failed to wait for container: {:?}", e);
        }
        None => {
            panic!("Wait stream ended unexpectedly");
        }
    };

    // Verify the function returns Ok(())
    assert!(
        stream_result.is_ok(),
        "attach_and_stream_logs should return Ok, got: {:?}",
        stream_result
    );

    // Verify container exited with code 0
    assert_eq!(
        exit_code, 0,
        "Container should exit with code 0, got: {:?}",
        exit_code
    );

    // Verify that a log file was created for the agent
    let agent_log_dir = log_dir.path().join(&config.agent_name);
    assert!(
        agent_log_dir.exists(),
        "Agent log directory should exist at {:?}",
        agent_log_dir
    );
    assert!(
        agent_log_dir.is_dir(),
        "Agent log path should be a directory"
    );

    // Find the log file
    let entries = match fs::read_dir(&agent_log_dir) {
        Ok(entries) => entries,
        Err(e) => {
            panic!("Failed to read agent log directory: {:?}", e);
        }
    };

    let log_files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    assert_eq!(
        log_files.len(),
        1,
        "There should be exactly one log file, found: {}",
        log_files.len()
    );

    // Verify the log file has the correct extension
    let log_file = log_files[0].path();
    assert_eq!(
        log_file.extension().unwrap_or_default(),
        "log",
        "Log file should have .log extension"
    );

    // Verify the log file contains the expected output messages
    let log_content = match fs::read_to_string(&log_file) {
        Ok(content) => content,
        Err(e) => {
            panic!("Failed to read log file: {:?}", e);
        }
    };

    // The log file should contain both stdout and stderr messages
    assert!(
        log_content.contains("stdout message"),
        "Log content should contain 'stdout message', got: {}",
        log_content
    );
    assert!(
        log_content.contains("stderr message"),
        "Log content should contain 'stderr message', got: {}",
        log_content
    );

    // Clean up: container is removed automatically (auto_remove: true)
    // TempDir is automatically cleaned up when it goes out of scope
}
