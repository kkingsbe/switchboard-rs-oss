// Process utilities for CLI

use std::fs;
use std::path::Path;
use std::sync::Arc;

#[cfg(unix)]
use libc;

use crate::docker::DockerClient;
use crate::traits::{ProcessExecutorTrait, RealProcessExecutor};

/// Check if a Docker image exists locally
pub async fn check_image_exists(
    client: &DockerClient,
    image_name: &str,
    image_tag: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let docker = client
        .docker()
        .ok_or_else(|| "Docker client unavailable".to_string())?;

    let images = docker
        .list_images::<String>(None)
        .await
        .map_err(|e| format!("Failed to list Docker images: {}", e))?;

    let target_image = format!("{}:{}", image_name, image_tag);

    for image in &images {
        for tag in &image.repo_tags {
            if tag == &target_image {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Get the default process executor
///
/// It can be replaced with a mock implementation for testing.
fn default_executor() -> Arc<dyn ProcessExecutorTrait> {
    Arc::new(RealProcessExecutor::new())
}

/// Check if a process is running using the provided executor
///
/// This version allows dependency injection for testing.
///
/// * `pid` - The process ID to check
/// * `executor` - The process executor to use
///
/// # Returns
/// Returns `true` if the process is running, `false` otherwise
#[cfg(windows)]
fn is_process_running_with_executor(pid: u32, executor: Arc<dyn ProcessExecutorTrait>) -> bool {
    // On Windows, use tasklist to check if the process exists
    // This is a simple approach that doesn't require additional dependencies
    let args = vec![
        "/FI".to_string(),
        format!("PID eq {}", pid),
        "/NH".to_string(),
    ];

    match executor.execute("tasklist", &args) {
        Ok(output) => {
            let stdout = &output.stdout;
            // tasklist returns the process info if it exists, empty string otherwise
            stdout.contains(&pid.to_string()) || !stdout.trim().is_empty()
        }
        Err(e) => {
            tracing::warn!("Failed to execute tasklist: {}", e);
            false
        }
    }
}

/// Check if a process is running (Unix implementation)
#[cfg(unix)]
pub fn is_process_running(pid: u32) -> bool {
    unsafe {
        match libc::kill(pid as libc::pid_t, 0) {
            0 | libc::EPERM => true,
            libc::ESRCH => false,
            _ => false,
        }
    }
}

/// Check if a process is running (Windows implementation)
#[cfg(windows)]
pub fn is_process_running(pid: u32) -> bool {
    is_process_running_with_executor(pid, default_executor())
}

/// Check and clean stale PID file
pub fn check_and_clean_stale_pid_file(
    pid_file_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !pid_file_path.exists() {
        return Ok(());
    }

    let pid_content =
        fs::read_to_string(pid_file_path).map_err(|e| format!("Failed to read PID file: {}", e))?;

    let pid: u32 = pid_content
        .trim()
        .parse()
        .map_err(|e| format!("Failed to parse PID: {}", e))?;

    if !is_process_running(pid) {
        tracing::debug!("Found stale PID file for process {}, removing...", pid);
        fs::remove_file(pid_file_path)
            .map_err(|e| format!("Failed to remove stale PID file: {}", e))?;
        tracing::debug!("Stale PID file removed");
    } else {
        tracing::debug!("PID file found, process {} is running", pid);
    }

    Ok(())
}
