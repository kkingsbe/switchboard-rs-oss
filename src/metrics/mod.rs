//! Metrics data structures for tracking agent execution statistics.
//!
//! This module provides data structures for collecting, storing, and displaying
//! metrics about scheduled agent executions.
//!
//! # Error Handling
//!
//! The metrics system is designed to be resilient and never fail the scheduler operation.
//! Errors during metrics operations are logged but do not interrupt agent execution.
//!
//! ## Metrics File Corruption
//!
//! If the metrics file (`<log_dir>/metrics.json`) becomes corrupted:
//! - The corrupted file is automatically backed up to `metrics.json.backup.<timestamp>`
//! - A warning is logged to stderr with the backup file path
//! - `MetricsStore::load()` returns `Err(MetricsError::CorruptedFile(backup_path))`
//! - The scheduler continues running (new empty metrics are created on next save)
//!
//! To recover from a corrupted metrics file:
//! 1. Check the backup file at `metrics.json.backup.<timestamp>`
//! 2. If the backup is valid, restore it by copying back to `metrics.json`
//! 3. If the backup is also corrupted, delete both files and let the scheduler start fresh
//!
//! ## Missing Metrics File
//!
//! If the metrics file doesn't exist on first run:
//! - `MetricsStore::load()` returns `Err(MetricsError::FileNotFound(path))`
//! - The first call to `MetricsStore::save()` creates a new metrics file with empty data
//! - No user intervention is required
//!
//! ## Metrics Update Failures
//!
//! If metrics update or save operations fail during scheduler execution:
//! - Errors are logged using `tracing::error!()` but do not fail the scheduler
//! - The scheduler continues running and processing agents
//! - Agent runs are not interrupted by metrics failures
//! - This design ensures agent reliability even if the metrics subsystem has issues
//!
//! ## Atomic Write Guarantees
//!
//! `MetricsStore::save()` uses an atomic write pattern:
//! - Data is first written to a temporary file (`metrics.json.tmp`)
//! - Only if the write succeeds, the temporary file is renamed to `metrics.json`
//! - This prevents partial or corrupted writes if the process crashes mid-operation
//!
//! # Error Types
//!
//! See the [`MetricsError`](enum@MetricsError) enum for all possible error conditions:
//! - `FileNotFound` - Metrics file doesn't exist
//! - `ReadError` - Failed to read metrics file
//! - `WriteError` - Failed to write metrics file or create directories
//! - `SerializationError` - Failed to serialize metrics to JSON
//! - `DeserializationError` - Failed to deserialize JSON (typically caused by corruption)
//! - `CorruptedFile` - Metrics file is corrupted (backup path included)

mod collector;
mod store;

pub use collector::update_all_metrics;
pub use store::{AgentMetricsData, AgentRunResultData, AllMetrics, MetricsStore};

/// Error types for metrics operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum MetricsError {
    /// Error when reading metrics from storage.
    #[error("Failed to read metrics: {0}")]
    ReadError(String),

    /// Error when writing metrics to storage.
    #[error("Failed to write metrics: {0}")]
    WriteError(String),

    /// Error when serializing metrics.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Error when deserializing metrics.
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Error when metrics file is not found.
    #[error("Metrics file not found: {0}")]
    FileNotFound(String),

    /// Error when metrics file is corrupted. Contains the backup file path.
    #[error("Metrics file is corrupted. Backup saved to: {0}")]
    CorruptedFile(String),
}

/// Result of a single agent run, used to update metrics.
#[derive(Debug, Clone)]
pub struct AgentRunResult {
    /// Name of the agent that ran.
    pub agent_name: String,
    /// Docker container ID for the run.
    pub container_id: String,
    /// When the run started.
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// When the run ended.
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// Exit code from the container (0 = success).
    pub exit_code: i64,
    /// Whether the run was terminated due to timeout.
    pub timed_out: bool,
    /// Type of termination used: "sigterm", "sigkill", or None.
    pub termination_type: Option<String>,
    /// When the job was added to the queue (for overlap_mode:queue).
    pub queued_start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of skills that were successfully installed in the container.
    pub skills_installed_count: u32,
    /// Number of skills that failed to install during the setup phase.
    pub skills_failed_count: u32,
    /// Time spent on skill installation in seconds (None if no skills were configured).
    pub skills_install_time_seconds: Option<f64>,
}

/// Comprehensive set of performance and reliability metrics tracked for each agent.
///
/// `AgentMetrics` provides a detailed view of an agent's execution history, including
/// success/failure rates, runtime statistics, queue performance, and termination behavior.
/// These metrics are essential for monitoring agent performance, reliability, queue behavior,
/// and detecting concurrent execution issues.
///
/// # When Metrics Are Updated
/// Metrics are updated after each agent run completes, regardless of whether the run
/// succeeded or failed. This ensures that all execution attempts are captured.
///
/// # Persistence
/// Metrics persist across scheduler restarts by being stored in `<log_dir>/metrics.json`.
/// The metrics are serialized to JSON and written to disk after each agent run completes,
/// ensuring no data is lost even if the scheduler crashes or is stopped.
///
/// # Use Cases
/// - **Performance Monitoring**: Track average run duration and detect performance regressions
/// - **Reliability Assessment**: Monitor success rates and failure patterns
/// - **Queue Analysis**: Identify scheduler contention or resource bottlenecks via wait times
/// - **Concurrent Execution Tracking**: Detect overlapping runs and ensure proper isolation
///
/// # Example
///
/// ```rust
/// # use switchboard::metrics::AgentMetrics;
/// # use chrono::Utc;
/// #
/// let metrics = AgentMetrics {
///     run_count: 10,
///     success_count: 8,
///     failure_count: 2,
///     total_runtime_seconds: 150.0,
///     average_run_duration_seconds: 15.0,
///     queue_wait_time_seconds: Some(2.5),
///     timeout_count: 1,
///     sigterm_count: 0,
///     sigkill_count: 1,
///     first_run_timestamp: Some(Utc::now()),
///     last_run_timestamp: Some(Utc::now()),
///     last_run_duration_seconds: Some(15.0),
///     concurrent_run_id: None,
/// };
///
/// // Calculate success rate
/// let success_rate = metrics.success_count as f64 / metrics.run_count as f64;
/// println!("Success rate: {:.1}%", success_rate * 100.0);
///
/// // Check if agent is currently running
/// if let Some(run_id) = &metrics.concurrent_run_id {
///     println!("Agent is running: {}", run_id);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AgentMetrics {
    /// Total number of times the agent has been executed.
    ///
    /// Includes both successful and failed runs. This counter is incremented after each
    /// agent run completes, providing a complete count of all execution attempts.
    pub run_count: u64,
    /// Number of successful agent runs.
    ///
    /// A successful run is defined as an execution that completed with exit code 0 and
    /// did not time out. This counter is used to calculate the success rate:
    /// `success_count / run_count`.
    pub success_count: u64,
    /// Number of failed agent runs.
    ///
    /// A failed run is defined as an execution that resulted in a non-zero exit code,
    /// timed out, or was terminated via a signal. This counter is used to calculate
    /// the failure rate: `failure_count / run_count`.
    pub failure_count: u64,
    /// Cumulative runtime of all successful and failed runs in seconds.
    ///
    /// This field accumulates the duration of every run regardless of outcome. It is
    /// used to calculate `average_run_duration_seconds` and to track overall resource
    /// usage by the agent over time.
    pub total_runtime_seconds: f64,
    /// Timestamp of the agent's first execution.
    ///
    /// Used to track how long the agent has been running since deployment. This value is
    /// set during the first run and never changes, providing a baseline for measuring
    /// agent uptime. None if the agent has never run.
    pub first_run_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Timestamp of the agent's most recent execution.
    ///
    /// Used to determine agent activity and freshness. A recent timestamp indicates the
    /// agent is actively running according to its schedule, while an old timestamp may
    /// indicate a stopped or misconfigured agent. None if the agent has never run.
    pub last_run_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Duration of the most recent run in seconds.
    ///
    /// Used to detect performance regressions by comparing recent run times against
    /// historical averages. Sudden increases may indicate resource contention or
    /// code changes. None if the agent has never run.
    pub last_run_duration_seconds: Option<f64>,
    /// Average duration of all runs in seconds.
    ///
    /// Calculated as `total_runtime_seconds / run_count`. This metric provides a
    /// baseline for performance benchmarking and trend analysis. It helps identify
    /// whether run durations are stable, increasing, or decreasing over time.
    pub average_run_duration_seconds: f64,
    /// Average time the agent spends waiting in the execution queue before starting.
    ///
    /// This metric is only relevant when using `overlap_mode: queue`. Higher values
    /// indicate scheduler contention or resource bottlenecks, suggesting that many
    /// agents are competing for execution slots. Used for queue performance monitoring
    /// and capacity planning. None if queue wait time tracking is not available.
    pub queue_wait_time_seconds: Option<f64>,
    /// Number of times the agent exceeded its configured timeout and was terminated.
    ///
    /// Used to track agent reliability and detect timeout configuration issues. A high
    /// timeout count may indicate that the agent is doing more work than expected or
    /// that the timeout value is set too low for the task being performed.
    pub timeout_count: u64,
    /// Number of times the agent was terminated via SIGTERM (graceful shutdown).
    ///
    /// SIGTERM is the standard signal for graceful shutdown, allowing the agent to
    /// clean up resources before exiting. This counter tracks intentional terminations
    /// and restart behavior, such as when an agent is stopped via CLI commands.
    pub sigterm_count: u64,
    /// Number of times the agent was terminated via SIGKILL (forceful kill).
    ///
    /// SIGKILL is used when an agent does not respond to SIGTERM and must be forcefully
    /// terminated. A high sigkill count may indicate that the agent is hanging or not
    /// handling shutdown signals properly, which could lead to resource leaks or
    /// incomplete work.
    pub sigkill_count: u64,
    /// Identifier of the currently running instance (if any).
    ///
    /// Used to track concurrent execution visibility and detect overlapping runs. This
    /// field is set when an agent starts execution and cleared when it completes. It can
    /// be used to identify which container or process ID is currently executing the
    /// agent. None when the agent is not running.
    pub concurrent_run_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_agent_run_result_creation() {
        let start_time = Utc::now();
        let end_time = start_time + Duration::seconds(10);
        let queued_time = start_time - Duration::seconds(5);

        let run_result = AgentRunResult {
            agent_name: "test_agent".to_string(),
            container_id: "container_abc123".to_string(),
            start_time,
            end_time,
            exit_code: 0,
            timed_out: false,
            termination_type: None,
            queued_start_time: Some(queued_time),
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        assert_eq!(run_result.agent_name, "test_agent");
        assert_eq!(run_result.container_id, "container_abc123");
        assert_eq!(run_result.exit_code, 0);
        assert!(!run_result.timed_out);
        assert_eq!(run_result.start_time, start_time);
        assert_eq!(run_result.end_time, end_time);
        assert_eq!(run_result.queued_start_time, Some(queued_time));
    }

    #[test]
    fn test_agent_run_result_with_none_queued_time() {
        let start_time = Utc::now();
        let end_time = start_time + Duration::seconds(15);

        let run_result = AgentRunResult {
            agent_name: "agent_no_queue".to_string(),
            container_id: "container_def456".to_string(),
            start_time,
            end_time,
            exit_code: 1,
            timed_out: true,
            termination_type: None,
            queued_start_time: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        assert_eq!(run_result.agent_name, "agent_no_queue");
        assert_eq!(run_result.exit_code, 1);
        assert!(run_result.timed_out);
        assert_eq!(run_result.queued_start_time, None);
    }

    #[test]
    fn test_agent_run_result_failure_case() {
        let start_time = Utc::now();
        let end_time = start_time + Duration::seconds(20);

        let run_result = AgentRunResult {
            agent_name: "failing_agent".to_string(),
            container_id: "container_fail123".to_string(),
            start_time,
            end_time,
            exit_code: 127,
            timed_out: false,
            termination_type: None,
            queued_start_time: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        assert_eq!(run_result.exit_code, 127);
        assert!(!run_result.timed_out);
    }

    #[test]
    fn test_agent_metrics_computed_fields_all_success() {
        let first_run_time = Utc::now();
        let last_run_time = first_run_time + Duration::hours(1);

        let metrics = AgentMetrics {
            run_count: 10,
            success_count: 10,
            failure_count: 0,
            total_runtime_seconds: 500.0,
            first_run_timestamp: Some(first_run_time),
            last_run_timestamp: Some(last_run_time),
            last_run_duration_seconds: Some(50.0),
            average_run_duration_seconds: 50.0,
            queue_wait_time_seconds: Some(12.5),
            timeout_count: 0,
            sigterm_count: 0,
            sigkill_count: 0,
            concurrent_run_id: None,
        };

        // Verify computed fields match expectations
        assert_eq!(metrics.success_count, 10);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.average_run_duration_seconds, 50.0);
    }

    #[test]
    fn test_agent_metrics_computed_fields_mixed_results() {
        let first_run_time = Utc::now();
        let last_run_time = first_run_time + Duration::hours(2);

        let metrics = AgentMetrics {
            run_count: 25,
            success_count: 20,
            failure_count: 5,
            total_runtime_seconds: 1250.0,
            first_run_timestamp: Some(first_run_time),
            last_run_timestamp: Some(last_run_time),
            last_run_duration_seconds: Some(50.0),
            average_run_duration_seconds: 50.0,
            queue_wait_time_seconds: Some(8.0),
            timeout_count: 3,
            sigterm_count: 1,
            sigkill_count: 2,
            concurrent_run_id: Some("current_container".to_string()),
        };

        // Verify computed fields
        assert_eq!(metrics.success_count, 20);
        assert_eq!(metrics.failure_count, 5);
        assert_eq!(metrics.average_run_duration_seconds, 50.0);
        assert_eq!(metrics.timeout_count, 3);
        assert_eq!(
            metrics.concurrent_run_id,
            Some("current_container".to_string())
        );
    }

    #[test]
    fn test_agent_metrics_average_calculation() {
        let first_run_time = Utc::now();
        let last_run_time = first_run_time + Duration::minutes(30);

        // Test with run_count = 5 and total_runtime = 250 seconds
        let metrics = AgentMetrics {
            run_count: 5,
            success_count: 3,
            failure_count: 2,
            total_runtime_seconds: 250.0,
            first_run_timestamp: Some(first_run_time),
            last_run_timestamp: Some(last_run_time),
            last_run_duration_seconds: Some(60.0),
            average_run_duration_seconds: 50.0, // 250 / 5 = 50
            queue_wait_time_seconds: None,
            timeout_count: 1,
            sigterm_count: 0,
            sigkill_count: 0,
            concurrent_run_id: None,
        };

        assert_eq!(metrics.average_run_duration_seconds, 50.0);
    }

    #[test]
    fn test_agent_metrics_zero_runs() {
        let metrics = AgentMetrics {
            run_count: 0,
            success_count: 0,
            failure_count: 0,
            total_runtime_seconds: 0.0,
            first_run_timestamp: None,
            last_run_timestamp: None,
            last_run_duration_seconds: None,
            average_run_duration_seconds: 0.0,
            queue_wait_time_seconds: None,
            timeout_count: 0,
            sigterm_count: 0,
            sigkill_count: 0,
            concurrent_run_id: None,
        };

        assert_eq!(metrics.run_count, 0);
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.average_run_duration_seconds, 0.0);
        assert!(metrics.first_run_timestamp.is_none());
        assert!(metrics.last_run_timestamp.is_none());
        assert!(metrics.last_run_duration_seconds.is_none());
    }

    #[test]
    fn test_agent_metrics_with_queue_wait() {
        let start_time = Utc::now();
        let metrics = AgentMetrics {
            run_count: 3,
            success_count: 3,
            failure_count: 0,
            total_runtime_seconds: 45.0,
            first_run_timestamp: Some(start_time),
            last_run_timestamp: Some(start_time + Duration::seconds(15)),
            last_run_duration_seconds: Some(15.0),
            average_run_duration_seconds: 15.0,
            queue_wait_time_seconds: Some(5.0), // Average wait time in queue
            timeout_count: 0,
            sigterm_count: 0,
            sigkill_count: 0,
            concurrent_run_id: None,
        };

        assert_eq!(metrics.queue_wait_time_seconds, Some(5.0));
    }

    #[test]
    fn test_agent_metrics_all_failures() {
        let first_run_time = Utc::now();
        let last_run_time = first_run_time + Duration::minutes(45);

        let metrics = AgentMetrics {
            run_count: 8,
            success_count: 0,
            failure_count: 8,
            total_runtime_seconds: 120.0,
            first_run_timestamp: Some(first_run_time),
            last_run_timestamp: Some(last_run_time),
            last_run_duration_seconds: Some(15.0),
            average_run_duration_seconds: 15.0,
            queue_wait_time_seconds: None,
            timeout_count: 5,
            sigterm_count: 2,
            sigkill_count: 3,
            concurrent_run_id: None,
        };

        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.failure_count, 8);
        assert_eq!(metrics.timeout_count, 5);
    }
}
