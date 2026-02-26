//! Metrics command - Display agent execution statistics
//!
//! This module provides functionality to display metrics about
//! scheduled agent executions in table or detailed format.

use crate::metrics::{AllMetrics, MetricsError, MetricsStore};
use chrono::Local;
use comfy_table::{Attribute, Cell, Color, Table};
use std::collections::HashMap;
use std::path::PathBuf;

/// Format duration in seconds to human-readable format
///
/// # Arguments
///
/// * `seconds` - Duration in seconds
///
/// # Returns
///
/// Human-readable duration string (e.g., "1h 2m 3s", "45s", "2m 3s")
fn format_duration(seconds: f64) -> String {
    let total_seconds = seconds.round() as i64;

    if total_seconds < 60 {
        return format!("{}s", total_seconds);
    }

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else {
        format!("{}m {}s", minutes, secs)
    }
}

/// Calculate status icon based on success rate and total runs
///
/// # Arguments
///
/// * `total_runs` - Total number of runs
/// * `success_rate` - Success rate as percentage
///
/// # Returns
///
/// Status icon: "✓", "⚠", "✗", or "-"
fn get_status_icon(total_runs: u64, success_rate: f64) -> &'static str {
    if total_runs == 0 || total_runs < 3 {
        "-"
    } else if success_rate >= 95.0 {
        "✓"
    } else if success_rate >= 50.0 {
        "⚠"
    } else {
        "✗"
    }
}

/// Display metrics for all agents
///
/// This function loads metrics data from `metrics.json` in the specified log directory
/// and displays execution statistics for all configured agents or a specific agent.
/// Metrics include total runs, success/failure counts, duration statistics, and
/// individual run details.
///
/// # Arguments
///
/// * `log_dir` - Path to the log directory where `metrics.json` is stored
/// * `detailed` - Whether to show detailed view with individual run information.
///   When `false`, shows a summary table. When `true`, shows per-agent details.
/// * `agent` - Optional agent name to filter metrics for a specific agent.
///   If `None`, displays metrics for all agents.
/// * `agent_schedules` - Optional mapping of agent names to their cron schedules.
///   Used in detailed view to display agent schedules.
///
/// # Returns
///
/// * `Ok(())` - Successfully displayed metrics
/// * `Err(String)` - Error occurred while loading metrics:
///   - Corrupted metrics file (with backup file location)
///   - Failed to read metrics file
///   - Failed to write metrics file
///   - Failed to serialize/deserialize metrics
///   - Agent not found in metrics data
///
/// # Examples
///
/// Display metrics for all agents (summary table):
/// ```bash
/// switchboard metrics
/// ```
///
/// Display detailed metrics for all agents:
/// ```bash
/// switchboard metrics --detailed
/// ```
///
/// Display metrics for a specific agent:
/// ```bash
/// switchboard metrics --agent my-agent
/// ```
///
/// # Skill Installation Metrics
///
/// The "Skills" column shows skill installation status in "installed/failed" format:
/// - "5/0" - All 5 skills installed successfully
/// - "3/2" - 3 skills installed, 2 failed
/// - "0/5" - All 5 skills failed to install
/// - "0/0" - No skills configured or unknown status (timeout)
///
/// In detailed view, skill installation metrics include:
/// - `Total Skills Installed` - Cumulative count of successfully installed skills
/// - `Total Skills Failed` - Cumulative count of failed skill installations
/// - `Runs with Skill Failures` - Number of runs where at least one skill failed
/// - `Avg Skill Install Time` - Average time in seconds for skill installation
///
/// ```bash
/// # View metrics table with skill installation status
/// switchboard metrics
///
/// # View detailed metrics including skill installation statistics
/// switchboard metrics --detailed
///
/// # Filter runs with skill failures
/// switchboard metrics | grep "0/"  # Shows rows with failed skills
/// ```
///
/// # Notes
///
/// - If no metrics data exists, prints a friendly message and returns successfully
/// - Status icons indicate agent health: ✓ (>=95% success), ⚠ (50-95% success), ✗ (<50% success), - (<3 runs)
/// - Detailed view shows last 5 runs with timestamps, durations, and status
/// - Average duration is calculated across all successful and failed runs
/// - The metrics file is automatically created and updated by the scheduler
/// - Corrupted metrics files are backed up before error is reported
/// - Skill installation metrics are collected for each agent run that has skills configured
pub fn metrics(
    log_dir: &str,
    detailed: bool,
    agent: Option<&str>,
    agent_schedules: Option<&HashMap<String, String>>,
) -> Result<(), String> {
    let store = MetricsStore::new(PathBuf::from(log_dir));
    let all_metrics = match store.load() {
        Ok(metrics) => metrics,
        Err(MetricsError::FileNotFound(_)) => {
            // Friendly message for missing file - exit 0
            eprintln!("No metrics data available yet. Run agents to collect metrics.");
            return Ok(());
        }
        Err(MetricsError::CorruptedFile(backup_path)) => {
            // Display error with backup file location - exit 1
            eprintln!("Error: Metrics file is corrupted.");
            eprintln!("Backup saved to: {}", backup_path);
            return Err("Corrupted metrics file".to_string());
        }
        Err(MetricsError::ReadError(msg)) => {
            // Display clear read error message - exit 1
            eprintln!("Error: Failed to read metrics file: {}", msg);
            return Err("Failed to read metrics file".to_string());
        }
        Err(MetricsError::WriteError(msg)) => {
            // Display clear write error message - exit 1
            eprintln!("Error: Failed to write metrics file: {}", msg);
            return Err("Failed to write metrics file".to_string());
        }
        Err(MetricsError::SerializationError(msg)) => {
            // Display clear serialization error message - exit 1
            eprintln!("Error: Failed to serialize metrics: {}", msg);
            return Err("Failed to serialize metrics".to_string());
        }
        Err(MetricsError::DeserializationError(msg)) => {
            // Display clear deserialization error message - exit 1
            eprintln!("Error: Failed to parse metrics: {}", msg);
            return Err("Failed to parse metrics".to_string());
        }
    };

    // Check if there are any agents with metrics
    if all_metrics.agents.is_empty() {
        println!("No metrics data available yet");
        return Ok(());
    }

    // Filter to specific agent if provided
    let mut agent_names: Vec<_> = if let Some(agent_name) = agent {
        // Check if the agent exists in metrics
        if !all_metrics.agents.contains_key(agent_name) {
            // Show available agent names in the error message
            let available: Vec<String> = all_metrics.agents.keys().cloned().collect();
            let available_list = if available.is_empty() {
                "none (no agents have run yet)".to_string()
            } else {
                available.join(", ")
            };
            return Err(format!(
                "Agent '{}' not found in metrics. Available agents: {}",
                agent_name, available_list
            ));
        }
        // Find the agent key in the hashmap to get the correct type
        all_metrics
            .agents
            .keys()
            .find(|k| *k == agent_name)
            .into_iter()
            .collect()
    } else {
        // Return all agents
        all_metrics.agents.keys().collect()
    };
    agent_names.sort();

    if detailed {
        display_detailed_metrics(&all_metrics, &agent_names, agent_schedules)?;
    } else {
        display_table_metrics(&all_metrics, &agent_names)?;
    }

    Ok(())
}

/// Display metrics in table format
///
/// # Arguments
///
/// * `all_metrics` - All metrics data
/// * `agent_names` - Sorted list of agent names
fn display_table_metrics(all_metrics: &AllMetrics, agent_names: &[&String]) -> Result<(), String> {
    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_header(vec![
            Cell::new("Agent")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Runs")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Success")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Fail")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Skills")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Avg Duration")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Last Run")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Status")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
        ]);

    for agent_name in agent_names {
        let agent_data = &all_metrics.agents[*agent_name];
        let total_runs = agent_data.total_runs;

        let successful_runs = agent_data.successful_runs;
        let failed_runs = agent_data.failed_runs;

        // Calculate success rate for status icon only
        let success_rate = if total_runs > 0 {
            ((total_runs - failed_runs) as f64 / total_runs as f64) * 100.0
        } else {
            0.0
        };

        let avg_duration = if total_runs > 0 {
            let avg_duration_ms = agent_data.total_duration_ms as f64 / total_runs as f64;
            format_duration(avg_duration_ms / 1000.0)
        } else {
            "-".to_string()
        };

        // Format skills as "installed/failed"
        let skills_str =
            if agent_data.total_skills_installed == 0 && agent_data.total_skills_failed == 0 {
                "-".to_string()
            } else {
                format!(
                    "{}/{}",
                    agent_data.total_skills_installed, agent_data.total_skills_failed
                )
            };

        let status_icon = get_status_icon(total_runs, success_rate);

        // Get last run timestamp
        let last_run_str = if !agent_data.runs.is_empty() {
            let last_run = agent_data.runs.iter().max_by_key(|r| r.timestamp).unwrap();
            let last_run_dt = chrono::DateTime::from_timestamp(last_run.timestamp as i64, 0)
                .unwrap_or_else(chrono::Utc::now)
                .with_timezone(&Local);
            last_run_dt.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            "-".to_string()
        };

        table.add_row(vec![
            Cell::new(agent_name),
            Cell::new(total_runs.to_string()),
            Cell::new(successful_runs.to_string()),
            Cell::new(failed_runs.to_string()),
            Cell::new(skills_str),
            Cell::new(avg_duration),
            Cell::new(last_run_str),
            Cell::new(status_icon),
        ]);
    }

    println!();
    println!("{}", table);
    println!();

    Ok(())
}

/// Display metrics in detailed format
///
/// # Arguments
///
/// * `all_metrics` - All metrics data
/// * `agent_names` - Sorted list of agent names
/// * `agent_schedules` - Optional mapping of agent names to their cron schedules
fn display_detailed_metrics(
    all_metrics: &AllMetrics,
    agent_names: &[&String],
    agent_schedules: Option<&HashMap<String, String>>,
) -> Result<(), String> {
    for agent_name in agent_names {
        let agent_data = &all_metrics.agents[*agent_name];

        println!();
        println!(
            "================================================================================"
        );
        println!("Agent: {}", agent_name);
        println!(
            "================================================================================"
        );

        // Get schedule from agent_schedules map if available
        let schedule = agent_schedules
            .and_then(|s| s.get(*agent_name).cloned())
            .unwrap_or_else(|| "-".to_string());
        println!("  Schedule:        {}", schedule);

        let total_runs = agent_data.total_runs;
        let successful_runs = agent_data.successful_runs;
        let failed_runs = agent_data.failed_runs;
        let success_rate = if total_runs > 0 {
            ((total_runs - failed_runs) as f64 / total_runs as f64) * 100.0
        } else {
            0.0
        };

        let avg_duration = if total_runs > 0 {
            format_duration((agent_data.total_duration_ms as f64 / total_runs as f64) / 1000.0)
        } else {
            "-".to_string()
        };

        let total_runtime = if total_runs > 0 {
            format_duration(agent_data.total_duration_ms as f64 / 1000.0)
        } else {
            "-".to_string()
        };

        let status_icon = get_status_icon(total_runs, success_rate);

        let success_rate_str = if total_runs > 0 {
            format!("{:.1}%", success_rate)
        } else {
            "-".to_string()
        };

        println!("  Total Runs:      {}", total_runs);
        println!("  Successful Runs: {}", successful_runs);
        println!("  Failed Runs:     {}", failed_runs);
        println!("  Success Rate:    {} {}", success_rate_str, status_icon);
        println!("  Total Runtime:   {}", total_runtime);
        println!("  Average Duration: {}", avg_duration);
        println!("  Concurrent Run ID: -");

        // Get first and last run timestamps from runs vector
        if !agent_data.runs.is_empty() {
            // Find first run (lowest timestamp)
            let first_run = agent_data.runs.iter().min_by_key(|r| r.timestamp).unwrap();

            // Find last run (highest timestamp)
            let last_run = agent_data.runs.iter().max_by_key(|r| r.timestamp).unwrap();

            let first_run_dt = chrono::DateTime::from_timestamp(first_run.timestamp as i64, 0)
                .unwrap_or_else(chrono::Utc::now)
                .with_timezone(&Local);
            let last_run_dt = chrono::DateTime::from_timestamp(last_run.timestamp as i64, 0)
                .unwrap_or_else(chrono::Utc::now)
                .with_timezone(&Local);

            println!(
                "  First Run:       {}",
                first_run_dt.format("%Y-%m-%d %H:%M:%S")
            );
            println!(
                "  Last Run:        {}",
                last_run_dt.format("%Y-%m-%d %H:%M:%S")
            );

            let last_run_duration = format_duration(last_run.duration_ms as f64 / 1000.0);
            println!("  Last Duration:   {}", last_run_duration);
        } else {
            println!("  First Run:       -");
            println!("  Last Run:        -");
            println!("  Last Duration:   -");
        }

        // Calculate timeout count (runs with status containing "timeout")
        let timeout_count = agent_data
            .runs
            .iter()
            .filter(|r| r.status.to_lowercase().contains("timeout"))
            .count() as u64;
        println!("  Timeout Count:   {}", timeout_count);

        // Queue wait time if available
        if let Some(wait_time) = agent_data.queue_wait_time_seconds {
            println!("  Queue Wait Time: {}s", wait_time);
        } else {
            println!("  Queue Wait Time: -");
        }

        // Skill installation metrics
        println!(
            "  Total Skills Installed: {}",
            agent_data.total_skills_installed
        );
        println!(
            "  Total Skills Failed:    {}",
            agent_data.total_skills_failed
        );
        println!(
            "  Runs with Skill Failures: {}",
            agent_data.runs_with_skill_failures
        );

        // Average skill install time
        if let Some(skills_time) = agent_data.skills_install_time_seconds {
            let total_skill_runs =
                agent_data.total_skills_installed as f64 + agent_data.total_skills_failed as f64;
            let avg_skill_time = if total_skill_runs > 0.0 {
                skills_time / total_skill_runs
            } else {
                0.0
            };
            println!(
                "  Avg Skill Install Time: {}",
                format_duration(avg_skill_time)
            );
        } else {
            println!("  Avg Skill Install Time: -");
        }

        // Show last 5 runs
        println!();
        println!("  Last {} runs:", 5.min(agent_data.runs.len()));
        println!(
            "  -------------------------------------------------------------------------------"
        );
        println!("  Run ID       | Timestamp          | Duration   | Status     | Error");
        println!(
            "  -------------------------------------------------------------------------------"
        );

        let mut recent_runs: Vec<_> = agent_data.runs.iter().collect();
        recent_runs.sort_by_key(|r| r.timestamp);
        recent_runs.reverse();

        for run in recent_runs.iter().take(5) {
            let run_dt = chrono::DateTime::from_timestamp(run.timestamp as i64, 0)
                .unwrap_or_else(chrono::Utc::now)
                .with_timezone(&Local);
            let timestamp_str = run_dt.format("%Y-%m-%d %H:%M:%S").to_string();
            let duration_str = format_duration(run.duration_ms as f64 / 1000.0);
            let error_msg = run
                .error_message
                .as_ref()
                .map(|e| truncate_text(e, 20))
                .unwrap_or_else(|| "-".to_string());

            println!(
                "  {:<12} | {} | {:>10} | {:<10} | {}",
                truncate_text(&run.run_id, 12),
                timestamp_str,
                duration_str,
                run.status,
                error_msg,
            );
        }
        println!(
            "  -------------------------------------------------------------------------------"
        );
        println!();
    }

    Ok(())
}

/// Truncate a string to a maximum length with "..." suffix
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}
