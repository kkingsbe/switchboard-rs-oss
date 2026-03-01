//! Collector module for updating metrics from agent run results.

use crate::metrics::{
    AgentMetricsData, AgentRunResult, AgentRunResultData, AllMetrics, MetricsError,
};

/// Update all metrics with the result of an agent run.
///
/// This function processes an AgentRunResult and updates the corresponding
/// agent's metrics in the AllMetrics structure. If the agent doesn't exist,
/// a new entry is created.
///
/// # Arguments
///
/// * `all_metrics` - Mutable reference to the AllMetrics structure to update
/// * `run_result` - Reference to the AgentRunResult containing execution data
///
/// # Returns
///
/// * `Result<(), MetricsError>` - Ok(()) on success, Err on failure
///
/// # Example
///
/// ```no_run
/// use switchboard::metrics::{AllMetrics, AgentRunResult, update_all_metrics};
/// use chrono::Utc;
///
/// let mut all_metrics = AllMetrics::default();
/// let run_result = AgentRunResult {
///     agent_name: "test_agent".to_string(),
///     container_id: "abc123".to_string(),
///     start_time: Utc::now(),
///     end_time: Utc::now(),
///     exit_code: 0,
///     timed_out: false,
///     termination_type: None,
///     queued_start_time: None,
///     skills_installed_count: 0,
///     skills_failed_count: 0,
///     skills_install_time_seconds: None,
/// };
/// update_all_metrics(&mut all_metrics, &run_result).unwrap();
/// ```
pub fn update_all_metrics(
    all_metrics: &mut AllMetrics,
    run_result: &AgentRunResult,
) -> Result<(), MetricsError> {
    let agent_data = all_metrics
        .agents
        .entry(run_result.agent_name.clone())
        .or_insert_with(|| AgentMetricsData {
            total_runs: 0,
            successful_runs: 0,
            failed_runs: 0,
            total_duration_ms: 0,
            runs: Vec::new(),
            queue_wait_time_seconds: None,
            queue_wait_times: Vec::new(),
            sigterm_count: 0,
            sigkill_count: 0,
            timeout_count: 0,
            total_skills_installed: 0,
            total_skills_failed: 0,
            skills_install_time_seconds: None,
            runs_with_skill_failures: 0,
            max_run_duration_ms: None,
            min_run_duration_ms: None,
            last_success_timestamp: None,
        });

    // Determine if the run was successful
    let is_success = run_result.exit_code == 0 && !run_result.timed_out;

    // Increment counters
    agent_data.total_runs += 1;
    if is_success {
        agent_data.successful_runs += 1;
    } else {
        agent_data.failed_runs += 1;
    }

    // Increment termination signal counters
    if let Some(ref termination_type) = run_result.termination_type {
        match termination_type.as_str() {
            "sigterm" => agent_data.sigterm_count += 1,
            "sigkill" => agent_data.sigkill_count += 1,
            _ => {}
        }
    }

    // Increment timeout counter
    if run_result.timed_out {
        agent_data.timeout_count += 1;
    }

    // Calculate and add duration
    let duration_ms = (run_result.end_time - run_result.start_time).num_milliseconds() as u64;
    agent_data.total_duration_ms += duration_ms;

    // Calculate queue wait time if available
    if let Some(queued_time) = run_result.queued_start_time {
        let queue_wait_seconds = (run_result.start_time - queued_time).num_seconds() as u64;
        agent_data.queue_wait_time_seconds =
            Some(agent_data.queue_wait_time_seconds.unwrap_or(0) + queue_wait_seconds);
        // Track individual wait times
        agent_data.queue_wait_times.push(queue_wait_seconds);
    }

    // Add skill installation counts to aggregated metrics
    agent_data.total_skills_installed += run_result.skills_installed_count as u64;
    agent_data.total_skills_failed += run_result.skills_failed_count as u64;

    // Track runs with skill failures
    if run_result.skills_failed_count > 0 {
        agent_data.runs_with_skill_failures += 1;
    }

    // Accumulate skill install time (for calculating average)
    if let Some(time) = run_result.skills_install_time_seconds {
        agent_data.skills_install_time_seconds =
            Some(agent_data.skills_install_time_seconds.unwrap_or(0.0) + time);
    }

    // Determine status and error message
    let status = if is_success {
        "success".to_string()
    } else {
        "failure".to_string()
    };

    let error_message = if run_result.timed_out {
        Some("timed_out".to_string())
    } else if run_result.exit_code != 0 {
        Some(format!("exit_code: {}", run_result.exit_code))
    } else {
        None
    };

    // Create and append run result data
    let run_result_data = AgentRunResultData {
        run_id: run_result.container_id.clone(),
        timestamp: run_result.start_time.timestamp() as u64,
        duration_ms,
        status,
        error_message,
        skills_installed_count: run_result.skills_installed_count,
        skills_failed_count: run_result.skills_failed_count,
        skills_install_time_seconds: run_result.skills_install_time_seconds,
    };

    agent_data.runs.push(run_result_data);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn create_test_run_result(
        agent_name: &str,
        container_id: &str,
        exit_code: i64,
        timed_out: bool,
        queue_delay_seconds: Option<i64>,
    ) -> AgentRunResult {
        let start_time = Utc::now();
        let end_time = start_time + Duration::seconds(10);

        AgentRunResult {
            agent_name: agent_name.to_string(),
            container_id: container_id.to_string(),
            start_time,
            end_time,
            exit_code,
            timed_out,
            termination_type: None,
            queued_start_time: queue_delay_seconds
                .map(|delay| start_time - Duration::seconds(delay)),
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        }
    }

    #[test]
    fn test_update_metrics_for_new_agent() {
        let mut all_metrics = AllMetrics::default();
        let run_result = create_test_run_result("new_agent", "container_123", 0, false, None);

        update_all_metrics(&mut all_metrics, &run_result).unwrap();

        assert!(all_metrics.agents.contains_key("new_agent"));
        let agent_data = &all_metrics.agents["new_agent"];
        assert_eq!(agent_data.total_runs, 1);
        assert_eq!(agent_data.successful_runs, 1);
        assert_eq!(agent_data.failed_runs, 0);
        assert_eq!(agent_data.total_duration_ms, 10000);
        assert_eq!(agent_data.runs.len(), 1);
        assert_eq!(agent_data.runs[0].run_id, "container_123");
        assert_eq!(agent_data.runs[0].status, "success");
        assert_eq!(agent_data.runs[0].error_message, None);
    }

    #[test]
    fn test_update_metrics_for_existing_agent() {
        let mut all_metrics = AllMetrics::default();
        let run_result1 = create_test_run_result("existing_agent", "container_123", 0, false, None);
        let run_result2 = create_test_run_result("existing_agent", "container_456", 0, false, None);

        update_all_metrics(&mut all_metrics, &run_result1).unwrap();
        update_all_metrics(&mut all_metrics, &run_result2).unwrap();

        assert!(all_metrics.agents.contains_key("existing_agent"));
        let agent_data = &all_metrics.agents["existing_agent"];
        assert_eq!(agent_data.total_runs, 2);
        assert_eq!(agent_data.successful_runs, 2);
        assert_eq!(agent_data.failed_runs, 0);
        assert_eq!(agent_data.total_duration_ms, 20000);
        assert_eq!(agent_data.runs.len(), 2);
    }

    #[test]
    fn test_failure_tracking_with_non_zero_exit_code() {
        let mut all_metrics = AllMetrics::default();
        let run_result = create_test_run_result("failing_agent", "container_123", 1, false, None);

        update_all_metrics(&mut all_metrics, &run_result).unwrap();

        let agent_data = &all_metrics.agents["failing_agent"];
        assert_eq!(agent_data.total_runs, 1);
        assert_eq!(agent_data.successful_runs, 0);
        assert_eq!(agent_data.failed_runs, 1);
        assert_eq!(agent_data.runs[0].status, "failure");
        assert_eq!(
            agent_data.runs[0].error_message,
            Some("exit_code: 1".to_string())
        );
    }

    #[test]
    fn test_failure_tracking_with_timeout() {
        let mut all_metrics = AllMetrics::default();
        let run_result = create_test_run_result("timeout_agent", "container_123", 0, true, None);

        update_all_metrics(&mut all_metrics, &run_result).unwrap();

        let agent_data = &all_metrics.agents["timeout_agent"];
        assert_eq!(agent_data.total_runs, 1);
        assert_eq!(agent_data.successful_runs, 0);
        assert_eq!(agent_data.failed_runs, 1);
        assert_eq!(agent_data.runs[0].status, "failure");
        assert_eq!(
            agent_data.runs[0].error_message,
            Some("timed_out".to_string())
        );
    }

    #[test]
    fn test_queue_wait_time_tracking() {
        let mut all_metrics = AllMetrics::default();
        let run_result1 =
            create_test_run_result("queued_agent", "container_123", 0, false, Some(5));
        let run_result2 =
            create_test_run_result("queued_agent", "container_456", 0, false, Some(10));

        update_all_metrics(&mut all_metrics, &run_result1).unwrap();
        update_all_metrics(&mut all_metrics, &run_result2).unwrap();

        let agent_data = &all_metrics.agents["queued_agent"];
        assert_eq!(agent_data.queue_wait_time_seconds, Some(15));
    }

    #[test]
    fn test_queue_wait_time_none_when_not_provided() {
        let mut all_metrics = AllMetrics::default();
        let run_result = create_test_run_result("no_queue_agent", "container_123", 0, false, None);

        update_all_metrics(&mut all_metrics, &run_result).unwrap();

        let agent_data = &all_metrics.agents["no_queue_agent"];
        assert_eq!(agent_data.queue_wait_time_seconds, None);
    }

    #[test]
    fn test_mixed_success_and_failure() {
        let mut all_metrics = AllMetrics::default();

        let success_run = create_test_run_result("mixed_agent", "container_123", 0, false, None);
        let failure_run = create_test_run_result("mixed_agent", "container_456", 1, false, None);
        let timeout_run = create_test_run_result("mixed_agent", "container_789", 0, true, None);

        update_all_metrics(&mut all_metrics, &success_run).unwrap();
        update_all_metrics(&mut all_metrics, &failure_run).unwrap();
        update_all_metrics(&mut all_metrics, &timeout_run).unwrap();

        let agent_data = &all_metrics.agents["mixed_agent"];
        assert_eq!(agent_data.total_runs, 3);
        assert_eq!(agent_data.successful_runs, 1);
        assert_eq!(agent_data.failed_runs, 2);
    }

    #[test]
    fn test_termination_signal_tracking() {
        let mut all_metrics = AllMetrics::default();
        let start_time = Utc::now();
        let end_time = start_time + Duration::seconds(10);

        // Create run results with different termination types
        let sigterm_run = AgentRunResult {
            agent_name: "signal_agent".to_string(),
            container_id: "container_sigterm".to_string(),
            start_time,
            end_time,
            exit_code: 143, // SIGTERM exit code
            timed_out: false,
            termination_type: Some("sigterm".to_string()),
            queued_start_time: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        let sigkill_run = AgentRunResult {
            agent_name: "signal_agent".to_string(),
            container_id: "container_sigkill".to_string(),
            start_time,
            end_time,
            exit_code: 137, // SIGKILL exit code
            timed_out: false,
            termination_type: Some("sigkill".to_string()),
            queued_start_time: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        let normal_run = AgentRunResult {
            agent_name: "signal_agent".to_string(),
            container_id: "container_normal".to_string(),
            start_time,
            end_time,
            exit_code: 0,
            timed_out: false,
            termination_type: None,
            queued_start_time: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        // Update metrics for each run
        update_all_metrics(&mut all_metrics, &sigterm_run).unwrap();
        update_all_metrics(&mut all_metrics, &sigkill_run).unwrap();
        update_all_metrics(&mut all_metrics, &normal_run).unwrap();

        // Verify counters
        let agent_data = &all_metrics.agents["signal_agent"];
        assert_eq!(agent_data.total_runs, 3);
        assert_eq!(agent_data.successful_runs, 1); // normal_run
        assert_eq!(agent_data.failed_runs, 2); // sigterm_run, sigkill_run
        assert_eq!(agent_data.sigterm_count, 1);
        assert_eq!(agent_data.sigkill_count, 1);
    }

    #[test]
    fn test_multiple_sigterm_sigkill_counts() {
        let mut all_metrics = AllMetrics::default();
        let start_time = Utc::now();
        let end_time = start_time + Duration::seconds(10);

        // Create multiple runs with sigterm and sigkill
        let sigterm_run1 = AgentRunResult {
            agent_name: "multi_signal_agent".to_string(),
            container_id: "container_sigterm1".to_string(),
            start_time,
            end_time,
            exit_code: 143,
            timed_out: false,
            termination_type: Some("sigterm".to_string()),
            queued_start_time: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        let sigterm_run2 = AgentRunResult {
            agent_name: "multi_signal_agent".to_string(),
            container_id: "container_sigterm2".to_string(),
            start_time,
            end_time,
            exit_code: 143,
            timed_out: false,
            termination_type: Some("sigterm".to_string()),
            queued_start_time: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        let sigkill_run1 = AgentRunResult {
            agent_name: "multi_signal_agent".to_string(),
            container_id: "container_sigkill1".to_string(),
            start_time,
            end_time,
            exit_code: 137,
            timed_out: false,
            termination_type: Some("sigkill".to_string()),
            queued_start_time: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        let sigkill_run2 = AgentRunResult {
            agent_name: "multi_signal_agent".to_string(),
            container_id: "container_sigkill2".to_string(),
            start_time,
            end_time,
            exit_code: 137,
            timed_out: false,
            termination_type: Some("sigkill".to_string()),
            queued_start_time: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        let sigkill_run3 = AgentRunResult {
            agent_name: "multi_signal_agent".to_string(),
            container_id: "container_sigkill3".to_string(),
            start_time,
            end_time,
            exit_code: 137,
            timed_out: false,
            termination_type: Some("sigkill".to_string()),
            queued_start_time: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        // Update metrics for each run
        update_all_metrics(&mut all_metrics, &sigterm_run1).unwrap();
        update_all_metrics(&mut all_metrics, &sigterm_run2).unwrap();
        update_all_metrics(&mut all_metrics, &sigkill_run1).unwrap();
        update_all_metrics(&mut all_metrics, &sigkill_run2).unwrap();
        update_all_metrics(&mut all_metrics, &sigkill_run3).unwrap();

        // Verify counters accumulate correctly
        let agent_data = &all_metrics.agents["multi_signal_agent"];
        assert_eq!(agent_data.total_runs, 5);
        assert_eq!(agent_data.failed_runs, 5);
        assert_eq!(agent_data.sigterm_count, 2);
        assert_eq!(agent_data.sigkill_count, 3);
    }
}
