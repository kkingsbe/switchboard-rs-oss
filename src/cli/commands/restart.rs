//! CLI restart command implementation
//!
//! This module contains the restart command handler that stops and restarts the scheduler.

use crate::cli::process::{default_executor, is_process_running};
use std::fs;
use std::path::Path;
use tokio::time::sleep;
use tokio::time::Duration;

/// Run the 'restart' command - Stop and start the scheduler
///
/// This command stops the scheduler if it's running and then starts it again.
/// It supports starting in either foreground or detached mode.
///
/// # Functionality
///
/// The function performs the following steps:
/// 1. Check if scheduler is running (read PID file)
/// 2. If running, stop the scheduler (similar to down command)
/// 3. Start the scheduler using the up command
/// 4. Print status
///
/// # Arguments
///
/// * `args` - The [`RestartCommand`] containing CLI arguments:
///   - `args.detach`: Run in detached mode after restart
/// * `config_path` - Optional path to the configuration file
///   - If `None`, defaults to `./switchboard.toml`
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Failed to stop the running scheduler
/// - Failed to start the scheduler
pub async fn run_restart(
    args: crate::cli::RestartCommand,
    config_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pid_file_path = Path::new(".switchboard/scheduler.pid");
    let config_path = config_path.unwrap_or_else(|| "./switchboard.toml".to_string());

    // Step 1: Check if scheduler is running
    let mut _was_running = false;
    
    if pid_file_path.exists() {
        // Read PID file
        let pid_content = fs::read_to_string(pid_file_path)?;
        
        // Parse PID as u32
        let pid: u32 = pid_content.trim().parse().map_err(|e| {
            format!("Failed to parse PID: {}", e)
        })?;

        // Check if process is actually running
        if is_process_running(pid) {
            _was_running = true;
            println!("Stopping scheduler (PID: {})...", pid);

            // Send SIGTERM to stop the scheduler
            let executor = default_executor();
            let kill_result = executor.execute("kill", &["-15".to_string(), pid.to_string()]);

            match kill_result {
                Ok(output) if output.status.success() => {
                    println!("✓ Scheduler stopped");
                }
                Ok(output) => {
                    let exit_code = output.status.code().unwrap_or(-1);
                    if exit_code == 1 {
                        println!("Scheduler process no longer running");
                    } else {
                        eprintln!("✗ Failed to stop scheduler (exit code: {})", exit_code);
                        return Err(format!("Failed to stop scheduler (exit code: {})", exit_code).into());
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string().to_lowercase();
                    if error_msg.contains("no such process") || error_msg.contains("esrch") {
                        println!("Scheduler process no longer running");
                    } else {
                        eprintln!("✗ Failed to stop scheduler: {}", e);
                        return Err(format!("Failed to stop scheduler: {}", e).into());
                    }
                }
            }

            // Give the process time to terminate
            sleep(Duration::from_secs(1)).await;
        }
    }

    // Clean up PID file if it still exists
    if pid_file_path.exists() {
        let _ = fs::remove_file(pid_file_path);
    }

    // Step 2: Start the scheduler
    println!();
    
    // Build the up command arguments
    let up_args = crate::cli::UpCommand {
        detach: args.detach,
        daemon: false,
    };

    // Call run_up to start the scheduler
    // This will handle starting in foreground or detached mode
    crate::cli::run_up(up_args, Some(config_path)).await?;

    Ok(())
}
