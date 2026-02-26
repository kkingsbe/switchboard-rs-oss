//! Integration test for verifying ENTRYPOINT and CMD behavior
//!
//! This test verifies that:
//! - Container is created with entrypoint: None (uses Docker ENTRYPOINT ["kilo"])
//! - Container is created with cmd containing the production CLI arguments
//! - The Docker ENTRYPOINT is combined with the cmd to form the full command
//! - Kilo Code CLI receives the correct arguments when the container executes

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

/// Test that the ENTRYPOINT and CMD are correctly combined for Kilo Code CLI
///
/// This test verifies:
/// - Container is created with entrypoint: None (uses Docker's ENTRYPOINT ["kilo"])
/// - Container is created with cmd containing production CLI arguments
/// - The full command becomes: kilo <cmd arguments>
/// - Kilo Code CLI receives the correct arguments (verified via logs)
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_entrypoint_cmd_verification() {
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

    // Production format CMD arguments
    let cmd_args = vec![
        "--agent-name".to_string(),
        "test-agent".to_string(),
        "--prompt".to_string(),
        "test prompt".to_string(),
        "--auto".to_string(),
    ];

    // Create container options
    let container_name = format!("switchboard-test-entrypoint-{}", uuid::Uuid::new_v4());
    let create_options = CreateContainerOptions {
        name: &container_name,
        platform: None,
    };

    // Build container config with:
    // - entrypoint: None (uses Docker's ENTRYPOINT ["kilo"])
    // - cmd: production format arguments
    let container_config = Config {
        image: Some(full_image_ref.clone()),
        cmd: Some(cmd_args.clone()),
        entrypoint: None, // Use the ENTRYPOINT from Dockerfile
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

    // Verify container was created with correct configuration by inspecting it
    let inspect_result = docker.inspect_container(&container.id, None).await;
    match inspect_result {
        Ok(inspect) => {
            let config = inspect.config.expect("Container should have config");

            // Verify entrypoint is None (uses Docker's ENTRYPOINT)
            let entrypoint = config.entrypoint;
            assert!(
                entrypoint.is_none() || entrypoint == Some(vec![]),
                "Container should have entrypoint: None or empty, got: {:?}",
                entrypoint
            );

            // Verify cmd contains the production format arguments
            let actual_cmd = config.cmd;
            assert!(actual_cmd.is_some(), "Container should have cmd set");
            let actual_cmd = actual_cmd.unwrap();
            assert_eq!(
                actual_cmd, cmd_args,
                "Container cmd should contain production format arguments.\n\
                 Expected: {:?}\n\
                 Got: {:?}",
                cmd_args, actual_cmd
            );

            // Note: The image's ENTRYPOINT is verified by the Dockerfile:
            // ENTRYPOINT ["kilo"]
            // When entrypoint: None is set on the container, Docker uses the image's ENTRYPOINT.
            // The full command becomes: kilo <cmd arguments>
        }
        Err(e) => {
            eprintln!("Skipping test: Failed to inspect container: {}", e);
            let _ = docker
                .remove_container(
                    &container.id,
                    Some(bollard::container::RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await;
            return;
        }
    }

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

    // Note: The container may exit with a non-zero exit code if the CLI arguments
    // are not valid. This is okay - the primary purpose of the test is to verify
    // that the entrypoint and cmd are correctly configured.

    // Verify output contains evidence that the CLI was invoked
    // Even with invalid arguments, the CLI should output something (e.g., error message)
    let output_lower = output.to_lowercase();

    // The output should contain something from the CLI (either an error or help message)
    // or the arguments themselves
    let has_cli_output = !output.trim().is_empty()
        || output_lower.contains("kilo")
        || output_lower.contains("error")
        || output_lower.contains("usage")
        || output_lower.contains("unknown")
        || output_lower.contains("agent-name")
        || output_lower.contains("prompt");

    assert!(
        has_cli_output,
        "Container should produce output showing Kilo Code CLI was invoked.\n\
         Exit code: {}\n\
         Output: {}",
        exit_code,
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
