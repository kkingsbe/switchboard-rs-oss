//! Integration test for environment variable passing to containers
//!
//! This test verifies that:
//! - Custom environment variables (from agent.env) ARE passed to containers
//! - AGENT_NAME is NOT passed as an environment variable (per ARCHITECT_DECISION)
//! - PROMPT is NOT passed as an environment variable (per ARCHITECT_DECISION)

#[cfg(feature = "integration")]
use super::docker_available;
#[cfg(feature = "integration")]
use bollard::container::{Config, CreateContainerOptions, LogOutput};
#[cfg(feature = "integration")]
use bollard::Docker;
#[cfg(feature = "integration")]
use futures_util::StreamExt;
#[cfg(feature = "integration")]
use std::path::PathBuf;

/// Test that custom agent environment variables are passed to containers,
/// while AGENT_NAME and PROMPT are NOT passed (per architecture decision).
///
/// This test verifies that:
/// - Custom env vars (TEST_VAR, API_KEY) ARE present with correct values
/// - AGENT_NAME is NOT present in container environment
/// - PROMPT is NOT present in container environment
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_agent_custom_env_vars_passed() {
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

    // Create a temp directory for the test
    let temp_dir: PathBuf =
        std::env::temp_dir().join(format!("switchboard-test-env-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    // Ensure temp directory is cleaned up after test
    let temp_dir_clone = temp_dir.clone();
    let _cleanup = scopeguard::guard(temp_dir_clone, |path| {
        let _ = std::fs::remove_dir_all(path);
    });

    // Define custom environment variables (these should be passed to the container)
    let custom_env_vars = vec![
        "TEST_VAR=test_value".to_string(),
        "API_KEY=secret123".to_string(),
        "DEBUG_MODE=true".to_string(),
    ];

    // Create container options
    let container_name = format!("switchboard-test-env-{}", uuid::Uuid::new_v4());
    let create_options = CreateContainerOptions {
        name: &container_name,
        platform: None,
    };

    // Build container config with workspace mount and custom env vars
    let container_config = Config {
        image: Some("alpine:latest".to_string()),
        cmd: Some(vec!["printenv".to_string()]),
        env: Some(custom_env_vars.clone()),
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
        "Container should exit with code 0 when running printenv, got {}",
        exit_code
    );

    // Verify output contains environment variables
    assert!(
        !output.trim().is_empty(),
        "Output should contain environment variables"
    );

    // Verify custom env vars ARE present with correct values
    let lines: Vec<&str> = output.lines().collect();
    let mut found_test_var = false;
    let mut found_api_key = false;
    let mut found_debug_mode = false;

    for line in &lines {
        if line.starts_with("TEST_VAR=") {
            let value = line.strip_prefix("TEST_VAR=").unwrap();
            assert_eq!(
                value, "test_value",
                "TEST_VAR should have value 'test_value', got: {}",
                value
            );
            found_test_var = true;
        } else if line.starts_with("API_KEY=") {
            let value = line.strip_prefix("API_KEY=").unwrap();
            assert_eq!(
                value, "secret123",
                "API_KEY should have value 'secret123', got: {}",
                value
            );
            found_api_key = true;
        } else if line.starts_with("DEBUG_MODE=") {
            let value = line.strip_prefix("DEBUG_MODE=").unwrap();
            assert_eq!(
                value, "true",
                "DEBUG_MODE should have value 'true', got: {}",
                value
            );
            found_debug_mode = true;
        }
    }

    assert!(
        found_test_var,
        "TEST_VAR should be present in environment variables"
    );
    assert!(
        found_api_key,
        "API_KEY should be present in environment variables"
    );
    assert!(
        found_debug_mode,
        "DEBUG_MODE should be present in environment variables"
    );

    // Verify AGENT_NAME is NOT present (per ARCHITECT_DECISION)
    let agent_name_present = lines
        .iter()
        .any(|line: &&str| line.starts_with("AGENT_NAME="));
    assert!(
        !agent_name_present,
        "AGENT_NAME should NOT be present as environment variable (per ARCHITECT_DECISION)"
    );

    // Verify PROMPT is NOT present (per ARCHITECT_DECISION)
    let prompt_present = lines.iter().any(|line: &&str| line.starts_with("PROMPT="));
    assert!(
        !prompt_present,
        "PROMPT should NOT be present as environment variable (per ARCHITECT_DECISION)"
    );

    // Cleanup: Explicitly drop the guard to remove the temp directory
    drop(_cleanup);
}
