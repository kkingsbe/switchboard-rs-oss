//! Integration tests for the `switchboard list` command

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use std::fs;
use tempfile::TempDir;

/// Test listing agents with a valid config file containing multiple agents
#[test]
fn test_list_with_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create the prompt file referenced in the config
    let prompt_path = temp_dir.path().join("prompt.txt");
    fs::write(&prompt_path, "Test prompt content").unwrap();

    // Create a valid switchboard.toml config file with multiple agents
    let config_content = r#"
[[agent]]
name = "test-agent-1"
schedule = "0 0 9 * * *"
prompt_file = "./prompt.txt"

[[agent]]
name = "test-agent-2"
schedule = "0 10 * * *"
prompt = "Inline prompt for agent 2"

[[agent]]
name = "test-agent-3"
schedule = "0 11 * * *"
prompt = "Another inline prompt"
readonly = true
timeout = "5m"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the list command with the config path
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("test-agent-1"))
        .stdout(predicates::str::contains("test-agent-2"))
        .stdout(predicates::str::contains("test-agent-3"))
        .stdout(predicates::str::contains("Name"))
        .stdout(predicates::str::contains("Schedule"))
        .stdout(predicates::str::contains("Prompt"))
        .stdout(predicates::str::contains("Readonly"))
        .stdout(predicates::str::contains("Timeout"))
        .stdout(predicates::str::contains("Next Run"));
}

/// Test listing agents using the default config path (no --config flag)
#[test]
fn test_list_with_default_config_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create the prompt file referenced in the config
    let prompt_path = temp_dir.path().join("prompt.txt");
    fs::write(&prompt_path, "Test prompt content").unwrap();

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt_file = "./prompt.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the list command without --config flag from the temp directory
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["list"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("test-agent"))
        .stdout(predicates::str::contains("0 9 * * *"));
}

/// Test listing agents with a missing config file
#[test]
fn test_list_with_missing_config() {
    // Run the list command with a non-existent config file
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", "/nonexistent/switchboard.toml", "list"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Configuration parsing failed"));
}

/// Test listing agents with an empty config (no agents defined)
#[test]
fn test_list_empty_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file with no agents
    let config_content = r#"
[settings]
timezone = "UTC"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the list command expecting failure due to validation
    // (CLI validation requires at least one agent to be defined)
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "list"])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Configuration must define at least one agent",
        ));
}

/// Test that long prompts are truncated in the list output
#[test]
fn test_list_prompt_truncation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create an agent with a very long prompt (>50 chars)
    let long_prompt = "This is an extremely long prompt that definitely exceeds fifty characters in length and should be truncated in the output";

    let config_content = format!(
        r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt = "{}"
"#,
        long_prompt
    );

    fs::write(&config_path, config_content).unwrap();

    // Run the list command and verify the prompt is truncated with "..."
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("test-agent"))
        .stdout(predicates::str::contains("..."))
        .stdout(predicates::str::is_match(long_prompt).unwrap().not());
}
