//! Integration tests for backwards compatibility with configurations without the skills field
//!
//! These tests verify that old-style projects without the `skills` field continue to work correctly.
//! The skills field is optional, and configurations without it should parse successfully,
//! validate without errors, and run all switchboard commands normally.
//!
//! See [`BACKWARDS_COMPATIBILITY_SKILLS.md`](../../BACKWARDS_COMPATIBILITY_SKILLS.md:1) for
//! detailed documentation on backwards compatibility behavior.

#[cfg(feature = "integration")]
use assert_cmd::Command;
#[cfg(feature = "integration")]
use predicates::prelude::*;

/// Test that `switchboard validate` succeeds with a config without the skills field
#[cfg(feature = "integration")]
#[test]
#[ignore = "Run with --ignored to execute integration tests"]
fn test_validate_command_no_skills() {
    // Run the validate command with test-no-skills.toml
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", "test-no-skills.toml", "validate"])
        .assert()
        .success()
        .stdout(predicates::str::contains("valid").or(predicates::str::contains("success")))
        .stderr(predicates::str::contains("skills").not());
}

/// Test that `switchboard list` shows both agents without the skills field
#[cfg(feature = "integration")]
#[test]
#[ignore = "Run with --ignored to execute integration tests"]
fn test_list_command_no_skills() {
    // Run the list command with test-no-skills.toml
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", "test-no-skills.toml", "list"])
        .assert()
        .success()
        // Should show both agents defined in test-no-skills.toml
        .stdout(predicates::str::contains("simple-agent"))
        .stdout(predicates::str::contains("comprehensive-agent"))
        // Should show standard list columns
        .stdout(predicates::str::contains("Name"))
        .stdout(predicates::str::contains("Schedule"))
        .stdout(predicates::str::contains("Prompt"))
        // Should not show any warnings about missing skills
        .stderr(predicates::str::contains("skills").not());
}

/// Test that `switchboard status` works without warnings about missing skills
#[cfg(feature = "integration")]
#[test]
#[ignore = "Run with --ignored to execute integration tests"]
fn test_status_command_no_skills() {
    // Run the status command with test-no-skills.toml
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", "test-no-skills.toml", "status"])
        .assert()
        .success()
        // Should not show any warnings or errors about missing skills field
        .stderr(predicates::str::contains("skills").not())
        .stderr(predicates::str::contains("warning").not().or(
            predicates::str::contains("warning").and(predicates::str::contains("skills").not()),
        ));
}

/// Test that `switchboard help` works with a config without the skills field
#[cfg(feature = "integration")]
#[test]
#[ignore = "Run with --ignored to execute integration tests"]
fn test_help_command_no_skills() {
    // Run the help command with test-no-skills.toml
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", "test-no-skills.toml", "--help"])
        .assert()
        .success()
        // Should show help output
        .stdout(predicates::str::contains("USAGE"))
        .stdout(predicates::str::contains("FLAGS"))
        .stdout(predicates::str::contains("OPTIONS"))
        .stdout(predicates::str::contains("COMMANDS"))
        // Should not show any errors about missing skills
        .stderr(predicates::str::contains("skills").not());
}

/// Test that no error messages or warnings about missing skills field are emitted
#[cfg(feature = "integration")]
#[test]
#[ignore = "Run with --ignored to execute integration tests"]
fn test_no_warnings_about_missing_skills() {
    // Test multiple commands to ensure no skills-related warnings anywhere

    // Test validate command
    let validate_result = Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", "test-no-skills.toml", "validate"])
        .unwrap();

    let validate_output = validate_result
        .output()
        .expect("Failed to execute validate command");
    let validate_stderr = String::from_utf8_lossy(&validate_output.stderr);

    // Test list command
    let list_result = Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", "test-no-skills.toml", "list"])
        .unwrap();

    let list_output = list_result
        .output()
        .expect("Failed to execute list command");
    let list_stderr = String::from_utf8_lossy(&list_output.stderr);

    // Test status command
    let status_result = Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", "test-no-skills.toml", "status"])
        .unwrap();

    let status_output = status_result
        .output()
        .expect("Failed to execute status command");
    let status_stderr = String::from_utf8_lossy(&status_output.stderr);

    // Verify none of the commands emit warnings or errors about missing skills
    assert!(
        !validate_stderr.contains("skills"),
        "Validate command should not mention 'skills' in stderr, got: {}",
        validate_stderr
    );
    assert!(
        !list_stderr.contains("skills"),
        "List command should not mention 'skills' in stderr, got: {}",
        list_stderr
    );
    assert!(
        !status_stderr.contains("skills"),
        "Status command should not mention 'skills' in stderr, got: {}",
        status_stderr
    );
}

/// Test config parsing succeeds without the skills field using ValidateCommand
#[tokio::test]
async fn test_config_parse_no_skills_field() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test-no-skills.toml");

    // Copy the test-no-skills.toml content to a temp file
    let test_config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"
overlap_mode = "skip"

[[agent]]
name = "simple-agent"
schedule = "0 * * * *"
prompt = """
Analyze the current state of the codebase and provide a brief summary.
"""

[[agent]]
name = "comprehensive-agent"
schedule = "*/15 * * * *"
prompt = """
Review the recent changes in the codebase and identify potential issues.
Focus on code quality, performance, and security concerns.
"""
env = { API_KEY = "your-api-key-here", LOG_LEVEL = "info", MAX_RETRIES = "3" }
readonly = false
timeout = "1h"
overlap_mode = "Queue"
max_queue_size = 5
"#;

    fs::write(&config_path, test_config_content).unwrap();

    // Try to parse the config
    use switchboard::config::Config;
    let result = Config::from_toml(&config_path);

    // Verify parsing succeeds
    assert!(
        result.is_ok(),
        "Config parsing should succeed without skills field, got: {:?}",
        result.err()
    );

    let config = result.unwrap();

    // Verify both agents are loaded
    assert_eq!(config.agents.len(), 2, "Should have 2 agents");
    assert_eq!(config.agents[0].name, "simple-agent");
    assert_eq!(config.agents[1].name, "comprehensive-agent");

    // Verify skills field is None for both agents
    assert!(
        config.agents[0].skills.is_none(),
        "First agent should have no skills"
    );
    assert!(
        config.agents[1].skills.is_none(),
        "Second agent should have no skills"
    );
}
