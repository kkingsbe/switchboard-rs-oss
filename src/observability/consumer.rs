//! Consumer module for computing derived metrics from events.
//!
//! This module provides functionality to read events from the event log
//! and compute derived metrics including throughput and reliability metrics.
//!
//! # Metrics Computed
//!
//! ## Throughput & Velocity
//! - Agent runs: Count of `container.started` events in window
//! - Productive runs: Count of `container.exited` events where `git.diff` has `commit_count > 0`
//! - Productive run rate: Productive runs / agent runs
//! - Commits: Sum of `git.diff.data.commit_count` across all runs
//! - Lines inserted: Sum of `git.diff.data.total_insertions`
//! - Lines deleted: Sum of `git.diff.data.total_deletions`
//! - Files changed: Sum of `git.diff.data.total_files_changed`
//! - Avg run duration: Mean of `container.exited.data.duration_seconds`
//! - Avg commits per run: Commits / productive runs
//!
//! ## Reliability
//! - Container failures: Count of `container.exited` where `exit_code != 0`
//! - Failure rate: Container failures / total `container.exited`
//! - Timeouts: Count of `container.exited` where `timeout_hit == true`
//! - Skipped runs: Count of `container.skipped` events
//! - Empty runs: Count of runs where `commit_count == 0` AND `exit_code == 0`
//! - Scheduler uptime: Time from first `scheduler.started` to last event
//!
//! ## Per-Agent Breakdown
//! All metrics support grouping by agent name.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use chrono::{DateTime, Utc};

use super::event::{Event, EventData, EventType};

/// Error types for consumer operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConsumerError {
    /// Error when reading events from file.
    #[error("Failed to read events: {0}")]
    ReadError(String),

    /// Error when parsing events.
    #[error("Failed to parse event: {0}")]
    ParseError(String),

    /// Error when no events file found.
    #[error("Events file not found: {0}")]
    FileNotFound(String),
}

/// Result type for consumer operations.
pub type ConsumerResult<T> = Result<T, ConsumerError>;

/// Throughput metrics computed from events.
#[derive(Debug, Clone, Default)]
pub struct ThroughputMetrics {
    /// Number of agent runs (container.started events)
    pub agent_runs: u64,
    /// Number of productive runs (runs with commits)
    pub productive_runs: u64,
    /// Rate of productive runs (productive_runs / agent_runs)
    pub productive_run_rate: f64,
    /// Total commits made
    pub commits: u64,
    /// Total lines inserted
    pub lines_inserted: u64,
    /// Total lines deleted
    pub lines_deleted: u64,
    /// Total files changed
    pub files_changed: u64,
    /// Average run duration in seconds
    pub avg_run_duration_seconds: f64,
    /// Average commits per productive run
    pub avg_commits_per_run: f64,
}

/// Reliability metrics computed from events.
#[derive(Debug, Clone, Default)]
pub struct ReliabilityMetrics {
    /// Number of container failures (non-zero exit codes)
    pub container_failures: u64,
    /// Failure rate (failures / total exits)
    pub failure_rate: f64,
    /// Number of timeouts
    pub timeouts: u64,
    /// Number of skipped runs
    pub skipped_runs: u64,
    /// Number of empty runs (exit_code == 0 && commit_count == 0)
    pub empty_runs: u64,
    /// Scheduler uptime in seconds
    pub scheduler_uptime_seconds: u64,
}

/// Complete derived metrics for an agent or overall.
#[derive(Debug, Clone, Default)]
pub struct DerivedMetrics {
    /// Throughput metrics
    pub throughput: ThroughputMetrics,
    /// Reliability metrics
    pub reliability: ReliabilityMetrics,
}

/// Event log consumer for computing derived metrics.
#[derive(Debug, Default)]
pub struct EventConsumer {
    /// Collected events for processing
    events: Vec<Event>,
    /// Run ID to agent mapping
    run_agent_map: HashMap<String, String>,
    /// Run ID to exit data mapping
    run_exit_data: HashMap<String, ExitData>,
    /// Agent name from scheduler.started events
    scheduler_agents: Vec<String>,
    /// First event timestamp
    first_event_timestamp: Option<DateTime<Utc>>,
    /// Last event timestamp
    last_event_timestamp: Option<DateTime<Utc>>,
}

/// Helper struct to store exit data for correlating with git diffs.
#[derive(Debug, Clone, Default)]
struct ExitData {
    /// Exit code
    exit_code: i32,
    /// Duration in seconds
    duration_seconds: u64,
    /// Whether timeout was hit
    timeout_hit: bool,
    /// Timestamp
    timestamp: DateTime<Utc>,
    /// Agent name
    agent: Option<String>,
}

impl EventConsumer {
    /// Create a new EventConsumer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Read events from a JSONL file.
    pub fn read_events(&mut self, path: impl AsRef<Path>) -> ConsumerResult<()> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(ConsumerError::FileNotFound(path.display().to_string()));
        }

        let file = File::open(path).map_err(|e| {
            ConsumerError::ReadError(format!("Failed to open events file: {}", e))
        })?;

        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line.map_err(|e| {
                ConsumerError::ReadError(format!("Failed to read line: {}", e))
            })?;

            if line.trim().is_empty() {
                continue;
            }

            let event: Event = serde_json::from_str(&line).map_err(|e| {
                ConsumerError::ParseError(format!("Failed to parse event: {}", e))
            })?;

            self.process_event(event);
        }

        Ok(())
    }

    /// Process a single event, updating internal state.
    fn process_event(&mut self, event: Event) {
        // Track first/last timestamps
        let timestamp = event.timestamp;
        if self.first_event_timestamp.is_none() {
            self.first_event_timestamp = Some(timestamp);
        }
        self.last_event_timestamp = Some(timestamp);

        // Store events for later processing
        self.events.push(event.clone());

        // Process based on event type
        match &event.payload {
            EventData::SchedulerStarted { agents, .. } => {
                self.scheduler_agents = agents.clone();
            }
            EventData::ContainerStarted { container_id, .. } => {
                // Map run_id to agent
                let agent = event.agent.clone().unwrap_or_else(|| "unknown".to_string());
                self.run_agent_map.insert(container_id.clone(), agent);
            }
            EventData::ContainerExited { exit_code, duration_seconds, timeout_hit } => {
                if let Some(run_id) = &event.run_id {
                    self.run_exit_data.insert(
                        run_id.clone(),
                        ExitData {
                            exit_code: *exit_code,
                            duration_seconds: *duration_seconds,
                            timeout_hit: *timeout_hit,
                            timestamp,
                            agent: event.agent.clone(),
                        },
                    );
                }
            }
            _ => {}
        }
    }

    /// Compute overall derived metrics (not per-agent).
    pub fn compute_metrics(&self) -> DerivedMetrics {
        let mut metrics = DerivedMetrics::default();

        // Count events
        let mut container_started_count = 0u64;
        let mut container_exited_count = 0u64;
        let mut container_skipped_count = 0u64;
        
        let mut total_duration: u64 = 0;
        let mut exit_with_diff: HashMap<String, (i32, bool, u64)> = HashMap::new(); // run_id -> (exit_code, timeout_hit, duration)

        for event in &self.events {
            match &event.payload {
                EventData::ContainerStarted { .. } => {
                    container_started_count += 1;
                }
                EventData::ContainerExited { exit_code, duration_seconds, timeout_hit } => {
                    container_exited_count += 1;
                    total_duration += duration_seconds;
                    
                    if let Some(run_id) = &event.run_id {
                        exit_with_diff.insert(
                            run_id.clone(),
                            (*exit_code, *timeout_hit, *duration_seconds),
                        );
                    }
                }
                EventData::ContainerSkipped { .. } => {
                    container_skipped_count += 1;
                }
                _ => {}
            }
        }

        // Count git diff commits per run
        let mut run_commits: HashMap<String, u32> = HashMap::new();
        let mut run_insertions: HashMap<String, u32> = HashMap::new();
        let mut run_deletions: HashMap<String, u32> = HashMap::new();
        let mut run_files_changed: HashMap<String, u32> = HashMap::new();

        for event in &self.events {
            if let EventData::GitDiff { commit_count, total_insertions, total_deletions, total_files_changed, commits: _, .. } = &event.payload {
                if let Some(run_id) = &event.run_id {
                    run_commits.insert(run_id.clone(), *commit_count);
                    run_insertions.insert(run_id.clone(), *total_insertions);
                    run_deletions.insert(run_id.clone(), *total_deletions);
                    run_files_changed.insert(run_id.clone(), *total_files_changed);
                }
            }
        }

        // Compute throughput metrics
        metrics.throughput.agent_runs = container_started_count;
        
        // Productive runs = runs with commit_count > 0
        let mut productive_runs = 0u64;
        let mut total_commits = 0u64;
        let mut total_insertions = 0u64;
        let mut total_deletions = 0u64;
        let mut total_files = 0u64;

        for (run_id, exit_data) in &exit_with_diff {
            let commits = run_commits.get(run_id).copied().unwrap_or(0);
            if commits > 0 {
                productive_runs += 1;
                total_commits += commits as u64;
                total_insertions += run_insertions.get(run_id).copied().unwrap_or(0) as u64;
                total_deletions += run_deletions.get(run_id).copied().unwrap_or(0) as u64;
                total_files += run_files_changed.get(run_id).copied().unwrap_or(0) as u64;
            }
        }

        metrics.throughput.productive_runs = productive_runs;
        metrics.throughput.productive_run_rate = if container_started_count > 0 {
            productive_runs as f64 / container_started_count as f64
        } else {
            0.0
        };
        metrics.throughput.commits = total_commits;
        metrics.throughput.lines_inserted = total_insertions;
        metrics.throughput.lines_deleted = total_deletions;
        metrics.throughput.files_changed = total_files;

        // Average run duration
        metrics.throughput.avg_run_duration_seconds = if container_exited_count > 0 {
            total_duration as f64 / container_exited_count as f64
        } else {
            0.0
        };

        // Average commits per run
        metrics.throughput.avg_commits_per_run = if productive_runs > 0 {
            total_commits as f64 / productive_runs as f64
        } else {
            0.0
        };

        // Compute reliability metrics
        let mut container_failures = 0u64;
        let mut timeouts = 0u64;
        let mut empty_runs = 0u64;

        for (run_id, (exit_code, timeout_hit, _)) in &exit_with_diff {
            let commits = run_commits.get(run_id).copied().unwrap_or(0);
            
            if *exit_code != 0 {
                container_failures += 1;
            }
            
            if *timeout_hit {
                timeouts += 1;
            }
            
            // Empty run: exit_code == 0 AND commit_count == 0
            if *exit_code == 0 && commits == 0 {
                empty_runs += 1;
            }
        }

        metrics.reliability.container_failures = container_failures;
        metrics.reliability.failure_rate = if container_exited_count > 0 {
            container_failures as f64 / container_exited_count as f64
        } else {
            0.0
        };
        metrics.reliability.timeouts = timeouts;
        metrics.reliability.skipped_runs = container_skipped_count;
        metrics.reliability.empty_runs = empty_runs;

        // Scheduler uptime
        if let (Some(first), Some(last)) = (self.first_event_timestamp, self.last_event_timestamp) {
            metrics.reliability.scheduler_uptime_seconds = (last - first).num_seconds() as u64;
        }

        metrics
    }

    /// Compute derived metrics grouped by agent.
    pub fn compute_per_agent_metrics(&self) -> HashMap<String, DerivedMetrics> {
        let mut agent_metrics: HashMap<String, DerivedMetrics> = HashMap::new();

        // Group events by agent
        let mut agent_events: HashMap<String, Vec<&Event>> = HashMap::new();
        
        for event in &self.events {
            // Use run_agent_map to determine agent if not set in event
            let agent = if let Some(ref agent) = event.agent {
                agent.clone()
            } else if let Some(ref run_id) = event.run_id {
                self.run_agent_map.get(run_id).cloned().unwrap_or_else(|| "unknown".to_string())
            } else {
                "unknown".to_string()
            };
            agent_events.entry(agent).or_default().push(event);
        }

        // Compute metrics for each agent
        for (agent, events) in agent_events {
            let metrics = compute_metrics_for_events(events.as_slice());
            agent_metrics.insert(agent, metrics);
        }

        agent_metrics
    }
}

/// Compute metrics for a slice of events (used for per-agent breakdown).
fn compute_metrics_for_events(events: &[&Event]) -> DerivedMetrics {
    let mut metrics = DerivedMetrics::default();

    // Count events
    let mut container_started_count = 0u64;
    let mut container_exited_count = 0u64;
    let mut container_skipped_count = 0u64;
    
    let mut total_duration: u64 = 0;
    let mut exit_with_diff: HashMap<String, (i32, bool, u64)> = HashMap::new();

    let mut first_ts: Option<DateTime<Utc>> = None;
    let mut last_ts: Option<DateTime<Utc>> = None;

    for event in events {
        let timestamp = event.timestamp;
        if first_ts.is_none() {
            first_ts = Some(timestamp);
        }
        last_ts = Some(timestamp);

        match &event.payload {
            EventData::ContainerStarted { .. } => {
                container_started_count += 1;
            }
            EventData::ContainerExited { exit_code, duration_seconds, timeout_hit } => {
                container_exited_count += 1;
                total_duration += duration_seconds;
                
                if let Some(run_id) = &event.run_id {
                    exit_with_diff.insert(
                        run_id.clone(),
                        (*exit_code, *timeout_hit, *duration_seconds),
                    );
                }
            }
            EventData::ContainerSkipped { .. } => {
                container_skipped_count += 1;
            }
            _ => {}
        }
    }

    // Count git diff commits per run
    let mut run_commits: HashMap<String, u32> = HashMap::new();
    let mut run_insertions: HashMap<String, u32> = HashMap::new();
    let mut run_deletions: HashMap<String, u32> = HashMap::new();
    let mut run_files_changed: HashMap<String, u32> = HashMap::new();

    for event in events {
        if let EventData::GitDiff { commit_count, total_insertions, total_deletions, total_files_changed, .. } = &event.payload {
            if let Some(run_id) = &event.run_id {
                run_commits.insert(run_id.clone(), *commit_count);
                run_insertions.insert(run_id.clone(), *total_insertions);
                run_deletions.insert(run_id.clone(), *total_deletions);
                run_files_changed.insert(run_id.clone(), *total_files_changed);
            }
        }
    }

    // Compute throughput metrics
    metrics.throughput.agent_runs = container_started_count;
    
    let mut productive_runs = 0u64;
    let mut total_commits = 0u64;
    let mut total_insertions = 0u64;
    let mut total_deletions = 0u64;
    let mut total_files = 0u64;

    for (run_id, exit_data) in &exit_with_diff {
        let commits = run_commits.get(run_id).copied().unwrap_or(0);
        if commits > 0 {
            productive_runs += 1;
            total_commits += commits as u64;
            total_insertions += run_insertions.get(run_id).copied().unwrap_or(0) as u64;
            total_deletions += run_deletions.get(run_id).copied().unwrap_or(0) as u64;
            total_files += run_files_changed.get(run_id).copied().unwrap_or(0) as u64;
        }
    }

    metrics.throughput.productive_runs = productive_runs;
    metrics.throughput.productive_run_rate = if container_started_count > 0 {
        productive_runs as f64 / container_started_count as f64
    } else {
        0.0
    };
    metrics.throughput.commits = total_commits;
    metrics.throughput.lines_inserted = total_insertions;
    metrics.throughput.lines_deleted = total_deletions;
    metrics.throughput.files_changed = total_files;

    metrics.throughput.avg_run_duration_seconds = if container_exited_count > 0 {
        total_duration as f64 / container_exited_count as f64
    } else {
        0.0
    };

    metrics.throughput.avg_commits_per_run = if productive_runs > 0 {
        total_commits as f64 / productive_runs as f64
    } else {
        0.0
    };

    // Compute reliability metrics
    let mut container_failures = 0u64;
    let mut timeouts = 0u64;
    let mut empty_runs = 0u64;

    for (run_id, (exit_code, timeout_hit, _)) in &exit_with_diff {
        let commits = run_commits.get(run_id).copied().unwrap_or(0);
        
        if *exit_code != 0 {
            container_failures += 1;
        }
        
        if *timeout_hit {
            timeouts += 1;
        }
        
        if *exit_code == 0 && commits == 0 {
            empty_runs += 1;
        }
    }

    metrics.reliability.container_failures = container_failures;
    metrics.reliability.failure_rate = if container_exited_count > 0 {
        container_failures as f64 / container_exited_count as f64
    } else {
        0.0
    };
    metrics.reliability.timeouts = timeouts;
    metrics.reliability.skipped_runs = container_skipped_count;
    metrics.reliability.empty_runs = empty_runs;

    if let (Some(first), Some(last)) = (first_ts, last_ts) {
        metrics.reliability.scheduler_uptime_seconds = (last - first).num_seconds() as u64;
    }

    metrics
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use uuid::Uuid;

    /// Helper to create a container started event
    fn create_container_started(run_id: &str, agent: &str) -> Event {
        Event::with_context(
            EventType::ContainerStarted,
            EventData::container_started(
                "test-image",
                "cron",
                Some("* * * * *".to_string()),
                run_id,
            ),
            Some(run_id.to_string()),
            Some(agent.to_string()),
        )
    }

    /// Helper to create a container exited event
    fn create_container_exited(run_id: &str, exit_code: i32, duration: u64, timeout: bool) -> Event {
        Event::with_context(
            EventType::ContainerExited,
            EventData::container_exited(exit_code, duration, timeout),
            Some(run_id.to_string()),
            None, // agent will be inferred from run_agent_map
        )
    }

    /// Helper to create a git diff event
    fn create_git_diff(run_id: &str, commits: u32, insertions: u32, deletions: u32, files: u32) -> Event {
        Event::with_context(
            EventType::GitDiff,
            EventData::git_diff(vec![]),
            Some(run_id.to_string()),
            None,
        )
    }

    /// Helper to manually set git diff data (since we use the helper that computes from commits)
    fn create_git_diff_with_data(run_id: &str, commit_count: u32, insertions: u32, deletions: u32, files: u32) -> Event {
        Event {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: EventType::GitDiff,
            run_id: Some(run_id.to_string()),
            agent: None,
            payload: EventData::GitDiff {
                commit_count,
                commits: vec![],
                total_insertions: insertions,
                total_deletions: deletions,
                total_files_changed: files,
            },
        }
    }

    /// Helper to create a container skipped event
    fn create_container_skipped(reason: &str) -> Event {
        Event::new(
            EventType::ContainerSkipped,
            EventData::container_skipped(reason, None),
        )
    }

    #[test]
    fn test_empty_consumer_returns_zero_metrics() {
        let consumer = EventConsumer::new();
        let metrics = consumer.compute_metrics();

        assert_eq!(metrics.throughput.agent_runs, 0);
        assert_eq!(metrics.throughput.productive_runs, 0);
        assert_eq!(metrics.reliability.container_failures, 0);
        assert_eq!(metrics.reliability.skipped_runs, 0);
    }

    #[test]
    fn test_agent_runs_counted() {
        let mut consumer = EventConsumer::new();
        
        // Add container started events
        consumer.process_event(create_container_started("run1", "agent1"));
        consumer.process_event(create_container_started("run2", "agent1"));
        consumer.process_event(create_container_started("run3", "agent2"));

        let metrics = consumer.compute_metrics();
        
        assert_eq!(metrics.throughput.agent_runs, 3);
    }

    #[test]
    fn test_productive_runs_counted() {
        let mut consumer = EventConsumer::new();
        
        // Run 1: successful with commits
        consumer.process_event(create_container_started("run1", "agent1"));
        consumer.process_event(create_container_exited("run1", 0, 10, false));
        consumer.process_event(create_git_diff_with_data("run1", 2, 100, 50, 5));
        
        // Run 2: successful but no commits (empty run)
        consumer.process_event(create_container_started("run2", "agent1"));
        consumer.process_event(create_container_exited("run2", 0, 10, false));
        consumer.process_event(create_git_diff_with_data("run2", 0, 0, 0, 0));
        
        // Run 3: failed but had commits
        consumer.process_event(create_container_started("run3", "agent1"));
        consumer.process_event(create_container_exited("run3", 1, 10, false));
        consumer.process_event(create_git_diff_with_data("run3", 1, 10, 5, 2));

        let metrics = consumer.compute_metrics();
        
        // Productive = runs with commit_count > 0
        assert_eq!(metrics.throughput.productive_runs, 2);
        assert_eq!(metrics.throughput.commits, 3); // 2 + 0 + 1
    }

    #[test]
    fn test_failure_rate_computed() {
        let mut consumer = EventConsumer::new();
        
        // Run 1: success
        consumer.process_event(create_container_started("run1", "agent1"));
        consumer.process_event(create_container_exited("run1", 0, 10, false));
        
        // Run 2: failure (non-zero exit code)
        consumer.process_event(create_container_started("run2", "agent1"));
        consumer.process_event(create_container_exited("run2", 1, 10, false));
        
        // Run 3: timeout (not counted as container failure per spec - counted separately as timeout)
        consumer.process_event(create_container_started("run3", "agent1"));
        consumer.process_event(create_container_exited("run3", 0, 10, true));

        let metrics = consumer.compute_metrics();
        
        // Only run2 (exit_code != 0) is a container failure
        assert_eq!(metrics.reliability.container_failures, 1);
        // failure rate = failures / total exits = 1 / 3
        assert!((metrics.reliability.failure_rate - 1.0/3.0).abs() < 0.001);
    }

    #[test]
    fn test_timeouts_counted() {
        let mut consumer = EventConsumer::new();
        
        // Run 1: timeout
        consumer.process_event(create_container_started("run1", "agent1"));
        consumer.process_event(create_container_exited("run1", 0, 300, true));
        
        // Run 2: not timeout
        consumer.process_event(create_container_started("run2", "agent1"));
        consumer.process_event(create_container_exited("run2", 0, 10, false));

        let metrics = consumer.compute_metrics();
        
        assert_eq!(metrics.reliability.timeouts, 1);
    }

    #[test]
    fn test_skipped_runs_counted() {
        let mut consumer = EventConsumer::new();
        
        consumer.process_event(create_container_skipped("overlap_skip"));
        consumer.process_event(create_container_skipped("overlap_skip"));
        consumer.process_event(create_container_skipped("manual_skip"));

        let metrics = consumer.compute_metrics();
        
        assert_eq!(metrics.reliability.skipped_runs, 3);
    }

    #[test]
    fn test_empty_runs_counted() {
        let mut consumer = EventConsumer::new();
        
        // Empty run: exit_code == 0 AND commit_count == 0
        consumer.process_event(create_container_started("run1", "agent1"));
        consumer.process_event(create_container_exited("run1", 0, 10, false));
        consumer.process_event(create_git_diff_with_data("run1", 0, 0, 0, 0));
        
        // Non-empty run: exit_code == 0 AND commit_count > 0
        consumer.process_event(create_container_started("run2", "agent1"));
        consumer.process_event(create_container_exited("run2", 0, 10, false));
        consumer.process_event(create_git_diff_with_data("run2", 1, 10, 5, 1));

        let metrics = consumer.compute_metrics();
        
        assert_eq!(metrics.reliability.empty_runs, 1);
    }

    #[test]
    fn test_avg_run_duration_computed() {
        let mut consumer = EventConsumer::new();
        
        consumer.process_event(create_container_started("run1", "agent1"));
        consumer.process_event(create_container_exited("run1", 0, 10, false));
        
        consumer.process_event(create_container_started("run2", "agent1"));
        consumer.process_event(create_container_exited("run2", 0, 20, false));
        
        consumer.process_event(create_container_started("run3", "agent1"));
        consumer.process_event(create_container_exited("run3", 0, 30, false));

        let metrics = consumer.compute_metrics();
        
        // Average = (10 + 20 + 30) / 3 = 20
        assert!((metrics.throughput.avg_run_duration_seconds - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_lines_inserted_deleted_computed() {
        let mut consumer = EventConsumer::new();
        
        consumer.process_event(create_container_started("run1", "agent1"));
        consumer.process_event(create_container_exited("run1", 0, 10, false));
        consumer.process_event(create_git_diff_with_data("run1", 1, 100, 50, 5));
        
        consumer.process_event(create_container_started("run2", "agent1"));
        consumer.process_event(create_container_exited("run2", 0, 10, false));
        consumer.process_event(create_git_diff_with_data("run2", 2, 200, 100, 10));

        let metrics = consumer.compute_metrics();
        
        assert_eq!(metrics.throughput.lines_inserted, 300); // 100 + 200
        assert_eq!(metrics.throughput.lines_deleted, 150); // 50 + 100
        assert_eq!(metrics.throughput.files_changed, 15); // 5 + 10
    }

    #[test]
    fn test_productive_run_rate_computed() {
        let mut consumer = EventConsumer::new();
        
        // 2 productive runs, 3 total runs
        consumer.process_event(create_container_started("run1", "agent1"));
        consumer.process_event(create_container_exited("run1", 0, 10, false));
        consumer.process_event(create_git_diff_with_data("run1", 1, 10, 5, 1));
        
        consumer.process_event(create_container_started("run2", "agent1"));
        consumer.process_event(create_container_exited("run2", 0, 10, false));
        consumer.process_event(create_git_diff_with_data("run2", 1, 10, 5, 1));
        
        consumer.process_event(create_container_started("run3", "agent1"));
        consumer.process_event(create_container_exited("run3", 0, 10, false));
        consumer.process_event(create_git_diff_with_data("run3", 0, 0, 0, 0));

        let metrics = consumer.compute_metrics();
        
        assert_eq!(metrics.throughput.agent_runs, 3);
        assert_eq!(metrics.throughput.productive_runs, 2);
        // rate = 2 / 3 = 0.666...
        assert!((metrics.throughput.productive_run_rate - 2.0/3.0).abs() < 0.001);
    }

    #[test]
    fn test_per_agent_breakdown() {
        let mut consumer = EventConsumer::new();
        
        // Agent 1 events
        consumer.process_event(create_container_started("run1", "agent1"));
        consumer.process_event(create_container_exited("run1", 0, 10, false));
        consumer.process_event(create_git_diff_with_data("run1", 1, 10, 5, 1));
        
        // Agent 2 events
        consumer.process_event(create_container_started("run2", "agent2"));
        consumer.process_event(create_container_exited("run2", 1, 10, false)); // failed
        consumer.process_event(create_git_diff_with_data("run2", 0, 0, 0, 0));

        let per_agent = consumer.compute_per_agent_metrics();
        
        assert!(per_agent.contains_key("agent1"));
        assert!(per_agent.contains_key("agent2"));
        
        // Agent 1 should have productive run
        let agent1_metrics = &per_agent["agent1"];
        assert_eq!(agent1_metrics.throughput.productive_runs, 1);
        assert_eq!(agent1_metrics.throughput.commits, 1);
        
        // Agent 2 should have failure
        let agent2_metrics = &per_agent["agent2"];
        assert_eq!(agent2_metrics.reliability.container_failures, 1);
    }

    #[test]
    fn test_scheduler_uptime_computed() {
        let mut consumer = EventConsumer::new();
        
        // Create events with specific timestamps
        let start_time = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end_time = Utc.with_ymd_and_hms(2025, 1, 1, 1, 0, 0).unwrap(); // 1 hour later
        
        let mut event1 = create_container_started("run1", "agent1");
        event1.timestamp = start_time;
        consumer.process_event(event1);
        
        let mut event2 = create_container_exited("run1", 0, 10, false);
        event2.timestamp = end_time;
        consumer.process_event(event2);

        let metrics = consumer.compute_metrics();
        
        // Should be 3600 seconds (1 hour)
        assert_eq!(metrics.reliability.scheduler_uptime_seconds, 3600);
    }
}
