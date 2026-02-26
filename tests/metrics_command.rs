//! Integration tests for `switchboard metrics` command

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test 'metrics' command with no metrics file
#[test]
fn test_metrics_command_no_metrics_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file with log_dir pointing to temp dir
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
log_dir = ".switchboard/logs"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the metrics command expecting success with "No metrics data available yet" message on stderr
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["metrics", "--config", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stderr(predicates::str::contains("No metrics data available yet"));
}

/// Test 'metrics' command with valid metrics data (table view)
#[test]
fn test_metrics_command_table_view() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let metrics_path = log_dir.join("metrics.json");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
log_dir = ".switchboard/logs"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create metrics.json with test data
    let metrics_content = r#"{
  "agents": {
    "agent_1": {
      "total_runs": 10,
      "successful_runs": 8,
      "failed_runs": 2,
      "total_duration_ms": 60000,
      "runs": [
        {
          "run_id": "container_1",
          "timestamp": 1234567890,
          "duration_ms": 6000,
          "status": "success",
          "error_message": null
        }
      ],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    },
    "agent_2": {
      "total_runs": 5,
      "successful_runs": 5,
      "failed_runs": 0,
      "total_duration_ms": 15000,
      "runs": [
        {
          "run_id": "container_2",
          "timestamp": 1234567891,
          "duration_ms": 3000,
          "status": "success",
          "error_message": null
        }
      ],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    }
  }
}"#;
    fs::write(&metrics_path, metrics_content).unwrap();

    // Run the metrics command expecting success
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["metrics", "--config", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("agent_1"))
        .stdout(predicates::str::contains("agent_2"))
        .stdout(predicates::str::contains("Runs"))
        .stdout(predicates::str::contains("Success"))
        .stdout(predicates::str::contains("Fail"))
        .stdout(predicates::str::contains("Avg Duration"))
        .stdout(predicates::str::contains("Last Run"))
        .stdout(predicates::str::contains("Status"));
}

/// Test 'metrics' command with --detailed flag
#[test]
fn test_metrics_command_detailed_view() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let metrics_path = log_dir.join("metrics.json");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
log_dir = ".switchboard/logs"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create metrics.json with test data
    let metrics_content = r#"{
  "agents": {
    "agent_1": {
      "total_runs": 5,
      "successful_runs": 4,
      "failed_runs": 1,
      "total_duration_ms": 50000,
      "runs": [
        {
          "run_id": "container_1",
          "timestamp": 1234567890,
          "duration_ms": 10000,
          "status": "success",
          "error_message": null
        },
        {
          "run_id": "container_2",
          "timestamp": 1234568000,
          "duration_ms": 10000,
          "status": "failure",
          "error_message": "exit_code: 1"
        }
      ],
      "queue_wait_time_seconds": 25,
      "queue_wait_times": [10, 15],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    }
  }
}"#;
    fs::write(&metrics_path, metrics_content).unwrap();

    // Run the metrics command with --detailed flag
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "metrics",
            "--detailed",
            "--config",
            config_path.to_str().unwrap(),
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Agent: agent_1"))
        .stdout(predicates::str::contains("Total Runs:"))
        .stdout(predicates::str::contains("Successful Runs:"))
        .stdout(predicates::str::contains("Failed Runs:"))
        .stdout(predicates::str::contains("Success Rate:"))
        .stdout(predicates::str::contains("Average Duration:"))
        .stdout(predicates::str::contains("First Run:"))
        .stdout(predicates::str::contains("Last Run:"))
        .stdout(predicates::str::contains("Timeout Count:"))
        .stdout(predicates::str::contains("Queue Wait Time:"));
}

/// Test 'metrics' command status icon thresholds
#[test]
fn test_metrics_command_status_icons() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let metrics_path = log_dir.join("metrics.json");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
log_dir = ".switchboard/logs"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create metrics.json with different success rates
    let metrics_content = r#"{
  "agents": {
    "agent_high_success": {
      "total_runs": 10,
      "successful_runs": 10,
      "failed_runs": 0,
      "total_duration_ms": 60000,
      "runs": [],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    },
    "agent_medium_success": {
      "total_runs": 10,
      "successful_runs": 7,
      "failed_runs": 3,
      "total_duration_ms": 60000,
      "runs": [],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    },
    "agent_low_success": {
      "total_runs": 10,
      "successful_runs": 3,
      "failed_runs": 7,
      "total_duration_ms": 60000,
      "runs": [],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    },
    "agent_insufficient_runs": {
      "total_runs": 2,
      "successful_runs": 2,
      "failed_runs": 0,
      "total_duration_ms": 12000,
      "runs": [],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    }
  }
}"#;
    fs::write(&metrics_path, metrics_content).unwrap();

    // Run the metrics command expecting status icons
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["metrics", "--config", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        // ✓ for ≥95% success rate
        .stdout(predicates::str::contains("✓"))
        // ⚠ for 50-95% success rate
        .stdout(predicates::str::contains("⚠"))
        // ✗ for <50% success rate
        .stdout(predicates::str::contains("✗"))
        // - for <3 runs
        .stdout(predicates::str::contains("-"));
}

/// Test 'metrics' command duration formatting
#[test]
fn test_metrics_command_duration_formatting() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let metrics_path = log_dir.join("metrics.json");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
log_dir = ".switchboard/logs"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create metrics.json with various durations
    // agent_seconds: 135000 ms total, 3 runs = 45000 ms avg = 45s (should show "45s")
    // agent_minutes: 450000 ms total, 3 runs = 150000 ms avg = 150s = 2m 30s (should show "2m 30s")
    // agent_hours: 11169000 ms total, 3 runs = 3723000 ms avg = 3723s = 1h 2m 3s (should show "1h 2m 3s")
    let metrics_content = r#"{
  "agents": {
    "agent_seconds": {
      "total_runs": 3,
      "successful_runs": 3,
      "failed_runs": 0,
      "total_duration_ms": 135000,
      "runs": [],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    },
    "agent_minutes": {
      "total_runs": 3,
      "successful_runs": 3,
      "failed_runs": 0,
      "total_duration_ms": 450000,
      "runs": [],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    },
    "agent_hours": {
      "total_runs": 3,
      "successful_runs": 3,
      "failed_runs": 0,
      "total_duration_ms": 11169000,
      "runs": [],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    }
  }
}"#;
    fs::write(&metrics_path, metrics_content).unwrap();

    // Run the metrics command expecting formatted durations
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["metrics", "--config", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        // Test second-only format: "45s"
        .stdout(predicates::str::contains("45s"))
        // Test minute-second format: "2m 30s"
        .stdout(predicates::str::contains("2m 30s"))
        // Test hour-minute-second format: "1h 2m 3s"
        .stdout(predicates::str::contains("1h 2m 3s"));
}

/// Test 'metrics' command with corrupted metrics file
#[test]
fn test_metrics_command_corrupted_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let metrics_path = log_dir.join("metrics.json");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
log_dir = ".switchboard/logs"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create corrupted metrics.json file
    fs::write(&metrics_path, "{ invalid json content }").unwrap();

    // Run the metrics command - should fail with error
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["metrics", "--config", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        // Should show error message about corrupted file and backup location
        .stderr(predicates::str::contains("Metrics file is corrupted"))
        .stderr(predicates::str::contains("Backup saved to:"));
}

/// Test 'metrics' command with --detailed flag and -c short flag
#[test]
fn test_metrics_command_short_config_flag() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let metrics_path = log_dir.join("metrics.json");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
log_dir = ".switchboard/logs"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create metrics.json with test data
    let metrics_content = r#"{
  "agents": {
    "agent_1": {
      "total_runs": 5,
      "successful_runs": 5,
      "failed_runs": 0,
      "total_duration_ms": 50000,
      "runs": [],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    }
  }
}"#;
    fs::write(&metrics_path, metrics_content).unwrap();

    // Run the metrics command with -c short flag
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["metrics", "-c", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("agent_1"));
}

/// Test 'metrics' command with zero runs
#[test]
fn test_metrics_command_zero_runs() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let metrics_path = log_dir.join("metrics.json");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
log_dir = ".switchboard/logs"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create metrics.json with zero runs
    let metrics_content = r#"{
  "agents": {
    "agent_no_runs": {
      "total_runs": 0,
      "successful_runs": 0,
      "failed_runs": 0,
      "total_duration_ms": 0,
      "runs": [],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    }
  }
}"#;
    fs::write(&metrics_path, metrics_content).unwrap();

    // Run the metrics command
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["metrics", "--config", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("agent_no_runs"))
        .stdout(predicates::str::contains("0")) // Should show 0 runs
        .stdout(predicates::str::contains("-")); // Duration and rate should show "-"
}

// ==================== Error Handling Tests ====================

/// Test 'metrics' command with missing metrics file (FileNotFound error)
#[test]
fn test_metrics_command_missing_file_error() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file with log_dir pointing to temp dir
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
log_dir = ".switchboard/logs"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Run the metrics command - should exit with code 0 (graceful handling)
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["metrics", "--config", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .assert()
        .success() // Exits with code 0 for missing file
        .stderr(predicates::str::contains(
            "No metrics data available yet. Run agents to collect metrics.",
        ));
}

/// Test 'metrics' command with corrupted metrics file (CorruptedFile error)
#[test]
fn test_metrics_command_corrupted_file_error() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let metrics_path = log_dir.join("metrics.json");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
log_dir = ".switchboard/logs"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create corrupted metrics.json file
    fs::write(&metrics_path, "{ invalid json content }").unwrap();

    // Run the metrics command - should fail with error code 1
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["metrics", "--config", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .assert()
        .failure() // Exits with code 1 for corrupted file
        .stderr(predicates::str::contains("Metrics file is corrupted"))
        .stderr(predicates::str::contains("Backup saved to:"));
}

/// Test 'metrics' command with agent not found when agents are available
#[test]
fn test_metrics_command_agent_not_found_with_agents() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let metrics_path = log_dir.join("metrics.json");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
log_dir = ".switchboard/logs"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create metrics.json with some agents
    let metrics_content = r#"{
  "agents": {
    "agent_1": {
      "total_runs": 10,
      "successful_runs": 8,
      "failed_runs": 2,
      "total_duration_ms": 60000,
      "runs": [],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    },
    "agent_2": {
      "total_runs": 5,
      "successful_runs": 5,
      "failed_runs": 0,
      "total_duration_ms": 15000,
      "runs": [],
      "queue_wait_time_seconds": null,
      "queue_wait_times": [],
      "sigterm_count": 0,
      "sigkill_count": 0,
      "timeout_count": 0
    }
  }
}"#;
    fs::write(&metrics_path, metrics_content).unwrap();

    // Run the metrics command with a non-existent agent name
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "metrics",
            "--agent",
            "nonexistent",
            "--config",
            config_path.to_str().unwrap(),
        ])
        .current_dir(temp_dir.path())
        .assert()
        .failure() // Exits with code 1
        .stderr(predicates::str::contains(
            "Agent 'nonexistent' not found in metrics",
        ))
        .stderr(predicates::str::contains("Available agents:"));
}

/// Test 'metrics' command with agent not found when no agents have run
#[test]
fn test_metrics_command_agent_not_found_no_agents() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let metrics_path = log_dir.join("metrics.json");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"

[settings]
log_dir = ".switchboard/logs"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create empty metrics.json (no agents)
    let metrics_content = r#"{
  "agents": {}
}"#;
    fs::write(&metrics_path, metrics_content).unwrap();

    // Run the metrics command with a non-existent agent name
    // Note: When agents HashMap is empty, the implementation returns a friendly message
    // instead of checking if the specific agent exists, so this exits with code 0
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "metrics",
            "--agent",
            "nonexistent",
            "--config",
            config_path.to_str().unwrap(),
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success() // Exits with code 0 when no agents exist
        .stdout(predicates::str::contains("No metrics data available yet"));
}
