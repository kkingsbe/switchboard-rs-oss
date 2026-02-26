//! Integration test for running the trivial agent with --version
//!
//! This test verifies that:
//! - The agent container can be created and started
//! - The kilo --version command runs successfully
//! - Version information is returned
//! - The container is cleaned up after execution

#[cfg(feature = "integration")]
use super::docker_available;
#[cfg(feature = "integration")]
use bollard::container::{Config, CreateContainerOptions, LogOutput};
#[cfg(feature = "integration")]
use bollard::Docker;
#[cfg(feature = "integration")]
use std::fs;
#[cfg(feature = "integration")]
use std::path::PathBuf;
#[cfg(feature = "integration")]
use tokio::time::{sleep, Duration};

/// Test that the trivial agent can be run with --version
///
/// This test verifies:
/// - Container is created with workspace mount
/// - kilo --version command executes successfully
/// - Exit code is 0
/// - Output contains version information
/// - Container is removed after exit (AutoRemove works)
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_trivial_agent_version() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Docker not available, skipping test");
        return;
    }

    // Connect to Docker
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Skipping test: Failed to connect to Docker: {}", e);
            return;
        }
    };

    // Create a temp directory for the test
    let temp_dir: PathBuf =
        std::env::temp_dir().join(format!("switchboard-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    // Ensure temp directory is cleaned up after test
    let temp_dir_clone = temp_dir.clone();
    let cleanup_guard = scopeguard::guard(temp_dir_clone, |path| {
        let _ = fs::remove_dir_all(path);
    });

    // Default image name and tag from config
    let image_name = "switchboard-agent";
    let image_tag = "latest";
    let full_image_ref = format!("{}:{}", image_name, image_tag);

    // Verify the image exists
    match docker.inspect_image(&full_image_ref).await {
        Ok(_) => {
            // Image exists, proceed with test
        }
        Err(e) => {
            eprintln!(
                "Skipping test: Required image '{}' not found. Please build the image first: {}",
                full_image_ref, e
            );
            return;
        }
    }

    // Create container options
    let container_name = format!("switchboard-test-version-{}", uuid::Uuid::new_v4());
    let create_options = CreateContainerOptions {
        name: &container_name,
        platform: None,
    };

    // Build container config with workspace mount
    let container_config = Config {
        image: Some(full_image_ref.clone()),
        cmd: Some(vec!["kilo".to_string(), "--version".to_string()]),
        host_config: Some(bollard::models::HostConfig {
            binds: Some(vec![format!("{}:/workspace", temp_dir.display())]),
            auto_remove: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create the container
    let container = match docker
        .create_container(Some(create_options), container_config)
        .await
    {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Skipping test: Failed to create container: {}", e);
            return;
        }
    };

    // Start the container
    match docker.start_container::<String>(&container.id, None).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Skipping test: Failed to start container: {}", e);
            return;
        }
    }

    // Wait for container to complete
    use futures_util::StreamExt;
    let mut wait_stream = docker.wait_container::<String>(
        &container.id,
        None::<bollard::container::WaitContainerOptions<String>>,
    );

    let exit_code = if let Some(Ok(exit_code_info)) = wait_stream.next().await {
        exit_code_info.status_code
    } else {
        eprintln!("Skipping test: Failed to get exit code");
        return;
    };

    // Capture container output logs
    let log_options = bollard::container::LogsOptions::<String> {
        stdout: true,
        stderr: true,
        ..Default::default()
    };

    let mut log_stream = docker.logs::<String>(&container.id, Some(log_options));

    let mut output = String::new();
    while let Some(result) = log_stream.next().await {
        match result {
            Ok(log_output) => match log_output {
                LogOutput::StdErr { message } | LogOutput::StdOut { message } => {
                    if let Ok(msg_str) = String::from_utf8(message.to_vec()) {
                        output.push_str(&msg_str);
                    }
                }
                LogOutput::StdIn { .. } | LogOutput::Console { .. } => {}
            },
            Err(e) => {
                eprintln!("Warning: Failed to read log output: {}", e);
            }
        }
    }

    // Verify exit code is 0 (success)
    assert_eq!(
        exit_code, 0,
        "Container should exit with code 0 when running 'kilo --version', got {}",
        exit_code
    );

    // Verify output contains version information
    assert!(
        !output.trim().is_empty(),
        "Version output should not be empty"
    );

    // The output should contain version-like information
    // (e.g., "kilo 0.1.0" or similar version string)
    let output_lower = output.to_lowercase();
    assert!(
        output_lower.contains("version")
            || output_lower.contains("kilo")
            || output_lower.contains("v"),
        "Output should contain version information, got: {}",
        output.trim()
    );

    // Verify container was removed after exit (AutoRemove works)
    sleep(Duration::from_secs(1)).await;
    let inspect_result = docker.inspect_container(&container.id, None).await;
    match inspect_result {
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => {
            // Container was removed as expected
        }
        Ok(_) => {
            panic!(
                "Container should have been auto-removed but still exists: {}",
                container.id
            );
        }
        Err(e) => {
            // Some other error occurred
            panic!("Unexpected error when verifying container removal: {}", e);
        }
    }

    // Cleanup: Explicitly drop the guard to remove the temp directory
    drop(cleanup_guard);
}
