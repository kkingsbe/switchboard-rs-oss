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
use crate::traits::{DockerClientTrait, RealDockerClient};

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
                    queue_mut.push(queued_run);

                    tracing::info!(
                        "Agent '{}' is running, queued run ({}/{})",
                        agent_name,
                        queue_mut.len(),
                        max_queue_size
                    );

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

        // Resolve the configured timezone for scheduling
        let tz = self.resolve_timezone()?;

        let docker_client_for_job = docker_client.clone();
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

        // Calculate next_run for all agents after starting
        if let Err(e) = self.calculate_next_run_for_agents() {
            tracing::warn!("Failed to calculate next_run for agents: {}", e);
        }

        // Start the heartbeat task
        self.start_heartbeat_task();

        Ok(())
    }

    /// Start the heartbeat task that periodically writes health status
    ///
    /// This writes a heartbeat file to `.switchboard/heartbeat.json` every minute
    /// that contains the scheduler PID, state, last heartbeat time, and agent info.
    fn start_heartbeat_task(&mut self) {
        let running = self.running.clone();
        let agents = self.agents.clone();
        let pid = self.pid;

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
                if let Err(e) = write_heartbeat(&agents, pid) {
                    tracing::warn!("Failed to write heartbeat: {}", e);
                }
            }
        }));

        // Write initial heartbeat
        if let Err(e) = write_heartbeat(&self.agents, self.pid) {
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
        self.running.store(false, Ordering::SeqCst);
        self.scheduler
            .shutdown()
            .await
            .map_err(|e| SchedulerError::SchedulerStopFailed {
                error: e.to_string(),
            })?;
        Ok(())
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
