//! CLI gateway command implementation
//!
//! This module contains the gateway subcommand handler for starting
//! the Discord Gateway service.

use crate::gateway::config::{GatewayConfig, GatewayConfigError};
use crate::gateway::pid::{PidFile, PidFileError};
use crate::gateway::server::GatewayServer;
use crate::logging::init_gateway_logging;
use clap::{Parser, Subcommand};
use reqwest;
use serde::Deserialize;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use tracing_appender::non_blocking::WorkerGuard;

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

/// Default log directory for gateway.
const DEFAULT_LOG_DIR: &str = ".switchboard/logs";

/// Initialize logging with file appender.
///
/// This function sets up tracing to write to both stdout and files in the
/// .switchboard/logs/ directory.
///
/// # Arguments
///
/// * `log_file` - Optional custom log file path from configuration (ignored for now,
///   logs always go to .switchboard/logs/)
///
/// # Returns
///
/// A tuple of `WorkerGuard`s (main guard, gateway guard) that must be kept alive, or an error.
fn init_file_logging(
    _log_file: &Option<String>,
) -> Result<(WorkerGuard, WorkerGuard), GatewayCommandError> {
    // Use the centralized logging in .switchboard/logs/
    let log_dir = PathBuf::from(DEFAULT_LOG_DIR);

    init_gateway_logging(log_dir).map_err(|e| GatewayCommandError::LoggingError(e.to_string()))
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
    // Check if we're running as a detached child process
    if std::env::var("SWITCHBOARD_DETACHED_CHILD").is_ok() {
        // We are the child process that was spawned for detached mode
        match args.subcommand {
            GatewaySubcommand::Up(up_args) => {
                // Run as child - this will write PID file and run server
                return run_gateway_as_child(up_args).await;
            }
            _ => {
                // For other subcommands, proceed normally
            }
        }
    }

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

    // Initialize file logging (writes to .switchboard/logs/ with daily rotation)
    let _guards = init_file_logging(&config.logging.file)?;

    // Log configuration details before moving
    let http_port = config.server.http_port;
    let ws_port = config.server.ws_port;
    let host = config.server.host.clone();
    tracing::info!(
        "Gateway configuration loaded: http_port={}, ws_port={}",
        http_port,
        ws_port
    );

    // Handle detached mode
    if args.detach {
        return run_gateway_detached(config, host, http_port, args.config).await;
    }

    // Create and start the gateway server
    let server = GatewayServer::new(config.server.clone(), config);

    tracing::info!("Starting gateway server on {}:{}", host, http_port);

    // Run the server (handles graceful shutdown internally)
    server.run().await?;

    tracing::info!("Gateway server stopped");
    Ok(())
}

/// Run the gateway in detached (background) mode.
///
/// This function daemonizes the process on Unix systems by spawning a child process,
/// redirecting stdio to /dev/null, writing the PID file, and running
/// the server in the background.
///
/// # Arguments
///
/// * `config` - The gateway configuration
/// * `host` - The host address
/// * `http_port` - The HTTP port
/// * `config_path` - The path to the config file
///
/// # Returns
///
/// Returns `Ok(())` on success (parent returns immediately, child runs server),
/// or an error if daemonization fails.
async fn run_gateway_detached(
    config: GatewayConfig,
    host: String,
    http_port: u32,
    config_path: String,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        // Get current executable path
        let exe_path = std::env::current_exe()?;

        // Build the command to spawn a detached child
        let mut cmd = std::process::Command::new(&exe_path);
        cmd.arg("gateway")
            .arg("up")
            .arg("--config")
            .arg(&config_path)
            // Use internal flag to indicate this is the child process
            .env("SWITCHBOARD_DETACHED_CHILD", "1");

        // Set up detached process on Unix
        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .process_group(0); // Create new process group

        // Spawn the child process
        let child = cmd.spawn()?;
        let child_pid = child.id();

        println!("Gateway started in detached mode (PID: {})", child_pid);
        Ok(())
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, we can't easily daemonize
        // For now, just run in foreground with a warning
        tracing::warn!("Detached mode is not fully supported on this platform. Running in foreground.");
        
        let server = GatewayServer::new(config.server.clone(), config);
        tracing::info!("Starting gateway server on {}:{}", host, http_port);
        server.run().await?;
        tracing::info!("Gateway server stopped");
        Ok(())
    }
}

/// Internal function to run gateway as a detached child process.
/// This is called when SWITCHBOARD_DETACHED_CHILD environment variable is set.
async fn run_gateway_as_child(
    args: GatewayUpArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = &args.config;

    // Validate config path exists
    let path = Path::new(config_path);
    if !path.exists() {
        return Err(GatewayCommandError::InvalidPath(format!(
            "Configuration file not found: {}",
            config_path
        ))
        .into());
    }

    // Load gateway configuration
    tracing::info!("Loading gateway configuration from: {}", config_path);
    let config = GatewayConfig::load(Some(config_path))?;

    // Initialize file logging
    let _guards = init_file_logging(&config.logging.file)?;

    let http_port = config.server.http_port;
    let host = config.server.host.clone();

    tracing::info!(
        "Gateway configuration loaded: http_port={}, ws_port={}",
        http_port,
        config.server.ws_port
    );

    // Write PID file before starting the server
    let pid_path = PidFile::default_path();
    PidFile::write_pid(&pid_path)?;

    // Create and start the gateway server
    let server = GatewayServer::new(config.server.clone(), config);

    tracing::info!("Starting gateway server on {}:{}", host, http_port);

    // Run the server
    server.run().await?;

    // Clean up PID file on shutdown
    let _ = PidFile::cleanup(&pid_path);

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
                Ok(response) => match response.json::<StatusResponse>().await {
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
                },
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
        .ok_or(GatewayCommandError::NotRunning)?
        .map_err(GatewayCommandError::IoError)?;

    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|_| GatewayCommandError::NotRunning)?;

    // Check if process is actually running
    #[cfg(unix)]
    {
        // Check if process exists by sending signal 0
        let result = Command::new("kill").arg("-0").arg(pid.to_string()).output();

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
                            let check =
                                Command::new("kill").arg("-0").arg(pid.to_string()).output();

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
                            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).output();

                            // Wait a bit more for the process to be killed
                            tokio::time::sleep(Duration::from_secs(1)).await;

                            // Clean up PID file
                            if let Err(e) = PidFile::cleanup(&pid_path) {
                                tracing::warn!("Failed to clean up PID file: {}", e);
                            }

                            println!("Gateway force stopped");
                            Ok(())
                        } else {
                            Err(GatewayCommandError::Timeout.into())
                        }
                    }
                    Err(e) => Err(GatewayCommandError::SignalError(format!(
                        "Failed to send SIGTERM: {}",
                        e
                    ))
                    .into()),
                }
            }
            _ => {
                // Process doesn't exist or we can't check
                Err(GatewayCommandError::NotRunning.into())
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

    /// Test that default log directory is used when no file is configured.
    #[test]
    fn test_default_log_directory() {
        // The new implementation uses .switchboard/logs/ by default
        let expected_dir = PathBuf::from(DEFAULT_LOG_DIR);

        // Verify the default log directory constant is correct
        assert_eq!(expected_dir, PathBuf::from(".switchboard/logs"));
    }

    /// Test that log directory path is correctly constructed.
    #[test]
    fn test_log_directory_path() {
        let log_dir = PathBuf::from(DEFAULT_LOG_DIR);

        // Verify the log directory path
        assert_eq!(log_dir, PathBuf::from(".switchboard/logs"));
    }
}
