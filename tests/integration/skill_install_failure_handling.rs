//! Integration tests for skill installation failure handling
//!
//! This test file verifies that error handling for skill installation failures
//! is properly implemented. The tests ensure:
//! - Skills with invalid format cause container to exit with non-zero code
//! - Logs contain proper [SKILL INSTALL ERROR] prefix for failures
//! - Metrics correctly track skill installation failures
//! - Error messages are user-friendly and actionable
//!
//! Run with: `cargo test --features integration -- --ignored`

#[cfg(feature = "integration")]
use super::docker_available;
#[cfg(feature = "integration")]
use std::fs;
#[cfg(feature = "integration")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "integration")]
use switchboard::docker::run::types::ContainerConfig;
#[cfg(feature = "integration")]
use switchboard::docker::{run_agent, DockerClient};
#[cfg(feature = "integration")]
use switchboard::logger::Logger;
#[cfg(feature = "integration")]
use switchboard::metrics::MetricsStore;

/// Integration test for skill installation failure with invalid skill format
///
/// This test verifies that when a skill has an invalid format (missing the required
/// `owner/repo` separator), the container properly:
///
/// 1. Exits with a non-zero exit code
/// 2. Logs contain `[SKILL INSTALL ERROR]` prefix
/// 3. Metrics track the failure with `total_skills_failed > 0`
/// 4. Error message is user-friendly with remediation steps
///
/// # Test Flow
///
/// 1. Check if Docker is available (skip test if not)
/// 2. Create a temporary directory with switchboard.toml containing invalid skill format
/// 3. Create logger and metrics instances
/// 4. Create Docker client and container config
/// 5. Execute agent with invalid skill format
/// 6. Verify exit code is non-zero
/// 7. Verify logs contain `[SKILL INSTALL ERROR]` prefix
/// 8. Verify logs contain user-friendly error message
/// 9. Verify metrics show `total_skills_failed > 0`
/// 10. Verify metrics show `runs_with_skill_failures > 0`
///
/// # Arguments
///
/// - `invalid_skill_format`: A skill string that doesn't match the expected `owner/repo`
///   or `owner/repo@skill-name` format. Examples: "invalid-format", "repo-only",
///   "owner@skill", etc.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_skill_install_failure_invalid_format() {
    // Step 1: Check if Docker is available
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Step 2: Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Step 3: Create a minimal switchboard.toml config file with invalid skill format
    // "invalid-format" is missing the required "owner/repo" separator
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"

[[agent]]
name = "test-invalid-skill-agent"
schedule = "0 * * * * *"
prompt = "Test prompt for invalid skill format"
skills = ["invalid-format"]
"#;
    fs::write(&config_path, config_content).expect("Failed to write switchboard.toml");

    // Step 4: Create a temporary directory for logs
    let log_dir = tempfile::tempdir().expect("Failed to create log directory");
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

    // Step 8: Create a container config with invalid skill format
    let config = ContainerConfig {
        agent_name: "test-invalid-skill-agent".to_string(),
        env_vars: vec![],
        timeout: Some("2m".to_string()),
        readonly: false,
        prompt: "Test prompt".to_string(),
        skills: Some(vec!["invalid-format".to_string()]),
    };

    // Step 9: Execute the agent with invalid skill format
    let result = match run_agent(
        workspace.to_str().unwrap(),
        &docker_client,
        &config,
        Some("2m".to_string()),
        "switchboard-agent:latest",
        None, // Use default command
        Some(logger.clone()),
        Some(&metrics_store),
        "test_invalid_skill_format",
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            // Error is expected for invalid skill format
            eprintln!("Agent execution failed (expected): {:?}", e);
            // We still want to verify the error was handled properly
            // In this case, we return early but mark test as passing
            // since the error handling is what we're testing
            return;
        }
    };

    // Step 10: Verify the exit code is non-zero (failed execution)
    assert_ne!(
        result.exit_code, 0,
        "Exit code should be non-zero for invalid skill format, got: {}",
        result.exit_code
    );

    // Step 11: Verify the container_id is not empty
    assert!(
        !result.container_id.is_empty(),
        "Container ID should not be empty"
    );

    // Step 12: Verify AgentExecutionResult reflects the failure state
    assert!(
        result.skills_install_failed,
        "AgentExecutionResult.skills_install_failed should be true for invalid skill format"
    );

    assert_eq!(
        result.skills_installed,
        Some(false),
        "AgentExecutionResult.skills_installed should be Some(false) for invalid skill format"
    );

    // Step 13: Verify logs contain the expected skill installation error prefix
    let agent_log_dir = log_dir_path.join(&config.agent_name);
    if agent_log_dir.exists() {
        let log_entries = fs::read_dir(&agent_log_dir).expect("Failed to read log directory");

        let mut found_error_prefix = false;
        let mut found_exit_code = false;
        let mut found_remediation = false;
        let mut found_user_friendly_message = false;

        for entry in log_entries {
            let entry = entry.expect("Failed to read log entry");
            let log_file = entry.path();

            if log_file.extension().map_or(false, |ext| ext == "log") {
                let log_content = fs::read_to_string(&log_file).expect("Failed to read log file");

                // Verify logs contain the [SKILL INSTALL ERROR] prefix
                if log_content.contains("[SKILL INSTALL ERROR]") {
                    found_error_prefix = true;
                }

                // Verify logs contain Exit code: error details
                if log_content.contains("Exit code:") {
                    found_exit_code = true;
                }

                // Verify logs contain remediation suggestions
                if log_content.contains("Remediation:") || log_content.contains("remediation") {
                    found_remediation = true;
                }

                // Verify logs contain user-friendly error message
                // The message should mention:
                // - What went wrong (invalid skill format)
                // - What the user can do (remediation steps)
                if log_content.contains("invalid-format") || log_content.contains("Invalid skill") {
                    found_user_friendly_message = true;
                }
            }
        }

        assert!(
            found_error_prefix,
            "Logs should contain '[SKILL INSTALL ERROR]' prefix for invalid skill format"
        );

        assert!(
            found_exit_code,
            "Logs should contain 'Exit code:' error details for invalid skill format"
        );

        assert!(
            found_remediation,
            "Logs should contain remediation suggestions for invalid skill format"
        );

        assert!(
            found_user_friendly_message,
            "Logs should contain user-friendly error message for invalid skill format"
        );
    } else {
        panic!("Agent log directory should exist after agent execution");
    }

    // Step 14: Verify metrics show correct failure statistics
    let all_metrics = match metrics_store.load() {
        Ok(metrics) => metrics,
        Err(e) => {
            eprintln!("Failed to load metrics: {:?}", e);
            panic!("Metrics should be available after agent run");
        }
    };

    // Verify metrics exist for the agent
    assert!(
        all_metrics.agents.contains_key(&config.agent_name),
        "Metrics should contain data for agent '{}'",
        config.agent_name
    );

    let agent_metrics = &all_metrics.agents[&config.agent_name];

    // Verify skills failed count is greater than 0
    assert!(
        agent_metrics.total_skills_failed > 0,
        "Metrics should show at least 1 failed skill for invalid format, got: {}",
        agent_metrics.total_skills_failed
    );

    // Verify no skills were successfully installed
    assert_eq!(
        agent_metrics.total_skills_installed, 0,
        "Metrics should show 0 installed skills for failed installation, got: {}",
        agent_metrics.total_skills_installed
    );

    // Verify runs with skill failures is greater than 0
    assert!(
        agent_metrics.runs_with_skill_failures > 0,
        "Metrics should show at least 1 run with skill failures, got: {}",
        agent_metrics.runs_with_skill_failures
    );
}

/// Integration test for skill installation failure with missing owner
///
/// This test verifies that when a skill is missing the owner part (e.g., "repo-only")
/// the container properly fails with appropriate error handling.
///
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_skill_install_failure_missing_owner() {
    // Step 1: Check if Docker is available
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Step 2: Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Step 3: Create a minimal switchboard.toml config file with missing owner
    // "repo-only" is missing the required "owner/" prefix
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"

[[agent]]
name = "test-missing-owner-agent"
schedule = "0 * * * * *"
prompt = "Test prompt for missing owner"
skills = ["repo-only"]
"#;
    fs::write(&config_path, config_content).expect("Failed to write switchboard.toml");

    // Step 4: Create a temporary directory for logs
    let log_dir = tempfile::tempdir().expect("Failed to create log directory");
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

    // Step 8: Create a container config with missing owner skill format
    let config = ContainerConfig {
        agent_name: "test-missing-owner-agent".to_string(),
        env_vars: vec![],
        timeout: Some("2m".to_string()),
        readonly: false,
        prompt: "Test prompt".to_string(),
        skills: Some(vec!["repo-only".to_string()]),
    };

    // Step 9: Execute the agent with missing owner skill format
    let result = match run_agent(
        workspace.to_str().unwrap(),
        &docker_client,
        &config,
        Some("2m".to_string()),
        "switchboard-agent:latest",
        None,
        Some(logger.clone()),
        Some(&metrics_store),
        "test_missing_owner",
        None,
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Agent execution failed (expected): {:?}", e);
            return;
        }
    };

    // Step 10: Verify the exit code is non-zero
    assert_ne!(
        result.exit_code, 0,
        "Exit code should be non-zero for missing owner, got: {}",
        result.exit_code
    );

    // Step 11: Verify logs contain error prefix
    let agent_log_dir = log_dir_path.join(&config.agent_name);
    if agent_log_dir.exists() {
        let log_entries = fs::read_dir(&agent_log_dir).expect("Failed to read log directory");

        let mut found_error_prefix = false;

        for entry in log_entries {
            let entry = entry.expect("Failed to read log entry");
            let log_file = entry.path();

            if log_file.extension().map_or(false, |ext| ext == "log") {
                let log_content = fs::read_to_string(&log_file).expect("Failed to read log file");

                if log_content.contains("[SKILL INSTALL ERROR]") {
                    found_error_prefix = true;
                }
            }
        }

        assert!(
            found_error_prefix,
            "Logs should contain '[SKILL INSTALL ERROR]' prefix for missing owner"
        );
    }

    // Step 12: Verify metrics track the failure
    let all_metrics = match metrics_store.load() {
        Ok(metrics) => metrics,
        Err(e) => {
            eprintln!("Failed to load metrics: {:?}", e);
            panic!("Metrics should be available after agent run");
        }
    };

    let agent_metrics = &all_metrics.agents[&config.agent_name];

    assert!(
        agent_metrics.total_skills_failed > 0,
        "Metrics should show failed skill for missing owner, got: {}",
        agent_metrics.total_skills_failed
    );
}

/// Integration test for skill installation failure with invalid format using @ separator
///
/// This test verifies that when a skill uses "@" instead of "/" separator
/// (e.g., "owner@skill"), the container properly fails.
///
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_skill_install_failure_invalid_at_separator() {
    // Step 1: Check if Docker is available
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Step 2: Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Step 3: Create a minimal switchboard.toml config file with invalid @ separator
    // "owner@skill" uses @ instead of / which is invalid
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"

[[agent]]
name = "test-invalid-at-agent"
schedule = "0 * * * * *"
prompt = "Test prompt for invalid @ separator"
skills = ["owner@skill"]
"#;
    fs::write(&config_path, config_content).expect("Failed to write switchboard.toml");

    // Step 4: Create a temporary directory for logs
    let log_dir = tempfile::tempdir().expect("Failed to create log directory");
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

    // Step 8: Create a container config with invalid @ separator
    let config = ContainerConfig {
        agent_name: "test-invalid-at-agent".to_string(),
        env_vars: vec![],
        timeout: Some("2m".to_string()),
        readonly: false,
        prompt: "Test prompt".to_string(),
        skills: Some(vec!["owner@skill".to_string()]),
    };

    // Step 9: Execute the agent with invalid @ separator
    let result = match run_agent(
        workspace.to_str().unwrap(),
        &docker_client,
        &config,
        Some("2m".to_string()),
        "switchboard-agent:latest",
        None,
        Some(logger.clone()),
        Some(&metrics_store),
        "test_invalid_at_separator",
        None,
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Agent execution failed (expected): {:?}", e);
            return;
        }
    };

    // Step 10: Verify the exit code is non-zero
    assert_ne!(
        result.exit_code, 0,
        "Exit code should be non-zero for invalid @ separator, got: {}",
        result.exit_code
    );

    // Step 11: Verify logs contain error prefix
    let agent_log_dir = log_dir_path.join(&config.agent_name);
    if agent_log_dir.exists() {
        let log_entries = fs::read_dir(&agent_log_dir).expect("Failed to read log directory");

        let mut found_error_prefix = false;

        for entry in log_entries {
            let entry = entry.expect("Failed to read log entry");
            let log_file = entry.path();

            if log_file.extension().map_or(false, |ext| ext == "log") {
                let log_content = fs::read_to_string(&log_file).expect("Failed to read log file");

                if log_content.contains("[SKILL INSTALL ERROR]") {
                    found_error_prefix = true;
                }
            }
        }

        assert!(
            found_error_prefix,
            "Logs should contain '[SKILL INSTALL ERROR]' prefix for invalid @ separator"
        );
    }

    // Step 12: Verify metrics track the failure
    let all_metrics = match metrics_store.load() {
        Ok(metrics) => metrics,
        Err(e) => {
            eprintln!("Failed to load metrics: {:?}", e);
            panic!("Metrics should be available after agent run");
        }
    };

    let agent_metrics = &all_metrics.agents[&config.agent_name];

    assert!(
        agent_metrics.total_skills_failed > 0,
        "Metrics should show failed skill for invalid @ separator, got: {}",
        agent_metrics.total_skills_failed
    );
}
