#![allow(clippy::async_yields_async)]
//! Logs command - View logs from agent executions
//!
//! This module provides functionality to view and follow logs
//! from agent executions.

use crate::config::Config;
use clap::Parser;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

/// List all available log files in the project
///
/// # Arguments
///
/// * `config` - The configuration containing agent definitions
///
/// # Returns
///
/// * `Vec<String>` - List of existing log file paths
fn list_available_log_files(config: &Config) -> Vec<String> {
    let mut available_logs = Vec::new();

    // Get log_dir from config (default: ".switchboard/logs")
    let log_dir = config
        .settings
        .as_ref()
        .map(|s| s.log_dir.as_str())
        .unwrap_or(".switchboard/logs");

    // Check for scheduler log
    let scheduler_log_path = format!("{}/switchboard.log", log_dir);
    if std::path::Path::new(&scheduler_log_path).exists() {
        available_logs.push(scheduler_log_path);
    }

    // Check for agent logs in nested structure: <log_dir>/<agent-name>/<timestamp>.log
    for agent in &config.agents {
        let agent_dir = format!("{}/{}", log_dir, agent.name);
        let agent_dir_path = std::path::Path::new(&agent_dir);

        // Check if agent subdirectory exists
        if agent_dir_path.exists() && agent_dir_path.is_dir() {
            // Scan for all .log files in the agent's subdirectory
            if let Ok(entries) = std::fs::read_dir(agent_dir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    // Only add .log files
                    if path.extension().and_then(|s| s.to_str()) == Some("log") {
                        if let Some(path_str) = path.to_str() {
                            available_logs.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }

    available_logs
}

/// Logs command arguments for viewing agent logs
///
/// This struct defines the command-line arguments for the logs command,
/// which allows viewing logs from agent executions or the scheduler.
///
/// # Fields
///
/// * `agent_name` - Optional agent name to filter logs for a specific agent.
///   If not provided, shows the scheduler log.
/// * `scheduler` - Explicit flag to show scheduler logs (use with or without agent_name)
/// * `follow` - Whether to follow log output (similar to `tail -f`). Continues
///   displaying new log lines as they are written.
/// * `tail` - Optional number of lines to show from the end of the log file.
///   If not specified, shows all log lines.
///
/// # Examples
///
/// View all scheduler logs:
/// ```bash
/// switchboard logs
/// ```
///
/// Explicitly view scheduler logs:
/// ```bash
/// switchboard logs --scheduler
/// ```
///
/// View logs for a specific agent:
/// ```bash
/// switchboard logs my-agent
/// ```
///
/// Follow logs in real-time:
/// ```bash
/// switchboard logs --follow
/// ```
///
/// Show last 100 lines:
/// ```bash
/// switchboard logs --tail 100
/// ```
///
/// Combine options:
/// ```bash
/// switchboard logs my-agent --follow --tail 50
/// ```
///
/// # Skill Installation Logs
///
/// Skill installation logs use special prefixes for filtering:
///
/// - `[SKILL INSTALL]` - Successful skill installation messages
/// - `[SKILL INSTALL ERROR]` - Skill installation failures
/// - `[SKILL INSTALL STDERR]` - Error output from skill installation
///
/// ```bash
/// # Filter skill installation logs
/// switchboard logs my-agent | grep "\[SKILL INSTALL\]"
///
/// # View only skill installation errors
/// switchboard logs my-agent | grep "\[SKILL INSTALL ERROR\]"
///
/// # View skill installation stderr
/// switchboard logs my-agent | grep "\[SKILL INSTALL STDERR\]"
/// ```
///
/// # Notes
///
/// - Agent logs are stored in `<log_dir>/<agent-name>/<timestamp>.log`
/// - Scheduler log is stored in `<log_dir>/switchboard.log`
/// - When filtering by agent, the most recent log file is selected
/// - If the specified log file doesn't exist, available logs are listed
/// - Skill installation logs are included in agent logs and use special prefixes
#[derive(Parser, Debug)]
#[command(about = "View logs from agent executions")]
pub struct LogsArgs {
    /// Optional agent name to filter logs
    #[arg(value_name = "AGENT")]
    pub agent_name: Option<String>,

    /// Show scheduler logs (explicit)
    #[arg(short, long)]
    pub scheduler: bool,

    /// Follow log output (like tail -f)
    #[arg(short, long)]
    pub follow: bool,

    /// Number of lines to show from the end of the logs
    #[arg(short = 'n', long, value_name = "N")]
    pub tail: Option<usize>,
}

/// Resolve the log file path based on agent name
///
/// # Arguments
///
/// * `agent_name` - Optional agent name to filter logs
/// * `scheduler` - Whether to show scheduler logs explicitly
/// * `config` - The configuration containing agent definitions
///
/// # Returns
///
/// * `Ok(String)` - The log file path ("<log_dir>/switchboard.log" for scheduler, "<log_dir>/<agent-name>/<timestamp>.log" for agents)
/// * `Err(Box<dyn std::error::Error>)` - Error if agent not found or no log files exist
fn resolve_log_path(
    agent_name: Option<&String>,
    scheduler: bool,
    config: &Config,
) -> Result<String, Box<dyn std::error::Error>> {
    // Determine if we should show scheduler logs
    // Show scheduler if: no agent_name provided, OR scheduler flag is set
    let show_scheduler = agent_name.is_none() || scheduler;

    // If showing scheduler, return scheduler log path
    if show_scheduler {
        let log_dir = config
            .settings
            .as_ref()
            .map(|s| s.log_dir.as_str())
            .unwrap_or(".switchboard/logs");
        return Ok(format!("{}/switchboard.log", log_dir));
    }

    // Otherwise, show agent log
    let name = agent_name.unwrap();

    // Check if the agent exists in config.agents
    let agent_exists = config.agents.iter().any(|agent| &agent.name == name);
    if !agent_exists {
        return Err(format!("Agent '{}' not found in config", name).into());
    }

    // Get log_dir from config (default: ".switchboard/logs")
    let log_dir = config
        .settings
        .as_ref()
        .map(|s| s.log_dir.as_str())
        .unwrap_or(".switchboard/logs");

    // Construct the agent's log directory path: <log_dir>/<agent-name>/
    let agent_dir = format!("{}/{}", log_dir, name);
    let agent_dir_path = std::path::Path::new(&agent_dir);

    // Check if agent subdirectory exists
    if !agent_dir_path.exists() || !agent_dir_path.is_dir() {
        return Err(
            format!("No log directory found for agent '{}': {}", name, agent_dir).into(),
        );
    }

    // Scan for all .log files in the agent's subdirectory
    let mut log_files: Vec<(String, std::time::SystemTime)> = Vec::new();

    if let Ok(entries) = std::fs::read_dir(agent_dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Only process .log files
            if path.extension().and_then(|s| s.to_str()) == Some("log") {
                // Get the file's modification time
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Some(path_str) = path.to_str() {
                            log_files.push((path_str.to_string(), modified));
                        }
                    }
                }
            }
        }
    }

    // Check if any log files were found
    if log_files.is_empty() {
        return Err(format!(
            "No log files found for agent '{}' in directory: {}",
            name, agent_dir
        )
        .into());
    }

    // Sort by modification time (most recent first) and return the most recent log file
    log_files.sort_by_key(|(_, time)| std::cmp::Reverse(*time));
    Ok(log_files[0].0.clone())
}

/// Read log file lines
///
/// # Arguments
///
/// * `path` - Path to the log file
/// * `tail` - Optional number of lines to show from the end
///
/// # Returns
///
/// * `Ok(Vec<String>)` - Vector of log lines
/// * `Err(Box<dyn std::error::Error>)` - Error if file not found or cannot be read
fn read_log_file(
    path: &str,
    tail: Option<usize>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Check if file exists
    let file_path = std::path::Path::new(path);
    if !file_path.exists() {
        return Err(format!("Log file not found: {}", path).into());
    }

    // Open the file
    let file = File::open(path)?;

    // Create a buffered reader
    let reader = BufReader::new(file);

    // Read all lines
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    // Return all lines or last n lines based on tail parameter
    match tail {
        Some(n) => {
            let start = if lines.len() > n { lines.len() - n } else { 0 };
            Ok(lines[start..].to_vec())
        }
        None => Ok(lines),
    }
}

/// Follow a log file, printing new lines as they are written
///
/// # Arguments
///
/// * `path` - Path to the log file to follow
/// * `agent_name` - Optional agent name for prefixing output
///
/// # Returns
///
/// * `Ok(())` - Following completed successfully (e.g., on Ctrl+C)
/// * `Err(Box<dyn std::error::Error>)` - Error occurred during following
async fn follow_log_file(
    path: &str,
    agent_name: Option<&String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Open the file and seek to the end
    let mut file = OpenOptions::new().read(true).open(path)?;
    file.seek(SeekFrom::End(0))?;

    // Track the last known file size and modification time
    let mut last_size = file.metadata()?.len();
    let mut last_modified = file.metadata()?.modified()?;
    let file_path = std::path::Path::new(path);

    // Create a signal handler for Ctrl+C
    let ctrl_c_signal = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c_signal);

    #[cfg(unix)]
    {
        // Create a signal handler for SIGTERM (Unix only)
        // Use .ok() to convert Result to Option - if setup fails, we continue with just Ctrl+C
        let sigterm = signal(SignalKind::terminate()).ok();
        tokio::pin!(sigterm);

        loop {
            tokio::select! {
                    _ = &mut ctrl_c_signal => {
                        // Ctrl+C pressed, exit gracefully
                        break;
                    }
                    _ = async {
                        // Only wait for SIGTERM if it was successfully set up
                        if let Some(ref mut sig) = *sigterm {
                            let _ = sig.recv().await;
                        }
                        // Never complete if sigterm is None (handler setup failed)
                        // This intentionally yields a pending future that never resolves,
                        // ensuring the loop continues until Ctrl+C is received
                        std::future::pending::<()>()
                    } => {
                        // SIGTERM received, exit gracefully
                        break;
                    }
                    _ = sleep(Duration::from_millis(100)) => {
                    // Get current file metadata
                    match file_path.metadata() {
                        Ok(metadata) => {
                            let current_size = metadata.len();
                            let current_modified = metadata.modified()?;

                            // Handle log rotation (modification time changed to an earlier time or size decreased)
                            // This cross-platform approach works on both Unix and Windows
                            if current_modified < last_modified || current_size < last_size {
                                // Log was rotated, print a message and reopen from beginning
                                println!("Log rotated, reading from new file");
                                drop(file);
                                file = OpenOptions::new().read(true).open(path)?;
                                file.seek(SeekFrom::Start(0))?;
                                last_modified = current_modified;
                                last_size = 0; // Read from the beginning of the new file
                                continue;
                            }

                            // If size increased, read and print new lines
                            if current_size > last_size {
                                let mut reader = BufReader::new(&mut file);
                                let mut line = String::new();

                                while reader.read_line(&mut line)? > 0 {
                                    match agent_name {
                                        Some(name) => println!("[{}] {}", name, line.trim_end()),
                                        None => println!("{}", line.trim_end()),
                                    }
                                    line.clear();
                                }

                                last_size = current_size;
                                last_modified = current_modified;
                            }
                        }
                        Err(_) => {
                            // File might have been deleted or the new file doesn't exist yet
                            // Wait and retry (log rotation in progress)
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        loop {
            tokio::select! {
                    _ = &mut ctrl_c_signal => {
                        // Ctrl+C pressed, exit gracefully
                        break;
                    }
                    _ = sleep(Duration::from_millis(100)) => {
                    // Get current file metadata
                    match file_path.metadata() {
                        Ok(metadata) => {
                            let current_size = metadata.len();
                            let current_modified = metadata.modified()?;

                            // Handle log rotation (modification time changed to an earlier time or size decreased)
                            // This cross-platform approach works on both Unix and Windows
                            if current_modified < last_modified || current_size < last_size {
                                // Log was rotated, print a message and reopen from beginning
                                println!("Log rotated, reading from new file");
                                drop(file);
                                file = OpenOptions::new().read(true).open(path)?;
                                file.seek(SeekFrom::Start(0))?;
                                last_modified = current_modified;
                                last_size = 0; // Read from the beginning of the new file
                                continue;
                            }

                            // If size increased, read and print new lines
                            if current_size > last_size {
                                let mut reader = BufReader::new(&mut file);
                                let mut line = String::new();

                                while reader.read_line(&mut line)? > 0 {
                                    match agent_name {
                                        Some(name) => println!("[{}] {}", name, line.trim_end()),
                                        None => println!("{}", line.trim_end()),
                                    }
                                    line.clear();
                                }

                                last_size = current_size;
                                last_modified = current_modified;
                            }
                        }
                        Err(_) => {
                            // File might have been deleted or the new file doesn't exist yet
                            // Wait and retry (log rotation in progress)
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Execute the logs command
///
/// This function loads the configuration, resolves the appropriate log file path,
/// reads and displays the log content, and optionally follows the log file for
/// new entries.
///
/// # Arguments
///
/// * `args` - The parsed logs command arguments containing agent_name, follow, and tail options
///
/// # Returns
///
/// * `Ok(())` - Command executed successfully
/// * `Err(Box<dyn std::error::Error>)` - Error occurred during execution:
///   - Configuration file not found or invalid
///   - Agent not found in configuration
///   - Log file not found
///   - Log directory not found for agent
///
/// # Examples
///
/// View all scheduler logs:
/// ```bash
/// switchboard logs
/// ```
///
/// View logs for a specific agent and follow:
/// ```bash
/// switchboard logs my-agent --follow
/// ```
///
/// Show last 50 lines:
/// ```bash
/// switchboard logs --tail 50
/// ```
///
/// # Notes
///
/// - Loads configuration from `./switchboard.toml`
/// - If no agent_name is specified, displays the scheduler log at `<log_dir>/switchboard.log`
/// - If agent_name is specified, displays the most recent log file for that agent
/// - When follow is enabled, continues reading until Ctrl+C or SIGTERM is received
/// - Supports log rotation by detecting file size or modification time changes
/// - If the specified log file doesn't exist, lists all available log files
/// - When viewing agent logs, prefixes each line with `[agent-name]`
pub async fn run(args: LogsArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Determine the config file path
    let config_path = PathBuf::from("./switchboard.toml");

    // Load the configuration from the file
    let config = Config::from_toml(&config_path)?;

    // Determine the log file path to read
    let log_path = resolve_log_path(args.agent_name.as_ref(), args.scheduler, &config)?;

    // Check if the log file exists before attempting to read it
    let file_path = std::path::Path::new(&log_path);
    if !file_path.exists() {
        // Log file not found, list available logs and return error
        match &args.agent_name {
            None => {
                // Scheduler log not found
                println!("Scheduler log file not found: {}", log_path);
            }
            Some(_agent_name) => {
                // Agent log not found
                println!("Log file not found: {}", log_path);
            }
        }

        // List available log files
        let available_logs = list_available_log_files(&config);
        if available_logs.is_empty() {
            println!("No log files found in the project.");
        } else {
            println!("Available log files: {}", available_logs.join(", "));
        }

        return Err(format!("Log file not found: {}", log_path).into());
    }

    // Read the log file
    let lines = read_log_file(&log_path, args.tail)?;

    // Print each line to stdout
    for line in lines {
        match &args.agent_name {
            Some(agent_name) => println!("[{}] {}", agent_name, line),
            None => println!("{}", line),
        }
    }

    // If follow mode is enabled, start following the log file
    if args.follow {
        follow_log_file(&log_path, args.agent_name.as_ref()).await?;
    }

    Ok(())
}
