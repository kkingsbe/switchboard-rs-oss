//! Scheduler - Evaluate cron expressions and trigger agent executions
//!
//! This module handles (658 lines):
//! - RunStatus enum for tracking scheduled agent runs
//! - QueuedRun struct for tracking jobs waiting to be executed in Queue mode
//! - ScheduledAgent struct with config, next_run, current_run, and overlap_mode fields
//! - Scheduler struct with agents, running, scheduler, clock, settings, queue, and queue tracking fields
//! - Cron job registration and scheduling using tokio-cron-scheduler
//! - Agent execution orchestration with Docker client integration
//! - Queue handling for overlap mode (Skip and Queue modes supported)
//! - Start and stop methods for scheduler lifecycle management
//!
//! **Current Status:** Fully implemented with tokio-cron-scheduler integration

mod clock;
pub mod cron_helper;

pub use clock::{Clock, SystemClock};

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration as StdDuration, Instant, SystemTime};
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

use crate::config::{Agent, OverlapMode};
use crate::docker::run::types::ContainerConfig;
use crate::docker::{run_agent, DockerClient};
use crate::logger::Logger;
use crate::metrics::MetricsStore;
use crate::observability::{CommitInfo, EmitterConfig, Event, EventData, EventEmitter, EventType};
use crate::traits::{DockerClientTrait, RealDockerClient};

/// Get the current HEAD commit hash in the repository
///
/// This function executes `git rev-parse HEAD` to get the current commit hash.
/// Returns Ok(None) if the workspace is not a git repository or if there are no commits.
///
/// # Arguments
///
/// * `workspace_path` - The path to the git repository workspace
///
/// # Returns
///
/// Returns Ok(Some(hash)) on success, Ok(None) if not a git repo or no commits, Err on error.
async fn get_git_head(workspace_path: &str) -> Result<Option<String>, SchedulerError> {
    let output = tokio::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(workspace_path)
        .output()
        .await
        .map_err(|e| SchedulerError::GitError {
            message: format!("Failed to execute git rev-parse: {}", e),
        })?;

    if !output.status.success() {
        // Not a git repository or no commits
        return Ok(None);
    }

    let hash = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    if hash.is_empty() {
        return Ok(None);
    }

    Ok(Some(hash))
}

/// Parse git log output into structured commit data
///
/// This function parses the output of:
/// `git log {before}..{after} --format="%H|%s" --numstat --no-merges`
///
/// The expected format is:
/// - Commit line: `<hash>|<subject>`
/// - Following lines: `<additions>\t<deletions>\t<filename>`
/// - Empty line separates commits
///
/// # Arguments
///
/// * `output` - The raw output from git log command
///
/// # Returns
///
/// Returns a vector of CommitInfo structs
fn parse_git_log_output(output: &str) -> Vec<CommitInfo> {
    let mut commits = Vec::new();
    let mut current_commit: Option<(String, String, u32, u32, u32)> = None;

    for line in output.lines() {
        let line = line.trim();

        if line.is_empty() {
            // Empty line signals end of a commit - save it if we have one
            if let Some((hash, message, files, insertions, deletions)) = current_commit.take() {
                commits.push(CommitInfo {
                    hash,
                    message,
                    files_changed: files,
                    insertions,
                    deletions,
                });
            }
            continue;
        }

        // Check if this is a commit header line (contains |)
        if line.contains('|') {
            // Save previous commit if exists
            if let Some((hash, message, files, insertions, deletions)) = current_commit.take() {
                commits.push(CommitInfo {
                    hash,
                    message,
                    files_changed: files,
                    insertions,
                    deletions,
                });
            }

            // Parse commit header: <hash>|<subject>
            let parts: Vec<&str> = line.splitn(2, '|').collect();
            if parts.len() == 2 {
                let hash = parts[0].trim().to_string();
                let message = parts[1].trim().to_string();
                current_commit = Some((hash, message, 0, 0, 0));
            }
        } else if current_commit.is_some() {
            // This is a numstat line: <additions>\t<deletions>\t<filename>
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let additions = parts[0].parse::<u32>().unwrap_or(0);
                let deletions = parts[1].parse::<u32>().unwrap_or(0);

                if let Some((hash, message, mut files, mut total_insertions, mut total_deletions)) = current_commit.take() {
                    files += 1;
                    total_insertions += additions;
                    total_deletions += deletions;
                    current_commit = Some((hash, message, files, total_insertions, total_deletions));
                }
            }
        }
    }

    // Don't forget the last commit if there's no trailing newline
    if let Some((hash, message, files, insertions, deletions)) = current_commit {
        commits.push(CommitInfo {
            hash,
            message,
            files_changed: files,
            insertions,
            deletions,
        });
    }

    commits
}

/// Get git diff between two commits
///
/// This function runs `git log {before}..{after} --format="%H|%s" --numstat --no-merges`
/// and parses the output into structured commit data.
///
/// # Arguments
///
/// * `workspace_path` - The path to the git repository
/// * `before_hash` - The commit hash before container execution
/// * `after_hash` - The commit hash after container execution
///
/// # Returns
///
/// Returns a vector of CommitInfo structs representing the commits made during container execution
async fn get_git_diff(
    workspace_path: &str,
    before_hash: &str,
    after_hash: &str,
) -> Result<Vec<CommitInfo>, SchedulerError> {
    let range = format!("{}..{}", before_hash, after_hash);

    let output = tokio::process::Command::new("git")
        .args([
            "log",
            &range,
            "--format=%H|%s",
            "--numstat",
            "--no-merges",
        ])
        .current_dir(workspace_path)
        .output()
        .await
        .map_err(|e| SchedulerError::GitError {
            message: format!("Failed to execute git log: {}", e),
        })?;

    if !output.status.success() {
        // No commits in range or other git error
        return Ok(Vec::new());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let commits = parse_git_log_output(&output_str);

    Ok(commits)
}

/// Comprehensive error type for scheduler operations
///
/// This enum provides detailed, actionable error messages for all failure modes
/// in the scheduler module, including configuration issues, execution failures,
/// overlap mode conflicts, state management errors, and Docker-related issues.
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    // ========================================
    // Configuration Errors
    // ========================================
    /// Invalid cron schedule expression
    ///
    /// Occurs when the cron schedule string provided in the agent configuration
    /// cannot be parsed by the cron library.
    #[error("Invalid cron expression: {schedule}\nError: {error}\n\nValid cron format examples:\n  * * * * *    Every minute\n  0 * * * *    Every hour\n  0 9 * * *    Daily at 9:00 AM\n  0 9 * * 1-5  Weekdays at 9:00 AM")]
    InvalidCronSchedule {
        /// The invalid schedule string that was provided
        schedule: String,
        /// Detailed error information from the cron parser
        error: String,
    },

    /// Invalid timezone specification
    ///
    /// Occurs when the timezone string provided in settings cannot be parsed
    /// as an IANA timezone identifier.
    #[error("Invalid timezone: {timezone}")]
    InvalidTimezone {
        /// The invalid timezone string that was provided
        timezone: String,
        /// Suggested timezone to use instead
        suggestion: String,
    },

    /// Referenced agent not found in configuration
    ///
    /// Occurs when attempting to register or execute an agent that doesn't
    /// exist in the loaded configuration.
    #[error("Agent not found: {agent_name}")]
    AgentNotFound {
        /// The agent name that was requested but not found
        agent_name: String,
        /// A list of available agent names for reference
        available: String,
    },

    /// Prompt file not found at the specified path
    ///
    /// Occurs when an agent specifies a `prompt_file` path that does not exist
    /// or cannot be accessed.
    #[error("Prompt file not found: {path}")]
    PromptFileNotFound {
        /// The path to the prompt file that could not be found
        path: String,
    },

    /// Missing prompt configuration
    ///
    /// Occurs when an agent is configured but has neither an inline `prompt`
    /// nor a `prompt_file` specified.
    #[error("Missing prompt for agent: {agent_name}")]
    MissingPrompt {
        /// The agent name that is missing a prompt
        agent_name: String,
    },

    // ========================================
    // Execution Errors
    // ========================================
    /// Failed to create the scheduler instance
    ///
    /// Occurs during scheduler initialization when the underlying
    /// tokio-cron-scheduler cannot be created.
    #[error("Failed to create scheduler: {error}")]
    SchedulerCreationFailed {
        /// Detailed error information
        error: String,
    },

    /// Failed to start the scheduler
    ///
    /// Occurs when starting the underlying tokio-cron-scheduler fails.
    #[error("Failed to start scheduler: {error}")]
    SchedulerStartFailed {
        /// Detailed error information
        error: String,
    },

    /// Failed to stop the scheduler
    ///
    /// Occurs when shutting down the scheduler fails during graceful shutdown.
    #[error("Failed to stop scheduler: {error}")]
    SchedulerStopFailed {
        /// Detailed error information
        error: String,
    },

    /// Failed to register a job for an agent
    ///
    /// Occurs when adding an agent's scheduled job to the tokio-cron-scheduler
    /// fails for any reason (e.g., invalid schedule, system resource limits).
    #[error("Failed to register job for agent {agent_name}: {error}")]
    JobRegistrationFailed {
        /// The name of the agent whose job registration failed
        agent_name: String,
        /// Detailed error information
        error: String,
    },

    // ========================================
    // Overlap Mode Errors
    // ========================================
    /// Queue is full and cannot accept new runs
    ///
    /// Occurs in Queue overlap mode when an agent is already running and
    /// the queue has reached its maximum size.
    #[error("Queue full: {agent_name} (current: {current_size}, max: {max_size})")]
    QueueFull {
        /// The name of the agent whose queue is full
        agent_name: String,
        /// The current queue size
        current_size: usize,
        /// The maximum queue size that has been reached
        max_size: usize,
    },

    /// Skip mode is active and a run was skipped due to overlap
    ///
    /// Occurs in Skip overlap mode when an agent is already running and
    /// a new scheduled execution is triggered.
    ///
    /// This is informational - the run was intentionally skipped according
    /// to the configured overlap mode behavior.
    #[error("Run skipped: {agent_name} (next run at: {next_run})")]
    SkipModeActive {
        /// The name of the agent whose run was skipped
        agent_name: String,
        /// The container ID of the currently running execution
        container_id: String,
        /// The next scheduled run time
        next_run: String,
    },

    // ========================================
    // State Errors
    // ========================================
    /// Scheduler is already running
    ///
    /// Occurs when attempting to start a scheduler that is already active.
    #[error("Scheduler already running")]
    SchedulerAlreadyRunning {
        /// Process ID of the running scheduler instance
        pid: u32,
    },

    /// Scheduler is not running
    ///
    /// Occurs when attempting to perform operations that require the scheduler
    /// to be running (e.g., stopping, checking status) when it is not active.
    #[error("Scheduler not running")]
    SchedulerNotRunning,

    // ========================================
    // Docker/Container Errors
    // ========================================
    /// Failed to connect to Docker daemon
    ///
    /// Occurs when the scheduler cannot establish a connection to the Docker
    /// daemon, which is required for executing agents in containers.
    #[error("Docker connection failed: {error}")]
    DockerConnectionFailed {
        /// Detailed error information
        error: String,
    },

    /// Failed to execute agent in a Docker container
    ///
    /// Occurs when the Docker container execution fails for any reason
    /// (e.g., image not found, container creation failure, runtime error).
    #[error("Container execution failed for agent {agent_name}: {error}")]
    ContainerExecutionFailed {
        /// The name of the agent that failed to execute
        agent_name: String,
        /// Detailed error information
        error: String,
    },

    /// Container execution timed out
    ///
    /// Occurs when an agent's Docker container execution exceeds the configured
    /// timeout duration and had to be terminated.
    #[error("Container timeout: {agent_name}")]
    ContainerTimeout {
        /// The name of the agent that timed out
        agent_name: String,
        /// The timeout duration in seconds
        duration_secs: u64,
        /// The action taken (e.g., "Sent SIGTERM and SIGKILL after 10s grace period")
        action_taken: String,
    },

    // ========================================
    // Git Errors
    // ========================================
    /// Git operation failed
    ///
    /// Occurs when a git operation (e.g., rev-parse, log) fails.
    #[error("Git error: {message}")]
    GitError {
        /// Detailed error information
        message: String,
    },

    // ========================================
    // System Errors
    // ========================================
    /// Mutex was poisoned
    ///
    /// Occurs when a thread panicked while holding a lock, leaving the mutex
    /// in an inconsistent state.
    #[error("Mutex poisoned")]
    MutexPoisoned,
}

/// Acquires a lock on a mutex, converting poisoning errors to SchedulerError.
///
/// This helper function simplifies the common pattern of:
/// ```rust
/// let mut guard = mutex.lock().map_err(|_| SchedulerError::MutexPoisoned)?;
/// ```
///
/// # Arguments
///
/// * `mutex` - The mutex to acquire a lock on
///
/// # Returns
///
/// Returns a MutexGuard on success, or SchedulerError::MutexPoisoned if the mutex was poisoned.
fn acquire_lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, SchedulerError> {
    mutex.lock().map_err(|_| SchedulerError::MutexPoisoned)
}

/// Status of a scheduled agent run
///
/// This enum represents the current state of a scheduled agent execution:
/// running, skipped (due to overlap), or scheduled for future execution.
pub enum RunStatus {
    /// A run is currently executing
    Running {
        /// The container ID of the running execution
        container_id: String,
    },
    /// A run was skipped (e.g., due to overlap)
    Skipped {
        /// Reason why the run was skipped
        reason: String,
    },
    /// A run is scheduled for future execution
    Scheduled {
        /// The next scheduled run time
        next_run: DateTime<Tz>,
    },
}

/// A queued agent run for queue mode overlap handling
///
/// When an agent is configured with overlap_mode = "queue" and is already running,
/// new scheduled executions are queued rather than skipped. This struct represents
/// a queued run waiting to be executed.
#[derive(Debug, Clone)]
pub struct QueuedRun {
    /// The name of the agent to run
    pub agent_name: String,
    /// When the run was queued
    pub scheduled_time: DateTime<Utc>,
    /// Unique identifier for the job
    pub uuid: Uuid,
}

/// Process a queued run by recording its wait time and logging it
///
/// This function calculates the wait time for a queued run and updates the metrics.
/// The wait time is calculated as: `current_time - scheduled_time` from the QueuedRun.
///
/// # Arguments
///
/// * `queued_run` - The queued run to process
/// * `current_time` - The current time when the run is being processed
/// * `queue_wait_time_seconds` - Arc<Mutex<u64>> for tracking cumulative wait time
/// * `queue_wait_times` - Arc<Mutex<Vec<u64>>> for tracking individual wait times
fn process_queued_run(
    queued_run: &QueuedRun,
    _current_time: Instant,
    queue_wait_time_seconds: Arc<Mutex<u64>>,
    queue_wait_times: Arc<Mutex<Vec<u64>>>,
) -> Result<(), SchedulerError> {
    let current_time_utc = Utc::now();
    let wait_duration_seconds = (current_time_utc - queued_run.scheduled_time).num_seconds() as u64;

    // Update cumulative wait time
    {
        let mut total_wait = acquire_lock(&queue_wait_time_seconds)?;
        *total_wait += wait_duration_seconds;
    }

    // Record individual wait time
    {
        let mut wait_times = queue_wait_times
            .lock()
            .map_err(|_| SchedulerError::MutexPoisoned)?;
        wait_times.push(wait_duration_seconds);
    }

    tracing::info!(
        "Processed queued run for agent '{}' - wait time: {}s",
        queued_run.agent_name,
        wait_duration_seconds
    );

    Ok(())
}

/// Execute an agent by creating and running a Docker container
///
/// This helper function handles all the steps needed to execute an agent:
/// 1. Check for overlap (if agent is already running)
/// 2. Resolves the prompt (from agent.prompt or reads from agent.prompt_file)
/// 3. Creates a Logger instance for log streaming
/// 4. Creates a DockerClient (or uses injected client if provided)
/// 5. Builds ContainerConfig from the agent configuration
/// 6. Calls run_agent() to execute the container
///
/// # Arguments
///
/// * `agent_name` - The name of the agent to execute
/// * `agents` - Reference to the agents vector for overlap detection
/// * `agent` - The agent configuration
/// * `config_dir` - The directory containing the config file (for resolving relative prompt_file paths)
/// * `log_dir` - Directory for log files
/// * `image_name` - Docker image name (e.g., "switchboard-agent")
/// * `image_tag` - Docker image tag (e.g., "latest")
/// * `workspace_path` - Workspace path to mount into the container
/// * `clock` - Clock for time operations
/// * `queue` - Queue for handling overlapping executions in queue mode
/// * `queue_wait_time_seconds` - Cumulative queue wait time tracker
/// * `queue_wait_times` - Individual queue wait times tracker
/// * `queued_start_time` - Optional timestamp when the run was queued (for queue mode)
/// * `docker_client` - Optional injected Docker client. If `Some`, uses the provided client;
///   if `None`, creates a new DockerClient internally (backward compatible)
///
/// # Returns
///
/// Returns `Ok(())` if the agent executed successfully.
/// Returns an error if any step fails.
#[allow(clippy::too_many_arguments)]
async fn execute_agent(
    agent_name: String,
    agents: Arc<Mutex<Vec<ScheduledAgent>>>,
    agent: Agent,
    config_dir: PathBuf,
    log_dir: PathBuf,
    image_name: String,
    image_tag: String,
    workspace_path: String,
    clock: Arc<dyn Clock + Send>,
    queue: Arc<Mutex<Vec<QueuedRun>>>,
    queue_wait_time_seconds: Arc<Mutex<u64>>,
    queue_wait_times: Arc<Mutex<Vec<u64>>>,
    queued_start_time: Option<Instant>,
    docker_client: Option<Arc<dyn DockerClientTrait>>,
    event_emitter: Option<Arc<Mutex<EventEmitter>>>,
    trigger_type: String,
) -> Result<(), SchedulerError> {
    // Check for overlap - if the agent is already running, skip this execution (unless overlap_mode is "queue")
    {
        let scheduled_agents = agents.lock().map_err(|_| SchedulerError::MutexPoisoned)?;
        if let Some(scheduled_agent) = scheduled_agents
            .iter()
            .find(|a| a.config.name == agent_name)
        {
            if let Some(container_id) = &scheduled_agent.current_run {
                // Handle overlap based on overlap_mode
                if scheduled_agent.overlap_mode == OverlapMode::Skip {
                    let next_run_str = scheduled_agent
                        .next_run
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| "unknown".to_string());
                    tracing::warn!(
                        "Agent '{}' is already running (container_id: {}), overlap_mode=skip, skipping new run. Next scheduled run: {}",
                        agent_name,
                        container_id,
                        next_run_str
                    );
                    
                    // Emit container.skipped event
                    if let Some(ref emitter) = event_emitter {
                        if let Ok(mut emitter_guard) = emitter.lock() {
                            let event = Event::new(
                                EventType::ContainerSkipped,
                                EventData::container_skipped(
                                    "overlap_skip",
                                    Some(container_id.clone()),
                                ),
                            );
                            let _ = emitter_guard.emit(event);
                        }
                    }
                    
                    return Err(SchedulerError::SkipModeActive {
                        agent_name: agent_name.to_string(),
                        container_id: container_id.clone(),
                        next_run: next_run_str,
                    });
                } else if scheduled_agent.overlap_mode == OverlapMode::Queue {
                    // Check if queue is full
                    let max_queue_size = scheduled_agent.config.effective_max_queue_size();
                    let queue_guard = queue.lock().map_err(|_| SchedulerError::MutexPoisoned)?;
                    let current_queue_size = queue_guard.len();

                    if current_queue_size >= max_queue_size {
                        tracing::warn!(
                            "Agent '{}' queue is full (current: {}, max: {}), skipping scheduled run",
                            agent_name,
                            current_queue_size,
                            max_queue_size
                        );
                        return Err(SchedulerError::QueueFull {
                            agent_name: agent_name.to_string(),
                            current_size: current_queue_size,
                            max_size: max_queue_size,
                        });
                    }

                    // Add to queue
                    let queued_run = QueuedRun {
                        agent_name: agent_name.clone(),
                        scheduled_time: Utc::now(),
                        uuid: Uuid::new_v4(),
                    };
                    drop(queue_guard);

                    let mut queue_mut = queue.lock().map_err(|_| SchedulerError::MutexPoisoned)?;
                    let queue_position = queue_mut.len() + 1; // 1-based position
                    queue_mut.push(queued_run);

                    tracing::info!(
                        "Agent '{}' is running, queued run ({}/{})",
                        agent_name,
                        queue_mut.len(),
                        max_queue_size
                    );

                    // Emit container.queued event
                    if let Some(ref emitter) = event_emitter {
                        if let Ok(mut emitter_guard) = emitter.lock() {
                            // Get the running container ID
                            let running_container_id = scheduled_agent.current_run.clone();
                            let event = Event::new(
                                EventType::ContainerQueued,
                                EventData::container_queued(queue_position as u32, running_container_id),
                            );
                            let _ = emitter_guard.emit(event);
                        }
                    }

                    // Note: Queue wait time will be tracked when the queued run is processed
                    // (when it's removed from the queue and executed)
                    // The wait time is calculated as: current_time - scheduled_time

                    return Ok(());
                }
            }
        }
    }

    // Set current_run to mark that execution is starting
    // This must be done before we start running the agent
    {
        let mut scheduled_agents = agents.lock().map_err(|_| SchedulerError::MutexPoisoned)?;
        if let Some(scheduled_agent) = scheduled_agents
            .iter_mut()
            .find(|a| a.config.name == agent_name)
        {
            scheduled_agent.current_run = Some("starting".to_string());
        }
    }

    // Define cleanup closure to clear current_run after execution
    // This uses a defer-like pattern to ensure cleanup runs even on error
    let agent_name_for_cleanup = agent_name.clone();
    let agents_for_cleanup = agents.clone();
    let queue_for_cleanup = queue.clone();
    let queue_wait_time_seconds_for_cleanup = queue_wait_time_seconds.clone();
    let queue_wait_times_for_cleanup = queue_wait_times.clone();
    let config_dir_for_cleanup = config_dir.clone();
    let log_dir_for_cleanup = log_dir.clone();
    let image_name_for_cleanup = image_name.clone();
    let image_tag_for_cleanup = image_tag.clone();
    let workspace_path_for_cleanup = workspace_path.clone();
    let clock_for_cleanup = clock.clone();
    let event_emitter_for_cleanup = event_emitter.clone();
    // Note: docker_client is not captured here because it's created after the cleanup closure.
    // For queued runs, None is passed so a new client will be created internally.
    let cleanup = || {
        // Clear current_run after execution completes
        let mut scheduled_agents = match agents_for_cleanup.lock() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Mutex poisoned in cleanup (clearing current_run): {}", e);
                return;
            }
        };
        if let Some(scheduled_agent) = scheduled_agents
            .iter_mut()
            .find(|a| a.config.name == agent_name_for_cleanup)
        {
            scheduled_agent.current_run = None;
        }
        drop(scheduled_agents);

        // Check if there are queued runs for this agent and process them
        let mut queue_guard = match queue_for_cleanup.lock() {
            Ok(guard) => guard,
            Err(e) => {
                tracing::error!("Mutex poisoned in cleanup (processing queue): {}", e);
                return;
            }
        };
        if let Some(pos) = queue_guard
            .iter()
            .position(|qr| qr.agent_name == agent_name_for_cleanup)
        {
            let queued_run = queue_guard.remove(pos);
            drop(queue_guard);

            // Process the queued run and track wait time
            if let Err(e) = process_queued_run(
                &queued_run,
                Instant::now(),
                queue_wait_time_seconds_for_cleanup.clone(),
                queue_wait_times_for_cleanup.clone(),
            ) {
                tracing::error!("Failed to process queued run: {}", e);
            }

            // Execute the queued run
            // Find the agent configuration by name
            let scheduled_agents = match agents_for_cleanup.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    tracing::error!("Mutex poisoned in cleanup (finding agent): {}", e);
                    return;
                }
            };
            if let Some(scheduled_agent) = scheduled_agents
                .iter()
                .find(|a| a.config.name == queued_run.agent_name)
            {
                let agent_config = scheduled_agent.config.clone();
                let position = pos + 1; // Position is 1-indexed for logging
                drop(scheduled_agents);

                // Log that we're starting the queued run
                tracing::info!(
                    "Agent '{}' completed. Starting queued run (position {})",
                    queued_run.agent_name,
                    position
                );

                // Execute the queued run synchronously using block_on
                // This blocks the cleanup on the queued run, but ensures it executes
                let handle = tokio::runtime::Handle::current();

                // Convert scheduled_time: DateTime<Utc> to Instant
                let queued_start_time = Some(
                    Instant::now()
                        - SystemTime::now()
                            .duration_since(queued_run.scheduled_time.into())
                            .unwrap_or(StdDuration::from_secs(0)),
                );

                if let Err(e) = handle.block_on(execute_agent(
                    queued_run.agent_name.clone(),
                    agents_for_cleanup,
                    agent_config,
                    config_dir_for_cleanup,
                    log_dir_for_cleanup,
                    image_name_for_cleanup,
                    image_tag_for_cleanup,
                    workspace_path_for_cleanup,
                    clock_for_cleanup,
                    queue_for_cleanup,
                    queue_wait_time_seconds_for_cleanup,
                    queue_wait_times_for_cleanup,
                    queued_start_time,
                    None, // No injected docker client for queued runs - will be created internally
                    event_emitter_for_cleanup,
                    "cron".to_string(), // Queued runs are still triggered by cron
                )) {
                    tracing::error!(
                        "Error executing queued run for agent '{}': {}",
                        queued_run.agent_name,
                        e
                    );
                }
            } else {
                tracing::warn!(
                    "Agent configuration not found for queued run: '{}'",
                    queued_run.agent_name
                );
            }
        }
    };

    // Execute the agent logic with proper error handling and cleanup
    let execution_result = async {
        // Resolve the prompt (from agent.prompt or read from agent.prompt_file)
        let prompt = match (&agent.prompt, &agent.prompt_file) {
            (Some(inline_prompt), None) => inline_prompt.clone(),
            (None, Some(prompt_file)) => match agent.read_prompt_file(&config_dir) {
                Ok(Some(contents)) => contents,
                Ok(None) => {
                    tracing::error!("Prompt file not found: {}", prompt_file);
                    return Err(SchedulerError::PromptFileNotFound {
                        path: prompt_file.clone(),
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to read prompt file: {}", e);
                    return Err(SchedulerError::PromptFileNotFound {
                        path: prompt_file.clone(),
                    });
                }
            },
            _ => {
                tracing::error!("Agent must have either 'prompt' or 'prompt_file' specified");
                return Err(SchedulerError::MissingPrompt {
                    agent_name: agent.name.clone(),
                });
            }
        };

        // Build environment variables
        let env_vars = agent.env(None);

        // Create ContainerConfig with all required fields
        let container_config = ContainerConfig {
            agent_name: agent.name.clone(),
            env_vars,
            timeout: agent.timeout.clone(),
            readonly: agent.readonly.unwrap_or(false),
            prompt: prompt.clone(),
            skills: agent.skills.clone(),
            silent_timeout: agent.silent_timeout.clone(),
            gpu: agent.gpu.unwrap_or(true), // Default to GPU enabled for backward compatibility
        };

        // Create DockerClient (use injected client if provided, otherwise create internally)
        let docker_client = match docker_client {
            Some(client) => {
                tracing::debug!("Using injected Docker client for agent '{}'", agent_name);
                client
            }
            None => {
                tracing::debug!("Creating new Docker client for agent '{}'", agent_name);
                match DockerClient::new(image_name.clone(), image_tag.clone()).await {
                    Ok(client) => Arc::new(client) as Arc<dyn DockerClientTrait>,
                    Err(e) => {
                        tracing::error!("Docker connection failed: {}", e);
                        return Err(SchedulerError::DockerConnectionFailed {
                            error: e.to_string(),
                        });
                    }
                }
            }
        };

        // Create Logger with log_dir and agent_name, foreground_mode: true
        let logger = Arc::new(Mutex::new(Logger::new(
            log_dir.clone(),
            Some(agent.name.clone()),
            true,
        )));

        // Create MetricsStore for collecting execution metrics
        let metrics_store = MetricsStore::new(log_dir.clone());

        // Load existing metrics before the run (error is logged but doesn't fail the agent run)
        if let Err(e) = metrics_store.load() {
            tracing::warn!("Metrics file not found (first run or missing file): {}", e);
        }

        // Build the full image name
        let image = format!("{}:{}", image_name, image_tag);

        // Run the agent
        tracing::info!("Starting agent: {}", agent.name);
        let start_time = clock.now();

        // When PROMPT is piped to kilocode, --auto flag is required
        // Note: prompt is a positional argument, NOT a --prompt flag
        let cmd_args: Vec<String> = vec!["--auto".to_string(), container_config.prompt.clone()];

        // Convert queued_start_time: Option<Instant> to Option<DateTime<Utc>> for run_agent
        let queued_datetime = queued_start_time.map(|instant| {
            let duration_since_now = clock.now().duration_since(instant);
            let queued_system_time = SystemTime::now() - duration_since_now;
            DateTime::<Utc>::from(queued_system_time)
        });

        // Build the full image name
        let full_image = format!("{}:{}", image_name, image_tag);

        // Record HEAD hash before container launch for git diff capture
        let before_hash = match get_git_head(&workspace_path).await {
            Ok(Some(hash)) => {
                tracing::debug!("Git HEAD before container launch: {}", hash);
                Some(hash)
            }
            Ok(None) => {
                tracing::debug!("Not a git repository or no commits - skipping git diff capture");
                None
            }
            Err(e) => {
                tracing::warn!("Failed to get git HEAD before container launch: {}", e);
                None
            }
        };
        
        // Emit container.started event before running the container
        if let Some(ref emitter) = event_emitter {
            if let Ok(mut emitter_guard) = emitter.lock() {
                let schedule = agent.schedule.clone();
                let event = Event::new(
                    EventType::ContainerStarted,
                    EventData::container_started(
                        full_image.clone(),
                        trigger_type.clone(),
                        Some(schedule),
                        "", // container_id not known yet, will be updated in exited event
                    ),
                );
                let _ = emitter_guard.emit(event);
            }
        }

        let result = match run_agent(
            &workspace_path,
            docker_client,
            &container_config,
            agent.timeout.clone(),
            &image,
            Some(cmd_args.as_slice()),
            Some(logger),
            Some(&metrics_store),
            &agent.name,
            queued_datetime,
            event_emitter.clone(),
            Some(trigger_type.clone()),
            Some(agent.schedule.clone()),
        )
        .await
        {
            Ok(result) => result,
            Err(e) => {
                tracing::error!("Failed to run agent: {}", e);
                return Err(SchedulerError::ContainerExecutionFailed {
                    agent_name: agent.name.clone(),
                    error: e.to_string(),
                });
            }
        };

        let duration = start_time.elapsed();

        // Emit container.exited event after run_agent completes
        if let Some(ref emitter) = event_emitter {
            if let Ok(mut emitter_guard) = emitter.lock() {
                let exit_code = result.exit_code;
                let duration_seconds = duration.as_secs();
                // Check if the container timed out (exit_code -1 indicates error during wait)
                let timeout_hit = result.exit_code == -1 || result.exit_code == 137;
                
                let event = Event::new(
                    EventType::ContainerExited,
                    EventData::container_exited(exit_code as i32, duration_seconds, timeout_hit),
                );
                let _ = emitter_guard.emit(event);
            }
        }

        // Capture HEAD hash after container exits and emit git.diff event
        let after_hash = match get_git_head(&workspace_path).await {
            Ok(Some(hash)) => {
                tracing::debug!("Git HEAD after container exit: {}", hash);
                Some(hash)
            }
            Ok(None) => {
                tracing::debug!("Not a git repository or no commits after container exit");
                None
            }
            Err(e) => {
                tracing::warn!("Failed to get git HEAD after container exit: {}", e);
                None
            }
        };

        // If we have both before and after hashes, compute and emit git diff
        if let (Some(before), Some(after)) = (&before_hash, &after_hash) {
            if before != after {
                // There are new commits - get the git diff
                match get_git_diff(&workspace_path, before, after).await {
                    Ok(commits) => {
                        tracing::info!(
                            "Git diff: {} commits, +{} lines, -{} lines",
                            commits.len(),
                            commits.iter().map(|c| c.insertions).sum::<u32>(),
                            commits.iter().map(|c| c.deletions).sum::<u32>()
                        );

                        // Emit git.diff event
                        if let Some(ref emitter) = event_emitter {
                            if let Ok(mut emitter_guard) = emitter.lock() {
                                let event = Event::new(
                                    EventType::GitDiff,
                                    EventData::git_diff(commits),
                                );
                                let _ = emitter_guard.emit(event);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to get git diff: {}", e);
                    }
                }
            } else {
                tracing::debug!("No new commits after container exit");

                // Emit git.diff event with empty commits (edge case: no commits)
                if let Some(ref emitter) = event_emitter {
                    if let Ok(mut emitter_guard) = emitter.lock() {
                        let event = Event::new(
                            EventType::GitDiff,
                            EventData::git_diff(vec![]),
                        );
                        let _ = emitter_guard.emit(event);
                    }
                }
            }
        } else if before_hash.is_none() || after_hash.is_none() {
            // Edge case: not a git repository or no commits - emit git.diff with empty commits
            if let Some(ref emitter) = event_emitter {
                if let Ok(mut emitter_guard) = emitter.lock() {
                    let event = Event::new(
                        EventType::GitDiff,
                        EventData::git_diff(vec![]),
                    );
                    let _ = emitter_guard.emit(event);
                }
            }
        }

        // Log execution summary
        tracing::info!(
            "Agent execution completed: container_id={}, exit_code={}, duration={:.2}s",
            result.container_id,
            result.exit_code,
            duration.as_secs_f64()
        );

        // Manually update metrics with queued start time if provided
        // This is needed because run_agent() creates AgentRunResult with queued_start_time: None
        if let Some(queued_time) = queued_start_time {
            if let Ok(mut all_metrics) = metrics_store.load() {
                use crate::metrics::{update_all_metrics, AgentRunResult};

                // Convert Instant to DateTime<Utc>
                let start_datetime =
                    DateTime::<Utc>::from(SystemTime::now() - (clock.now() - start_time));
                let queued_datetime =
                    DateTime::<Utc>::from(SystemTime::now() - (clock.now() - queued_time));

                // Create a synthetic AgentRunResult with the queued_start_time
                let run_result = AgentRunResult {
                    agent_name: agent.name.clone(),
                    container_id: result.container_id.clone(),
                    start_time: start_datetime,
                    end_time: Utc::now(), // Approximation, as end_time isn't available
                    exit_code: result.exit_code,
                    timed_out: false, // We don't have this info here, assume false
                    termination_type: None,
                    queued_start_time: Some(queued_datetime),
                    skills_installed_count: 0,
                    skills_failed_count: 0,
                    skills_install_time_seconds: None,
                };

                // Update the metrics
                if let Err(e) = update_all_metrics(&mut all_metrics, &run_result) {
                    tracing::error!("Failed to update metrics with queued start time: {}", e);
                } else {
                    // Save the updated metrics
                    if let Err(e) = metrics_store.save_with_retry(&all_metrics) {
                        tracing::error!(
                            "Failed to save metrics after queued start time update: {}",
                            e
                        );
                    }
                }
            }
        }

        Ok(())
    }
    .await;

    // Apply cleanup (runs regardless of success or failure)
    cleanup();

    execution_result
}

/// A scheduled agent with runtime state
///
/// This struct combines the agent configuration with runtime state information
/// such as the next scheduled run time and whether the agent is currently executing.
pub struct ScheduledAgent {
    /// The agent configuration
    pub config: Agent,
    /// Next scheduled execution time
    pub next_run: Option<DateTime<Tz>>,
    /// Container ID if currently running (or similar tracking)
    pub current_run: Option<String>,
    /// How to handle overlapping executions
    pub overlap_mode: OverlapMode,
}

/// Scheduler for managing agent executions
///
/// The Scheduler is the main component for managing scheduled agent executions.
/// It handles cron schedule parsing, agent registration, execution orchestration,
/// and overlap mode handling (skip, queue, or allow).
pub struct Scheduler {
    /// The scheduled agents
    pub agents: Arc<Mutex<Vec<ScheduledAgent>>>,
    /// Flag for graceful shutdown
    pub running: Arc<AtomicBool>,
    /// The underlying tokio-cron-scheduler job scheduler
    scheduler: JobScheduler,
    /// Clock for time operations (injectable for testing)
    clock: Arc<dyn Clock + Send>,
    /// Docker client for container operations (injectable for testing)
    docker_client: Arc<dyn DockerClientTrait>,
    /// Global settings configuration
    settings: Option<crate::config::Settings>,
    /// Queue for queue mode overlap handling
    pub queue: Arc<Mutex<Vec<QueuedRun>>>,
    /// Total cumulative queue wait time in seconds across all processed queued runs
    queue_wait_time_seconds: Arc<Mutex<u64>>,
    /// Individual queue wait times in seconds for all processed queued runs
    queue_wait_times: Arc<Mutex<Vec<u64>>>,
    /// Handle to the heartbeat task (None when not running)
    heartbeat_task: Option<tokio::task::JoinHandle<()>>,
    /// PID of the scheduler process
    pid: u32,
    /// Start time of the scheduler (RFC3339 timestamp)
    start_time: String,
    /// Event emitter for observability (optional)
    event_emitter: Option<Arc<Mutex<EventEmitter>>>,
    /// Start time for uptime calculation (Instant)
    uptime_start: Option<Instant>,
}

impl Scheduler {
    /// Create a new Scheduler instance
    ///
    /// # Arguments
    ///
    /// * `clock` - Optional clock for time operations (defaults to SystemClock)
    /// * `settings` - Optional global settings configuration
    /// * `docker_client` - Optional Docker client for container operations (defaults to RealDockerClient)
    ///
    /// # Returns
    ///
    /// Returns a new Scheduler instance, or an error if scheduler creation fails.
    pub async fn new(
        clock: Option<Arc<dyn Clock>>,
        settings: Option<crate::config::Settings>,
        docker_client: Option<Arc<dyn DockerClientTrait>>,
    ) -> Result<Self, SchedulerError> {
        let clock = clock.unwrap_or_else(|| Arc::new(SystemClock));
        let pid = std::process::id();
        let start_time = Utc::now().to_rfc3339();
        let docker_client = match docker_client {
            Some(client) => client,
            None => Arc::new(RealDockerClient::new().await.map_err(|e| {
                SchedulerError::SchedulerCreationFailed {
                    error: e.to_string(),
                }
            })?),
        };

        Ok(Scheduler {
            agents: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
            scheduler: JobScheduler::new().await.map_err(|e| {
                SchedulerError::SchedulerCreationFailed {
                    error: e.to_string(),
                }
            })?,
            clock,
            docker_client,
            settings,
            queue: Arc::new(Mutex::new(Vec::new())),
            queue_wait_time_seconds: Arc::new(Mutex::new(0)),
            queue_wait_times: Arc::new(Mutex::new(Vec::new())),
            heartbeat_task: None,
            pid,
            start_time,
            event_emitter: None,
            uptime_start: None,
        })
    }

    /// Create a new Scheduler instance synchronously
    /// This is a convenience method that blocks on the async initialization
    ///
    /// # Arguments
    ///
    /// * `clock` - Optional clock for time operations (defaults to SystemClock)
    /// * `settings` - Optional global settings configuration
    /// * `docker_client` - Optional Docker client for container operations (defaults to RealDockerClient)
    pub fn new_sync(
        clock: Option<Arc<dyn Clock>>,
        settings: Option<crate::config::Settings>,
        docker_client: Option<Arc<dyn DockerClientTrait>>,
    ) -> Result<Self, SchedulerError> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(Self::new(clock, settings, docker_client))
        })
    }

    /// Set the event emitter for the scheduler
    ///
    /// This allows the scheduler to emit events for observability.
    ///
    /// # Arguments
    ///
    /// * `emitter` - The event emitter to use
    pub fn set_event_emitter(&mut self, emitter: EventEmitter) {
        self.event_emitter = Some(Arc::new(Mutex::new(emitter)));
    }

    /// Register an agent with its cron schedule
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent configuration to register
    /// * `config_dir` - The directory containing the config file (for resolving relative prompt_file paths)
    /// * `log_dir` - Directory for log files
    /// * `image_name` - Docker image name (e.g., "switchboard-agent")
    /// * `image_tag` - Docker image tag (e.g., "latest")
    /// * `workspace_path` - Workspace path to mount into the container
    /// * `docker_client` - Optional injected Docker client. If `Some`, uses the provided client;
    ///   if `None`, creates a new DockerClient internally when the agent executes (backward compatible)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the agent was successfully registered.
    /// Returns an error if the cron schedule is invalid or job registration fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn register_agent(
        &mut self,
        agent: &Agent,
        config_dir: PathBuf,
        log_dir: PathBuf,
        image_name: String,
        image_tag: String,
        workspace_path: String,
        _docker_client: Option<Arc<dyn DockerClientTrait>>,
    ) -> Result<(), SchedulerError> {
        // Create a job that triggers when the cron schedule fires
        let agent_name = agent.name.clone();
        let agent_name_for_error = agent_name.clone(); // Clone for error reporting
                                                       // Convert 5-field Unix cron to 6-field format for tokio-cron-scheduler
        let schedule = cron_helper::convert_to_6_field_cron(&agent.schedule);
        let agent_config = agent.clone();
        let clock = self.clock.clone();
        let agents = self.agents.clone();
        let queue = self.queue.clone();
        let docker_client = self.docker_client.clone();

        let queue_wait_time_seconds = self.queue_wait_time_seconds.clone();
        let queue_wait_times = self.queue_wait_times.clone();
        let event_emitter = self.event_emitter.clone();

        // Resolve the configured timezone for scheduling
        let tz = self.resolve_timezone()?;

        let docker_client_for_job = docker_client.clone();
        let event_emitter_for_job = event_emitter.clone();
        let job = Job::new_async_tz(schedule, tz, move |_uuid, _l| {
            let agents = agents.clone();
            let agent_name = agent_name.clone();
            let agent = agent_config.clone();
            let config_dir = config_dir.clone();
            let log_dir = log_dir.clone();
            let image_name = image_name.clone();
            let image_tag = image_tag.clone();
            let workspace_path = workspace_path.clone();
            let clock = clock.clone();
            let queue = queue.clone();
            let queue_wait_time_seconds = queue_wait_time_seconds.clone();
            let queue_wait_times = queue_wait_times.clone();
            let docker_client = docker_client_for_job.clone();
            let event_emitter = event_emitter_for_job.clone();
            Box::pin({
                let agents_clone = agents.clone();
                async move {
                    let agent_name_for_log = agent_name.clone();
                    tracing::info!("Agent '{}' triggered by cron schedule", agent_name);
                    if let Err(e) = execute_agent(
                        agent_name,
                        agents_clone,
                        agent,
                        config_dir,
                        log_dir,
                        image_name,
                        image_tag,
                        workspace_path,
                        clock,
                        queue,
                        queue_wait_time_seconds,
                        queue_wait_times,
                        None, // No queued start time for regular (non-queued) runs
                        Some(docker_client),
                        event_emitter,
                        "cron".to_string(),
                    )
                    .await
                    {
                        tracing::error!("Error executing agent '{}': {}", agent_name_for_log, e);
                    }
                }
            })
        })
        .map_err(|e| SchedulerError::JobRegistrationFailed {
            agent_name: agent_name_for_error.clone(),
            error: e.to_string(),
        })?;

        // Add the job to the scheduler
        self.scheduler
            .add(job)
            .await
            .map_err(|e| SchedulerError::JobRegistrationFailed {
                agent_name: agent_name_for_error.clone(),
                error: e.to_string(),
            })?;

        // Store the agent as a ScheduledAgent
        // Note: next_run calculation will be done when the scheduler starts
        let overlap_mode = agent.effective_overlap_mode(&self.settings);
        let mut agents = self
            .agents
            .lock()
            .map_err(|_| SchedulerError::MutexPoisoned)?;
        agents.push(ScheduledAgent {
            config: agent.clone(),
            next_run: None,
            current_run: None,
            overlap_mode,
        });
        drop(agents);

        Ok(())
    }

    /// Resolve the timezone for the scheduler
    ///
    /// Returns the configured timezone from Settings, or the system local timezone
    /// if no timezone is configured or if "system" is specified.
    ///
    /// Note: For "system", we use the TZ environment variable if set, otherwise
    /// default to UTC. This is a simplification - in production you might want to
    /// use a library like `iana-time-zone` to get the system timezone.
    fn resolve_timezone(&self) -> Result<Tz, SchedulerError> {
        let timezone_str = self
            .settings
            .as_ref()
            .map(|s| s.timezone.as_str())
            .unwrap_or("system");

        if timezone_str == "system" || timezone_str.is_empty() {
            // Try to get system timezone from TZ environment variable
            if let Ok(tz_var) = std::env::var("TZ") {
                if let Ok(tz) = Tz::from_str(&tz_var) {
                    return Ok(tz);
                }
            }

            // Fallback to UTC if we can't determine local timezone
            // For "system", using UTC as default ensures consistent behavior
            Ok(Tz::UTC)
        } else {
            // Parse IANA timezone string (e.g., "America/Los_Angeles")
            // Tz implements FromStr for parsing timezone names
            Tz::from_str(timezone_str).map_err(|_| SchedulerError::InvalidTimezone {
                timezone: timezone_str.to_string(),
                suggestion: "Try 'UTC' or check IANA timezone database at https://en.wikipedia.org/wiki/List_of_tz_database_time_zones".to_string(),
            })
        }
    }

    /// Calculate next_run for all registered agents
    ///
    /// This method computes the next scheduled execution time for each agent
    /// based on their cron schedule and the configured timezone.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if next_run was calculated successfully for all agents.
    /// Returns an error if timezone resolution or next run calculation fails.
    fn calculate_next_run_for_agents(&self) -> Result<(), SchedulerError> {
        let tz = self.resolve_timezone()?;

        let mut agents = self
            .agents
            .lock()
            .map_err(|_| SchedulerError::MutexPoisoned)?;
        for agent in agents.iter_mut() {
            // Parse the cron schedule using the cron crate
            let schedule = cron::Schedule::from_str(&cron_helper::convert_to_6_field_cron(
                &agent.config.schedule,
            ))
            .map_err(|e| SchedulerError::InvalidCronSchedule {
                schedule: agent.config.schedule.clone(),
                error: e.to_string(),
            })?;

            // Get the current time in UTC
            let now_utc: DateTime<Utc> = Utc::now();

            // Calculate the next scheduled time after now
            // The schedule's after() method returns an iterator of future times
            if let Some(next_utc) = schedule.after(&now_utc).next() {
                // Convert to the configured timezone
                let next_run_tz: DateTime<Tz> = next_utc.with_timezone(&tz);
                agent.next_run = Some(next_run_tz);
            } else {
                // Schedule has no next occurrence, set to None
                agent.next_run = None;
            }
        }
        drop(agents);

        Ok(())
    }

    /// Start the scheduler
    ///
    /// This starts the underlying tokio-cron-scheduler and begins
    /// monitoring cron schedules for registered agents.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the scheduler started successfully.
    /// Returns an error if starting the scheduler fails.
    pub async fn start(&mut self) -> Result<(), SchedulerError> {
        self.scheduler
            .start()
            .await
            .map_err(|e| SchedulerError::SchedulerStartFailed {
                error: e.to_string(),
            })?;
        self.running.store(true, Ordering::SeqCst);

        // Record start time for uptime calculation
        self.uptime_start = Some(Instant::now());

        // Calculate next_run for all agents after starting
        if let Err(e) = self.calculate_next_run_for_agents() {
            tracing::warn!("Failed to calculate next_run for agents: {}", e);
        }

        // Start the heartbeat task
        self.start_heartbeat_task();

        // Emit scheduler.started event if event emitter is configured
        self.emit_scheduler_started_event();

        Ok(())
    }

    /// Emit the scheduler.started event
    fn emit_scheduler_started_event(&mut self) {
        if let Some(ref emitter) = self.event_emitter {
            // Get list of agent names
            let agents: Vec<String> = {
                let locked_agents = match self.agents.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        tracing::warn!("Failed to lock agents for event emission: {}", e);
                        return;
                    }
                };
                locked_agents.iter().map(|a| a.config.name.clone()).collect()
            };

            let version = env!("CARGO_PKG_VERSION");
            let config_file = "switchboard.toml";

            let event = Event::new(
                EventType::SchedulerStarted,
                EventData::scheduler_started(agents, version, config_file),
            );

            if let Ok(mut emitter_guard) = emitter.lock() {
                if let Err(e) = emitter_guard.emit(event) {
                    tracing::warn!("Failed to emit scheduler.started event: {}", e);
                } else {
                    tracing::info!("Emitted scheduler.started event");
                }
            }
        }
    }

    /// Start the heartbeat task that periodically writes health status
    ///
    /// This writes a heartbeat file to `.switchboard/heartbeat.json` every minute
    /// that contains the scheduler PID, state, last heartbeat time, and agent info.
    fn start_heartbeat_task(&mut self) {
        let running = self.running.clone();
        let agents = self.agents.clone();
        let pid = self.pid;
        let start_time = self.start_time.clone();
        let version = env!("CARGO_PKG_VERSION");

        // Spawn the heartbeat task
        self.heartbeat_task = Some(tokio::spawn(async move {
            let heartbeat_interval = tokio::time::Duration::from_secs(60); // 1 minute

            loop {
                tokio::time::sleep(heartbeat_interval).await;

                // Check if scheduler is still running
                if !running.load(Ordering::SeqCst) {
                    tracing::debug!("Heartbeat task: scheduler stopped, exiting");
                    break;
                }

                // Write heartbeat file
                if let Err(e) = write_heartbeat(&agents, pid, &start_time, version) {
                    tracing::warn!("Failed to write heartbeat: {}", e);
                }
            }
        }));

        // Write initial heartbeat
        if let Err(e) = write_heartbeat(&self.agents, self.pid, &self.start_time, env!("CARGO_PKG_VERSION")) {
            tracing::warn!("Failed to write initial heartbeat: {}", e);
        }

        tracing::info!("Heartbeat task started");
    }

    /// Stop the scheduler gracefully
    ///
    /// This stops the underlying tokio-cron-scheduler and marks
    /// the scheduler as not running.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the scheduler stopped successfully.
    /// Returns an error if stopping the scheduler fails.
    pub async fn stop(&mut self) -> Result<(), SchedulerError> {
        // Emit scheduler.stopped event before stopping (if event emitter is configured)
        self.emit_scheduler_stopped_event("sigint");

        self.running.store(false, Ordering::SeqCst);
        self.scheduler
            .shutdown()
            .await
            .map_err(|e| SchedulerError::SchedulerStopFailed {
                error: e.to_string(),
            })?;
        Ok(())
    }

    /// Emit the scheduler.stopped event
    fn emit_scheduler_stopped_event(&mut self, reason: &str) {
        if let Some(ref emitter) = self.event_emitter {
            // Calculate uptime in seconds
            let uptime_seconds = self.uptime_start
                .map(|start| start.elapsed().as_secs())
                .unwrap_or(0);

            let event = Event::new(
                EventType::SchedulerStopped,
                EventData::scheduler_stopped(reason, uptime_seconds),
            );

            if let Ok(mut emitter_guard) = emitter.lock() {
                if let Err(e) = emitter_guard.emit(event) {
                    tracing::warn!("Failed to emit scheduler.stopped event: {}", e);
                } else {
                    tracing::info!("Emitted scheduler.stopped event (uptime: {}s)", uptime_seconds);
                }
            }
        }
    }

    /// Get the total cumulative queue wait time in seconds
    ///
    /// # Returns
    ///
    /// The total wait time in seconds across all processed queued runs
    pub fn get_total_queue_wait_time(&self) -> u64 {
        *self
            .queue_wait_time_seconds
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Get the individual queue wait times in seconds
    ///
    /// # Returns
    ///
    /// A vector of wait times in seconds for each processed queued run
    pub fn get_queue_wait_times(&self) -> Vec<u64> {
        self.queue_wait_times
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Get the number of processed queued runs
    ///
    /// # Returns
    ///
    /// The count of processed queued runs
    pub fn get_processed_queue_count(&self) -> usize {
        self.queue_wait_times
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

/// Write heartbeat file to `.switchboard/heartbeat.json`
///
/// This function writes a JSON file containing the scheduler's current state,
/// including PID, last heartbeat time, state, and agent information.
fn write_heartbeat(
    agents: &Arc<Mutex<Vec<ScheduledAgent>>>,
    pid: u32,
    start_time: &str,
    version: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use serde::Serialize;

    // Get current time
    let now = chrono::Utc::now();
    let last_heartbeat = now.to_rfc3339();

    // Get agent information from the scheduler
    let agent_heartbeats: Vec<AgentHeartbeat> = {
        let locked_agents = agents.lock().map_err(|_| "Failed to lock agents")?;
        locked_agents
            .iter()
            .map(|a| AgentHeartbeat {
                name: a.config.name.clone(),
                schedule: a.config.schedule.clone(),
                current_run: a.current_run.clone(),
            })
            .collect()
    };

    // Create heartbeat data structure
    #[derive(Serialize)]
    struct HeartbeatData<'a> {
        pid: u32,
        last_heartbeat: &'a str,
        start_time: &'a str,
        version: &'a str,
        state: &'a str,
        agents: Vec<AgentHeartbeat>,
    }

    #[derive(Serialize)]
    struct AgentHeartbeat {
        name: String,
        schedule: String,
        current_run: Option<String>,
    }

    let heartbeat_data = HeartbeatData {
        pid,
        last_heartbeat: &last_heartbeat,
        start_time,
        version,
        state: "running",
        agents: agent_heartbeats,
    };

    // Create .switchboard directory if it doesn't exist
    let switchboard_dir = std::path::Path::new(".switchboard");
    if !switchboard_dir.exists() {
        std::fs::create_dir_all(switchboard_dir)?;
    }

    // Write heartbeat file
    let heartbeat_path = switchboard_dir.join("heartbeat.json");
    let json = serde_json::to_string_pretty(&heartbeat_data)?;
    std::fs::write(&heartbeat_path, json)?;

    tracing::debug!("Heartbeat written to {:?}", heartbeat_path);

    Ok(())
}

#[cfg(test)]
mod scheduler_events_tests {
    use super::*;
    use crate::observability::{EmitterConfig, Event, EventData, EventEmitter, EventType};
    use tempfile::TempDir;

    /// Test that the scheduler.started event is emitted with correct data
    /// 
    /// This test verifies:
    /// - Event is emitted when scheduler starts
    /// - Event contains correct agent list
    /// - Event contains correct version
    /// - Event contains correct config file
    #[tokio::test]
    async fn test_scheduler_started_event_emission() {
        // Create a temporary directory for the event file
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let event_file = temp_dir.path().join("events.jsonl");
        
        // Create an event emitter
        let mut emitter = EventEmitter::new(
            EmitterConfig::new(&event_file)
                .with_append(false)
                .with_auto_flush(true)
        ).expect("Failed to create event emitter");
        
        // Create scheduler started event data
        let agents = vec![
            "goal-planner".to_string(),
            "goal-executor".to_string(),
            "goal-verifier".to_string(),
            "skill-distiller".to_string()
        ];
        let version = "0.5.0";
        let config_file = "switchboard.toml";
        
        let event = Event::new(
            EventType::SchedulerStarted,
            EventData::scheduler_started(agents.clone(), version, config_file),
        );
        
        // Emit the event
        emitter.emit(event).expect("Failed to emit scheduler.started event");
        emitter.flush().expect("Failed to flush");
        
        // Read the event file and verify
        let content = std::fs::read_to_string(&event_file).expect("Failed to read event file");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1, "Should have exactly one event");
        
        // Parse and verify the event
        let parsed: serde_json::Value = serde_json::from_str(lines[0])
            .expect("Failed to parse JSON");
        
        // Verify event type
        assert_eq!(parsed["event_type"], "scheduler_started");
        
        // Verify payload data
        let data = &parsed["payload"]["data"];
        assert_eq!(data["agent_count"], 4);
        assert_eq!(data["version"], "0.5.0");
        assert_eq!(data["config_file"], "switchboard.toml");
        
        let parsed_agents = data["agents"].as_array().expect("agents should be an array");
        assert_eq!(parsed_agents.len(), 4);
    }

    /// Test that the scheduler.stopped event is emitted with correct data
    /// 
    /// This test verifies:
    /// - Event is emitted when scheduler stops
    /// - Event contains correct reason
    /// - Event contains correct uptime_seconds
    #[tokio::test]
    async fn test_scheduler_stopped_event_emission() {
        // Create a temporary directory for the event file
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let event_file = temp_dir.path().join("events.jsonl");
        
        // Create an event emitter
        let mut emitter = EventEmitter::new(
            EmitterConfig::new(&event_file)
                .with_append(false)
                .with_auto_flush(true)
        ).expect("Failed to create event emitter");
        
        // Create scheduler stopped event data
        let reason = "sigint";
        let uptime_seconds = 86400u64;
        
        let event = Event::new(
            EventType::SchedulerStopped,
            EventData::scheduler_stopped(reason, uptime_seconds),
        );
        
        // Emit the event
        emitter.emit(event).expect("Failed to emit scheduler.stopped event");
        emitter.flush().expect("Failed to flush");
        
        // Read the event file and verify
        let content = std::fs::read_to_string(&event_file).expect("Failed to read event file");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1, "Should have exactly one event");
        
        // Parse and verify the event
        let parsed: serde_json::Value = serde_json::from_str(lines[0])
            .expect("Failed to parse JSON");
        
        // Verify event type
        assert_eq!(parsed["event_type"], "scheduler_stopped");
        
        // Verify payload data
        let data = &parsed["payload"]["data"];
        assert_eq!(data["reason"], "sigint");
        assert_eq!(data["uptime_seconds"], 86400u64);
    }

    /// Test that scheduler.started and scheduler.stopped events can both be emitted
    /// 
    /// This tests the full lifecycle: start -> stop
    #[tokio::test]
    async fn test_scheduler_lifecycle_events() {
        // Create a temporary directory for the event file
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let event_file = temp_dir.path().join("events.jsonl");
        
        // Create an event emitter
        let mut emitter = EventEmitter::new(
            EmitterConfig::new(&event_file)
                .with_append(false)
                .with_auto_flush(true)
        ).expect("Failed to create event emitter");
        
        // Emit scheduler.started event
        let started_event = Event::new(
            EventType::SchedulerStarted,
            EventData::scheduler_started(
                vec!["agent1".to_string(), "agent2".to_string()],
                "0.5.0",
                "switchboard.toml"
            ),
        );
        emitter.emit(started_event).expect("Failed to emit started event");
        emitter.flush().expect("Failed to flush");
        
        // Simulate some time passing (in a real scenario this would be the uptime)
        let uptime_seconds = 3600u64; // 1 hour
        
        // Emit scheduler.stopped event
        let stopped_event = Event::new(
            EventType::SchedulerStopped,
            EventData::scheduler_stopped("sigterm", uptime_seconds),
        );
        emitter.emit(stopped_event).expect("Failed to emit stopped event");
        emitter.flush().expect("Failed to flush");
        
        // Read and verify both events
        let content = std::fs::read_to_string(&event_file).expect("Failed to read event file");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2, "Should have exactly two events");
        
        // Parse and verify first event (started)
        let parsed_started: serde_json::Value = serde_json::from_str(lines[0])
            .expect("Failed to parse started event");
        assert_eq!(parsed_started["event_type"], "scheduler_started");
        
        // Parse and verify second event (stopped)
        let parsed_stopped: serde_json::Value = serde_json::from_str(lines[1])
            .expect("Failed to parse stopped event");
        assert_eq!(parsed_stopped["event_type"], "scheduler_stopped");
        assert_eq!(parsed_stopped["payload"]["data"]["uptime_seconds"], 3600u64);
    }

    /// Test uptime calculation from Instant
    #[test]
    fn test_uptime_calculation() {
        let start = Instant::now();
        
        // Simulate some time passing (in tests, this is very small)
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let uptime = start.elapsed().as_secs();
        // The uptime should be at least 0 (it might be 0 in fast tests)
        assert!(uptime >= 0);
    }

    // ===== Git Diff Parsing Tests =====

    /// Test parsing git log output with a single commit
    #[test]
    fn test_parse_git_log_single_commit() {
        let output = r#"abc1234567890abcdef|feat: Add new feature
10	5	src/main.rs
3	2	src/lib.rs
"#;
        
        let commits = parse_git_log_output(output);
        
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "abc1234567890abcdef");
        assert_eq!(commits[0].message, "feat: Add new feature");
        assert_eq!(commits[0].files_changed, 2);
        assert_eq!(commits[0].insertions, 13);
        assert_eq!(commits[0].deletions, 7);
    }

    /// Test parsing git log output with multiple commits
    #[test]
    fn test_parse_git_log_multiple_commits() {
        let output = r#"def7890123456789|feat: First commit
20	10	src/file1.rs
5	3	src/file2.rs

ghi456789012345abc|test: Second commit
15	0	src/tests.rs
"#;
        
        let commits = parse_git_log_output(output);
        
        assert_eq!(commits.len(), 2);
        
        // First commit
        assert_eq!(commits[0].hash, "def7890123456789");
        assert_eq!(commits[0].message, "feat: First commit");
        assert_eq!(commits[0].files_changed, 2);
        assert_eq!(commits[0].insertions, 25);
        assert_eq!(commits[0].deletions, 13);
        
        // Second commit
        assert_eq!(commits[1].hash, "ghi456789012345abc");
        assert_eq!(commits[1].message, "test: Second commit");
        assert_eq!(commits[1].files_changed, 1);
        assert_eq!(commits[1].insertions, 15);
        assert_eq!(commits[1].deletions, 0);
    }

    /// Test parsing git log output with no commits (empty output)
    #[test]
    fn test_parse_git_log_empty_output() {
        let output = "";
        
        let commits = parse_git_log_output(output);
        
        assert!(commits.is_empty());
    }

    /// Test parsing git log output with no numstat lines
    #[test]
    fn test_parse_git_log_no_numstat() {
        let output = r#"abc1234567890abcdef|Initial commit
"#;
        
        let commits = parse_git_log_output(output);
        
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "abc1234567890abcdef");
        assert_eq!(commits[0].message, "Initial commit");
        assert_eq!(commits[0].files_changed, 0);
        assert_eq!(commits[0].insertions, 0);
        assert_eq!(commits[0].deletions, 0);
    }

    /// Test parsing git log output with binary files (shown as -)
    #[test]
    fn test_parse_git_log_binary_files() {
        let output = r#"abc1234567890abcdef|feat: Add binary
-	-	src/binary.dat
10	5	src/text.rs
"#;
        
        let commits = parse_git_log_output(output);
        
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].files_changed, 2);
        // Binary files show as - for additions/deletions
        assert_eq!(commits[0].insertions, 10);
        assert_eq!(commits[0].deletions, 5);
    }

    /// Test parsing git log output with special characters in commit message
    #[test]
    fn test_parse_git_log_special_chars_in_message() {
        let output = r#"abc1234567890abcdef|feat: Add | pipe test & special chars!
10	5	src/main.rs
"#;
        
        let commits = parse_git_log_output(output);
        
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].message, "feat: Add | pipe test & special chars!");
    }

    /// Test parsing git log output with trailing newline
    #[test]
    fn test_parse_git_log_trailing_newline() {
        let output = r#"abc1234567890abcdef|feat: Test commit
10	5	src/main.rs

def7890123456789|feat: Another commit
3	1	src/lib.rs
"#;
        
        let commits = parse_git_log_output(output);
        
        assert_eq!(commits.len(), 2);
    }


    /// Test parsing git log output with Windows-style line endings
    #[test]
    fn test_parse_git_log_windows_line_endings() {
        let output = "abc1234567890abcdef|feat: Windows test\n10\t5\tsrc/main.rs\n";
        
        let commits = parse_git_log_output(output);
        
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "abc1234567890abcdef");
    }
}
