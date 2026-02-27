//! Integration tests for `switchboard validate` command

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test validating a configuration with a valid config file
#[test]
fn test_validate_with_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create the prompt file referenced in the config
    let prompt_path = temp_dir.path().join("prompt.txt");
    fs::write(&prompt_path, "Test prompt content").unwrap();

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt_file = "./prompt.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the validate command with the config path
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "validate"])
        .assert()
        .success()
        .stdout(predicates::str::contains("✓ Configuration valid"))
        .stdout(predicates::str::contains("1 agent(s) defined"));
}

/// Test validating a configuration with a missing config file
#[test]
fn test_validate_with_missing_file() {
    // Run the validate command with a non-existent config file
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", "/nonexistent/file.toml", "validate"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("✗ Configuration parsing failed"));
}

/// Test validating a configuration with invalid TOML syntax
#[test]
fn test_validate_with_invalid_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create a file with invalid TOML syntax
    fs::write(
        &config_path,
        r#"
version = "1.0"
[[agents]
name = "test-agent"
"#,
    )
    .unwrap();

    // Run the validate command with the invalid config
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "validate"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("✗ Configuration parsing failed"));
}

/// Test validating a configuration using the default path (no --config flag)
#[test]
fn test_validate_default_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create the prompt file referenced in the config
    let prompt_path = temp_dir.path().join("prompt.txt");
    fs::write(&prompt_path, "Test prompt content").unwrap();

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt_file = "./prompt.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the validate command without --config flag from the temp directory
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["validate"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("✓ Configuration valid"));
}

/// Test that CLI binary exists and --help flag works
#[test]
fn test_cli_help() {
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage:"));
}

/// Test that CLI binary runs without crashing (basic invocation)
#[test]
fn test_cli_runs() {
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .arg("--help")
        .assert()
        .success();
}

/// Test 'up' command without arguments
#[test]
fn test_up_command() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[settings]
image_name = "test-agent"
image_tag = "latest"

[[agent]]
name = "test-agent"
schedule = "0 0 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the up command with the config path
    // Note: This will fail because Docker is not available in the test environment
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "up"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Docker connection error"));
}

/// Test 'up' command with --detach flag
#[test]
fn test_up_command_with_detach() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[settings]
image_name = "test-agent"
image_tag = "latest"

[[agent]]
name = "test-agent"
schedule = "0 0 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the up command with --detach flag
    // Note: This will fail because Docker is not available in the test environment
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "up", "--detach"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Docker connection error"));
}

/// Test 'up' command with -d flag (short form)
#[test]
fn test_up_command_with_short_detach() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[settings]
image_name = "test-agent"
image_tag = "latest"

[[agent]]
name = "test-agent"
schedule = "0 0 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the up command with -d (short form of --detach)
    // Note: This will fail because Docker is not available in the test environment
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "up", "-d"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Docker connection error"));
}

/// Test 'run' command with an agent name
#[test]
fn test_run_command() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create the prompt file referenced in the config
    let prompt_path = temp_dir.path().join("prompt.txt");
    fs::write(&prompt_path, "Test prompt content").unwrap();

    // Create a valid switchboard.toml config file with test-agent
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt_file = "./prompt.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the run command with the config path
    // Note: This will fail because Docker is not available in the test environment
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "run",
            "test-agent",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Docker connection error"));
}

/// Test 'run' command with a non-existent agent
#[test]
fn test_run_command_agent_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create the prompt file referenced in the config
    let prompt_path = temp_dir.path().join("prompt.txt");
    fs::write(&prompt_path, "Test prompt content").unwrap();

    // Create a valid switchboard.toml config file with test-agent
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt_file = "./prompt.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the run command with a non-existent agent name
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "run",
            "nonexistent-agent",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Agent 'nonexistent-agent' not found in configuration",
        ));
}

/// Test 'run' command with a missing config file
#[test]
fn test_run_command_missing_config_file() {
    // Run the run command with a non-existent config file
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            "/nonexistent/switchboard.toml",
            "run",
            "test-agent",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Configuration parsing failed"));
}

/// Test 'run' command with invalid TOML config
#[test]
fn test_run_command_invalid_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create a file with invalid TOML syntax
    fs::write(
        &config_path,
        r#"
version = "1.0"
[[agents]
name = "test-agent"
"#,
    )
    .unwrap();

    // Run the run command with the invalid config
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "run",
            "test-agent",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Configuration parsing failed"));
}

/// Test 'run' command with a missing prompt file
#[test]
fn test_run_command_prompt_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file with a non-existent prompt file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt_file = "./nonexistent.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the run command expecting failure
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "run",
            "test-agent",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Prompt file not found"));
}

/// Test 'run' command with both prompt and prompt_file (validation error)
#[test]
fn test_run_command_both_prompt_and_prompt_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file with both prompt and prompt_file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Inline prompt"
prompt_file = "./prompt.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the run command expecting validation failure
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "run",
            "test-agent",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Configuration validation failed"))
        .stderr(predicates::str::contains(
            "exactly one of 'prompt' or 'prompt_file' specified, not both",
        ));
}

/// Test 'run' command with neither prompt nor prompt_file (validation error)
#[test]
fn test_run_command_neither_prompt_nor_prompt_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file without prompt or prompt_file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the run command expecting validation failure
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "run",
            "test-agent",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Configuration validation failed"))
        .stderr(predicates::str::contains(
            "must have either 'prompt' (inline text) or 'prompt_file' (path to file) specified",
        ));
}

/// Test 'run' command with --config flag properly parsed
#[test]
fn test_run_command_with_config_flag() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create the prompt file referenced in the config
    let prompt_path = temp_dir.path().join("prompt.txt");
    fs::write(&prompt_path, "Test prompt content").unwrap();

    // Create a valid switchboard.toml config file with test-agent
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt_file = "./prompt.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the run command with --config flag
    // Note: This will fail because Docker is not available in the test environment
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "run",
            "test-agent",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Docker connection error"));
}

/// Test 'run' command without required argument (should fail)
#[test]
fn test_run_command_missing_agent_name() {
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .arg("run")
        .assert()
        .failure()
        .stderr(predicates::str::contains("error"));
}

/// Test 'build' command without arguments
#[test]
fn test_build_command() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a Dockerfile in the temp directory
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    fs::write(&dockerfile_path, "FROM alpine:latest\nRUN echo 'test'").unwrap();

    // Create a valid switchboard.toml config file
    let config_content = r#"
[settings]
image_name = "test-agent"
image_tag = "latest"

[[agent]]
name = "test-agent"
schedule = "0 0 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the build command with the config path
    // Note: This will fail because Docker is not available in the test environment
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "build"])
        .assert()
        .failure()
        .stdout(predicates::str::contains("Config loaded"));
}

/// Test 'build' command with --no-cache flag
#[test]
fn test_build_command_with_no_cache() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a Dockerfile in the temp directory
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    fs::write(&dockerfile_path, "FROM alpine:latest\nRUN echo 'test'").unwrap();

    // Create a valid switchboard.toml config file
    let config_content = r#"
[settings]
image_name = "test-agent"
image_tag = "latest"

[[agent]]
name = "test-agent"
schedule = "0 0 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the build command with --no-cache flag
    // Note: This will fail because Docker is not available in the test environment
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "build",
            "--no-cache",
        ])
        .assert()
        .failure()
        .stdout(predicates::str::contains("Config loaded"));
}

/// Test validating a configuration with invalid skill source formats
///
/// This test validates that the `switchboard validate` command properly detects
/// and reports invalid skill source formats. Valid formats are:
/// - `owner/repo`
/// - `owner/repo@skill-name`
///
/// Invalid formats tested:
/// - Missing owner: `repo-only`
/// - Missing repo: `owner@only`
/// - Multiple slashes: `owner/repo/extra`
/// - Empty string
#[test]
fn test_validate_invalid_skill_format() {
    // Test invalid skill format: missing owner (repo-only)
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            "tests/fixtures/invalid-skill-missing-owner.toml",
            "validate",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Configuration validation failed"));

    // Test invalid skill format: missing repo (owner@only)
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            "tests/fixtures/invalid-skill-missing-repo.toml",
            "validate",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Configuration validation failed"));

    // Test invalid skill format: multiple slashes (owner/repo/extra)
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            "tests/fixtures/invalid-skill-multiple-slashes.toml",
            "validate",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Configuration validation failed"));

    // Test invalid skill format: empty string
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            "tests/fixtures/invalid-skill-empty.toml",
            "validate",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Configuration validation failed"));
}
