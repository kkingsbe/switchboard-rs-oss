//! Integration tests to verify scheduler logs are written to correct location
//!
//! This module tests complete logging flow from initialization to
//! writing logs and verifying logs command can read them.

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use tempfile::TempDir;
use tracing_appender::non_blocking::WorkerGuard;

// Use a static to ensure tracing subscriber is only initialized once
// This is necessary because tracing::subscriber::set_global_default can only be called once
static INIT: Once = Once::new();
static mut GLOBAL_GUARD: Option<WorkerGuard> = None;
static mut GLOBAL_LOG_DIR: Option<PathBuf> = None;
static mut TEMP_DIR: Option<TempDir> = None;

/// Helper function to initialize logging for all tests
/// Uses a static flag to ensure tracing is only initialized once across all tests
/// All tests share same log directory and subscriber
#[allow(static_mut_refs)]
fn get_test_log_dir() -> &'static Path {
    unsafe {
        INIT.call_once(|| {
            // Create a temp dir that lives for duration of test run
            let temp = TempDir::new().unwrap();
            let log_dir = temp.path().join("logs");

            // Initialize logging using switchboard's init_logging
            let guard = switchboard::logging::init_logging(log_dir.clone()).expect("Failed to initialize logging");

            TEMP_DIR = Some(temp);
            GLOBAL_LOG_DIR = Some(log_dir);
            GLOBAL_GUARD = Some(guard);
        });

        GLOBAL_LOG_DIR.as_ref().unwrap().as_path()
    }
}

/// Test that log directory is created when init_logging() is called
#[test]
fn test_init_logging_creates_log_directory() {
    let log_dir = get_test_log_dir();

    // Verify log directory exists
    assert!(
        log_dir.exists(),
        "Log directory should exist at {}",
        log_dir.display()
    );
    assert!(
        log_dir.is_dir(),
        "Log path should be a directory at {}",
        log_dir.display()
    );
}

/// Test that scheduler logs are written to <log_dir>/switchboard.log
#[test]
fn test_scheduler_logs_written_to_correct_location() {
    let log_dir = get_test_log_dir();
    let log_file_path = log_dir.join("switchboard.log");

    // Write some test log messages
    tracing::info!("TEST1: Scheduler started successfully");
    tracing::info!("TEST1: Processing agent task");
    tracing::warn!("TEST1: This is a warning message");
    tracing::error!("TEST1: This is an error message");

    // Give non-blocking writer time to flush
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Verify log file exists
    assert!(
        log_file_path.exists(),
        "Log file should exist at {}",
        log_file_path.display()
    );
    assert!(
        log_file_path.is_file(),
        "Log path should be a file at {}",
        log_file_path.display()
    );

    // Verify log file contains expected messages
    let contents = fs::read_to_string(&log_file_path).unwrap();
    assert!(
        contents.contains("TEST1: Scheduler started successfully"),
        "Log file should contain 'TEST1: Scheduler started successfully'"
    );
    assert!(
        contents.contains("TEST1: Processing agent task"),
        "Log file should contain 'TEST1: Processing agent task'"
    );
    assert!(
        contents.contains("TEST1: This is a warning message"),
        "Log file should contain 'TEST1: This is a warning message'"
    );
    assert!(
        contents.contains("TEST1: This is an error message"),
        "Log file should contain 'TEST1: This is an error message'"
    );
}

/// Test that logs command can read scheduler logs from correct location
#[test]
fn test_logs_command_reads_scheduler_logs_correctly() {
    // Get shared test log directory (initializes logging if not already)
    let log_dir = get_test_log_dir();

    // Write some test log messages
    tracing::info!("TEST2: Scheduler initialization complete");
    tracing::info!("TEST2: Agent scheduled to run at 09:00");
    tracing::warn!("TEST2: Agent execution delayed");

    // Give non-blocking writer time to flush
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create a unique test directory for this test
    let test_temp_dir = TempDir::new().unwrap();
    let actual_log_dir = test_temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&actual_log_dir).unwrap();

    // Copy log file to expected location
    fs::copy(
        log_dir.join("switchboard.log"),
        actual_log_dir.join("switchboard.log"),
    )
    .unwrap();

    // Create a valid switchboard.toml config file
    let config_path = test_temp_dir.path().join("switchboard.toml");
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the logs command to verify it can read scheduler logs
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs"])
        .current_dir(test_temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "TEST2: Scheduler initialization complete",
        ))
        .stdout(predicates::str::contains(
            "TEST2: Agent scheduled to run at 09:00",
        ))
        .stdout(predicates::str::contains("TEST2: Agent execution delayed"));
}

/// Comprehensive end-to-end test of scheduler logging flow
#[test]
fn test_complete_scheduler_logging_flow() {
    // Get shared test log directory (initializes logging if not already)
    let log_dir = get_test_log_dir();
    let log_file_path = log_dir.join("switchboard.log");

    // Step 1: Verify log directory is created (already created by first test)
    assert!(
        log_dir.exists(),
        "Log directory should be created after initialization"
    );

    // Step 2: Write multiple log messages
    tracing::info!("TEST3: === Scheduler Log Test ===");
    tracing::info!("TEST3: Starting scheduler at 2025-01-15 10:00:00");
    for i in 1..=5 {
        tracing::info!("TEST3: Processing task number {}", i);
    }
    tracing::info!("TEST3: All tasks completed successfully");
    tracing::warn!("TEST3: Low memory warning: 85% usage");
    tracing::error!("TEST3: Failed to connect to agent API (will retry)");

    // Step 3: Give non-blocking writer time to flush
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Step 4: Verify log file exists and contains all expected messages
    assert!(
        log_file_path.exists(),
        "Log file should exist at {}",
        log_file_path.display()
    );

    let contents = fs::read_to_string(&log_file_path).unwrap();

    // Verify all log messages are present
    assert!(contents.contains("TEST3: === Scheduler Log Test ==="));
    assert!(contents.contains("TEST3: Starting scheduler at 2025-01-15 10:00:00"));
    assert!(contents.contains("TEST3: Processing task number 1"));
    assert!(contents.contains("TEST3: Processing task number 5"));
    assert!(contents.contains("TEST3: All tasks completed successfully"));
    assert!(contents.contains("TEST3: Low memory warning: 85% usage"));
    assert!(contents.contains("TEST3: Failed to connect to agent API (will retry)"));

    // Step 5: Create unique test directory and config file
    let test_temp_dir = TempDir::new().unwrap();
    let actual_log_dir = test_temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&actual_log_dir).unwrap();
    fs::copy(log_file_path, actual_log_dir.join("switchboard.log")).unwrap();

    let config_path = test_temp_dir.path().join("switchboard.toml");
    let config_content = r#"
[[agent]]
name = "scheduler-agent"
schedule = "0 0 9 * * *"
prompt = "Scheduler agent prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Step 6: Verify logs command can read logs
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs"])
        .current_dir(test_temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "TEST3: === Scheduler Log Test ===",
        ))
        .stdout(predicates::str::contains(
            "TEST3: Starting scheduler at 2025-01-15 10:00:00",
        ))
        .stdout(predicates::str::contains(
            "TEST3: All tasks completed successfully",
        ));
}

/// Test that log messages persist across multiple writes
#[test]
fn test_logs_persist_across_multiple_writes() {
    let log_dir = get_test_log_dir();
    let log_file_path = log_dir.join("switchboard.log");

    // Write initial log messages
    tracing::info!("TEST4: Initial message 1");
    tracing::info!("TEST4: Initial message 2");
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Write additional log messages
    tracing::info!("TEST4: Additional message 1");
    tracing::info!("TEST4: Additional message 2");
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Verify all messages are present
    let contents = fs::read_to_string(&log_file_path).unwrap();
    assert!(contents.contains("TEST4: Initial message 1"));
    assert!(contents.contains("TEST4: Initial message 2"));
    assert!(contents.contains("TEST4: Additional message 1"));
    assert!(contents.contains("TEST4: Additional message 2"));
}

/// Test that custom log directory structure is handled correctly
#[test]
fn test_custom_log_directory_structure() {
    // Get shared test log directory (initializes logging if not already)
    let log_dir = get_test_log_dir();
    let log_file_path = log_dir.join("switchboard.log");

    // Write test log
    tracing::info!("TEST5: Custom log directory structure test");
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Verify directory structure is created
    assert!(log_dir.exists());
    assert!(log_dir.is_dir());
    assert!(log_file_path.exists());

    // Verify log content
    let contents = fs::read_to_string(&log_file_path).unwrap();
    assert!(contents.contains("TEST5: Custom log directory structure test"));
}

/// Test that logs command with --tail option correctly shows the last N lines
#[test]
fn test_logs_command_tail_scheduler_logs() {
    // Create a unique test directory for this test with isolated logging
    let test_temp_dir = TempDir::new().unwrap();
    let actual_log_dir = test_temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&actual_log_dir).unwrap();

    let log_file_path = actual_log_dir.join("switchboard.log");

    // Write 15 test log lines directly to the log file
    // We write directly instead of using tracing to avoid interference from the global shared log
    let mut log_content = String::new();
    for i in 1..=15 {
        // Simulate the log format that tracing produces
        log_content.push_str(&format!(
            "2025-01-15T10:00:00.000000Z INFO scheduler_logging_integration: TEST5: TAIL_TEST Log line {}\n",
            i
        ));
    }
    fs::write(&log_file_path, log_content).unwrap();

    // Verify all 15 lines are in log file first
    let contents = fs::read_to_string(&log_file_path).unwrap();
    for i in 1..=15 {
        assert!(
            contents.contains(&format!("TEST5: TAIL_TEST Log line {}", i)),
            "Log file should contain line {}",
            i
        );
    }

    // Create a valid switchboard.toml config file
    let config_path = test_temp_dir.path().join("switchboard.toml");
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run logs command with --tail 5 to get last 5 lines (lines 11-15)
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs", "--tail", "5"])
        .current_dir(test_temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("TEST5: TAIL_TEST Log line 11"))
        .stdout(predicates::str::contains("TEST5: TAIL_TEST Log line 12"))
        .stdout(predicates::str::contains("TEST5: TAIL_TEST Log line 13"))
        .stdout(predicates::str::contains("TEST5: TAIL_TEST Log line 14"))
        .stdout(predicates::str::contains("TEST5: TAIL_TEST Log line 15"));
}
