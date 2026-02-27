//! List command - Display configured agents and their details
//!
//! This module provides functionality to list all configured agents
//! and display their key properties in a formatted table.

use crate::config::Config;
use crate::ui::colors::{color_error, color_success, color_warning};
use chrono::Utc;
use chrono_tz::Tz;
use comfy_table::{Attribute, Cell, Color, Table};
use cron::Schedule;

/// Determine if a cron schedule is valid
fn is_valid_schedule(schedule: &str) -> bool {
    schedule.parse::<Schedule>().is_ok()
}

/// Get status text based on schedule validity
fn get_status(schedule: &str) -> String {
    if is_valid_schedule(schedule) {
        "Active".to_string()
    } else if schedule.is_empty() {
        "Never".to_string()
    } else {
        "Invalid".to_string()
    }
}

/// Parse timeout string to seconds
///
/// Supports:
/// - "30s" -> 30 seconds
/// - "5m" -> 5 minutes
/// - "1h" -> 1 hour
///
/// Returns None if parsing fails.
fn parse_timeout_secs(timeout: &str) -> Option<u64> {
    if timeout.is_empty() {
        return None;
    }

    let timeout = timeout.trim();
    let last_char = timeout.chars().last()?;
    let value_str = &timeout[..timeout.len() - 1];
    let value: u64 = value_str.parse().ok()?;

    match last_char {
        's' => Some(value),
        'm' => Some(value * 60),
        'h' => Some(value * 3600),
        _ => None,
    }
}

/// Truncate a string to a maximum length with "..." suffix
///
/// # Arguments
///
/// * `text` - The text to truncate
/// * `max_len` - Maximum length (including the "..." suffix)
///
/// # Returns
///
/// Truncated text if longer than max_len, otherwise original text
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

/// Get the timezone from config, defaulting to UTC
fn get_timezone(config: &Config) -> Result<Tz, String> {
    config
        .settings
        .as_ref()
        .ok_or_else(|| "Missing settings in config".to_string())
        .and_then(|s| {
            s.timezone
                .parse::<Tz>()
                .map_err(|e| format!("Invalid timezone '{}': {}", s.timezone, e))
        })
}

/// Calculate the next run time for a cron schedule
///
/// # Arguments
///
/// * `schedule` - The cron schedule string
/// * `timezone` - The timezone to use for calculation
///
/// # Returns
///
/// Formatted next run time string, or error message
fn calculate_next_run(schedule: &str, timezone: Tz) -> String {
    match schedule.parse::<Schedule>() {
        Ok(cron_schedule) => {
            // Get current time in the target timezone
            let now = Utc::now().with_timezone(&timezone);

            // Find the next scheduled time after now
            match cron_schedule.after(&now).next() {
                Some(next_time) => {
                    // Convert to the target timezone and format
                    let local_time = next_time.with_timezone(&timezone);
                    local_time.format("%Y-%m-%d %H:%M:%S").to_string()
                }
                None => "No future runs".to_string(),
            }
        }
        Err(_) => "Invalid cron".to_string(),
    }
}

/// List all configured agents and their details in a table format
///
/// This function displays a formatted table containing all agents defined in the
/// configuration. The table includes the following columns:
///
/// - **Name**: Agent name
/// - **Schedule**: Cron schedule expression
/// - **Prompt**: Inline prompt or prompt file reference (truncated to 50 chars)
/// - **Readonly**: Whether the agent runs in readonly mode
/// - **Timeout**: Timeout duration for the agent
/// - **Next Run**: Calculated next execution time in the configured timezone
///
/// # Arguments
///
/// * `config` - Reference to the loaded configuration containing agent definitions
///
/// # Returns
///
/// * `Ok(())` - Successfully displayed agent information
/// * `Err(String)` - Error occurred while listing agents (e.g., timezone parsing error)
///
/// # Examples
///
/// The function is typically called by the CLI list command:
/// ```bash
/// switchboard list
/// ```
///
/// # Notes
///
/// - If no agents are configured, prints "No agents configured" and returns successfully
/// - Uses the timezone from `config.settings.timezone` (defaults to UTC)
/// - Prompts are truncated to 50 characters with "..." suffix
/// - Invalid cron schedules display "Invalid cron" in the Next Run column
/// - Next run times are calculated based on current time in the configured timezone
pub fn list_agents(config: &Config) -> Result<(), String> {
    if config.agents.is_empty() {
        println!("No agents configured");
        return Ok(());
    }

    // Get the timezone from config
    let timezone = get_timezone(config)?;

    // Create a table with proper styling
    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_header(vec![
            Cell::new("Name")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Status")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Schedule")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Prompt")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Readonly")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Timeout")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new("Next Run")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
        ]);

    // Add a row for each agent
    for agent in &config.agents {
        // Get prompt text (inline or file-based)
        let prompt_text = match (&agent.prompt, &agent.prompt_file) {
            (Some(inline_prompt), None) => truncate_text(inline_prompt, 50),
            (None, Some(prompt_file)) => truncate_text(&format!("@{}", prompt_file), 50),
            (Some(_), Some(_)) => {
                // This should not happen due to validation, but handle gracefully
                "(config error)".to_string()
            }
            (None, None) => {
                // This should not happen due to validation, but handle gracefully
                "(none)".to_string()
            }
        };

        // Get readonly flag
        let readonly = agent
            .readonly
            .map(|r| r.to_string())
            .unwrap_or_else(|| "".to_string());

        // Get timeout
        let timeout = agent.timeout.as_deref().unwrap_or("");

        // Determine timeout color - yellow for short timeouts (< 5 minutes)
        let timeout_cell = if !timeout.is_empty() {
            // Parse timeout and check if less than 5 minutes (300 seconds)
            let is_short_timeout = parse_timeout_secs(timeout).map(|secs| secs < 300).unwrap_or(false);
            if is_short_timeout {
                // Less than 5 minutes - use warning color
                Cell::new(color_warning(timeout))
            } else {
                Cell::new(timeout)
            }
        } else {
            Cell::new(timeout)
        };

        // Calculate next run time
        let next_run = calculate_next_run(&agent.schedule, timezone);

        // Color-code next run column
        let next_run_cell = if next_run == "Invalid cron" {
            Cell::new(color_error(&next_run))
        } else if next_run == "No future runs" {
            Cell::new(color_warning(&next_run))
        } else {
            Cell::new(&next_run)
        };

        // Determine status based on schedule validity
        let status = get_status(&agent.schedule);
        let status_cell = if is_valid_schedule(&agent.schedule) {
            Cell::new(color_success(&status))
        } else if agent.schedule.is_empty() {
            Cell::new(color_warning(&status))
        } else {
            Cell::new(color_error(&status))
        };

        // Color-code schedule column
        let schedule_cell = if is_valid_schedule(&agent.schedule) {
            Cell::new(&agent.schedule)
        } else {
            Cell::new(color_error(&agent.schedule))
        };

        // Add row to table
        table.add_row(vec![
            Cell::new(&agent.name),
            status_cell,
            schedule_cell,
            Cell::new(&prompt_text),
            Cell::new(&readonly),
            timeout_cell,
            next_run_cell,
        ]);
    }

    // Print the table
    println!();
    println!("{}", table);
    println!();

    Ok(())
}
