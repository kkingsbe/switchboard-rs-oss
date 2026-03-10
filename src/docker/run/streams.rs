//! Container log streaming functionality
//!
//! This module provides functionality for streaming logs from Docker containers,
//! supporting both foreground terminal output and file-based logging.

use crate::docker::DockerError;
use crate::logger::Logger;
use bollard::container::{LogOutput, LogsOptions};
use futures::StreamExt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Shared state for tracking the last time a log was received.
///
/// This is used by the silent timeout monitoring to detect when an agent
/// has been silent (no log output) for a specified duration.
#[derive(Clone)]
pub struct LogTimestampTracker {
    /// Unix timestamp (seconds since epoch) of when the last log was received
    last_log_time: Arc<AtomicU64>,
}

impl LogTimestampTracker {
    /// Create a new LogTimestampTracker, initialized to the current time
    pub fn new() -> Self {
        let now = Instant::now().elapsed().as_secs();
        Self {
            last_log_time: Arc::new(AtomicU64::new(now)),
        }
    }

    /// Update the last log time to now
    pub fn update(&self) {
        let now = Instant::now().elapsed().as_secs();
        self.last_log_time.store(now, Ordering::SeqCst);
    }

    /// Get the seconds since the last log was received
    pub fn seconds_since_last_log(&self) -> u64 {
        let last_log = self.last_log_time.load(Ordering::SeqCst);
        let now = Instant::now().elapsed().as_secs();
        now.saturating_sub(last_log)
    }

    /// Check if the silent timeout has been exceeded
    /// Returns true if no logs received for longer than the timeout duration
    pub fn is_silent_timeout_exceeded(&self, timeout: Duration) -> bool {
        let timeout_secs = timeout.as_secs();
        self.seconds_since_last_log() > timeout_secs
    }
}

impl Default for LogTimestampTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Attach to a container and stream its logs
///
/// This function connects to a running Docker container and streams its stdout/stderr
/// output. If a logger is provided, logs are written to both the terminal (if in
/// foreground mode) and the agent's log file.
///
/// If a `LogTimestampTracker` is provided, it will be updated whenever a log message
/// is received, which is used by the silent timeout monitoring feature.
///
/// # Arguments
///
/// * `client` - Reference to the DockerClientTrait
/// * `container_id` - The ID of the container to stream logs from
/// * `agent_name` - Name of the agent (used for log file naming)
/// * `logger` - Optional logger for writing container logs
/// * `follow` - Whether to follow logs as they are generated (true) or get existing logs (false)
/// * `timestamp_tracker` - Optional tracker for updating last log timestamp (for silent timeout)
///
/// # Returns
///
/// Returns `Ok(())` on successful log streaming, or `DockerError` on failure.
///
/// # Errors
///
/// Returns `DockerError::ConnectionError` if there's an issue with the Docker connection
/// or log stream.
pub async fn attach_and_stream_logs<T: crate::traits::DockerClientTrait>(
    client: &T,
    container_id: &str,
    agent_name: &str,
    logger: Option<Arc<Mutex<Logger>>>,
    follow: bool,
    timestamp_tracker: Option<LogTimestampTracker>,
) -> Result<(), DockerError> {
    let docker = client
        .docker()
        .ok_or_else(|| DockerError::DockerUnavailable {
            reason: "Docker client not initialized".to_string(),
            suggestion: "Ensure Docker is running and properly initialized".to_string(),
        })?;

    // Configure logs options
    let options = LogsOptions {
        follow, // follow logs as they are generated or get existing logs
        stdout: true,
        stderr: true,
        tail: "0".to_string(), // Get all logs from the beginning
        timestamps: false,
        since: 0,
        until: 0,
    };

    // Get logs from the container
    let logs_stream = docker.logs::<String>(container_id, Some(options));

    // Process the stream
    tokio::pin!(logs_stream);

    while let Some(result) = logs_stream.next().await {
        match result {
            Ok(output) => {
                // Convert LogOutput to bytes
                let bytes = match output {
                    LogOutput::StdErr { message } => message,
                    LogOutput::StdOut { message } => message,
                    LogOutput::Console { message } => message,
                    LogOutput::StdIn { .. } => continue, // Skip stdin
                };

                // Convert bytes to string, handling potential UTF-8 errors
                let message = match String::from_utf8_lossy(&bytes).into_owned() {
                    s if !s.is_empty() => s,
                    _ => continue, // Skip empty messages
                };

                // Write to logger if present
                if let Some(logger) = &logger {
                    // Write to terminal if in foreground mode
                    if let Ok(logger_guard) = logger.lock() {
                        if logger_guard.foreground_mode {
                            let _ = logger_guard.write_terminal_output(&message);
                        }
                    }
                    // Write to log file
                    match logger.lock() {
                        Ok(logger_guard) => {
                            if let Err(e) = logger_guard.write_agent_log(agent_name, &message) {
                                eprintln!("Failed to write agent log: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to acquire logger lock: {}", e);
                        }
                    }
                }

                // Update timestamp tracker if provided (for silent timeout monitoring)
                if let Some(ref tracker) = timestamp_tracker {
                    tracker.update();
                }
            }
            Err(e) => {
                // Handle stream errors
                // Check if this is an expected termination (container removed/exited)
                let error_msg = format!("{}", e);
                let is_expected_termination = error_msg.contains("no such container")
                    || error_msg.contains("container not found")
                    || error_msg.contains("EOF")
                    || error_msg.contains("connection closed")
                    || error_msg.contains("broken pipe");

                if is_expected_termination {
                    // Container finished or was removed (expected with auto_remove: true)
                    // This is not an error, just the stream ending normally
                    return Ok(());
                }

                // Actual error - return it
                let error_msg = format!(
                    "Failed to read from stream for container '{}': {}",
                    container_id, e
                );
                return Err(DockerError::ConnectionError(error_msg));
            }
        }
    }

    Ok(())
}
