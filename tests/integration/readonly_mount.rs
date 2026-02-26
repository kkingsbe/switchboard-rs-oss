//! Integration test for read-only mount enforcement
//!
//! This test verifies that when a workspace is mounted as read-only,
//! the container cannot write to it, and the host filesystem is not
//! modified.

#[cfg(feature = "integration")]
use super::docker_available;
#[cfg(feature = "integration")]
use switchboard::docker::run::types::ContainerConfig;
#[cfg(feature = "integration")]
use switchboard::docker::DockerClient;

/// Test that read-only mount enforcement prevents writes to the workspace
///
/// This test verifies that:
/// - When a container runs with `readonly: true`, the workspace is mounted as read-only
/// - The container cannot create files in the workspace
/// - The host filesystem is not modified by the container's attempt to write
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_readonly_mount_enforcement() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Create a temporary directory to use as the workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace_path = temp_dir.path();

    // Use scopeguard to ensure the temporary directory is cleaned up even on panic
    let _cleanup = scopeguard::guard(&temp_dir, |dir| {
        let _ = std::fs::remove_dir_all(dir.path());
    });

    // Create a DockerClient instance
    let client = match DockerClient::new("test-image".to_string(), "latest".to_string()).await {
        Ok(c) => c,
        Err(e) => {
            panic!("Failed to create DockerClient: {:?}", e);
        }
    };

    // Create a ContainerConfig with readonly: true
    let config = ContainerConfig {
        agent_name: "test-readonly-agent".to_string(),
        env_vars: vec![],
        timeout: None,
        readonly: true, // This is the key setting - mount as read-only
        prompt: String::new(),
        skills: None,
    };

    // Prepare a command that attempts to write to the workspace
    let cmd = vec![
        "sh".to_string(),
        "-c".to_string(),
        "touch /workspace/test.txt".to_string(),
    ];

    // Run the agent with the temporary directory as workspace
    let result = match switchboard::docker::run::run_agent(
        workspace_path
            .to_str()
            .expect("Failed to convert workspace path to string"),
        &client,
        &config,
        None,            // No timeout
        "alpine:latest", // Use a simple image for testing
        Some(&cmd),
        None, // No logger
        None, // No metrics store
        "test_readonly_mount",
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            panic!("Failed to run agent: {:?}", e);
        }
    };

    // Verify the exit code is non-zero (the write should fail)
    assert!(
        result.exit_code != 0,
        "Exit code should be non-zero when writing to read-only mount, got: {}",
        result.exit_code
    );

    // Verify the container ID is not empty
    assert!(
        !result.container_id.is_empty(),
        "Container ID should not be empty"
    );

    // Verify the test.txt file was NOT created in the temporary directory on the host
    let test_file_path = workspace_path.join("test.txt");
    assert!(
        !test_file_path.exists(),
        "File should not exist on host when workspace is mounted read-only: {:?}",
        test_file_path
    );

    // The temporary directory will be automatically cleaned up when temp_dir goes out of scope
}
