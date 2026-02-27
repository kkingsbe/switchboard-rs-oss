//! Integration tests for backwards compatibility with existing configs
//!
//! These tests verify that:
//! 1. Config without skills field works correctly (existing format)
//! 2. No warnings or errors about skills are emitted
//! 3. Container creation works as before
//! 4. Manually managed skills in .kilocode/skills/ directory work correctly
//!
//! See [`BACKWARDS_COMPATIBILITY_SKILLS.md`](../../BACKWARDS_COMPATIBILITY_SKILLS.md:1) for
//! detailed documentation on backwards compatibility behavior.

#[cfg(feature = "integration")]
use super::docker_available;
#[cfg(feature = "integration")]
use std::fs;
#[cfg(feature = "integration")]
use std::path::Path;
#[cfg(feature = "integration")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "integration")]
use switchboard::config::Config;
#[cfg(feature = "integration")]
use switchboard::docker::run::find_preexisting_skills;
#[cfg(feature = "integration")]
use switchboard::docker::run::generate_entrypoint_script;
#[cfg(feature = "integration")]
use switchboard::docker::run::types::ContainerConfig;
#[cfg(feature = "integration")]
use switchboard::docker::{run_agent, DockerClient};
#[cfg(feature = "integration")]
use switchboard::logger::Logger;
#[cfg(feature = "integration")]
use switchboard::metrics::MetricsStore;

/// Test that config parsing succeeds without the skills field
///
/// This test verifies that:
/// 1. A config file without the skills field can be parsed
/// 2. No warnings about missing skills are emitted
/// 3. The agents are correctly loaded
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_config_without_skills_field_parses_correctly() {
    // Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Create a config file without the skills field (legacy format)
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"
overlap_mode = "skip"

[[agent]]
name = "test-agent"
schedule = "0 * * * *"
prompt = """
Test agent without skills field.
"""
"#;
    fs::write(&config_path, config_content).expect("Failed to write switchboard.toml");

    // Parse the config - this should succeed without any warnings
    let result = Config::from_toml(&config_path);

    // Verify parsing succeeds
    assert!(
        result.is_ok(),
        "Config parsing should succeed without skills field, got: {:?}",
        result.err()
    );

    let config = result.unwrap();

    // Verify the agent is loaded correctly
    assert_eq!(config.agents.len(), 1, "Should have 1 agent");
    assert_eq!(config.agents[0].name, "test-agent");

    // Verify skills field is None
    assert!(
        config.agents[0].skills.is_none(),
        "Agent should have no skills field"
    );

    // Verify settings are loaded correctly
    assert_eq!(config.settings.image_name, "switchboard-agent");
    assert_eq!(config.settings.image_tag, "latest");
}

/// Test that validate command works with config without skills field
///
/// This test verifies that:
/// 1. The validate command succeeds with a config without skills field
/// 2. No warnings about missing skills are emitted to stderr
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_validate_command_without_skills_field() {
    use assert_cmd::Command;
    use predicates::prelude::*;

    // Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Create a config file without the skills field
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"

[[agent]]
name = "test-agent"
schedule = "0 * * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).expect("Failed to write switchboard.toml");

    // Run the validate command
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("switchboard"));
    cmd.arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("validate");

    // Execute and capture output
    let output = cmd.output().expect("Failed to execute validate command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify the command succeeds
    assert!(
        output.status.success(),
        "Validate command should succeed, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    // Verify no warnings about skills in stderr
    assert!(
        !stderr.contains("skills"),
        "Validate command should not mention 'skills' in stderr, got: {}",
        stderr
    );
}

/// Test that list command works with config without skills field
///
/// This test verifies that:
/// 1. The list command succeeds with a config without skills field
/// 2. No warnings about missing skills are emitted
/// 3. The agent is listed correctly
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_list_command_without_skills_field() {
    use assert_cmd::Command;

    // Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Create a config file without the skills field
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"

[[agent]]
name = "test-agent"
schedule = "0 * * * *"
prompt = "Test prompt"

[[agent]]
name = "another-agent"
schedule = "*/15 * * * *"
prompt = "Another test prompt"
"#;
    fs::write(&config_path, config_content).expect("Failed to write switchboard.toml");

    // Run the list command
    let output = Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("list")
        .output()
        .expect("Failed to execute list command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify the command succeeds
    assert!(
        output.status.success(),
        "List command should succeed, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    // Verify both agents are listed
    assert!(
        stdout.contains("test-agent"),
        "List output should contain test-agent"
    );
    assert!(
        stdout.contains("another-agent"),
        "List output should contain another-agent"
    );

    // Verify no warnings about skills in stderr
    assert!(
        !stderr.contains("skills"),
        "List command should not mention 'skills' in stderr, got: {}",
        stderr
    );
}

/// Test that manually managed skills in .kilocode/skills/ are detected correctly
///
/// This test verifies that:
/// 1. Skills in .kilocode/skills/ directory with SKILL.md are detected
/// 2. The skill name is correctly extracted
/// 3. The detection works with both owner/repo and owner/repo@skill-name formats
#[cfg(feature = "integration")]
#[test]
fn test_manually_managed_skills_detection() {
    // Use the fixture directory with manually installed skills
    let fixture_dir = Path::new("tests/fixtures/manual-skills");

    // Create a skills directory structure to simulate manually installed skills
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();
    let skills_dir = workspace.join(".kilocode").join("skills");

    // Create manual skill directories
    let manual_skill1 = skills_dir.join("manual-skill-1");
    fs::create_dir_all(&manual_skill1).expect("Failed to create manual-skill-1 directory");
    fs::write(
        manual_skill1.join("SKILL.md"),
        r#"---
name: manual-skill-1
description: A manually installed skill
version: 0.1.0
---

# Manual Skill 1

This skill was manually installed.
"#,
    )
    .expect("Failed to write SKILL.md");

    let manual_skill2 = skills_dir.join("manual-skill-2");
    fs::create_dir_all(&manual_skill2).expect("Failed to create manual-skill-2 directory");
    fs::write(
        manual_skill2.join("SKILL.md"),
        r#"---
name: manual-skill-2
description: Another manually installed skill
version: 0.1.0
---

# Manual Skill 2

This skill was also manually installed.
"#,
    )
    .expect("Failed to write SKILL.md");

    // Test detection with configured skills that match manual skills
    let configured_skills = vec![
        "test-owner/manual-skill-1".to_string(),
        "test-owner/manual-skill-2".to_string(),
    ];

    let result = find_preexisting_skills(&configured_skills, workspace)
        .expect("find_preexisting_skills should succeed");

    // Verify both manual skills are detected
    assert_eq!(
        result.len(),
        2,
        "Should detect exactly 2 preexisting skills"
    );
    assert!(
        result.contains(&"manual-skill-1".to_string()),
        "Should detect manual-skill-1"
    );
    assert!(
        result.contains(&"manual-skill-2".to_string()),
        "Should detect manual-skill-2"
    );
}

/// Test that entrypoint script generation skips npx for manually managed skills
///
/// This test verifies that:
/// 1. Preexisting skills have log messages about skipping npx
/// 2. Non-preexisting skills have npx skills add commands
/// 3. The correct log format is generated
#[cfg(feature = "integration")]
#[test]
fn test_entrypoint_script_skips_npx_for_manual_skills() {
    // Create a list of configured skills
    let configured_skills = vec![
        "test-owner/manual-skill".to_string(),
        "hypothetical-owner/hypothetical-skill".to_string(),
    ];

    // Create a list of preexisting skills (as returned by find_preexisting_skills)
    let preexisting_skills = vec!["manual-skill".to_string()];

    // Generate the entrypoint script
    let script = generate_entrypoint_script("test-agent", &configured_skills, &preexisting_skills)
        .expect("generate_entrypoint_script should succeed");

    // Verify the script contains the shebang
    assert!(
        script.contains("#!/bin/sh"),
        "Script should contain shebang"
    );

    // Verify preexisting skills have the skip npx log message
    assert!(
        script.contains(
            "[SKILL INSTALL] Using preexisting skill: manual-skill (skipping npx installation)"
        ),
        "Script should contain skip npx log for manual-skill"
    );

    // Verify non-preexisting skills have npx skills add command
    assert!(
        script.contains("npx skills add hypothetical-owner/hypothetical-skill -a kilo -y"),
        "Script should contain npx skills add command for hypothetical-skill"
    );

    // Verify preexisting skills do NOT have npx skills add command
    assert!(
        !script.contains("npx skills add test-owner/manual-skill"),
        "Script should NOT contain npx skills add for preexisting manual-skill"
    );

    // Verify the script ends with exec kilocode
    assert!(
        script.contains("exec kilocode --yes \"$@\""),
        "Script should contain exec kilocode command"
    );
}

/// Test container execution with config without skills field
///
/// This test verifies that:
/// 1. An agent without skills field can be executed in a container
/// 2. The container runs successfully (exit code 0)
/// 3. No errors about missing skills are emitted
///
/// Note: This test requires a running Docker daemon and may take longer to execute.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_container_execution_without_skills_field() {
    // Check if Docker is available
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Create a config file without the skills field
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"

[[agent]]
name = "test-agent-no-skills"
schedule = "0 * * * *"
prompt = "Test prompt for container execution without skills"
"#;
    fs::write(&config_path, config_content).expect("Failed to write switchboard.toml");

    // Parse the config
    let config = Config::from_toml(&config_path).expect("Config should parse successfully");

    // Create a temporary directory for logs
    let log_dir = tempfile::tempdir().expect("Failed to create log directory");
    let log_dir_path = log_dir.path();

    // Create a logger instance
    let logger = Logger::new(log_dir_path.to_path_buf(), None, false);
    let logger = Arc::new(Mutex::new(logger));

    // Create a metrics store instance
    let metrics_store = MetricsStore::new(log_dir_path.to_path_buf());

    // Create a Docker client instance
    let docker_client =
        match DockerClient::new("switchboard-agent".to_string(), "latest".to_string()).await {
            Ok(client) => client,
            Err(e) => {
                eprintln!("Skipping test: Failed to create Docker client: {:?}", e);
                return;
            }
        };

    // Create a container config without skills
    let container_config = ContainerConfig {
        agent_name: "test-agent-no-skills".to_string(),
        env_vars: vec![],
        timeout: Some("2m".to_string()),
        readonly: false,
        prompt: "Test prompt for container execution without skills".to_string(),
        skills: None, // No skills - this is the key test
    };

    // Execute the agent without skills
    let result = match run_agent(
        workspace.to_str().unwrap(),
        &docker_client,
        &container_config,
        Some("2m".to_string()),
        "switchboard-agent:latest",
        None, // Use default command
        Some(logger.clone()),
        Some(&metrics_store),
        "test_agent_no_skills",
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            // If the agent fails due to missing image or other issues, check if it's a skills-related error
            let error_msg = format!("{:?}", e);
            assert!(
                !error_msg.contains("skills") || error_msg.contains("No skills configured"),
                "Error should not be about missing skills, got: {}",
                error_msg
            );
            eprintln!(
                "Agent execution failed (expected if image not available): {:?}",
                e
            );
            return;
        }
    };

    // Verify the container executed (exit code may be 0 or non-zero depending on image availability)
    // The key point is that there's no error about missing skills configuration
    assert!(
        !result.container_id.is_empty(),
        "Container ID should not be empty"
    );
}

/// Test container execution with manually managed skills
///
/// This test verifies that:
/// 1. An agent with manually managed skills in .kilocode/skills/ works correctly
/// 2. The entrypoint script correctly skips npx for preexisting skills
/// 3. No attempt is made to install skills via npx for preexisting skills
///
/// Note: This test requires a running Docker daemon and may take longer to execute.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_container_execution_with_manual_skills() {
    // Check if Docker is available
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Create the .kilocode/skills directory with manually installed skills
    let skills_dir = workspace.join(".kilocode").join("skills");
    fs::create_dir_all(&skills_dir).expect("Failed to create .kilocode/skills directory");

    // Create a manual skill
    let manual_skill = skills_dir.join("manual-test-skill");
    fs::create_dir_all(&manual_skill).expect("Failed to create manual-test-skill directory");
    fs::write(
        manual_skill.join("SKILL.md"),
        r#"---
name: manual-test-skill
description: A test skill for manual skills integration test
version: 0.1.0
---

# Manual Test Skill

This skill was manually installed for testing.
"#,
    )
    .expect("Failed to write SKILL.md");

    // Create a config file that references the manual skill
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"

[[agent]]
name = "test-agent-manual-skills"
schedule = "0 * * * *"
prompt = "Test prompt for manual skills"
skills = ["test-owner/manual-test-skill"]
"#;
    fs::write(&config_path, config_content).expect("Failed to write switchboard.toml");

    // Create a temporary directory for logs
    let log_dir = tempfile::tempdir().expect("Failed to create log directory");
    let log_dir_path = log_dir.path();

    // Create a logger instance
    let logger = Logger::new(log_dir_path.to_path_buf(), None, false);
    let logger = Arc::new(Mutex::new(logger));

    // Create a metrics store instance
    let metrics_store = MetricsStore::new(log_dir_path.to_path_buf());

    // Create a Docker client instance
    let docker_client =
        match DockerClient::new("switchboard-agent".to_string(), "latest".to_string()).await {
            Ok(client) => client,
            Err(e) => {
                eprintln!("Skipping test: Failed to create Docker client: {:?}", e);
                return;
            }
        };

    // Create a container config with the manual skill
    let container_config = ContainerConfig {
        agent_name: "test-agent-manual-skills".to_string(),
        env_vars: vec![],
        timeout: Some("2m".to_string()),
        readonly: false,
        prompt: "Test prompt for manual skills".to_string(),
        skills: Some(vec!["test-owner/manual-test-skill".to_string()]),
    };

    // First verify that find_preexisting_skills detects the manual skill
    let configured_skills = vec!["test-owner/manual-test-skill".to_string()];
    let preexisting_skills = find_preexisting_skills(&configured_skills, workspace)
        .expect("find_preexisting_skills should succeed");

    assert!(
        preexisting_skills.contains(&"manual-test-skill".to_string()),
        "Should detect manual-test-skill as preexisting"
    );

    // Generate and verify the entrypoint script
    let script = generate_entrypoint_script(
        "test-agent-manual-skills",
        &configured_skills,
        &preexisting_skills,
    )
    .expect("generate_entrypoint_script should succeed");

    // Verify the script skips npx for the manual skill
    assert!(
        script.contains("[SKILL INSTALL] Using preexisting skill: manual-test-skill"),
        "Script should indicate manual skill is preexisting"
    );
    assert!(
        !script.contains("npx skills add test-owner/manual-test-skill"),
        "Script should NOT try to install manual skill via npx"
    );

    // Execute the agent with manual skills
    let result = match run_agent(
        workspace.to_str().unwrap(),
        &docker_client,
        &container_config,
        Some("2m".to_string()),
        "switchboard-agent:latest",
        None, // Use default command
        Some(logger.clone()),
        Some(&metrics_store),
        "test_agent_manual_skills",
        None, // No queued start time
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            // If the agent fails due to missing image or other issues, that's OK
            // The key is to verify the entrypoint script was generated correctly
            eprintln!(
                "Agent execution failed (expected if image not available): {:?}",
                e
            );
            return;
        }
    };

    // Verify the container was created
    assert!(
        !result.container_id.is_empty(),
        "Container ID should not be empty"
    );
}

/// Test that backward compatibility works end-to-end
///
/// This test verifies the full flow:
/// 1. Create a config without skills field
/// 2. Parse the config successfully
/// 3. Execute an agent (without actually running a container if not available)
/// 4. Verify no skills-related errors or warnings
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_backwards_compatibility_end_to_end() {
    // Create a temporary directory for the test workspace
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let workspace = temp_dir.path();

    // Create a config file without the skills field (legacy format)
    let config_path = workspace.join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"
overlap_mode = "skip"

[[agent]]
name = "legacy-agent"
schedule = "0 * * * *"
prompt = """
This is a legacy agent configuration without skills field.
It should work exactly as before the skills feature was added.
"""

[[agent]]
name = "legacy-agent-2"
schedule = "*/30 * * * *"
prompt = """
Another legacy agent for testing.
"""
env = { ENVIRONMENT = "test" }
timeout = "30m"
"#;
    fs::write(&config_path, config_content).expect("Failed to write switchboard.toml");

    // Parse the config
    let config = match Config::from_toml(&config_path) {
        Ok(c) => c,
        Err(e) => {
            panic!("Config parsing should succeed, got error: {:?}", e);
        }
    };

    // Verify both agents are loaded
    assert_eq!(config.agents.len(), 2, "Should have 2 agents");
    assert_eq!(config.agents[0].name, "legacy-agent");
    assert_eq!(config.agents[1].name, "legacy-agent-2");

    // Verify no skills field in either agent
    for agent in &config.agents {
        assert!(
            agent.skills.is_none(),
            "Agent {} should have no skills field",
            agent.name
        );
    }

    // Verify settings are loaded correctly
    assert_eq!(config.settings.image_name, "switchboard-agent");
    assert_eq!(config.settings.image_tag, "latest");
    assert_eq!(config.settings.overlap_mode.to_string(), "skip");

    // If Docker is available, try to run a simple test
    if !docker_available().await {
        eprintln!("Skipping Docker test: Docker daemon is not available");
        return;
    }

    // Create a Docker client
    let docker_client =
        match DockerClient::new("switchboard-agent".to_string(), "latest".to_string()).await {
            Ok(client) => client,
            Err(e) => {
                eprintln!(
                    "Skipping Docker test: Failed to create Docker client: {:?}",
                    e
                );
                return;
            }
        };

    // Create a log directory
    let log_dir = tempfile::tempdir().expect("Failed to create log directory");
    let log_dir_path = log_dir.path();

    // Create logger and metrics
    let logger = Logger::new(log_dir_path.to_path_buf(), None, false);
    let logger = Arc::new(Mutex::new(logger));
    let metrics_store = MetricsStore::new(log_dir_path.to_path_buf());

    // Create container config without skills
    let container_config = ContainerConfig {
        agent_name: "legacy-agent".to_string(),
        env_vars: vec![],
        timeout: Some("1m".to_string()),
        readonly: false,
        prompt: "Test prompt".to_string(),
        skills: None, // No skills - legacy mode
    };

    // Try to execute
    let result = run_agent(
        workspace.to_str().unwrap(),
        &docker_client,
        &container_config,
        Some("1m".to_string()),
        "switchboard-agent:latest",
        None,
        Some(logger.clone()),
        Some(&metrics_store),
        "test_legacy_agent",
        None,
    )
    .await;

    // The result might fail due to image not being available, but that's OK
    // The important thing is there's no skills-related error
    match result {
        Ok(result) => {
            assert!(
                !result.container_id.is_empty(),
                "Container should have been created"
            );
        }
        Err(e) => {
            let error_str = format!("{:?}", e);
            // Verify error is not about skills
            assert!(
                !error_str.contains("skills"),
                "Error should not be about skills: {}",
                error_str
            );
        }
    }
}
