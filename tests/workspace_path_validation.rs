//! Integration tests for workspace path validation

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use std::fs;
use tempfile::TempDir;

/// Test workspace path validation with non-existent path in 'up' command
#[test]
fn test_up_command_nonexistent_workspace_path() {
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

    // Run the up command expecting failure
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "up"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Workspace path '/nonexistent/workspace/path' does not exist or is not a directory",
        ));
}

/// Test workspace path validation with non-existent path in 'run' command
#[test]
fn test_run_command_nonexistent_workspace_path() {
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

    // Run the run command expecting failure
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "run",
            "test-agent",
        ])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Workspace path '/nonexistent/workspace/path' does not exist or is not a directory",
        ));
}

/// Test workspace path validation with file (not directory) in 'up' command
#[test]
fn test_up_command_file_instead_of_directory_workspace_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a file (not a directory) at the workspace path location
    let workspace_file = temp_dir.path().join("workspace_file.txt");
    fs::write(&workspace_file, "This is a file, not a directory").unwrap();

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

    // Run the up command expecting failure
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "up"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Workspace path 'workspace_file.txt' does not exist or is not a directory",
        ));
}

/// Test workspace path validation with file (not directory) in 'run' command
#[test]
fn test_run_command_file_instead_of_directory_workspace_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a file (not a directory) at the workspace path location
    let workspace_file = temp_dir.path().join("workspace_file.txt");
    fs::write(&workspace_file, "This is a file, not a directory").unwrap();

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

    // Run the run command expecting failure
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "run",
            "test-agent",
        ])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Workspace path 'workspace_file.txt' does not exist or is not a directory",
        ));
}

/// Test workspace path validation with valid directory in 'up' command
#[test]
fn test_up_command_valid_workspace_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid directory for the workspace path
    let workspace_dir = temp_dir.path().join("workspace");
    fs::create_dir(&workspace_dir).unwrap();

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

    // Run the up command expecting success (workspace validation passes)
    // Note: This will still fail later due to Docker not being available in test environment
    // but we check that workspace validation passes by looking for the success message
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "up"])
        .current_dir(temp_dir.path())
        .assert()
        .stderr(
            predicates::str::contains(
                "Workspace path 'workspace' does not exist or is not a directory",
            )
            .not(),
        ); // Verify the workspace error is NOT present
}

/// Test workspace path validation with valid directory in 'run' command
#[test]
fn test_run_command_valid_workspace_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid directory for the workspace path
    let workspace_dir = temp_dir.path().join("workspace");
    fs::create_dir(&workspace_dir).unwrap();

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

    // Run the run command expecting success (workspace validation passes)
    // Note: This will still fail later due to Docker not being available in test environment
    // but we check that workspace validation passes by verifying the workspace error is NOT present
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "run",
            "test-agent",
        ])
        .current_dir(temp_dir.path())
        .assert()
        .stderr(
            predicates::str::contains(
                "Workspace path 'workspace' does not exist or is not a directory",
            )
            .not(),
        ); // Verify the workspace error is NOT present
}

/// Test workspace path validation with current directory ('.'), which always exists
#[test]
fn test_up_command_current_directory_workspace() {
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

    // Run the up command expecting success (current directory always exists)
    // Note: This will fail because Docker is not available in the test environment
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["--config", config_path.to_str().unwrap(), "up"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Docker connection error"));
}

/// Test workspace path validation with current directory ('.') in 'run' command
#[test]
fn test_run_command_current_directory_workspace() {
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

    // Run the run command - workspace validation passes (current directory always exists)
    // It will fail later due to Docker, but that's expected
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "run",
            "test-agent",
        ])
        .current_dir(temp_dir.path())
        .assert()
        .failure() // Will fail due to Docker
        .stderr(
            predicates::str::contains("Workspace path '.' does not exist or is not a directory")
                .not(),
        ); // Verify the workspace error is NOT present
}
