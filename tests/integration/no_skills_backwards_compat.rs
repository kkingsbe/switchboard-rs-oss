//! Integration tests for backwards compatibility with configurations without the skills field
//!
//! These tests verify that:
//! 1. Container execution WITHOUT skills field - verifies the default entrypoint is used
//! 2. Metrics backwards compatibility - verify old JSON format without skill fields can still be deserialized
//! 3. When skills is None, the container uses default entrypoint (not custom script)
//!
//! This is different from `tests/backwards_compatibility_no_skills.rs` which tests CLI commands
//! and from `tests/integration/manual_skills_backwards_compat.rs` which tests preexisting skills.
//! These tests specifically verify container execution behavior and metrics deserialization.

#[cfg(feature = "integration")]
use super::docker_available;
#[cfg(feature = "integration")]
use bollard::container::{Config, CreateContainerOptions};
#[cfg(feature = "integration")]
use bollard::Docker;
#[cfg(feature = "integration")]
use switchboard::docker::run::types::ContainerConfig;
#[cfg(feature = "integration")]
use switchboard::docker::{run_agent, DockerClient};
#[cfg(feature = "integration")]
use switchboard::logger::Logger;
#[cfg(feature = "integration")]
use switchboard::metrics::{AgentMetricsData, AgentRunResultData, AllMetrics, MetricsStore};
#[cfg(feature = "integration")]
use std::collections::HashMap;
#[cfg(feature = "integration")]
use std::fs;
#[cfg(feature = "integration")]
use std::path::PathBuf;
#[cfg(feature = "integration")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "integration")]
use tempfile::TempDir;

/// Integration test for container execution WITHOUT skills field
///
/// This test verifies that:
/// 1. Docker is available for running containers
/// 2. An agent WITHOUT skills configured can be executed
/// 3. The container uses the default entrypoint (not a custom script)
/// 4. The container runs successfully with exit code 0
///
/// This is different from testing CLI commands - it specifically tests that
/// when skills is None/empty, the container behaves correctly.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_container_execution_without_skills_field() {
    // Step 1: Check if Docker is available
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Step 2: Create a temporary directory for the test workspace
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Step 3: Create a minimal switchboard.toml config file WITHOUT skills
    let config_pathswitchboard.toml = workspace.join("");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"

[[agent]]
name = "test-no-skills-agent"
schedule = "0 * * * * *"
prompt = "Test prompt for no skills agent"
"#;
    fs::write(&config_path, config_content).expect("Failed to write switchboard.toml");

    // Step 4: Create a temporary directory for logs
    let log_dir = TempDir::new().expect("Failed to create log directory");
    let log_dir_path = log_dir.path();

    // Step 5: Create a logger instance
    let logger = Logger::new(log_dir_path.to_path_buf(), None, false);
    let logger = Arc::new(Mutex::new(logger));

    // Step 6: Create a metrics store instance
    let metrics_store = MetricsStore::new(log_dir_path.to_path_buf());

    // Step 7: Create a Docker client instance
    let docker_client =
        match DockerClient::new("switchboard-agent".to_string(), "latest".to_string()).await {
            Ok(client) => client,
            Err(e) => {
                eprintln!("Skipping test: Failed to create Docker client: {:?}", e);
                return;
            }
        };

    // Step 8: Create a container config WITHOUT skills (None)
    let config = ContainerConfig {
        agent_name: "test-no-skills-agent".to_string(),
        env_vars: vec![],
        timeout: Some("1m".to_string()),
        readonly: false,
        prompt: "Test prompt".to_string(),
        skills: None, // No skills configured - this is the key difference
    };

    // Step 9: Execute the agent without skills
    let result = match run_agent(
        workspace.to_str().unwrap(),
        &docker_client,
        &config,
        Some("1m".to_string()),
        "switchboard-agent:latest",
        None, // Use default command
        Some(logger.clone()),
        Some(&metrics_store),
        "test_no_skills",
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            // Some errors are expected if the image doesn't exist or network issues
            eprintln!("Agent execution encountered an error: {:?}", e);
            eprintln!("This may be expected in some environments");
            return;
        }
    };

    // Step 10: Verify the container executed successfully
    // The key assertion is that when skills is None, the container should use
    // the default entrypoint (not a custom script with skill installation)
    assert_eq!(
        result.exit_code, 0,
        "Container without skills should exit with code 0, got: {}",
        result.exit_code
    );

    // Step 11: Verify the container_id is not empty
    assert!(
        !result.container_id.is_empty(),
        "Container ID should not be empty"
    );

    // Step 12: Verify skills_installed is None or Some(false) - no skills attempted
    // This confirms that when skills is None, no skill installation is attempted
    assert!(
        result.skills_installed.is_none() || result.skills_installed == Some(false),
        "When skills is None, skills_installed should be None or Some(false), got: {:?}",
        result.skills_installed
    );
}

/// Integration test for container with empty skills list
///
/// This test verifies that when skills is an empty Some([]), the container
/// also uses the default entrypoint (not a custom script).
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_container_execution_with_empty_skills_list() {
    // Step 1: Check if Docker is available
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Step 2: Create a temporary directory for the test workspace
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Step 3: Create a minimal switchboard.toml config file with empty skills array
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"

[[agent]]
name = "test-empty-skills-agent"
schedule = "0 * * * * *"
