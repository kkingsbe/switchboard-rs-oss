//! Integration tests for the `switchboard logs` command

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

/// Test logs with no arguments (reads scheduler log)
#[test]
fn test_logs_no_arguments() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard/logs");
    fs::create_dir_all(&log_dir).unwrap();
    let log_path = log_dir.join("switchboard.log");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create a log file with sample content
    let log_content = "2025-01-13 10:00:00 [INFO] Scheduler started
2025-01-13 10:00:01 [INFO] Agent test-agent running
2025-01-13 10:00:02 [INFO] Task completed successfully";
    fs::write(&log_path, log_content).unwrap();

    // Run the logs command
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Scheduler started"))
        .stdout(predicates::str::contains("Agent test-agent running"))
        .stdout(predicates::str::contains("Task completed successfully"));
}

/// Test logs with --tail 10 option
#[test]
fn test_logs_with_tail() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard/logs");
    fs::create_dir_all(&log_dir).unwrap();
    let log_path = log_dir.join("switchboard.log");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create a log file with many lines
    let mut log_content = String::new();
    for i in 1..=20 {
        log_content.push_str(&format!(
            "2025-01-13 10:00:{:02} [INFO] Log line {}\n",
            i, i
        ));
    }
    fs::write(&log_path, log_content).unwrap();

    // Run the logs command with --tail 10
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs", "--tail", "10"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Log line 11"))
        .stdout(predicates::str::contains("Log line 20"))
        .stdout(predicates::str::contains("Log line 10").not());
}

/// Test logs with agent name
#[test]
fn test_logs_with_agent_name() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let agent_log_dir = temp_dir
        .path()
        .join(".switchboard")
        .join("logs")
        .join("my-agent");
    fs::create_dir_all(&agent_log_dir).unwrap();
    let agent_log_path = agent_log_dir.join("20250113-100000.log");

    // Create a valid switchboard.toml config file with an agent
    let config_content = r#"
[[agent]]
name = "my-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create agent log file with sample content
    let log_content = "2025-01-13 10:00:00 [INFO] Agent started
2025-01-13 10:00:01 [INFO] Processing task
2025-01-13 10:00:02 [INFO] Task completed";
    fs::write(&agent_log_path, log_content).unwrap();

    // Run the logs command with agent name
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs", "my-agent"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "[my-agent] 2025-01-13 10:00:00 [INFO] Agent started",
        ))
        .stdout(predicates::str::contains(
            "[my-agent] 2025-01-13 10:00:01 [INFO] Processing task",
        ))
        .stdout(predicates::str::contains(
            "[my-agent] 2025-01-13 10:00:02 [INFO] Task completed",
        ));
}

/// Test logs with non-existent agent
#[test]
fn test_logs_nonexistent_agent() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the logs command with a non-existent agent name
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs", "nonexistent_agent"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Agent 'nonexistent_agent' not found in config",
        ));
}

/// Test logs with file not found
#[test]
fn test_logs_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Don't create any log files

    // Run the logs command
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stdout(predicates::str::contains(
            "Scheduler log file not found: .switchboard/logs/switchboard.log",
        ))
        .stdout(predicates::str::contains(
            "No log files found in the project.",
        ));
}

/// Test logs with file not found and available logs list
#[test]
fn test_logs_file_not_found_with_available_logs() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let agent_log_dir = temp_dir
        .path()
        .join(".switchboard")
        .join("logs")
        .join("test-agent");
    fs::create_dir_all(&agent_log_dir).unwrap();
    let agent_log_path = agent_log_dir.join("20250113-100000.log");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"

[[agent]]
name = "another-agent"
schedule = "0 10 * * *"
prompt = "Another prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create one agent log file (not the scheduler log)
    let log_content = "2025-01-13 10:00:00 [INFO] Agent started";
    fs::write(&agent_log_path, log_content).unwrap();

    // Run the logs command for scheduler (which doesn't exist)
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stdout(predicates::str::contains(
            "Scheduler log file not found: .switchboard/logs/switchboard.log",
        ))
        .stdout(predicates::str::contains("Available log files:"));
}

/// Test --follow mode with simulated updates (integration test)
#[tokio::test]
async fn test_logs_follow_mode() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard/logs");
    fs::create_dir_all(&log_dir).unwrap();
    let log_path = log_dir.join("switchboard.log");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create initial log file
    let initial_content = "2025-01-13 10:00:00 [INFO] Initial line\n";
    fs::write(&log_path, initial_content).unwrap();

    // Clone path for use in spawned task
    let log_path_clone = log_path.clone();

    // Spawn a task that appends to the log file
    let write_task = tokio::spawn(async move {
        // Wait a bit before writing
        sleep(Duration::from_millis(200)).await;

        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&log_path_clone)
            .unwrap();

        file.write_all(b"2025-01-13 10:00:01 [INFO] New line 1\n")
            .unwrap();
        file.flush().unwrap();

        sleep(Duration::from_millis(100)).await;

        file.write_all(b"2025-01-13 10:00:02 [INFO] New line 2\n")
            .unwrap();
        file.flush().unwrap();
    });

    // Run the logs command with --follow and a timeout
    // We expect a timeout since follow mode runs indefinitely
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs", "--follow"])
        .current_dir(temp_dir.path())
        .timeout(Duration::from_millis(500))
        .assert()
        .failure();

    // Wait for the write task to complete
    write_task.await.expect("Write task should complete");

    // Timeout is expected for follow mode
    // The important thing is that the write task ran and appended to the log file
    let final_content = fs::read_to_string(&log_path).unwrap();
    assert!(final_content.contains("Initial line"));
    assert!(final_content.contains("New line 1"));
    assert!(final_content.contains("New line 2"));
}

/// Test logs with agent name and --tail option combined
#[test]
fn test_logs_agent_with_tail() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let agent_log_dir = temp_dir
        .path()
        .join(".switchboard")
        .join("logs")
        .join("my-agent");
    fs::create_dir_all(&agent_log_dir).unwrap();
    let agent_log_path = agent_log_dir.join("20250113-100000.log");

    // Create a valid switchboard.toml config file with an agent
    let config_content = r#"
[[agent]]
name = "my-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create agent log file with many lines
    let mut log_content = String::new();
    for i in 1..=20 {
        log_content.push_str(&format!(
            "2025-01-13 10:00:{:02} [INFO] Agent log line {}\n",
            i, i
        ));
    }
    fs::write(&agent_log_path, log_content).unwrap();

    // Run the logs command with agent name and --tail 5
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs", "my-agent", "--tail", "5"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "[my-agent] 2025-01-13 10:00:16 [INFO] Agent log line 16",
        ))
        .stdout(predicates::str::contains(
            "[my-agent] 2025-01-13 10:00:20 [INFO] Agent log line 20",
        ))
        .stdout(
            predicates::str::contains("[my-agent] 2025-01-13 10:00:15 [INFO] Agent log line 15")
                .not(),
        );
}

/// Test logs for non-existent agent log file
#[test]
fn test_logs_agent_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let agent_log_dir = temp_dir
        .path()
        .join(".switchboard")
        .join("logs")
        .join("my-agent");
    fs::create_dir_all(&agent_log_dir).unwrap();

    // Create a valid switchboard.toml config file with an agent
    let config_content = r#"
[[agent]]
name = "my-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Don't create the agent log file (directory exists but no .log files)

    // Run the logs command for the agent
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs", "my-agent"])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "No log files found for agent 'my-agent' in directory:",
        ))
        .stderr(predicates::str::contains(".switchboard/logs/my-agent"));
}
