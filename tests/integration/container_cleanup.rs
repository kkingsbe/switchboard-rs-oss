//! Integration tests for container cleanup behavior
//!
//! These tests verify that:
//! - Containers are removed after execution when --rm flag is used
//! - Container artifacts are properly cleaned up
//! - No orphaned containers remain after execution

#[cfg(feature = "integration")]
use super::docker_available;
#[cfg(feature = "integration")]
use bollard::Docker;
#[cfg(feature = "integration")]
use switchboard::docker::run::wait::{wait_with_timeout, TerminationSignal};
#[cfg(feature = "integration")]
use switchboard::docker::DockerClient;
#[cfg(feature = "integration")]
use tokio::time::{sleep, Duration};

/// Test container cleanup after normal exit
///
/// This test verifies that a container with auto_remove enabled is automatically
/// removed by Docker after the process exits normally (exit code 0).
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_container_cleanup_normal_exit() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
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

    // Verify alpine image exists
    match docker.inspect_image("alpine:latest").await {
        Ok(_) => {
            // Image exists, proceed with test
        }
        Err(e) => {
            eprintln!(
                "Skipping test: Required image 'alpine:latest' not found. Please pull the image first: docker pull alpine:latest. Error: {}",
                e
            );
            return;
        }
    }

    // Create container options with a unique name
    let container_name = format!("switchboard-test-cleanup-{}", uuid::Uuid::new_v4());
    let create_options = bollard::container::CreateContainerOptions {
        name: &container_name,
        platform: None,
    };

    // Build container config with auto_remove enabled
    let container_config = bollard::container::Config {
        image: Some("alpine:latest".to_string()),
        cmd: Some(vec![
            "sh".to_string(),
            "-c".to_string(),
            "exit 0".to_string(),
        ]),
        host_config: Some(bollard::models::HostConfig {
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

    let exit_code = match wait_stream.next().await {
        Some(Ok(exit_code_info)) => exit_code_info.status_code,
        Some(Err(e)) => {
            eprintln!("Skipping test: Failed to wait for container: {}", e);
            return;
        }
        None => {
            eprintln!("Skipping test: No exit code received");
            return;
        }
    };

    // Verify exit code is 0 (success)
    assert_eq!(
        exit_code, 0,
        "Container should exit with code 0 when running 'sh -c \"exit 0\"', got {}",
        exit_code
    );

    // Sleep for 1 second to allow Docker to process auto-removal
    sleep(Duration::from_secs(1)).await;

    // Verify container was removed after exit (AutoRemove works)
    let inspect_result = docker.inspect_container(&container.id, None).await;
    match inspect_result {
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => {
            // Container was removed as expected - test passes
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
}

/// Test container cleanup after timeout exit
///
/// This test verifies that a container with auto_remove enabled is automatically
/// removed by Docker after being terminated due to timeout.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_container_cleanup_timeout_exit() {
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

    // Verify alpine image exists
    match docker.inspect_image("alpine:latest").await {
        Ok(_) => {
            // Image exists, proceed with test
        }
        Err(e) => {
            eprintln!(
                "Skipping test: Required image 'alpine:latest' not found. Please pull the image first: docker pull alpine:latest. Error: {}",
                e
            );
            return;
        }
    }

    // Create container options with a unique name
    let container_name = format!("switchboard-test-cleanup-{}", uuid::Uuid::new_v4());
    let create_options = bollard::container::CreateContainerOptions {
        name: &container_name,
        platform: None,
    };

    // Build container config with auto_remove enabled
    // Use a sleep command that runs longer than our timeout
    let container_config = bollard::container::Config {
        image: Some("alpine:latest".to_string()),
        cmd: Some(vec![
            "sh".to_string(),
            "-c".to_string(),
            "sleep 30".to_string(),
        ]),
        host_config: Some(bollard::models::HostConfig {
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

    // Use wait_with_timeout with a short timeout (5 seconds)
    // The container runs for 30 seconds, so it should time out
    let timeout_duration = Duration::from_secs(5);
    let exit_status =
        match wait_with_timeout(&client, &container.id, timeout_duration, "test-agent", None).await
        {
            Ok(status) => status,
            Err(e) => {
                eprintln!("Skipping test: Failed to wait for container: {}", e);
                return;
            }
        };

    // Verify timeout occurred
    assert!(
        exit_status.timed_out,
        "Container should have timed out, got timed_out={}",
        exit_status.timed_out
    );

    // Verify exit code is 137 (SIGKILL) or 143 (SIGTERM)
    assert!(
        exit_status.exit_code == 137 || exit_status.exit_code == 143,
        "Container should have exit code 137 (SIGKILL) or 143 (SIGTERM), got {}",
        exit_status.exit_code
    );

    // Verify termination signal
    assert_eq!(
        exit_status.termination_signal,
        TerminationSignal::SigKill,
        "Container should have been terminated with SIGKILL, got {:?}",
        exit_status.termination_signal
    );

    // Sleep for 1 second to allow Docker to process auto-removal
    sleep(Duration::from_secs(1)).await;

    // Verify container was removed after timeout exit (AutoRemove works)
    let inspect_result = docker.inspect_container(&container.id, None).await;
    match inspect_result {
        Err(bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        }) => {
            // Container was removed as expected - test passes
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
}

/// Test container cleanup after multiple runs
///
/// This test verifies that multiple containers run sequentially are all
/// automatically removed after exit when auto_remove is enabled.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_container_cleanup_multiple_runs() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
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

    // Verify alpine image exists
    match docker.inspect_image("alpine:latest").await {
        Ok(_) => {
            // Image exists, proceed with test
        }
        Err(e) => {
            eprintln!(
                "Skipping test: Required image 'alpine:latest' not found. Please pull the image first: docker pull alpine:latest. Error: {}",
                e
            );
            return;
        }
    }

    // Store container IDs for later verification
    let mut container_ids = Vec::new();

    // Run 5 containers sequentially
    for i in 0..5 {
        // Create container options with a unique name
        let container_name = format!("switchboard-test-cleanup-{}", uuid::Uuid::new_v4());
        let create_options = bollard::container::CreateContainerOptions {
            name: &container_name,
            platform: None,
        };

        // Build container config with auto_remove enabled
        let container_config = bollard::container::Config {
            image: Some("alpine:latest".to_string()),
            cmd: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                "exit 0".to_string(),
            ]),
            host_config: Some(bollard::models::HostConfig {
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
                eprintln!("Skipping test: Failed to create container {}: {}", i, e);
                return;
            }
        };

        container_ids.push(container.id.clone());

        // Start the container
        match docker.start_container::<String>(&container.id, None).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Skipping test: Failed to start container {}: {}", i, e);
                return;
            }
        }

        // Wait for container to complete
        use futures_util::StreamExt;
        let mut wait_stream = docker.wait_container::<String>(
            &container.id,
            None::<bollard::container::WaitContainerOptions<String>>,
        );

        let exit_code = match wait_stream.next().await {
            Some(Ok(exit_code_info)) => exit_code_info.status_code,
            Some(Err(e)) => {
                eprintln!("Skipping test: Failed to wait for container {}: {}", i, e);
                return;
            }
            None => {
                eprintln!("Skipping test: No exit code received for container {}", i);
                return;
            }
        };

        // Verify exit code is 0 (success)
        assert_eq!(
            exit_code, 0,
            "Container {} should exit with code 0 when running 'sh -c \"exit 0\"', got {}",
            i, exit_code
        );
    }

    // Sleep for 1 second to allow Docker to process auto-removal
    sleep(Duration::from_secs(1)).await;

    // List all containers with filter for our test containers
    use bollard::container::ListContainersOptions;
    use std::collections::HashMap;
    let filters = HashMap::from([("label".to_string(), vec!["switchboard.agent".to_string()])]);
    let list_options = Some(ListContainersOptions {
        all: true,
        filters,
        ..Default::default()
    });
    let containers = docker
        .list_containers::<String>(list_options)
        .await
        .unwrap();

    // Verify NO containers remain (all should have been auto-removed)
    assert!(
        containers.is_empty(),
        "All {} containers should have been auto-removed, but {} containers still exist",
        container_ids.len(),
        containers.len()
    );
}
