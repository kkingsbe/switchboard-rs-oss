//! CLI ps command implementation
//!
//! This module implements the `ps` command similar to `docker-compose ps` to show
//! the status of all switchboard processes.

use crate::api::registry::derive_instance_id_from_config;
use crate::cli::process::is_process_running;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Process status entry
#[derive(Debug, Clone)]
pub struct ProcessStatus {
    pub name: String,
    pub status: String,
    pub pid: Option<u32>,
    pub created: Option<String>,
}

/// Run the ps command - display status of all switchboard processes
///
/// This command shows running scheduler status from `.switchboard/scheduler.pid`
/// and `.switchboard/heartbeat.json` in a docker-compose ps-like format.
///
/// # Output format
/// ```
/// NAME              STATUS          PID     CREATED
/// scheduler        Running         12345   2 minutes ago
/// ```
///
/// # Edge cases handled:
/// - Scheduler not running (no PID file)
/// - Stale PID file (process not running but file exists)
/// - No heartbeat file
pub fn run_ps(config: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = config.unwrap_or_else(|| "./switchboard.toml".to_string());
    
    // Derive instance_id from config path to match the instance-specific PID file location
    let instance_id = derive_instance_id_from_config(&config_path);
    tracing::debug!("Derived instance ID from config path: {}", instance_id);
    
    let pid_file_path = Path::new(".switchboard").join("instances").join(&instance_id).join("scheduler.pid");
    let heartbeat_path = Path::new(".switchboard").join("instances").join(&instance_id).join("heartbeat.json");

    // Header for the table
    println!(
        "{:<16} {:<15} {:<10} {}",
        "NAME", "STATUS", "PID", "CREATED"
    );
    println!(
        "{:-<16} {:-<15} {:-<10} {}",
        "", "", "", ""
    );

    // Try to get scheduler status
    let status = get_scheduler_status(&pid_file_path, &heartbeat_path);

    // Print the scheduler row
    println!(
        "{:<16} {:<15} {:<10} {}",
        status.name,
        status.status,
        status.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string()),
        status.created.unwrap_or_else(|| "-".to_string())
    );

    Ok(())
}

/// Get the scheduler status from PID file and heartbeat
fn get_scheduler_status(
    pid_file_path: &Path,
    heartbeat_path: &Path,
) -> ProcessStatus {
    // Try to read PID file first
    let pid = match fs::read_to_string(pid_file_path) {
        Ok(content) => content.trim().parse::<u32>().ok(),
        Err(_) => None,
    };

    // Check if heartbeat file exists
    if !heartbeat_path.exists() {
        // No heartbeat file - check if PID file exists (stale)
        return ProcessStatus {
            name: "scheduler".to_string(),
            status: if pid.is_some() { "Stopped".to_string() } else { "Not running".to_string() },
            pid,
            created: None,
        };
    }

    // Read and parse heartbeat file
    match fs::read_to_string(heartbeat_path) {
        Ok(content) => {
            match serde_json::from_str::<HeartbeatData>(&content) {
                Ok(heartbeat) => {
                    // Check if the process is actually running
                    let process_running = pid.map(|p| is_process_running(p)).unwrap_or(false);

                    // Parse last heartbeat time for "created" display
                    let created = chrono::DateTime::parse_from_rfc3339(&heartbeat.last_heartbeat)
                        .ok()
                        .map(|dt| {
                            let now = chrono::Utc::now();
                            let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
                            format_duration(duration)
                        });

                    let status = if heartbeat.state == "running" && process_running {
                        "Running".to_string()
                    } else if heartbeat.state == "running" && !process_running {
                        "Stopped".to_string()
                    } else {
                        heartbeat.state
                    };

                    ProcessStatus {
                        name: "scheduler".to_string(),
                        status,
                        pid,
                        created,
                    }
                }
                Err(e) => {
                    // Failed to parse heartbeat
                    ProcessStatus {
                        name: "scheduler".to_string(),
                        status: "Error".to_string(),
                        pid,
                        created: Some(format!("parse error: {}", e)),
                    }
                }
            }
        }
        Err(_) => {
            // Can't read heartbeat
            ProcessStatus {
                name: "scheduler".to_string(),
                status: "Not running".to_string(),
                pid,
                created: None,
            }
        }
    }
}

/// Format a chrono Duration into a human-readable string
fn format_duration(duration: chrono::Duration) -> String {
    let total_secs = duration.num_seconds().abs();
    if total_secs < 60 {
        format!("{} seconds ago", total_secs)
    } else if total_secs < 3600 {
        let mins = total_secs / 60;
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if total_secs < 86400 {
        let hours = total_secs / 3600;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else {
        let days = total_secs / 86400;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    }
}

/// Heartbeat data structure from `.switchboard/heartbeat.json`
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HeartbeatData {
    pid: u32,
    last_heartbeat: String,
    state: String,
    agents: Vec<AgentHeartbeat>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AgentHeartbeat {
    name: String,
    schedule: String,
    current_run: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_seconds() {
        let duration = chrono::Duration::seconds(45);
        assert_eq!(format_duration(duration), "45 seconds ago");
    }

    #[test]
    fn test_format_duration_minutes() {
        let duration = chrono::Duration::minutes(5);
        assert_eq!(format_duration(duration), "5 minutes ago");
    }

    #[test]
    fn test_format_duration_hours() {
        let duration = chrono::Duration::hours(3);
        assert_eq!(format_duration(duration), "3 hours ago");
    }

    #[test]
    fn test_format_duration_days() {
        let duration = chrono::Duration::days(2);
        assert_eq!(format_duration(duration), "2 days ago");
    }
}
