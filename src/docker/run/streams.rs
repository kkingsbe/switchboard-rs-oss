//! Container log streaming functionality
//!
//! This module provides functionality for streaming logs from Docker containers,
//! supporting both foreground terminal output and file-based logging.

use crate::docker::{DockerClient, DockerError};
use crate::logger::Logger;
use bollard::container::{LogOutput, LogsOptions};
use futures::StreamExt;
use std::sync::{Arc, Mutex};

/// Attach to a container and stream its logs
///
/// This function connects to a running Docker container and streams its stdout/stderr
/// output. If a logger is provided, logs are written to both the terminal (if in
/// foreground mode) and the agent's log file.
///
/// # Arguments
///
/// * `client` - Reference to the DockerClient
/// * `container_id` - The ID of the container to stream logs from
/// * `agent_name` - Name of the agent (used for log file naming)
/// * `logger` - Optional logger for writing container logs
/// * `follow` - Whether to follow logs as they are generated (true) or get existing logs (false)
///
/// # Returns
///
/// Returns `Ok(())` on successful log streaming, or `DockerError` on failure.
///
/// # Errors
///
/// Returns `DockerError::ConnectionError` if there's an issue with the Docker connection
/// or log stream.
pub async fn attach_and_stream_logs(
    client: &DockerClient,
    container_id: &str,
    agent_name: &str,
    logger: Option<Arc<Mutex<Logger>>>,
    follow: bool,
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
                    if let Err(e) = logger.lock().unwrap().write_agent_log(agent_name, &message) {
                        eprintln!("Failed to write agent log: {}", e);
                    }
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
