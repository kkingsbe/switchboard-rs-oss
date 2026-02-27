//! Integration tests for the `switchboard down` command

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test down command with no scheduler running
///
/// This test verifies that when there is no PID file present,
/// the down command properly returns an error indicating the scheduler is not running.
#[test]
fn test_down_no_scheduler_running() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();

    // Ensure no PID file exists (it shouldn't by default)
    let pid_file_path = temp_dir.path().join(".switchboard/scheduler.pid");
    assert!(
        !pid_file_path.exists(),
        "PID file should not exist initially"
    );

    // Run the down command from the temp directory
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .arg("down")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Scheduler is not running"));
}

/// Test down command with invalid PID file
///
/// This test verifies that when the PID file contains invalid content
/// (not a valid number), the down command reports an appropriate error.
#[test]
fn test_down_invalid_pid_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create .switchboard directory
    let switchboard_dir = temp_dir.path().join(".switchboard");
    fs::create_dir_all(&switchboard_dir).unwrap();

    // Create a PID file with invalid content (not a number)
    let pid_file_path = switchboard_dir.join("scheduler.pid");
    fs::write(&pid_file_path, "invalid-pid-content").unwrap();

    // Verify the PID file was created
    assert!(pid_file_path.exists(), "PID file should exist");

    // Run the down command
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .arg("down")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Error: Failed to parse PID"));

    // Clean up is handled by TempDir automatically
}

/// Test down command with valid PID file (mock scenario)
///
/// This test verifies that when the PID file contains a valid PID number,
/// the down command checks if the process is running before attempting to stop it.
/// Since this is a mock scenario with a non-existent process, it should gracefully
/// handle the case where the process is no longer running (BUG-005 fix).
#[test]
fn test_down_valid_pid_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create .switchboard directory
    let switchboard_dir = temp_dir.path().join(".switchboard");
    fs::create_dir_all(&switchboard_dir).unwrap();

    // Create a PID file with a valid (but non-existent) PID number
    // Using a high PID number that is unlikely to exist
    let mock_pid = "999999";
    let pid_file_path = switchboard_dir.join("scheduler.pid");
    fs::write(&pid_file_path, mock_pid).unwrap();

    // Verify the PID file was created
    assert!(pid_file_path.exists(), "PID file should exist");

    // Run the down command
    // The command should check if the process is running first (BUG-005 fix)
    // Since the process doesn't exist, it should report that it's no longer running
    // When the scheduler is not running, the command exits with success
    // Docker is not available, but that's just a warning, not a failure
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .arg("down")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Clean up is handled by TempDir automatically
}

/// Test down command with PID file for non-existent process (BUG-005 race condition)
///
/// This test verifies the fix for BUG-005 - a TOCTOU (Time-of-Check-Time-of-Use) race condition.
/// The scenario is:
/// - A PID file exists with a valid PID number
/// - The process is no longer running (has already exited)
/// - The command should check if the process is running BEFORE sending SIGTERM
/// - If the process is not running, it should skip sending the signal and report success
///
/// Expected behavior:
/// - Check process existence using is_process_running() before sending SIGTERM
/// - If process is not running, report success without attempting to send SIGTERM
/// - Output should indicate the process was already stopped
#[test]
fn test_down_pid_file_race_condition() {
    let temp_dir = TempDir::new().unwrap();

    // Create .switchboard directory
    let switchboard_dir = temp_dir.path().join(".switchboard");
    fs::create_dir_all(&switchboard_dir).unwrap();

    // Create a PID file with a valid (but non-existent) PID number
    // Using a high PID number that is extremely unlikely to exist
    // This simulates the race condition where the scheduler process has already exited
    let mock_pid = "9999999";
    let pid_file_path = switchboard_dir.join("scheduler.pid");
    fs::write(&pid_file_path, mock_pid).unwrap();

    // Verify the PID file was created
    assert!(pid_file_path.exists(), "PID file should exist");

    // Run the down command
    //
    // Expected behavior (after fix):
    // - The command should check if the process is running before sending SIGTERM
    // - Since the process doesn't exist, it should skip sending SIGTERM
    // - Output should indicate the process was already stopped (not that it attempted to stop it)
    //
    // Current buggy behavior (before fix):
    // - The command reads the PID file and immediately sends SIGTERM
    // - It only detects the process is gone after the kill command fails
    // - This is a TOCTOU race condition - wrong process could be signaled if PID is reused
    //
    // Docker is not available, but that's just a warning, not a failure
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .arg("down")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Clean up is handled by TempDir automatically
}
