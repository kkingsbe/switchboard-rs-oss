//! CLI gateway command implementation
//!
//! This module contains the gateway subcommand handler for starting
//! the Discord Gateway service.

use crate::gateway::config::{GatewayConfig, GatewayConfigError};
use crate::gateway::pid::{PidFile, PidFileError};
use crate::gateway::server::GatewayServer;
use clap::{Parser, Subcommand};
use reqwest;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Error types for gateway command operations.
#[derive(Debug, Error)]
pub enum GatewayCommandError {
    /// Failed to load configuration.
    #[error("Failed to load configuration: {0}")]
    ConfigError(#[from] GatewayConfigError),

    /// Failed to start the gateway server.
    #[error("Failed to start gateway server: {0}")]
    ServerError(#[from] crate::gateway::server::GatewayServerError),

    /// Invalid configuration path.
    #[error("Invalid configuration path: {0}")]
    InvalidPath(String),

    /// Failed to initialize logging.
    #[error("Failed to initialize logging: {0}")]
    LoggingError(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Gateway is not running.
    #[error("Gateway is not running")]
    NotRunning,

    /// Failed to send signal to gateway process.
    #[error("Failed to send signal to gateway process: {0}")]
    SignalError(String),

    /// Gateway process did not exit in time.
    #[error("Gateway process did not exit in time")]
    Timeout,
}

/// Gateway subcommand.
#[derive(Parser)]
#[command(name = "gateway", about = "Start the Discord Gateway service")]
pub struct GatewayCommand {
    #[command(subcommand)]
    pub subcommand: GatewaySubcommand,
}

/// Gateway subcommand variants.
#[derive(Subcommand)]
pub enum GatewaySubcommand {
    /// Start the gateway server
    Up(GatewayUpArgs),
    /// Check gateway status
    Status(GatewayStatusArgs),
    /// Stop the gateway server
    Down(GatewayDownArgs),
}

/// Arguments for the gateway up command.
#[derive(Parser)]
#[command(about = "Start the Discord Gateway service")]
pub struct GatewayUpArgs {
    /// Path to the gateway configuration file (default: gateway.toml)
    #[arg(short, long, value_name = "PATH", default_value = "gateway.toml")]
    pub config: String,

    /// Run in detached mode (background)
    /// Note: This is a placeholder for future implementation.
    #[arg(long)]
    pub detach: bool,
}

/// Arguments for the gateway status command.
#[derive(Parser)]
#[command(about = "Check gateway status")]
pub struct GatewayStatusArgs {
    /// Path to the gateway configuration file (default: gateway.toml)
    #[arg(short, long, value_name = "PATH", default_value = "gateway.toml")]
    pub config: String,
}

/// Status response from the gateway /status endpoint.
#[derive(Debug, Deserialize)]
struct StatusResponse {
    /// Whether the gateway is running.
    #[allow(dead_code)]
    gateway_running: bool,
    /// Whether Discord is connected.
    discord_connected: bool,
    /// List of connected projects with their channel subscriptions.
    connected_projects: Vec<ProjectStatus>,
}

/// Project status from the gateway /status endpoint.
#[derive(Debug, Deserialize)]
struct ProjectStatus {
    /// Project name.
    name: String,
    /// List of channels the project is subscribed to.
    channels: Vec<String>,
}

/// Arguments for the gateway down command.
#[derive(Parser)]
#[command(about = "Stop the gateway server")]
pub struct GatewayDownArgs {
    /// Timeout in seconds to wait for graceful shutdown (default: 30)
    #[arg(short, long, value_name = "SECONDS", default_value = "30")]
    pub timeout: u64,

    /// Force kill the gateway if it doesn't stop gracefully
    #[arg(short, long)]
    pub force: bool,
}

/// Default log file path for gateway.
const DEFAULT_LOG_FILE: &str = ".switchboard/gateway.log";

/// Initialize logging with file appender.
///
/// This function sets up tracing to write to both stdout and a file.
/// The log file path is taken from the config, or defaults to `.switchboard/gateway.log`.
///
/// # Arguments
///
/// * `log_file` - Optional path to the log file from configuration
///
/// # Returns
///
/// A `Result` containing the `WorkerGuard` that must be kept alive, or an error.
fn init_file_logging(log_file: &Option<String>) -> Result<WorkerGuard, GatewayCommandError> {
    // Determine log file path: use config value or default
    let log_path = log_file
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOG_FILE));

    // Get the parent directory (e.g., ".switchboard" from ".switchboard/gateway.log")
    let log_dir = log_path.parent().ok_or_else(|| {
        GatewayCommandError::LoggingError(format!("Invalid log path: {}", log_path.display()))
    })?;

    // Create the directory if it doesn't exist
    if !log_dir.exists() {
        fs::create_dir_all(log_dir)?;
        tracing::debug!("Created log directory: {}", log_dir.display());
    }

    // Get the filename from the path
    let log_filename = log_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            GatewayCommandError::LoggingError(format!(
                "Invalid log filename: {}",
                log_path.display()
            ))
        })?;

    // Create file appender (using 'never' rotation for single file)
    let file_appender = tracing_appender::rolling::never(log_dir, log_filename);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Build subscriber with both stdout and file writers
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    // Use registry to combine layers properly
    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer);

    subscriber.try_init().map_err(|e| {
        GatewayCommandError::LoggingError(format!("Failed to set tracing subscriber: {}", e))
    })?;

    tracing::info!("Logging initialized: file={}", log_path.display());

    Ok(guard)
}

/// Run the gateway command.
///
/// This function loads the gateway configuration and starts the
/// GatewayServer with graceful shutdown handling.
///
/// # Arguments
///
/// * `args` - The [`GatewayCommand`] containing CLI arguments
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Configuration loading fails
/// - Configuration validation fails
/// - Server fails to start
pub async fn run_gateway(args: GatewayCommand) -> Result<(), Box<dyn std::error::Error>> {
    match args.subcommand {
        GatewaySubcommand::Up(up_args) => run_gateway_up(up_args).await,
        GatewaySubcommand::Status(status_args) => run_gateway_status(status_args).await,
        GatewaySubcommand::Down(down_args) => run_gateway_down(down_args).await,
    }
}

/// Run the gateway up subcommand.
async fn run_gateway_up(args: GatewayUpArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = &args.config;

    // Validate config path exists
    let path = Path::new(config_path);
    if !path.exists() {
        tracing::error!("Configuration file not found: {}", config_path);
        return Err(GatewayCommandError::InvalidPath(format!(
            "Configuration file not found: {}",
            config_path
        ))
        .into());
    }

    // Load gateway configuration
    tracing::info!("Loading gateway configuration from: {}", config_path);
    let config = GatewayConfig::load(Some(config_path))?;

    // Initialize file logging (must be done after config load to get log file path)
    let _guard = init_file_logging(&config.logging.file)?;

    // Log configuration details before moving
    let http_port = config.server.http_port;
    let ws_port = config.server.ws_port;
    let host = config.server.host.clone();
    tracing::info!(
        "Gateway configuration loaded: http_port={}, ws_port={}",
        http_port,
        ws_port
    );

    // Create and start the gateway server
    let server = GatewayServer::new(config.server.clone(), config);

    tracing::info!("Starting gateway server on {}:{}", host, http_port);

    // Run the server (handles graceful shutdown internally)
    server.run().await?;

    tracing::info!("Gateway server stopped");
    Ok(())
}

/// Run the gateway status subcommand.
///
/// This function checks if the gateway is currently running by checking
/// the PID file and verifying the process.
///
/// # Arguments
///
/// * `args` - The [`GatewayStatusArgs`] containing CLI arguments
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Configuration file validation fails
/// - PID file check fails unexpectedly
async fn run_gateway_status(args: GatewayStatusArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = &args.config;

    // Validate config path exists (optional - we can still check status without it)
    let path = Path::new(config_path);
    if !path.exists() {
        tracing::warn!(
            "Configuration file not found: {}, will check PID file anyway",
            config_path
        );
    }

    // Check if gateway is running using PID file
    let pid_path = PidFile::default_path();

    match PidFile::check_existing(&pid_path) {
        Ok(()) => {
            println!("Gateway: Stopped");
        }
        Err(PidFileError::AlreadyRunning(pid)) => {
            // Gateway is running, try to get additional status from HTTP endpoint
            let http_port = match GatewayConfig::load(Some(config_path)) {
                Ok(config) => config.server.http_port,
                Err(e) => {
                    tracing::warn!("Failed to load config for port: {}, using default", e);
                    8080 // default port
                }
            };

            let status_url = format!("http://localhost:{}/status", http_port);

            match reqwest::get(&status_url).await {
                Ok(response) => {
                    match response.json::<StatusResponse>().await {
                        Ok(status) => {
                            println!("Gateway: Running (PID: {})", pid);
                            println!(
                                "Discord: {}",
                                if status.discord_connected {
                                    "Connected"
                                } else {
                                    "Disconnected"
                                }
                            );

                            if status.connected_projects.is_empty() {
                                println!("Connected Projects: None");
                            } else {
                                println!("Connected Projects:");
                                for project in &status.connected_projects {
                                    let channels = if project.channels.is_empty() {
                                        "(no channels)".to_string()
                                    } else {
                                        project.channels.join(", ")
                                    };
                                    println!("  - {}: {}", project.name, channels);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse status response: {}", e);
                            println!("Gateway: Running (PID: {}) - Status unavailable", pid);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to query status endpoint: {}", e);
                    println!("Gateway: Running (PID: {}) - Status unavailable", pid);
                }
            }
        }
        Err(e) => {
            // For other errors (like IO errors), we'll still report not running
            // but log the error
            tracing::debug!("PID file check error: {}, reporting not running", e);
            println!("Gateway: Stopped");
        }
    }

    Ok(())
}

/// Run the gateway down subcommand.
///
/// This function stops the gateway server by reading the PID file,
/// sending SIGTERM to the process, and waiting for graceful shutdown.
///
/// # Arguments
///
/// * `args` - The [`GatewayDownArgs`] containing CLI arguments
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Gateway is not running (no PID file)
/// - Failed to send signal to gateway process
/// - Gateway did not exit in time (and --force was not specified)
async fn run_gateway_down(args: GatewayDownArgs) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::process::Command;
    use std::time::Duration;

    let pid_path = PidFile::default_path();

    // Check if PID file exists
    if !pid_path.exists() {
        return Err(GatewayCommandError::NotRunning.into());
    }

    // Read the PID from the file
    let file = File::open(&pid_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let pid_str = lines
        .next()
        .ok_or_else(|| GatewayCommandError::NotRunning)?
        .map_err(|e| GatewayCommandError::IoError(e))?;

    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|_| GatewayCommandError::NotRunning)?;

    // Check if process is actually running
    #[cfg(unix)]
    {
        // Check if process exists by sending signal 0
        let result = Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output();

        match result {
            Ok(output) if output.status.success() => {
                // Process exists, send SIGTERM
                println!("Sending SIGTERM to gateway (PID: {})...", pid);
                
                let kill_result = Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output();

                match kill_result {
                    Ok(_) => {
                        // Wait for the process to exit
                        let timeout_secs = args.timeout;
                        let poll_interval = Duration::from_secs(1);
                        let mut elapsed = 0u64;

                        while elapsed < timeout_secs {
                            let check = Command::new("kill")
                                .arg("-0")
                                .arg(pid.to_string())
                                .output();

                            match check {
                                Ok(output) if !output.status.success() => {
                                    // Process has exited
                                    println!("Gateway stopped successfully");
                                    
                                    // Clean up PID file
                                    if let Err(e) = PidFile::cleanup(&pid_path) {
                                        tracing::warn!("Failed to clean up PID file: {}", e);
                                    }
                                    
                                    return Ok(());
                                }
                                _ => {
                                    // Process still running, wait
                                    tokio::time::sleep(poll_interval).await;
                                    elapsed += 1;
                                }
                            }
                        }

                        // Timeout reached
                        if args.force {
                            println!("Gateway did not stop gracefully, forcing kill...");
                            let _ = Command::new("kill")
                                .arg("-9")
                                .arg(pid.to_string())
                                .output();
                            
                            // Wait a bit more for the process to be killed
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            
                            // Clean up PID file
                            if let Err(e) = PidFile::cleanup(&pid_path) {
                                tracing::warn!("Failed to clean up PID file: {}", e);
                            }
                            
                            println!("Gateway force stopped");
                            return Ok(());
                        } else {
                            return Err(GatewayCommandError::Timeout.into());
                        }
                    }
                    Err(e) => {
                        return Err(GatewayCommandError::SignalError(format!(
                            "Failed to send SIGTERM: {}",
                            e
                        )).into());
                    }
                }
            }
            _ => {
                // Process doesn't exist or we can't check
                return Err(GatewayCommandError::NotRunning.into());
            }
        }
    }

    #[cfg(not(unix))]
    {
        return Err("Signal handling is only supported on Unix systems".into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_gateway_command_parsing() {
        // Test that the command can be parsed
        let cmd = GatewayCommand::parse_from(["gateway", "up"]);
        match cmd.subcommand {
            GatewaySubcommand::Up(args) => {
                assert_eq!(args.config, "gateway.toml");
                assert!(!args.detach);
            }
            GatewaySubcommand::Status(_) => unreachable!("Expected Up subcommand"),
            GatewaySubcommand::Down(_) => unreachable!("Expected Up subcommand"),
        }
    }

    #[test]
    fn test_gateway_command_with_custom_config() {
        let cmd = GatewayCommand::parse_from(["gateway", "up", "--config", "custom.toml"]);
        match cmd.subcommand {
            GatewaySubcommand::Up(args) => {
                assert_eq!(args.config, "custom.toml");
            }
            GatewaySubcommand::Status(_) => unreachable!("Expected Up subcommand"),
            GatewaySubcommand::Down(_) => unreachable!("Expected Up subcommand"),
        }
    }

    #[test]
    fn test_gateway_command_with_detach() {
        let cmd = GatewayCommand::parse_from(["gateway", "up", "--detach"]);
        match cmd.subcommand {
            GatewaySubcommand::Up(args) => {
                assert!(args.detach);
            }
            GatewaySubcommand::Status(_) => unreachable!("Expected Up subcommand"),
            GatewaySubcommand::Down(_) => unreachable!("Expected Up subcommand"),
        }
    }

    #[test]
    fn test_gateway_up_args_defaults() {
        let args = GatewayUpArgs::parse_from(["up"]);
        assert_eq!(args.config, "gateway.toml");
        assert!(!args.detach);
    }

    #[test]
    fn test_gateway_status_args_defaults() {
        let args = GatewayStatusArgs::parse_from(["status"]);
        assert_eq!(args.config, "gateway.toml");
    }

    #[test]
    fn test_gateway_status_args_with_custom_config() {
        let args = GatewayStatusArgs::parse_from(["status", "--config", "custom.toml"]);
        assert_eq!(args.config, "custom.toml");
    }

    #[test]
    fn test_gateway_status_command_parsing() {
        let cmd = GatewayCommand::parse_from(["gateway", "status"]);
        match cmd.subcommand {
            GatewaySubcommand::Status(args) => {
                assert_eq!(args.config, "gateway.toml");
            }
            GatewaySubcommand::Up(_) => unreachable!("Expected Status subcommand"),
            GatewaySubcommand::Down(_) => unreachable!("Expected Status subcommand"),
        }
    }

    #[test]
    fn test_gateway_status_command_with_config() {
        let cmd = GatewayCommand::parse_from(["gateway", "status", "--config", "custom.toml"]);
        match cmd.subcommand {
            GatewaySubcommand::Status(args) => {
                assert_eq!(args.config, "custom.toml");
            }
            GatewaySubcommand::Up(_) => unreachable!("Expected Status subcommand"),
            GatewaySubcommand::Down(_) => unreachable!("Expected Status subcommand"),
        }
    }

    #[test]
    fn test_gateway_down_command_parsing() {
        let cmd = GatewayCommand::parse_from(["gateway", "down"]);
        match cmd.subcommand {
            GatewaySubcommand::Down(args) => {
                assert_eq!(args.timeout, 30);
                assert!(!args.force);
            }
            _ => unreachable!("Expected Down subcommand"),
        }
    }

    #[test]
    fn test_gateway_down_command_with_timeout() {
        let cmd = GatewayCommand::parse_from(["gateway", "down", "--timeout", "60"]);
        match cmd.subcommand {
            GatewaySubcommand::Down(args) => {
                assert_eq!(args.timeout, 60);
                assert!(!args.force);
            }
            _ => unreachable!("Expected Down subcommand"),
        }
    }

    #[test]
    fn test_gateway_down_command_with_force() {
        let cmd = GatewayCommand::parse_from(["gateway", "down", "--force"]);
        match cmd.subcommand {
            GatewaySubcommand::Down(args) => {
                assert_eq!(args.timeout, 30);
                assert!(args.force);
            }
            _ => unreachable!("Expected Down subcommand"),
        }
    }

    #[test]
    fn test_gateway_down_command_with_timeout_and_force() {
        let cmd = GatewayCommand::parse_from(["gateway", "down", "--timeout", "15", "--force"]);
        match cmd.subcommand {
            GatewaySubcommand::Down(args) => {
                assert_eq!(args.timeout, 15);
                assert!(args.force);
            }
            _ => unreachable!("Expected Down subcommand"),
        }
    }

    #[test]
    fn test_gateway_down_args_defaults() {
        let args = GatewayDownArgs::parse_from(["down"]);
        assert_eq!(args.timeout, 30);
        assert!(!args.force);
    }

    #[test]
    fn test_gateway_command_help() {
        // Test that --help works by parsing with help flag
        let result = GatewayUpArgs::try_parse_from(["up", "--help"]);
        // This should fail with clap's error message containing help text
        assert!(result.is_err());
    }

    /// Test that default log file path is returned when no file is configured.
    #[test]
    fn test_default_log_file_path() {
        let log_file: Option<String> = None;
        let expected_path = PathBuf::from(DEFAULT_LOG_FILE);

        // The log path should be the default when no file is configured
        let log_path = log_file
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_LOG_FILE));

        assert_eq!(log_path, expected_path);
    }

    /// Test that custom log file path is returned when configured.
    #[test]
    fn test_custom_log_file_path() {
        let log_file = Some("/var/log/my-gateway.log".to_string());
        let expected_path = PathBuf::from("/var/log/my-gateway.log");

        let log_path = log_file
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_LOG_FILE));

        assert_eq!(log_path, expected_path);
    }

    /// Test that parent directory is correctly extracted from log path.
    #[test]
    fn test_log_parent_directory() {
        let log_path = PathBuf::from(".switchboard/gateway.log");
        let log_dir = log_path.parent().expect("Should have parent directory");

        assert_eq!(log_dir, PathBuf::from(".switchboard"));
    }

    /// Test that log filename is correctly extracted from log path.
    #[test]
    fn test_log_filename() {
        let log_path = PathBuf::from(".switchboard/gateway.log");
        let log_filename = log_path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("Should have filename");

        assert_eq!(log_filename, "gateway.log");
    }
}
