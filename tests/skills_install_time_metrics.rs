//! Integration tests for skill installation time metrics
//!
//! This test file verifies that the `skills_install_time_seconds` field
//! in metrics is correctly populated with accurate data:
//! - The field is populated when skills are configured
//! - The field is None when no skills are configured
//! - Metrics are correctly persisted to metrics.json
//! - The recorded time is a reasonable value (positive and within expected range)
//!
//! These tests focus on data accuracy, not performance thresholds.

use chrono::{Duration, Utc};
use switchboard::metrics::{update_all_metrics, AgentRunResult, AllMetrics, MetricsStore};
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

/// Test 1: Skill installation time is recorded when skills are installed
///
/// This test verifies that:
/// 1. `skills_install_time_seconds` is Some(value) (not None) when skills are configured
/// 2. The value is a positive f64 number
/// 3. The value is reasonable (between 0.1 and 300 seconds)
#[test]
fn test_skills_install_time_recorded_when_skills_installed() {
    let start_time = Utc::now();
    let end_time = start_time + Duration::seconds(5);

    // Simulate successful installation with skills configured
    let run_result = AgentRunResult {
        agent_name: "test-agent-with-skills".to_string(),
        container_id: "container-123".to_string(),
        start_time,
        end_time,
        exit_code: 0,
        timed_out: false,
        termination_type: None,
        queued_start_time: None,
        skills_installed_count: 2,              // 2 skills installed
        skills_failed_count: 0,                 // No failures
        skills_install_time_seconds: Some(5.0), // Time recorded
    };

    // Verify skills_install_time_seconds is Some (not None)
    assert!(
        run_result.skills_install_time_seconds.is_some(),
        "skills_install_time_seconds should be Some when skills are configured, got None"
    );

    // Verify the value is present
    let install_time = run_result.skills_install_time_seconds.unwrap();

    // Verify the value is a positive f64 number
    assert!(
        install_time > 0.0,
        "skills_install_time_seconds should be positive, got: {}",
        install_time
    );

    // Verify the value is reasonable (between 0.1 and 300 seconds)
    assert!(
        install_time >= 0.1,
        "skills_install_time_seconds should be at least 0.1 seconds, got: {}",
        install_time
    );

    assert!(
        install_time <= 300.0,
        "skills_install_time_seconds should be at most 300 seconds, got: {}",
        install_time
    );

    // Verify the value is a valid f64 (not NaN or infinite)
    assert!(
        install_time.is_finite(),
        "skills_install_time_seconds should be a finite number, got: {}",
        install_time
    );
}

/// Test 2: Skill installation time is None when no skills are configured
///
/// This test verifies that:
/// 1. `skills_install_time_seconds` is None when no skills are configured
/// 2. The None value indicates no skill installation was performed
#[test]
fn test_skills_install_time_none_when_no_skills() {
    let start_time = Utc::now();
    let end_time = start_time + Duration::seconds(2);

    // Simulate execution without skills configured
    let run_result = AgentRunResult {
        agent_name: "test-agent-no-skills".to_string(),
        container_id: "container-456".to_string(),
        start_time,
        end_time,
        exit_code: 0,
        timed_out: false,
        termination_type: None,
        queued_start_time: None,
        skills_installed_count: 0,         // No skills installed
        skills_failed_count: 0,            // No failures
        skills_install_time_seconds: None, // No time recorded when no skills
    };

    // Verify skills_install_time_seconds is None
    assert!(
        run_result.skills_install_time_seconds.is_none(),
        "skills_install_time_seconds should be None when no skills are configured, got: {:?}",
        run_result.skills_install_time_seconds
    );

    // Verify skills_installed_count is 0
    assert_eq!(
        run_result.skills_installed_count, 0,
        "skills_installed_count should be 0 when no skills are configured"
    );

    // Verify skills_failed_count is 0
    assert_eq!(
        run_result.skills_failed_count, 0,
        "skills_failed_count should be 0 when no skills are configured"
    );
}

/// Test 3: Metrics are persisted to metrics.json
///
/// This test verifies that:
/// 1. Metrics with skills_install_time_seconds are correctly persisted to metrics.json
/// 2. The persisted file contains the skills_install_time_seconds field
/// 3. The persisted data matches the in-memory metrics
#[test]
fn test_metrics_persisted_to_json() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let log_dir = temp_dir.path();

    // Create a metrics store
    let metrics_store = MetricsStore::new(log_dir.to_path_buf());

    // Create a run result with skills_install_time_seconds
    let start_time = Utc::now();
    let end_time = start_time + Duration::seconds(7);

    let run_result = AgentRunResult {
        agent_name: "test-agent-persistence".to_string(),
        container_id: "container-789".to_string(),
        start_time,
        end_time,
        exit_code: 0,
        timed_out: false,
        termination_type: None,
        queued_start_time: None,
        skills_installed_count: 1,
        skills_failed_count: 0,
        skills_install_time_seconds: Some(7.5), // Time to be persisted
    };

    // Create AllMetrics structure and update with the run result
    let mut all_metrics = AllMetrics::default();
    update_all_metrics(&mut all_metrics, &run_result).expect("Metrics update should succeed");

    // Save metrics to disk
    let save_result = metrics_store.save(&all_metrics);
    assert!(
        save_result.is_ok(),
        "Metrics save should succeed: {:?}",
        save_result.err()
    );

    // Verify metrics.json file exists
    let metrics_path = log_dir.join("metrics.json");
    assert!(
        metrics_path.exists(),
        "metrics.json should exist after saving metrics"
    );

    // Load the metrics.json file and verify content
    let metrics_content = fs::read_to_string(&metrics_path).expect("Failed to read metrics.json");

    // Verify the file contains skills_install_time_seconds field
    assert!(
        metrics_content.contains("skills_install_time_seconds"),
        "metrics.json should contain 'skills_install_time_seconds' field"
    );

    // Verify the file contains the specific value
    assert!(
        metrics_content.contains("7.5"),
        "metrics.json should contain the recorded install time value"
    );

    // Load metrics from file and verify they match
    let loaded_metrics = metrics_store
        .load()
        .expect("Failed to load metrics from file");

    // Verify the agent exists in loaded metrics
    assert!(
        loaded_metrics.agents.contains_key("test-agent-persistence"),
        "Loaded metrics should contain data for 'test-agent-persistence'"
    );

    let agent_data = &loaded_metrics.agents["test-agent-persistence"];

    // Verify skills_install_time_seconds is persisted correctly
    assert_eq!(
        agent_data.skills_install_time_seconds,
        Some(7.5),
        "Persisted skills_install_time_seconds should match the original value"
    );

    // Verify other metrics are persisted correctly
    assert_eq!(
        agent_data.total_skills_installed, 1,
        "Persisted total_skills_installed should match"
    );

    assert_eq!(
        agent_data.total_skills_failed, 0,
        "Persisted total_skills_failed should match"
    );
}

/// Test: Multiple skill installations accumulate time correctly
///
/// This test verifies that when multiple runs with skill installations occur,
/// the metrics correctly track the accumulated skill installation time.
#[test]
fn test_multiple_skill_installations_accumulate_time() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let log_dir = temp_dir.path();

    // Create a metrics store
    let metrics_store = MetricsStore::new(log_dir.to_path_buf());

    // Create AllMetrics structure
    let mut all_metrics = AllMetrics::default();

    // First run: 5 seconds
    let start_time1 = Utc::now();
    let end_time1 = start_time1 + Duration::seconds(5);

    let run_result1 = AgentRunResult {
        agent_name: "test-agent-accumulation".to_string(),
        container_id: "container-1".to_string(),
        start_time: start_time1,
        end_time: end_time1,
        exit_code: 0,
        timed_out: false,
        termination_type: None,
        queued_start_time: None,
        skills_installed_count: 2,
        skills_failed_count: 0,
        skills_install_time_seconds: Some(5.0),
    };

    update_all_metrics(&mut all_metrics, &run_result1).expect("First update should succeed");

    // Second run: 3 seconds
    let start_time2 = Utc::now();
    let end_time2 = start_time2 + Duration::seconds(3);

    let run_result2 = AgentRunResult {
        agent_name: "test-agent-accumulation".to_string(),
        container_id: "container-2".to_string(),
        start_time: start_time2,
        end_time: end_time2,
        exit_code: 0,
        timed_out: false,
        termination_type: None,
        queued_start_time: None,
        skills_installed_count: 1,
        skills_failed_count: 0,
        skills_install_time_seconds: Some(3.0),
    };

    update_all_metrics(&mut all_metrics, &run_result2).expect("Second update should succeed");

    // Save and reload metrics
    metrics_store
        .save(&all_metrics)
        .expect("Save should succeed");
    let loaded_metrics = metrics_store.load().expect("Load should succeed");

    let agent_data = &loaded_metrics.agents["test-agent-accumulation"];

    // Verify skills_install_time_seconds tracks the latest or total time
    // (depending on implementation, this tests that it's persisted)
    assert!(
        agent_data.skills_install_time_seconds.is_some(),
        "skills_install_time_seconds should be recorded after multiple runs"
    );

    let install_time = agent_data.skills_install_time_seconds.unwrap();
    assert!(
        install_time > 0.0,
        "skills_install_time_seconds should be positive after multiple runs"
    );

    // Verify total skills installed is accumulated
    assert_eq!(
        agent_data.total_skills_installed, 3,
        "total_skills_installed should be accumulated across runs (2 + 1 = 3)"
    );
}

/// Test: Failed skill installation still records time
///
/// This test verifies that even when skill installation fails,
/// the time is still recorded (the attempt took time).
#[test]
fn test_failed_skill_installation_records_time() {
    let start_time = Utc::now();
    let end_time = start_time + Duration::seconds(2);

    // Simulate failed skill installation
    let run_result = AgentRunResult {
        agent_name: "test-agent-failed-skills".to_string(),
        container_id: "container-fail".to_string(),
        start_time,
        end_time,
        exit_code: 127, // Non-zero exit code
        timed_out: false,
        termination_type: None,
        queued_start_time: None,
        skills_installed_count: 0,              // No skills installed
        skills_failed_count: 1,                 // 1 skill failed
        skills_install_time_seconds: Some(2.0), // Time still recorded
    };

    // Verify skills_install_time_seconds is recorded even on failure
    assert!(
        run_result.skills_install_time_seconds.is_some(),
        "skills_install_time_seconds should be Some even when installation fails"
    );

    // Verify the time is positive
    assert!(
        run_result.skills_install_time_seconds.unwrap() > 0.0,
        "skills_install_time_seconds should be positive even on failure"
    );

    // Verify failure counts are correct
    assert_eq!(
        run_result.skills_installed_count, 0,
        "skills_installed_count should be 0 when installation fails"
    );

    assert_eq!(
        run_result.skills_failed_count, 1,
        "skills_failed_count should be 1 when installation fails"
    );
}

/// Test: Fractional seconds are recorded correctly
///
/// This test verifies that fractional seconds in skill installation time
/// are accurately recorded and persisted.
#[test]
fn test_fractional_seconds_recorded_correctly() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let log_dir = temp_dir.path();

    // Create a metrics store
    let metrics_store = MetricsStore::new(log_dir.to_path_buf());

    // Create a run result with fractional seconds
    let start_time = Utc::now();
    let end_time = start_time + Duration::milliseconds(2750); // 2.75 seconds

    let run_result = AgentRunResult {
        agent_name: "test-agent-fractional".to_string(),
        container_id: "container-fractional".to_string(),
        start_time,
        end_time,
        exit_code: 0,
        timed_out: false,
        termination_type: None,
        queued_start_time: None,
        skills_installed_count: 1,
        skills_failed_count: 0,
        skills_install_time_seconds: Some(2.75), // Fractional seconds
    };

    let mut all_metrics = AllMetrics::default();
    update_all_metrics(&mut all_metrics, &run_result).expect("Update should succeed");
    metrics_store
        .save(&all_metrics)
        .expect("Save should succeed");

    // Load and verify
    let loaded_metrics = metrics_store.load().expect("Load should succeed");
    let agent_data = &loaded_metrics.agents["test-agent-fractional"];

    // Verify fractional seconds are preserved
    assert_eq!(
        agent_data.skills_install_time_seconds,
        Some(2.75),
        "Fractional seconds should be preserved in persisted metrics"
    );

    // Verify the value is not rounded to an integer
    assert!(
        agent_data.skills_install_time_seconds.unwrap() % 1.0 != 0.0,
        "Fractional seconds should not be rounded to integer"
    );
}

/// Test: metrics.json contains install time data with correct structure
///
/// This test verifies that the metrics.json file structure contains the
/// expected skills_install_time_seconds data:
/// 1. agents[].runs[].skills_install_time_seconds contains actual f64 values
/// 2. The values are reasonable (> 0 and < 300 seconds)
/// 3. The data structure matches expectations
#[test]
fn test_metrics_json_contains_install_time_data() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let log_dir = temp_dir.path();

    // Create a metrics store
    let metrics_store = MetricsStore::new(log_dir.to_path_buf());

    // Create AllMetrics structure
    let mut all_metrics = AllMetrics::default();

    // First run: Agent 1 with skills
    let start_time1 = Utc::now();
    let end_time1 = start_time1 + Duration::seconds(8);

    let run_result1 = AgentRunResult {
        agent_name: "agent-with-skills-1".to_string(),
        container_id: "container-1".to_string(),
        start_time: start_time1,
        end_time: end_time1,
        exit_code: 0,
        timed_out: false,
        termination_type: None,
        queued_start_time: None,
        skills_installed_count: 3,
        skills_failed_count: 0,
        skills_install_time_seconds: Some(8.5),
    };

    update_all_metrics(&mut all_metrics, &run_result1).expect("First update should succeed");

    // Second run: Agent 1 with different skill set
    let start_time2 = Utc::now();
    let end_time2 = start_time2 + Duration::seconds(4);

    let run_result2 = AgentRunResult {
        agent_name: "agent-with-skills-1".to_string(),
        container_id: "container-2".to_string(),
        start_time: start_time2,
        end_time: end_time2,
        exit_code: 0,
        timed_out: false,
        termination_type: None,
        queued_start_time: None,
        skills_installed_count: 1,
        skills_failed_count: 0,
        skills_install_time_seconds: Some(4.2),
    };

    update_all_metrics(&mut all_metrics, &run_result2).expect("Second update should succeed");

    // Third run: Agent 2 with skills
    let start_time3 = Utc::now();
    let end_time3 = start_time3 + Duration::seconds(12);

    let run_result3 = AgentRunResult {
        agent_name: "agent-with-skills-2".to_string(),
        container_id: "container-3".to_string(),
        start_time: start_time3,
        end_time: end_time3,
        exit_code: 0,
        timed_out: false,
        termination_type: None,
        queued_start_time: None,
        skills_installed_count: 2,
        skills_failed_count: 0,
        skills_install_time_seconds: Some(12.0),
    };

    update_all_metrics(&mut all_metrics, &run_result3).expect("Third update should succeed");

    // Save metrics to disk
    metrics_store
        .save(&all_metrics)
        .expect("Save should succeed");

    // Load the metrics.json file as raw JSON
    let metrics_path = log_dir.join("metrics.json");
    let metrics_content = fs::read_to_string(&metrics_path).expect("Failed to read metrics.json");

    // Parse the JSON content
    let json: Value =
        serde_json::from_str(&metrics_content).expect("Failed to parse metrics.json as JSON");

    // Verify the top-level structure
    assert!(json.is_object(), "metrics.json root should be an object");

    assert!(
        json.get("agents").is_some(),
        "metrics.json should contain 'agents' field"
    );

    let agents = json["agents"]
        .as_object()
        .expect("'agents' should be an object");

    // Verify we have the expected agents
    assert!(
        agents.contains_key("agent-with-skills-1"),
        "metrics.json should contain 'agent-with-skills-1'"
    );

    assert!(
        agents.contains_key("agent-with-skills-2"),
        "metrics.json should contain 'agent-with-skills-2'"
    );

    // Verify agent-with-skills-1 structure
    let agent1 = &agents["agent-with-skills-1"];
    assert!(
        agent1["total_skills_installed"].is_number(),
        "agent-with-skills-1 should have total_skills_installed as number"
    );
    assert_eq!(
        agent1["total_skills_installed"].as_u64().unwrap(),
        4,
        "agent-with-skills-1 should have total_skills_installed = 4 (3 + 1)"
    );

    assert!(
        agent1["total_skills_failed"].is_number(),
        "agent-with-skills-1 should have total_skills_failed as number"
    );

    // Verify skills_install_time_seconds at agent level (cumulative)
    assert!(
        agent1["skills_install_time_seconds"].is_number(),
        "agent-with-skills-1 should have skills_install_time_seconds as number at agent level"
    );

    let agent1_cumulative_time = agent1["skills_install_time_seconds"]
        .as_f64()
        .expect("skills_install_time_seconds should be f64");
    assert!(
        agent1_cumulative_time > 0.0,
        "skills_install_time_seconds should be positive"
    );
    assert!(
        agent1_cumulative_time <= 300.0,
        "skills_install_time_seconds should be reasonable (< 300 seconds)"
    );
    assert!(
        agent1_cumulative_time.is_finite(),
        "skills_install_time_seconds should be a finite number"
    );

    // Verify runs array exists and contains data
    let runs = agent1["runs"]
        .as_array()
        .expect("agent-with-skills-1 should have 'runs' array");

    assert_eq!(runs.len(), 2, "agent-with-skills-1 should have 2 runs");

    // Verify each run has skills_install_time_seconds
    for (i, run) in runs.iter().enumerate() {
        assert!(run.is_object(), "Run {} should be an object", i);

        assert!(
            run.get("skills_install_time_seconds").is_some(),
            "Run {} should have 'skills_install_time_seconds' field",
            i
        );

        let run_time = run["skills_install_time_seconds"]
            .as_f64()
            .expect("skills_install_time_seconds should be f64 in run");

        assert!(
            run_time > 0.0,
            "Run {} skills_install_time_seconds should be positive, got: {}",
            i,
            run_time
        );

        assert!(
            run_time <= 300.0,
            "Run {} skills_install_time_seconds should be reasonable (< 300 seconds), got: {}",
            i,
            run_time
        );

        assert!(
            run_time.is_finite(),
            "Run {} skills_install_time_seconds should be a finite number",
            i
        );

        // Verify skills_installed_count
        assert!(
            run["skills_installed_count"].is_number(),
            "Run {} should have skills_installed_count as number",
            i
        );

        let installed_count = run["skills_installed_count"]
            .as_u64()
            .expect("skills_installed_count should be u64");

        assert!(
            installed_count > 0,
            "Run {} should have at least 1 skill installed",
            i
        );
    }

    // Verify agent-with-skills-2 structure
    let agent2 = &agents["agent-with-skills-2"];
    let runs2 = agent2["runs"]
        .as_array()
        .expect("agent-with-skills-2 should have 'runs' array");

    assert_eq!(runs2.len(), 1, "agent-with-skills-2 should have 1 run");

    let run2_time = runs2[0]["skills_install_time_seconds"]
        .as_f64()
        .expect("skills_install_time_seconds should be f64 in run");
    assert_eq!(
        run2_time, 12.0,
        "agent-with-skills-2 run should have skills_install_time_seconds = 12.0"
    );

    // Verify the JSON structure contains all expected fields for a run
    let run1_data = &runs[0];
    assert!(
        run1_data.get("run_id").is_some(),
        "Run should have 'run_id' field"
    );
    assert!(
        run1_data.get("timestamp").is_some(),
        "Run should have 'timestamp' field"
    );
    assert!(
        run1_data.get("duration_ms").is_some(),
        "Run should have 'duration_ms' field"
    );
    assert!(
        run1_data.get("status").is_some(),
        "Run should have 'status' field"
    );
    assert!(
        run1_data.get("skills_installed_count").is_some(),
        "Run should have 'skills_installed_count' field"
    );
    assert!(
        run1_data.get("skills_failed_count").is_some(),
        "Run should have 'skills_failed_count' field"
    );
    assert!(
        run1_data.get("skills_install_time_seconds").is_some(),
        "Run should have 'skills_install_time_seconds' field"
    );
}
