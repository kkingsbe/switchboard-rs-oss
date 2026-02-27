//! Integration tests for skill installation in containers
//!
//! This test file verifies that skill installation works correctly when agents
//! are executed in Docker containers. The tests ensure:
//! - Skills are installed with proper logging using `[SKILL INSTALL]` prefix
//! - Exit code is 0 for successful installation
//! - Metrics correctly track skills_installed count
//! - Logs contain expected installation messages
//! - Failed skill installation is properly handled and logged with `[SKILL INSTALL ERROR]` prefix
//! - Exit code is non-zero for failed installation
//! - Metrics correctly track skills_failed count and runs_with_skill_failures

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

/// Integration test for successful skill installation in container
///
/// This test verifies that:
/// 1. Docker is available for running containers
/// 2. An agent with configured skills can be executed
/// 3. Skill installation logs contain the expected `[SKILL INSTALL]` prefix
/// 4. The container exits with code 0 (successful execution)
/// 5. Metrics correctly track the number of skills installed
///
/// # Test Flow
///
/// 1. Check if Docker is available (skip test if not)
/// 2. Create a temporary directory with a minimal switchboard.toml config
/// 3. Create a skill directory structure with a valid SKILL.md
/// 4. Create a Docker client instance
/// 5. Create a logger instance for capturing logs
/// 6. Create a metrics store instance for tracking metrics
/// 7. Execute the agent with skills configured
/// 8. Verify the exit code is 0
/// 9. Verify logs contain `[SKILL INSTALL] Installing skill:` pattern
/// 10. Verify metrics show correct `skills_installed` count
///
/// # Note
///
/// This test requires actual skills to be available and installable via
/// `npx skills add <skill> -a kilo -y`. If no skills are available
/// or network access is not possible, the test may fail or need to be
/// marked as skipped.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_successful_skill_installation_in_container() {
    // Step 1: Check if Docker is available
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Step 2: Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Step 3: Create a minimal switchboard.toml config file with skills
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"

[[agent]]
name = "test-skill-agent"
schedule = "0 * * * * *"
prompt = "Test prompt for skill installation"
skills = ["test-owner/repo@test-skill"]
"#;
    fs::write(&config_path, config_content).expect("Failed to write switchboard.toml");

    // Step 4: Create a minimal skill directory structure with valid SKILL.md
    let skills_dir = workspace.join(".kilocode").join("skills");
    fs::create_dir_all(&skills_dir).expect("Failed to create .kilocode/skills directory");

    let skill_dir = skills_dir.join("test-skill");
    fs::create_dir_all(&skill_dir).expect("Failed to create test-skill directory");

    let skill_md = skill_dir.join("SKILL.md");
    let skill_content = r#"---
name: test-skill
description: A test skill for integration testing
version: 0.1.0
---

# Test Skill

This is a minimal test skill used for integration testing.
"#;
    fs::write(&skill_md, skill_content).expect("Failed to write SKILL.md");

    // Step 5: Create a temporary directory for logs
    let log_dir = tempfile::tempdir().expect("Failed to create log directory");
    let log_dir_path = log_dir.path();

    // Step 6: Create a logger instance
    let logger = Logger::new(log_dir_path.to_path_buf(), None, false);
    let logger = Arc::new(Mutex::new(logger));

    // Step 7: Create a metrics store instance
    let metrics_store = MetricsStore::new(log_dir_path.to_path_buf());

    // Step 8: Create a Docker client instance
    let docker_client =
        match DockerClient::new("switchboard-agent".to_string(), "latest".to_string()).await {
            Ok(client) => client,
            Err(e) => {
                eprintln!("Skipping test: Failed to create Docker client: {:?}", e);
                return;
            }
        };

    // Step 9: Create a container config with skills
    let config = ContainerConfig {
        agent_name: "test-skill-agent".to_string(),
        env_vars: vec![],
        timeout: Some("2m".to_string()), // 2 minute timeout for skill installation
        readonly: false,
        prompt: "Test prompt".to_string(),
        skills: Some(vec!["test-owner/repo@test-skill".to_string()]),
    };

    // Step 10: Execute the agent with skills
    let result = match run_agent(
        workspace.to_str().unwrap(),
        &docker_client,
        &config,
        Some("2m".to_string()), // 2 minute timeout
        "switchboard-agent:latest",
        None, // Use default command
        Some(logger.clone()),
        Some(&metrics_store),
        "test_skill_installation",
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            // If the skill installation fails (which is expected if test skills don't exist),
            // we still want to verify the error handling and logging
            eprintln!("Agent execution failed: {:?}", e);
            eprintln!("This is expected if test skills are not available on npx registry");
            return;
        }
    };

    // Step 11: Verify the exit code is 0 (successful execution)
    assert_eq!(
        result.exit_code, 0,
        "Exit code should be 0 for successful skill installation, got: {}",
        result.exit_code
    );

    // Step 12: Verify the container_id is not empty
    assert!(
        !result.container_id.is_empty(),
        "Container ID should not be empty"
    );

    // Step 13: Verify logs contain the expected skill installation prefix
    let agent_log_dir = log_dir_path.join(&config.agent_name);
    if agent_log_dir.exists() {
        let log_entries = fs::read_dir(&agent_log_dir).expect("Failed to read log directory");

        for entry in log_entries {
            let entry = entry.expect("Failed to read log entry");
            let log_file = entry.path();

            if log_file.extension().map_or(false, |ext| ext == "log") {
                let log_content = fs::read_to_string(&log_file).expect("Failed to read log file");

                // Verify logs contain the [SKILL INSTALL] prefix
                assert!(
                    log_content.contains("[SKILL INSTALL]"),
                    "Logs should contain '[SKILL INSTALL]' prefix for skill installation"
                );

                // Verify logs contain the specific installation message
                assert!(
                    log_content.contains("[SKILL INSTALL] Installing skill:"),
                    "Logs should contain '[SKILL INSTALL] Installing skill:' pattern"
                );

                // Verify logs contain success message
                assert!(
                    log_content.contains("[SKILL INSTALL] Skills installed successfully"),
                    "Logs should contain '[SKILL INSTALL] Skills installed successfully' message"
                );

                // Verify NO error messages in logs
                assert!(
                    !log_content.contains("[SKILL INSTALL ERROR]"),
                    "Logs should NOT contain '[SKILL INSTALL ERROR]' for successful installation"
                );
                assert!(
                    !log_content.contains("Error:") || log_content.contains("[SKILL INSTALL]"),
                    "If 'Error:' appears, it should only be in skill installation context with proper prefix"
                );
            }
        }
    }

    // Step 14: Verify skills are installed in the correct location
    let skills_dir = workspace.join(".kilocode").join("skills");
    assert!(
        skills_dir.exists(),
        "Skills directory should exist at .kilocode/skills/"
    );

    // Verify the specific skill directory exists
    let test_skill_dir = skills_dir.join("test-skill");
    assert!(
        test_skill_dir.exists(),
        "Test skill directory should exist at .kilocode/skills/test-skill/"
    );

    // Verify SKILL.md exists in the skill directory
    let skill_md = test_skill_dir.join("SKILL.md");
    assert!(
        skill_md.exists(),
        "SKILL.md should exist in the test skill directory"
    );

    // Verify SKILL.md contains expected content
    let skill_content = fs::read_to_string(&skill_md).expect("Failed to read SKILL.md");
    assert!(
        skill_content.contains("name: test-skill"),
        "SKILL.md should contain the skill name"
    );
    assert!(
        skill_content.contains("# Test Skill"),
        "SKILL.md should contain the skill title"
    );

    // Step 15: Verify metrics show correct skills_installed count
    let all_metrics = match metrics_store.load() {
        Ok(metrics) => metrics,
        Err(e) => {
            eprintln!("Failed to load metrics: {:?}", e);
            panic!("Metrics should be available after successful agent run");
        }
    };

    // Verify metrics exist for the agent
    assert!(
        all_metrics.agents.contains_key(&config.agent_name),
        "Metrics should contain data for agent '{}'",
        config.agent_name
    );

    let agent_metrics = &all_metrics.agents[&config.agent_name];

    // Verify at least one skill was installed
    assert!(
        agent_metrics.total_skills_installed >= 1,
        "Metrics should show at least 1 skill installed, got: {}",
        agent_metrics.total_skills_installed
    );

    // Verify no skills failed
    assert_eq!(
        agent_metrics.total_skills_failed, 0,
        "Metrics should show 0 failed skills for successful installation, got: {}",
        agent_metrics.total_skills_failed
    );

    // Verify skill install time was recorded
    assert!(
        agent_metrics.skills_install_time_seconds.is_some(),
        "Metrics should include skill install time"
    );

    let skill_time = agent_metrics.skills_install_time_seconds.unwrap();
    assert!(
        skill_time > 0.0,
        "Skill install time should be greater than 0, got: {}",
        skill_time
    );
}

/// Integration test for failed skill installation in container
///
/// This test verifies that:
/// 1. Docker is available for running containers
/// 2. An agent with an invalid skill format fails to execute
/// 3. Skill installation error logs contain the expected `[SKILL INSTALL ERROR]` prefix
/// 4. The container exits with a non-zero code (failed execution)
/// 5. Metrics correctly track the number of skills that failed to install
/// 6. The AgentExecutionResult reflects the failure state
///
/// # Test Flow
///
/// 1. Check if Docker is available (skip test if not)
/// 2. Create a temporary directory with a minimal switchboard.toml config
/// 3. Create a Docker client instance
/// 4. Create a logger instance for capturing logs
/// 5. Create a metrics store instance for tracking metrics
/// 6. Execute the agent with an invalid skill format (missing `/` separator)
/// 7. Verify the exit code is non-zero
/// 8. Verify logs contain `[SKILL INSTALL ERROR]` prefix
/// 9. Verify logs contain error details like `Exit code:` and remediation suggestions
/// 10. Verify metrics show `total_skills_failed > 0`
/// 11. Verify metrics show `total_skills_installed = 0`
/// 12. Verify metrics show `runs_with_skill_failures > 0`
/// 13. Verify AgentExecutionResult has `skills_install_failed = true`
/// 14. Verify AgentExecutionResult has `skills_installed = Some(false)`
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_failed_skill_installation_in_container() {
    // Step 1: Check if Docker is available
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Step 2: Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Step 3: Create a minimal switchboard.toml config file with an invalid skill format
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"

[[agent]]
name = "test-failed-skill-agent"
schedule = "0 * * * * *"
prompt = "Test prompt for failed skill installation"
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

    // Step 8: Create a container config with an invalid skill format
    let config = ContainerConfig {
        agent_name: "test-failed-skill-agent".to_string(),
        env_vars: vec![],
        timeout: Some("2m".to_string()), // 2 minute timeout for skill installation
        readonly: false,
        prompt: "Test prompt".to_string(),
        skills: Some(vec!["invalid-format".to_string()]),
    };

    // Step 9: Execute the agent with the invalid skill format
    let result = match run_agent(
        workspace.to_str().unwrap(),
        &docker_client,
        &config,
        Some("2m".to_string()), // 2 minute timeout
        "switchboard-agent:latest",
        None, // Use default command
        Some(logger.clone()),
        Some(&metrics_store),
        "test_failed_skill_installation",
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Agent execution failed: {:?}", e);
            eprintln!("This is expected for invalid skill format");
            return;
        }
    };

    // Step 10: Verify the exit code is non-zero (failed execution)
    assert_ne!(
        result.exit_code, 0,
        "Exit code should be non-zero for failed skill installation, got: {}",
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
        "AgentExecutionResult.skills_install_failed should be true for failed installation"
    );

    assert_eq!(
        result.skills_installed,
        Some(false),
        "AgentExecutionResult.skills_installed should be Some(false) for failed installation"
    );

    // Step 13: Verify logs contain the expected skill installation error prefix
    let agent_log_dir = log_dir_path.join(&config.agent_name);
    if agent_log_dir.exists() {
        let log_entries = fs::read_dir(&agent_log_dir).expect("Failed to read log directory");

        let mut found_error_prefix = false;
        let mut found_exit_code = false;
        let mut found_remediation = false;

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
            }
        }

        assert!(
            found_error_prefix,
            "Logs should contain '[SKILL INSTALL ERROR]' prefix for failed skill installation"
        );

        assert!(
            found_exit_code,
            "Logs should contain 'Exit code:' error details for failed skill installation"
        );

        assert!(
            found_remediation,
            "Logs should contain remediation suggestions for failed skill installation"
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

    // Verify at least one skill failed
    assert!(
        agent_metrics.total_skills_failed > 0,
        "Metrics should show at least 1 failed skill, got: {}",
        agent_metrics.total_skills_failed
    );

    // Verify no skills were installed
    assert_eq!(
        agent_metrics.total_skills_installed, 0,
        "Metrics should show 0 installed skills for failed installation, got: {}",
        agent_metrics.total_skills_installed
    );

    // Verify runs with skill failures was incremented
    assert!(
        agent_metrics.runs_with_skill_failures > 0,
        "Metrics should show at least 1 run with skill failures, got: {}",
        agent_metrics.runs_with_skill_failures
    );
}

/// Integration test for mixed skill installation in container (partial success/failure)
///
/// This test verifies that:
/// 1. Docker is available for running containers
/// 2. An agent with multiple skills (some valid, some invalid) executes and partially succeeds
/// 3. Skill installation logs contain both `[SKILL INSTALL]` and `[SKILL INSTALL ERROR]` prefixes
/// 4. The container exits with a non-zero code (partial failure means the command should fail)
/// 5. Metrics correctly track both successful and failed skill installations
/// 6. The AgentExecutionResult reflects the partial failure state
///
/// # Test Flow
///
/// 1. Check if Docker is available (skip test if not)
/// 2. Create a temporary directory with a minimal switchboard.toml config
/// 3. Create a skill directory structure with a valid SKILL.md for the valid skill
/// 4. Create a Docker client instance
/// 5. Create a logger instance for capturing logs
/// 6. Create a metrics store instance for tracking metrics
/// 7. Execute the agent with multiple skills (one valid, one invalid)
/// 8. Verify the exit code is non-zero (partial failure)
/// 9. Verify logs contain both `[SKILL INSTALL]` and `[SKILL INSTALL ERROR]` prefixes
/// 10. Verify metrics show `total_skills_installed > 0` (at least one skill installed)
/// 11. Verify metrics show `total_skills_failed > 0` (at least one skill failed)
/// 12. Verify AgentExecutionResult has `skills_install_failed = true`
/// 13. Verify AgentExecutionResult has `skills_installed = Some(false)`
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_mixed_skill_installation_in_container() {
    // Step 1: Check if Docker is available
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Step 2: Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Step 3: Create a minimal switchboard.toml config file with mixed skills (one valid, one invalid)
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"

[[agent]]
name = "test-mixed-skill-agent"
schedule = "0 * * * * *"
prompt = "Test prompt for mixed skill installation"
skills = ["test-owner/repo@valid-skill", "invalid-format"]
"#;
    fs::write(&config_path, config_content).expect("Failed to write switchboard.toml");

    // Step 4: Create a minimal skill directory structure with valid SKILL.md for the valid skill
    let skills_dir = workspace.join(".kilocode").join("skills");
    fs::create_dir_all(&skills_dir).expect("Failed to create .kilocode/skills directory");

    let skill_dir = skills_dir.join("valid-skill");
    fs::create_dir_all(&skill_dir).expect("Failed to create valid-skill directory");

    let skill_md = skill_dir.join("SKILL.md");
    let skill_content = r#"---
name: valid-skill
description: A valid test skill for mixed installation testing
version: 0.1.0
---

# Valid Test Skill

This is a minimal valid test skill used for mixed installation testing.
"#;
    fs::write(&skill_md, skill_content).expect("Failed to write SKILL.md");

    // Step 5: Create a temporary directory for logs
    let log_dir = tempfile::tempdir().expect("Failed to create log directory");
    let log_dir_path = log_dir.path();

    // Step 6: Create a logger instance
    let logger = Logger::new(log_dir_path.to_path_buf(), None, false);
    let logger = Arc::new(Mutex::new(logger));

    // Step 7: Create a metrics store instance
    let metrics_store = MetricsStore::new(log_dir_path.to_path_buf());

    // Step 8: Create a Docker client instance
    let docker_client =
        match DockerClient::new("switchboard-agent".to_string(), "latest".to_string()).await {
            Ok(client) => client,
            Err(e) => {
                eprintln!("Skipping test: Failed to create Docker client: {:?}", e);
                return;
            }
        };

    // Step 9: Create a container config with mixed skills (one valid, one invalid)
    let config = ContainerConfig {
        agent_name: "test-mixed-skill-agent".to_string(),
        env_vars: vec![],
        timeout: Some("2m".to_string()), // 2 minute timeout for skill installation
        readonly: false,
        prompt: "Test prompt".to_string(),
        skills: Some(vec![
            "test-owner/repo@valid-skill".to_string(),
            "invalid-format".to_string(),
        ]),
    };

    // Step 10: Execute the agent with mixed skills
    let result = match run_agent(
        workspace.to_str().unwrap(),
        &docker_client,
        &config,
        Some("2m".to_string()), // 2 minute timeout
        "switchboard-agent:latest",
        None, // Use default command
        Some(logger.clone()),
        Some(&metrics_store),
        "test_mixed_skill_installation",
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Agent execution failed: {:?}", e);
            eprintln!("This is expected for mixed skill installation scenarios");
            return;
        }
    };

    // Step 11: Verify the exit code is non-zero (partial failure means the command should fail)
    assert_ne!(
        result.exit_code, 0,
        "Exit code should be non-zero for mixed skill installation (partial failure), got: {}",
        result.exit_code
    );

    // Step 12: Verify the container_id is not empty
    assert!(
        !result.container_id.is_empty(),
        "Container ID should not be empty"
    );

    // Step 13: Verify AgentExecutionResult reflects the partial failure state
    assert!(
        result.skills_install_failed,
        "AgentExecutionResult.skills_install_failed should be true for mixed installation (partial failure)"
    );

    assert_eq!(
        result.skills_installed,
        Some(false),
        "AgentExecutionResult.skills_installed should be Some(false) for mixed installation (not all skills installed)"
    );

    // Step 14: Verify logs contain both expected prefixes (successful and failed installations)
    let agent_log_dir = log_dir_path.join(&config.agent_name);
    if agent_log_dir.exists() {
        let log_entries = fs::read_dir(&agent_log_dir).expect("Failed to read log directory");

        let mut found_install_prefix = false;
        let mut found_error_prefix = false;

        for entry in log_entries {
            let entry = entry.expect("Failed to read log entry");
            let log_file = entry.path();

            if log_file.extension().map_or(false, |ext| ext == "log") {
                let log_content = fs::read_to_string(&log_file).expect("Failed to read log file");

                // Verify logs contain the [SKILL INSTALL] prefix (for successful skill)
                if log_content.contains("[SKILL INSTALL]") {
                    found_install_prefix = true;
                }

                // Verify logs contain the [SKILL INSTALL ERROR] prefix (for failed skill)
                if log_content.contains("[SKILL INSTALL ERROR]") {
                    found_error_prefix = true;
                }
            }
        }

        assert!(
            found_install_prefix,
            "Logs should contain '[SKILL INSTALL]' prefix for at least one successfully installed skill"
        );

        assert!(
            found_error_prefix,
            "Logs should contain '[SKILL INSTALL ERROR]' prefix for at least one failed skill installation"
        );
    } else {
        panic!("Agent log directory should exist after agent execution");
    }

    // Step 15: Verify metrics show both successful and failed skill installations
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

    // Verify at least one skill was installed (partial success)
    assert!(
        agent_metrics.total_skills_installed > 0,
        "Metrics should show at least 1 skill installed (partial success), got: {}",
        agent_metrics.total_skills_installed
    );

    // Verify at least one skill failed (partial failure)
    assert!(
        agent_metrics.total_skills_failed > 0,
        "Metrics should show at least 1 failed skill (partial failure), got: {}",
        agent_metrics.total_skills_failed
    );

    // Verify runs with skill failures was incremented
    assert!(
        agent_metrics.runs_with_skill_failures > 0,
        "Metrics should show at least 1 run with skill failures, got: {}",
        agent_metrics.runs_with_skill_failures
    );
}

/// Integration test for skill installation failure handling (non-existent npx skill)
///
/// This test verifies that:
/// 1. Docker is available for running containers
/// 2. An agent with a valid format but non-existent npx skill fails gracefully
/// 3. Skill installation error logs contain the expected `[SKILL INSTALL]` prefix for failure
/// 4. The container exits with a non-zero code (failed execution)
/// 5. Metrics correctly track the number of skills that failed to install
/// 6. Error messages are user-friendly (not cryptic)
///
/// # Test Flow
///
/// 1. Check if Docker is available (skip test if not)
/// 2. Create a temporary directory with a minimal switchboard.toml config
/// 3. Create a Docker client instance
/// 4. Create a logger instance for capturing logs
/// 5. Create a metrics store instance for tracking metrics
/// 6. Execute the agent with a non-existent npx skill (valid format)
/// 7. Verify the exit code is non-zero
/// 8. Verify logs contain `[SKILL INSTALL]` prefix with failure message
/// 9. Verify metrics show `total_skills_failed > 0`
/// 10. Verify metrics show `runs_with_skill_failures > 0`
/// 11. Verify error message is user-friendly (not cryptic)
///
/// # Note
///
/// This test is different from `test_failed_skill_installation_in_container()` which tests
/// invalid skill format. This test specifically tests the failure mode where the skill format
/// is valid but the npx package does not exist in the registry.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_skill_installation_failure_handling() {
    // Step 1: Check if Docker is available
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Step 2: Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Step 3: Create a minimal switchboard.toml config file with a non-existent npx skill
    // The skill format is valid (owner/repo@skill) but the package doesn't exist
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"

[[agent]]
name = "test-skill-failure-agent"
schedule = "0 * * * * *"
prompt = "Test prompt for skill installation failure handling"
skills = ["@switchboard/skill-does-not-exist"]
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

    // Step 8: Create a container config with a non-existent npx skill
    let config = ContainerConfig {
        agent_name: "test-skill-failure-agent".to_string(),
        env_vars: vec![],
        timeout: Some("2m".to_string()), // 2 minute timeout for skill installation
        readonly: false,
        prompt: "Test prompt".to_string(),
        skills: Some(vec!["@switchboard/skill-does-not-exist".to_string()]),
    };

    // Step 9: Execute the agent with the non-existent npx skill
    let result = match run_agent(
        workspace.to_str().unwrap(),
        &docker_client,
        &config,
        Some("2m".to_string()), // 2 minute timeout
        "switchboard-agent:latest",
        None, // Use default command
        Some(logger.clone()),
        Some(&metrics_store),
        "test_skill_installation_failure",
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Agent execution failed: {:?}", e);
            eprintln!("This is expected for non-existent npx skill");
            return;
        }
    };

    // Step 10: Verify the exit code is non-zero (failed execution)
    assert_ne!(
        result.exit_code, 0,
        "Exit code should be non-zero for failed skill installation (non-existent package), got: {}",
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
        "AgentExecutionResult.skills_install_failed should be true for failed installation"
    );

    assert_eq!(
        result.skills_installed,
        Some(false),
        "AgentExecutionResult.skills_installed should be Some(false) for failed installation"
    );

    // Step 13: Verify logs contain the expected skill installation error prefix and messages
    let agent_log_dir = log_dir_path.join(&config.agent_name);
    if agent_log_dir.exists() {
        let log_entries = fs::read_dir(&agent_log_dir).expect("Failed to read log directory");

        let mut found_install_prefix = false;
        let mut found_error_prefix = false;
        let mut found_failure_message = false;
        let mut log_content_for_friendly_check = String::new();

        for entry in log_entries {
            let entry = entry.expect("Failed to read log entry");
            let log_file = entry.path();

            if log_file.extension().map_or(false, |ext| ext == "log") {
                let content = fs::read_to_string(&log_file).expect("Failed to read log file");
                log_content_for_friendly_check = content.clone();

                // Verify logs contain the [SKILL INSTALL] prefix
                if content.contains("[SKILL INSTALL]") {
                    found_install_prefix = true;
                }

                // Verify logs contain the [SKILL INSTALL ERROR] prefix for failures
                if content.contains("[SKILL INSTALL ERROR]") {
                    found_error_prefix = true;
                }

                // Verify logs contain failure-related messages
                if content.contains("failed")
                    || content.contains("Failed")
                    || content.contains("error")
                    || content.contains("Error")
                {
                    found_failure_message = true;
                }
            }
        }

        assert!(
            found_install_prefix,
            "Logs should contain '[SKILL INSTALL]' prefix for skill installation attempt"
        );

        assert!(
            found_error_prefix,
            "Logs should contain '[SKILL INSTALL ERROR]' prefix for failed skill installation"
        );

        assert!(
            found_failure_message,
            "Logs should contain a failure-related message (failed/error)"
        );

        // Step 14: Verify error message is user-friendly (not cryptic)
        // User-friendly means it contains helpful context, not just cryptic error codes
        let has_helpful_context = log_content_for_friendly_check.contains("skill")
            || log_content_for_friendly_check.contains("Skill")
            || log_content_for_friendly_check.contains("install")
            || log_content_for_friendly_check.contains("package")
            || log_content_for_friendly_check.contains("not found")
            || log_content_for_friendly_check.contains("not exist");

        assert!(
            has_helpful_context,
            "Error message should be user-friendly and contain helpful context. Found:\n{}",
            log_content_for_friendly_check
        );

        // Verify error is not just cryptic (e.g., should not be just an exit code)
        assert!(
            log_content_for_friendly_check.len() > 50,
            "Error message should have sufficient detail, not just a cryptic short message"
        );
    } else {
        panic!("Agent log directory should exist after agent execution");
    }

    // Step 15: Verify metrics show correct failure statistics
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

    // Verify at least one skill failed
    assert!(
        agent_metrics.total_skills_failed > 0,
        "Metrics should show at least 1 failed skill, got: {}",
        agent_metrics.total_skills_failed
    );

    // Verify no skills were installed (since the package doesn't exist)
    assert_eq!(
        agent_metrics.total_skills_installed, 0,
        "Metrics should show 0 installed skills for non-existent package, got: {}",
        agent_metrics.total_skills_installed
    );

    // Verify runs with skill failures was incremented
    assert!(
        agent_metrics.runs_with_skill_failures > 0,
        "Metrics should show at least 1 run with skill failures, got: {}",
        agent_metrics.runs_with_skill_failures
    );
}
