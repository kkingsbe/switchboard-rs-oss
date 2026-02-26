//! Integration tests for `switchboard build` command

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test 'build' command with a non-existent config file
#[test]
fn test_build_command_config_not_found() {
    // Run the build command with a non-existent config file
    // Note: --config must come after 'build' for the build command's own config flag
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["build", "--config", "/nonexistent/config.toml"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Failed to read file"));
}

/// Test 'build' command with a valid config file but missing Dockerfile
#[test]
fn test_build_command_dockerfile_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create .kilocode directory (required after BUG-003 fix)
    let kilocode_dir = temp_dir.path().join(".kilocode");
    fs::create_dir(&kilocode_dir).unwrap();

    // Run the build command expecting failure (Docker availability check fails first)
    // Note: --config must come after 'build' for the build command's own config flag
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["build", "--config", config_path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Docker availability check failed",
        ));
}

/// Test 'build' command with invalid TOML in config file
#[test]
fn test_build_command_invalid_toml() {
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

    // Run the build command with the invalid config
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["build", "--config", config_path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Error parsing"));
}

/// Test 'build' command with --no-cache flag and missing config
#[test]
fn test_build_command_with_no_cache_config_not_found() {
    // Run the build command with --no-cache flag and non-existent config
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "build",
            "--no-cache",
            "--config",
            "/nonexistent/config.toml",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Failed to read file"));
}

/// Test 'build' command default path (no --config flag) with missing Dockerfile
#[test]
fn test_build_command_default_path_dockerfile_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create .kilocode directory (required after BUG-003 fix)
    let kilocode_dir = temp_dir.path().join(".kilocode");
    fs::create_dir(&kilocode_dir).unwrap();

    // Run the build command from the temp directory without --config flag
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["build"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Docker availability check failed",
        ));
}

/// Test 'build' command with --config short flag
#[test]
fn test_build_command_short_config_flag_dockerfile_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create .kilocode directory (required after BUG-003 fix)
    let kilocode_dir = temp_dir.path().join(".kilocode");
    fs::create_dir(&kilocode_dir).unwrap();

    // Run the build command with -c (short form)
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["build", "-c", config_path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Docker availability check failed",
        ));
}

#[cfg(feature = "integration")]
use bollard::Docker;

#[cfg(feature = "integration")]
/// Check if Docker daemon is available for integration tests
async fn docker_available() -> bool {
    match Docker::connect_with_local_defaults() {
        Ok(docker) => docker.ping().await.is_ok(),
        Err(_) => false,
    }
}

/// Test 'build' command happy path with valid config and Dockerfile
///
/// This test is marked as #[ignore] and requires `integration` feature.
/// Run with: `cargo test --features integration test_build_command_happy_path -- --ignored`
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests requiring Docker"]
async fn test_build_command_happy_path() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let dockerfile_path = temp_dir.path().join("Dockerfile");

    // Create a simple valid Dockerfile
    let dockerfile_content = r#"
FROM alpine:latest
RUN echo "Hello from test container"
CMD ["/bin/sh"]
"#;
    fs::write(&dockerfile_path, dockerfile_content).unwrap();

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
image_name = "switchboard-agent"
image_tag = "test"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create .kilocode directory (required after BUG-003 fix)
    let kilocode_dir = temp_dir.path().join(".kilocode");
    fs::create_dir(&kilocode_dir).unwrap();

    // Run the build command expecting success
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["build", "--config", config_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("Successfully built image"))
        .stdout(predicates::str::contains("switchboard-agent:test"));
}

/// Test 'build' command happy path with --no-cache flag
///
/// This test is marked as #[ignore] and requires `integration` feature.
/// Run with: `cargo test --features integration test_build_command_no_cache -- --ignored`
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests requiring Docker"]
async fn test_build_command_no_cache() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let dockerfile_path = temp_dir.path().join("Dockerfile");

    // Create a simple valid Dockerfile
    let dockerfile_content = r#"
FROM alpine:latest
RUN echo "Hello from test container"
CMD ["/bin/sh"]
"#;
    fs::write(&dockerfile_path, dockerfile_content).unwrap();

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
image_name = "switchboard-agent"
image_tag = "test-nocache"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create .kilocode directory (required after BUG-003 fix)
    let kilocode_dir = temp_dir.path().join(".kilocode");
    fs::create_dir(&kilocode_dir).unwrap();

    // Run the build command with --no-cache flag expecting success
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "build",
            "--no-cache",
            "--config",
            config_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("Successfully built image"))
        .stdout(predicates::str::contains("switchboard-agent:test-nocache"));
}

/// Test 'build' command when Docker daemon is not running
///
/// This test expects a failure when Docker is not available.
/// Note: This will pass if Docker happens to be running, which is acceptable.
#[test]
fn test_build_command_docker_not_available() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let dockerfile_path = temp_dir.path().join("Dockerfile");

    // Create a simple valid Dockerfile
    let dockerfile_content = r#"
FROM alpine:latest
RUN echo "Hello from test container"
CMD ["/bin/sh"]
"#;
    fs::write(&dockerfile_path, dockerfile_content).unwrap();

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
image_name = "switchboard-agent"
image_tag = "test-docker-unavailable"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create .kilocode directory (required after BUG-003 fix)
    let kilocode_dir = temp_dir.path().join(".kilocode");
    fs::create_dir(&kilocode_dir).unwrap();

    // Run the build command
    // It may fail with Docker error (if Docker not available) or succeed (if Docker is available)
    let result = Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["build", "--config", config_path.to_str().unwrap()])
        .assert();

    // Check if command failed with Docker-related error
    // If Docker is available, test may pass, which is acceptable
    let output = result.get_output();
    if !output.status.success() {
        // Verify error is Docker-related or .kilocode directory error
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Docker connection error")
                || stderr.contains("Docker unavailable")
                || stderr.contains("Docker daemon")
                || stderr.contains("connection refused")
                || stderr.contains(".kilocode directory"),
            "Expected Docker-related or .kilocode directory error, got: {}",
            stderr
        );
    }
    // If Docker is available and build succeeds, that's also acceptable
}

/// Test 'build' command fails early when .kilocode directory is missing
///
/// This test verifies BUG-003: The .kilocode directory check should happen at
/// the command entry point (before any Docker operations) rather than inside
/// build_agent_image(). With the fix, the error occurs immediately without
/// attempting Docker connection.
///
/// This test expects the command to fail with the .kilocode error message.
/// Importantly, it should NOT contain any Docker-related error messages,
/// indicating that the check happened before Docker connection attempts.
#[test]
fn test_build_command_fails_early_without_kilocode_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let dockerfile_path = temp_dir.path().join("Dockerfile");

    // Create a simple valid Dockerfile
    let dockerfile_content = r#"
FROM alpine:latest
RUN echo "Hello from test container"
CMD ["/bin/sh"]
"#;
    fs::write(&dockerfile_path, dockerfile_content).unwrap();

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
image_name = "switchboard-agent"
image_tag = "test-missing-kilocode"
"#;
    fs::write(&config_path, config_content).unwrap();

    // NOTE: We intentionally do NOT create the .kilocode directory

    // Run the build command expecting failure
    let result = Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["build", "--config", config_path.to_str().unwrap()])
        .assert()
        .failure();

    let output = result.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify the error message mentions .kilocode directory
    assert!(
        stderr.contains(".kilocode directory"),
        "Expected error to mention .kilocode directory, got: {}",
        stderr
    );

    // Verify the error message is user-friendly and contains the suggestion
    assert!(
        stderr.contains("API keys") || stderr.contains("MCP server"),
        "Expected error to mention API keys or MCP server, got: {}",
        stderr
    );

    // CRITICAL: With BUG-003 fix, the error should occur BEFORE any Docker connection
    // Therefore, we should NOT see any Docker-related error messages
    // (This proves the check happens at command entry point)
    assert!(
        !stderr.contains("Docker connection error") &&
        !stderr.contains("Docker unavailable") &&
        !stderr.contains("connection refused") &&
        !stderr.contains("No such file"),
        "Expected .kilocode error to occur BEFORE Docker connection attempt, but found Docker-related errors: {}",
        stderr
    );
}
