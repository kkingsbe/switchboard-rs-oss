//! List command - Display configured agents and their details
//!
//! This module provides functionality to list all configured agents
//! and display their key properties in a formatted table.

use crate::config::Config;
use chrono::Utc;
use chrono_tz::Tz;
use comfy_table::{Attribute, Cell, Color, Table};
use cron::Schedule;

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
fn get_timezone(config: &Config) -> Tz {
    config
        .settings
        .as_ref()
        .and_then(|s| s.timezone.parse::<Tz>().ok())
        .unwrap_or(Tz::UTC)
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
    let timezone = get_timezone(config);

    // Create a table with proper styling
    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_header(vec![
            Cell::new("Name")
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

        // Calculate next run time
        let next_run = calculate_next_run(&agent.schedule, timezone);

        // Add row to table
        table.add_row(vec![
            Cell::new(&agent.name),
            Cell::new(&agent.schedule),
            Cell::new(&prompt_text),
            Cell::new(&readonly),
            Cell::new(timeout),
            Cell::new(&next_run),
        ]);
    }

    // Print the table
    println!();
    println!("{}", table);
    println!();

    Ok(())
}
