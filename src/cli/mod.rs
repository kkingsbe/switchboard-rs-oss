//! CLI Layer - Parse CLI commands and dispatch to appropriate modules
//!
//! This module provides:
//! - Command argument parsing using clap
//! - Command dispatch to scheduler, docker, and other modules
//! - Individual command implementations (up, run, build, list, logs, down, validate)
//!
//! **Current Status:**
//! - All command structures fully implemented with clap derive macros
//! - run_up() fully implemented with scheduler initialization and execution
//! - run_run() fully implemented with complete agent execution flow
//! - run_validate() implemented with config validation logic
//! - All command handlers are fully implemented

use crate::commands::logs::{run as logs_run, LogsArgs};
use crate::commands::{list_agents, metrics, BuildCommand, SkillsCommand, ValidateCommand};
use crate::config::{Config, ConfigError};
use crate::docker::run::types::ContainerConfig;
use crate::docker::{check_docker_available, run_agent, DockerClient};
use crate::logger::Logger;
use crate::logging::init_logging;
use crate::metrics::MetricsStore;
use crate::scheduler::Scheduler;
use crate::scheduler::SchedulerError;
use crate::traits::{DockerClientTrait, ProcessExecutorTrait, RealProcessExecutor};
use crate::ui::ColorMode;
use bollard::container::{ListContainersOptions, StopContainerOptions};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

#[cfg(feature = "discord")]
use crate::config::env::resolve_config_value;
#[cfg(feature = "discord")]
use crate::discord::config::LlmConfig;
#[cfg(feature = "discord")]
use crate::discord::start_discord_listener_with_shutdown;
#[cfg(feature = "discord")]
use tokio::sync::broadcast;

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

/// Check if Discord configuration is present in environment variables or switchboard.toml
///
/// First checks for environment variables, then falls back to switchboard.toml config.
/// Returns true only if Discord can be configured.
#[cfg(feature = "discord")]
fn is_discord_configured() -> bool {
    // First check if environment variables are already set
    if std::env::var("DISCORD_TOKEN").is_ok()
        && std::env::var("OPENROUTER_API_KEY").is_ok()
        && std::env::var("DISCORD_CHANNEL_ID").is_ok()
    {
        tracing::debug!("Discord configured via environment variables");
        return true;
    }

    // Try to load Discord config from switchboard.toml
    match load_discord_config_from_toml("./switchboard.toml") {
        Ok(full_config) => {
            if full_config.enabled {
                tracing::debug!("Discord config loaded from switchboard.toml: enabled={}, token_env={}, channel_id={}", 
                    full_config.enabled, full_config.token_env, full_config.channel_id);

                // Handle token: use resolve_config_value to handle ${VAR} syntax
                // This checks both switchboard.env and system environment variables
                let token = resolve_config_value(&full_config.token_env);

                if !token.is_empty() {
                    env::set_var("DISCORD_TOKEN", &token);
                }

                // Set channel ID (use as-is from TOML, or resolve if it has ${VAR} syntax)
                let channel_id = resolve_config_value(&full_config.channel_id);
                if !channel_id.is_empty() {
                    env::set_var("DISCORD_CHANNEL_ID", &channel_id);
                }

                // Handle LLM API key
                let api_key = if let Some(llm_config) = &full_config.llm {
                    // Use resolve_config_value to handle ${VAR} syntax
                    resolve_config_value(&llm_config.api_key_env)
                } else {
                    String::new()
                };

                if !api_key.is_empty() {
                    env::set_var("OPENROUTER_API_KEY", &api_key);
                }

                // Verify we now have all required env vars
                let configured = std::env::var("DISCORD_TOKEN").is_ok()
                    && std::env::var("OPENROUTER_API_KEY").is_ok()
                    && std::env::var("DISCORD_CHANNEL_ID").is_ok();

                tracing::debug!(
                    "Discord env vars set: token={}, api_key={}, channel_id={}",
                    std::env::var("DISCORD_TOKEN").is_ok(),
                    std::env::var("OPENROUTER_API_KEY").is_ok(),
                    std::env::var("DISCORD_CHANNEL_ID").is_ok()
                );

                configured
            } else {
                tracing::debug!("Discord disabled in switchboard.toml");
                false
            }
        }
        Err(e) => {
            tracing::debug!("Failed to load Discord config from switchboard.toml: {}", e);
            false
        }
    }
}

/// Full Discord configuration including LLM settings
#[cfg(feature = "discord")]
#[derive(Debug, Clone, serde::Deserialize)]
struct DiscordFullConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub token_env: String,
    #[serde(default)]
    pub channel_id: String,
    #[serde(default)]
    pub llm: Option<LlmConfig>,
}

/// Discord configuration specifically for the [discord] section in switchboard.toml
#[cfg(feature = "discord")]
#[derive(Debug, Clone, serde::Deserialize)]
struct DiscordTomlSection {
    #[serde(default)]
    pub discord: Option<DiscordFullConfig>,
}

/// Load Discord configuration from switchboard.toml file
#[cfg(feature = "discord")]
fn load_discord_config_from_toml(
    path: &str,
) -> Result<DiscordFullConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;

    // First try to parse as a struct that contains [discord] section
    if let Ok(section_config) = toml::from_str::<DiscordTomlSection>(&content) {
        if let Some(discord_config) = section_config.discord {
            tracing::debug!("Successfully parsed [discord] section from switchboard.toml");
            return Ok(discord_config);
        }
    }

    // Fallback: try parsing as DiscordFullConfig at root level (for backward compatibility)
    let config: DiscordFullConfig = toml::from_str(&content)?;
    Ok(config)
}

/// Create a default process executor
///
/// This function provides a default RealProcessExecutor for use in CLI commands.
/// It can be replaced with a mock implementation for testing.
fn default_executor() -> Arc<dyn ProcessExecutorTrait> {
    Arc::new(RealProcessExecutor::new())
}

/// Switchboard - AI coding agent prompt scheduler
///
/// Schedule AI coding agent prompts via Docker containers
#[derive(Parser)]
#[command(name = "switchboard")]
pub struct Cli {
    /// Path to the configuration file (default: ./switchboard.toml)
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<String>,

    /// Control colored output (auto, always, never)
    #[arg(long, value_enum, default_value = "auto")]
    pub color: ColorMode,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands for Switchboard
///
/// This enum represents all subcommands available in the Switchboard CLI. Each variant
/// corresponds to a specific operation and contains the necessary arguments for that
/// command.
///
/// # Variants
///
/// - [`Commands::Up`] - Build agent image and start the scheduler
/// - [`Commands::Run`] - Immediately execute a single agent (bypassing scheduler)
/// - [`Commands::Build`] - Build or rebuild the agent Docker image
/// - [`Commands::List`] - Print all configured agents, their schedules, and prompts
/// - [`Commands::Logs`] - View logs from agent runs
/// - [`Commands::Metrics`] - Display agent execution metrics
/// - [`Commands::Down`] - Stop scheduler and any running agent containers
/// - [`Commands::Validate`] - Parse and validate configuration file
/// - [`Commands::Skills`] - Manage Kilo skills
/// - [`Commands::Status`] - Check scheduler health and status
///
/// # Command Dispatch
///
/// When the CLI is invoked, clap parses the command-line arguments into a [`Commands`]
/// variant, which is then dispatched to the appropriate handler function:
///
/// - [`Commands::Up`] → [`run_up()`]
/// - [`Commands::Run`] → [`run_run()`]
/// - [`Commands::Build`] → [`run_build()`]
/// - [`Commands::List`] → [`run_list()`]
/// - [`Commands::Logs`] → [`run_logs()`]
/// - [`Commands::Metrics`] → [`run_metrics()`]
/// - [`Commands::Down`] → [`run_down()`]
/// - [`Commands::Validate`] → [`run_validate()`]
/// - [`Commands::Skills`] → [`run_skills()`]
/// - [`Commands::Status`] → [`run_status()`]
///
/// # Examples
///
/// Each command can be invoked from the command line:
///
/// ```text
/// switchboard up              # Start the scheduler
/// switchboard run dev-agent   # Run a specific agent immediately
/// switchboard list            # List all configured agents
/// switchboard logs dev-agent  # View logs for an agent
/// switchboard metrics         # Display execution metrics
/// switchboard down            # Stop the scheduler
/// switchboard validate        # Validate configuration
/// switchboard skills list     # List available skills
/// switchboard status          # Check scheduler status
/// ```
#[derive(Subcommand)]
pub enum Commands {
    /// Build agent image and start scheduler
    Up(UpCommand),

    /// Immediately execute a single agent
    Run(RunCommand),

    /// Build or rebuild agent Docker image
    Build(BuildCommand),

    /// Print all configured agents, their schedules, and prompts
    List,

    /// View logs from agent runs
    Logs(LogsCommand),

    /// Display agent execution metrics
    Metrics(MetricsCommand),

    /// Stop scheduler and any running agent containers
    Down(DownCommand),

    /// Parse and validate config file
    Validate(ValidateCommand),

    /// Manage Kilo skills
    Skills(SkillsCommand),

    /// Check scheduler health and status
    Status,
}

/// Build agent image and start scheduler
#[derive(Parser)]
pub struct UpCommand {
    /// Run in background
    #[arg(short, long)]
    pub detach: bool,
}

/// Immediately execute a single agent
#[derive(Parser)]
pub struct RunCommand {
    /// Name of the agent to execute
    #[arg(value_name = "AGENT_NAME")]
    pub agent_name: String,
}

/// View logs from agent runs
#[derive(Parser)]
pub struct LogsCommand {
    /// Name of the agent to view logs for (optional)
    #[arg(value_name = "AGENT_NAME")]
    pub agent_name: Option<String>,

    /// Stream logs as they are generated
    #[arg(short, long)]
    pub follow: bool,

    /// Show the last N lines (default: 50)
    #[arg(short, long, value_name = "N", default_value_t = 50)]
    pub tail: usize,
}

/// Display agent execution metrics
#[derive(Parser)]
pub struct MetricsCommand {
    /// Show detailed metrics view
    #[arg(short, long)]
    pub detailed: bool,

    /// Show detailed metrics for a specific agent
    #[arg(long, value_name = "NAME")]
    pub agent: Option<String>,

    /// Path to configuration file
    #[arg(long, short = 'c', value_name = "PATH")]
    pub config: Option<String>,
}

/// Stop scheduler and any running agent containers
#[derive(Parser)]
pub struct DownCommand {
    /// Clean up .switchboard directory (logs, PID files, etc.)
    #[arg(short = 'c', long)]
    pub cleanup: bool,
}

/// Check if a Docker image exists locally
///
/// This function uses the bollard Docker client API to list images and check
/// if the specified image (name:tag) exists in the local Docker image cache.
///
/// # Arguments
///
/// * `client` - Reference to the DockerClient instance
/// * `image_name` - The Docker image name (e.g., "switchboard-agent")
/// * `image_tag` - The Docker image tag (e.g., "latest")
///
/// # Returns
///
/// Returns `Ok(true)` if the image exists locally, `Ok(false)` if it doesn't.
/// Returns an error if the Docker API call fails.
async fn check_image_exists(
    client: &DockerClient,
    image_name: &str,
    image_tag: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let docker = client
        .docker()
        .ok_or_else(|| "Docker client unavailable".to_string())?;

    // List all local images
    let images = docker
        .list_images::<String>(None)
        .await
        .map_err(|e| format!("Failed to list Docker images: {}", e))?;

    // Construct the full image reference we're looking for
    let target_image = format!("{}:{}", image_name, image_tag);

    // Check if any image has a matching repository tag
    for image in &images {
        for tag in &image.repo_tags {
            if tag == &target_image {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Check if a process with the given PID is running
///
/// # Arguments
/// * `pid` - The process ID to check
///
/// # Returns
/// Returns `true` if the process is running, `false` otherwise
#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    // On Unix, use kill(pid, 0) to check if process exists
    // ESRCH (error 3) means the process does not exist
    // EPERM (error 1) means the process exists but we don't have permission
    unsafe {
        match libc::kill(pid as libc::pid_t, 0) {
            0 | libc::EPERM => true, // Process exists (success or permission denied)
            libc::ESRCH => false,    // Process does not exist
            _ => false,              // Other errors, assume not running
        }
    }
}

/// Check if a process with the given PID is running (Windows)
///
/// # Arguments
/// * `pid` - The process ID to check
///
/// # Returns
/// Returns `true` if the process is running, `false` otherwise
#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    is_process_running_with_executor(pid, default_executor())
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

/// Check for and clean up stale PID files
///
/// This function checks if the PID file exists and if the process it refers to
/// is still running. If the process is not running, it removes the stale PID file.
///
/// # Returns
/// Ok(()) on success, or an error if the PID file cannot be read or removed
fn check_and_clean_stale_pid_file(pid_file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !pid_file_path.exists() {
        // No PID file, nothing to do
        return Ok(());
    }

    // Read PID file
    let pid_content =
        fs::read_to_string(pid_file_path).map_err(|e| format!("Failed to read PID file: {}", e))?;

    // Parse PID as u32
    let pid: u32 = pid_content
        .trim()
        .parse()
        .map_err(|e| format!("Failed to parse PID: {}", e))?;

    // Check if process is running
    if !is_process_running(pid) {
        // Process is not running, remove stale PID file
        tracing::debug!("Found stale PID file for process {}, removing...", pid);
        fs::remove_file(pid_file_path)
            .map_err(|e| format!("Failed to remove stale PID file: {}", e))?;
        tracing::debug!("Stale PID file removed");
    } else {
        tracing::debug!("PID file found, process {} is running", pid);
    }

    Ok(())
}

/// Run the CLI application and dispatch to the appropriate command handler
///
/// This is the main entry point for the Switchboard CLI. It parses command-line
/// arguments using clap and dispatches to the appropriate handler function based
/// on the subcommand specified by the user.
///
/// # Functionality
///
/// The function performs the following steps:
/// 1. Parses CLI arguments into the [`Cli`] structure
/// 2. Matches the command variant and dispatches to the corresponding handler
///
/// # Supported Commands
///
/// - [`Commands::Up`] - Build agent image and start scheduler (handled by [`run_up`])
/// - [`Commands::Run`] - Immediately execute a single agent (handled by [`run_run`])
/// - [`Commands::Build`] - Build or rebuild agent Docker image (handled by [`run_build`])
/// - [`Commands::List`] - Print all configured agents (handled by [`run_list`])
/// - [`Commands::Logs`] - View logs from agent runs (handled by [`run_logs`])
/// - [`Commands::Metrics`] - Display agent execution metrics (handled by [`run_metrics`])
/// - [`Commands::Down`] - Stop scheduler and containers (handled by [`run_down`])
/// - [`Commands::Validate`] - Parse and validate config file (handled by [`run_validate`])
/// - [`Commands::Skills`] - Manage Kilo skills (handled by [`run_skills`])
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if command execution fails.
///
/// # Errors
///
/// This function will return an error if:
/// - CLI argument parsing fails
/// - The dispatched command handler returns an error
///
/// # Examples
///
/// ```no_run
/// use switchboard::cli::run;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     run().await
/// }
/// ```
///
/// # CLI Usage
///
/// ```text
/// switchboard [OPTIONS] <COMMAND>
///
/// Options:
///   -c, --config <PATH>  Path to the configuration file (default: ./switchboard.toml)
///   -h, --help          Print help
///   -V, --version       Print version
///
/// Commands:
///   up        Build agent image and start scheduler
///   run       Immediately execute a single agent
///   build     Build or rebuild agent Docker image
///   list      Print all configured agents, their schedules, and prompts
///   logs      View logs from agent runs
///   metrics   Display agent execution metrics
///   down      Stop scheduler and any running agent containers
///   validate  Parse and validate config file
///   skills    Manage Kilo skills
/// ```
pub async fn run() -> Result<ColorMode, Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let color_mode = cli.color;

    let result = match cli.command {
        Commands::Up(args) => run_up(args, cli.config).await,
        Commands::Run(args) => run_run(args, cli.config).await,
        Commands::Build(args) => run_build(args, cli.config).await,
        Commands::List => run_list(cli.config),
        Commands::Logs(args) => run_logs(args, cli.config).await,
        Commands::Metrics(args) => run_metrics(args, cli.config),
        Commands::Down(args) => run_down(args, cli.config).await,
        Commands::Validate(args) => run_validate(args, cli.config).await,
        Commands::Skills(args) => run_skills(args, cli.config).await,
        Commands::Status => run_status(cli.config),
    };

    // Return the color_mode regardless of success or failure
    result?;
    Ok(color_mode)
}

/// Handler for the 'up' command - Build agent image and start the scheduler
///
/// This is the main command for starting the Switchboard scheduler. It loads the
/// configuration, initializes the scheduler, registers all configured agents,
/// and starts the agent execution loop.
///
/// # Functionality
///
/// The function performs the following steps:
/// 1. Loads and validates the configuration file from the specified path
/// 2. Checks for and cleans up stale PID files from previous runs
/// 3. Verifies the scheduler is not already running
/// 4. Initializes logging with file-based output
/// 5. Validates the workspace path exists
/// 6. Creates and initializes the scheduler
/// 7. Registers all configured agents with their schedules
/// 8. Checks Docker availability and image presence
/// 9. Starts the scheduler in either foreground or detached mode
///
/// # Arguments
///
/// * `args` - The [`UpCommand`] containing CLI arguments:
///   - `args.detach`: If `true`, runs in detached (background) mode
///   - If `false`, runs in foreground mode
/// * `config_path` - Optional path to the configuration file
///   - If `None`, defaults to `./switchboard.toml`
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Configuration parsing or validation fails
/// - Prompt file is not found
/// - Scheduler is already running
/// - Workspace path does not exist
/// - Docker is not available
/// - Scheduler initialization fails
///
/// # Errors
///
/// This function will return an error in the following cases:
///
/// - **Configuration Errors**: Invalid TOML syntax, missing required fields,
///   validation failures
/// - **Prompt File Errors**: Referenced prompt files do not exist
/// - **Scheduler Errors**: Another instance is already running, agent registration fails
/// - **Path Errors**: Workspace path does not exist or is not a directory
/// - **Docker Errors**: Docker daemon is not running or not accessible
///
/// # Modes
///
/// ## Foreground Mode (`--detach` not set)
///
/// The scheduler runs in the foreground and waits for Ctrl+C to stop:
/// - Provides immediate feedback on agent execution
/// - Logs are written to both console and file
/// - Pressing Ctrl+C triggers graceful shutdown
///
/// ```bash
/// switchboard up
/// ```
///
/// ## Detached Mode (`--detach` set)
///
/// The scheduler runs in the background:
/// - Process ID is written to `.switchboard/scheduler.pid`
/// - Logs are only written to files
/// - Use `switchboard down` to stop the scheduler
/// - The process continues running after the terminal closes
///
/// ```bash
/// switchboard up --detach
/// ```
///
/// # Configuration File
///
/// The configuration file (default: `./switchboard.toml`) must contain:
///
/// ```toml
/// [settings]
/// image_name = "switchboard-agent"
/// image_tag = "latest"
/// log_dir = ".switchboard/logs"
///
/// [[agents]]
/// name = "dev-agent"
/// schedule = "0 * * * *"
/// prompt_file = "prompts/dev.md"
/// ```
///
/// # Examples
///
/// ## Start scheduler in foreground mode
///
/// ```no_run
/// # use switchboard::cli::UpCommand;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = UpCommand { detach: false };
/// switchboard::cli::run_up(args, Some("./switchboard.toml".to_string())).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Start scheduler in detached mode
///
/// ```no_run
/// # use switchboard::cli::UpCommand;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = UpCommand { detach: true };
/// switchboard::cli::run_up(args, None).await?;
/// # Ok(())
/// # }
/// ```
///
/// # PID File
///
/// In detached mode, a PID file is created at `.switchboard/scheduler.pid`:
/// - Contains the process ID of the running scheduler
/// - Used to detect if a scheduler is already running
/// - Automatically cleaned up when the scheduler stops
/// - Stale PID files are cleaned up on startup
///
/// # Docker Requirements
///
/// - Docker daemon must be running
/// - The agent image (default: `switchboard-agent:latest`) should exist locally
/// - If the image doesn't exist, a warning is displayed and manual build is suggested
pub async fn run_up(
    args: UpCommand,
    config_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from default path
    let config_path = config_path.unwrap_or_else(|| "./switchboard.toml".to_string());
    let path = Path::new(&config_path);

    let config = match Config::from_toml(path) {
        Ok(config) => config,
        Err(e @ ConfigError::ParseError { .. }) => {
            eprintln!("✗ Configuration parsing failed: {}", e);
            return Err(e.into());
        }
        Err(e @ ConfigError::ValidationError { .. }) => {
            eprintln!("✗ Configuration validation failed");
            eprintln!("Error: {}", e);
            return Err(e.into());
        }
        Err(ConfigError::PromptFileNotFound {
            agent_name: _,
            prompt_file,
        }) => {
            eprintln!("✗ Prompt file not found: {}", prompt_file);
            return Err(format!("Prompt file not found: {}", prompt_file).into());
        }
    };

    // Check for and clean up stale PID file before starting
    let pid_file_path = Path::new(".switchboard/scheduler.pid");
    if let Err(e) = check_and_clean_stale_pid_file(pid_file_path) {
        eprintln!("  ⚠ Warning: Failed to check/clean stale PID file: {}", e);
        // Continue anyway, the error is not fatal
    }

    // Check if scheduler is already running (PID file exists and process is running)
    if pid_file_path.exists() {
        // Read PID file to get the running process
        if let Ok(pid_content) = fs::read_to_string(pid_file_path) {
            if let Ok(pid) = pid_content.trim().parse::<u32>() {
                if is_process_running(pid) {
                    eprintln!("✗ Scheduler is already running (PID: {}). Use 'switchboard list' to see active agents or 'switchboard down' to stop it first", pid);
                    return Err(Box::new(SchedulerError::SchedulerAlreadyRunning { pid }));
                }
            }
        }
    }

    // Print configuration summary
    let settings = config.settings.as_ref();
    let image_name = settings
        .map(|s| s.image_name.clone())
        .unwrap_or_else(|| "switchboard-agent".to_string());
    let image_tag = settings
        .map(|s| s.image_tag.clone())
        .unwrap_or_else(|| "latest".to_string());
    let log_dir = settings
        .map(|s| s.log_dir.clone())
        .unwrap_or_else(|| ".switchboard/logs".to_string());

    // Derive workspace path from config file path (parent directory)
    let workspace_path = Path::new(&config_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    // Initialize tracing with file appender for scheduler logs
    // This must be kept alive for the duration of the program
    let log_dir_path = PathBuf::from(&log_dir);
    let _logging_guard = init_logging(log_dir_path);

    // Validate that workspace path exists and is a directory
    let workspace_path_obj = Path::new(&workspace_path);
    if !workspace_path_obj.exists() || !workspace_path_obj.is_dir() {
        let error_msg = format!("Workspace path '{}' does not exist or is not a directory. Check your switchboard.toml configuration or create the directory.", workspace_path);
        eprintln!("✗ {}", error_msg);
        std::process::exit(1);
    }

    println!("\n✓ Configuration loaded: {}", config_path);
    println!("  Image: {}:{}", image_name, image_tag);
    println!("  Agents: {}", config.agents.len());

    for agent in &config.agents {
        println!("    - {} (schedule: {})", agent.name, agent.schedule);
    }

    // Create scheduler and register agents (before Docker check so it runs even without Docker)
    println!();
    let scheduler_result = Scheduler::new(None, config.settings.clone(), None).await;
    let mut scheduler = match scheduler_result {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("  ⚠ Warning: Failed to create scheduler: {}", e);
            None
        }
    };

    // Get configuration directory for resolving relative paths
    let config_dir = path.parent().unwrap_or_else(|| Path::new("."));

    let mut registered_count = 0;
    if let Some(ref mut sched) = scheduler {
        for agent in &config.agents {
            match sched
                .register_agent(
                    agent,
                    config_dir.to_path_buf(),
                    PathBuf::from(&log_dir),
                    image_name.clone(),
                    image_tag.clone(),
                    workspace_path.clone(),
                    None, // No injected Docker client - will be created internally
                )
                .await
            {
                Ok(_) => {
                    println!(
                        "  ✓ Registered agent '{}' (schedule: {})",
                        agent.name, agent.schedule
                    );
                    registered_count += 1;
                }
                Err(e) => {
                    eprintln!(
                        "  ⚠ Warning: Failed to register agent '{}': {}",
                        agent.name, e
                    );
                    eprintln!("    Schedule: {}", agent.schedule);
                }
            }
        }
    }

    if registered_count > 0 {
        println!(
            "  ✓ Scheduler initialized with {} agent(s) registered",
            registered_count
        );
    }

    // Print detach status before scheduler starts
    println!("\n  detach: {}", args.detach);

    // Check Docker availability before attempting any Docker operations
    // This ensures consistent error handling across all Docker-dependent commands
    check_docker_available()
        .await
        .map_err(|e| format!("Docker availability check failed: {}", e))?;
    println!("\n✓ Docker is available");

    // Create Docker client for checking image existence
    let docker_client = match DockerClient::new(image_name.clone(), image_tag.clone()).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("\n⚠ Warning: Docker is not available");
            eprintln!("  {}", e);
            return Ok(());
        }
    };

    // Check if the target image exists locally
    let image_exists = match check_image_exists(&docker_client, &image_name, &image_tag).await {
        Ok(exists) => exists,
        Err(e) => {
            eprintln!("\n⚠ Warning: Failed to check image availability: {}", e);
            false
        }
    };

    if image_exists {
        println!("  ✓ Image {}:{} found locally", image_name, image_tag);
    } else {
        println!(
            "  ⚠ Warning: Image {}:{} not found locally",
            image_name, image_tag
        );
        println!();
        println!("  Note: Automatic image building is not yet implemented (coming in Sprint 3).");
        println!("  If the image doesn't exist, you can build it manually:");
        println!("    docker build -t {}:{} .", image_name, image_tag);
    }

    // Foreground mode: start the scheduler and wait for Ctrl+C
    if !args.detach {
        if let Some(ref mut sched) = scheduler {
            if registered_count > 0 {
                println!("\n✓ Starting scheduler in foreground mode");
                println!("  Press Ctrl+C to stop the scheduler and exit");

                // Start the scheduler
                if let Err(e) = sched.start().await {
                    eprintln!("✗ Failed to start scheduler: {}", e);
                    return Err(Box::new(e));
                }

                // Spawn Discord listener concurrently (if discord feature is enabled and configured)
                #[cfg(feature = "discord")]
                {
                    if is_discord_configured() {
                        tracing::info!(
                            "Discord configuration detected, starting Discord listener..."
                        );

                        // Create shutdown channel for graceful Discord shutdown
                        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

                        let discord_handle = tokio::spawn(async move {
                            if let Err(e) =
                                start_discord_listener_with_shutdown(Some(shutdown_rx), None).await
                            {
                                tracing::warn!("Discord listener failed to start: {}. Scheduler will continue running.", e);
                            }
                        });
                        println!("  ✓ Discord listener spawned (optional)");

                        // Wait indefinitely for Ctrl+C signal
                        if let Err(e) = tokio::signal::ctrl_c().await {
                            eprintln!("Failed to listen for Ctrl+C: {}", e);
                        }
                        println!("\n\n⚠ Received Ctrl+C, shutting down scheduler...");
                        if shutdown_tx.send(()).is_err() {
                            eprintln!("Warning: shutdown receiver was dropped");
                        }

                        // Wait for Discord to finish (with timeout)
                        match tokio::time::timeout(
                            std::time::Duration::from_secs(5),
                            discord_handle,
                        )
                        .await
                        {
                            Ok(_) => tracing::info!("Discord listener shut down gracefully"),
                            Err(_) => tracing::warn!("Discord listener shutdown timed out"),
                        }

                        if let Err(e) = sched.stop().await {
                            eprintln!("  ⚠ Warning: Error stopping scheduler: {}", e);
                        } else {
                            println!("  ✓ Scheduler stopped");
                        }
                    } else {
                        tracing::debug!(
                            "Discord configuration not found, skipping Discord listener"
                        );

                        // Wait indefinitely for Ctrl+C signal
                        if let Err(e) = tokio::signal::ctrl_c().await {
                            eprintln!("Failed to listen for Ctrl+C: {}", e);
                        }
                        println!("\n\n⚠ Received Ctrl+C, shutting down scheduler...");

                        if let Err(e) = sched.stop().await {
                            eprintln!("  ⚠ Warning: Error stopping scheduler: {}", e);
                        } else {
                            println!("  ✓ Scheduler stopped");
                        }
                    }
                }

                // When discord feature is not enabled, just wait for Ctrl+C
                #[cfg(not(feature = "discord"))]
                {
                    // Wait indefinitely for Ctrl+C signal
                    if let Err(e) = tokio::signal::ctrl_c().await {
                        eprintln!("Failed to listen for Ctrl+C: {}", e);
                    }
                    println!("\n\n⚠ Received Ctrl+C, shutting down scheduler...");

                    if let Err(e) = sched.stop().await {
                        eprintln!("  ⚠ Warning: Error stopping scheduler: {}", e);
                    } else {
                        println!("  ✓ Scheduler stopped");
                    }
                }
            } else {
                println!("\n⚠ No agents registered, scheduler not started");
            }
        }
    } else {
        // Detach mode: start the scheduler in the background
        if let Some(mut sched) = scheduler {
            if registered_count > 0 {
                println!("\n✓ Starting scheduler in detached mode");

                // Get current process ID
                let current_pid = std::process::id();

                // Create .switchboard directory if it doesn't exist
                let switchboard_dir = Path::new(".switchboard");
                if !switchboard_dir.exists() {
                    if let Err(e) = std::fs::create_dir_all(switchboard_dir) {
                        eprintln!(
                            "  ⚠ Warning: Failed to create .switchboard directory: {}",
                            e
                        );
                        return Err(e.into());
                    }
                }

                // Write PID file
                let pid_file_path = switchboard_dir.join("scheduler.pid");
                if let Err(e) = std::fs::write(&pid_file_path, current_pid.to_string()) {
                    eprintln!("  ⚠ Warning: Failed to write PID file: {}", e);
                    return Err(e.into());
                }

                println!("  ✓ PID file created: .switchboard/scheduler.pid");

                // Spawn the scheduler as a background task
                let pid_file_path_for_cleanup = pid_file_path.clone();
                let task = tokio::spawn(async move {
                    // Use scopeguard to ensure PID file is cleaned up regardless of how the task exits
                    // This handles cases like: scheduler start failure, panic, or unexpected termination
                    let _guard = scopeguard::guard(pid_file_path_for_cleanup.clone(), |path| {
                        tracing::debug!("Cleaning up PID file: {}", path.display());
                        if let Err(e) = std::fs::remove_file(&path) {
                            tracing::error!("Failed to remove PID file: {}", e);
                        } else {
                            tracing::debug!("PID file removed: {}", path.display());
                        }
                    });

                    // Start the scheduler
                    if let Err(e) = sched.start().await {
                        tracing::error!("Failed to start scheduler: {}", e);
                        return;
                    }

                    tracing::info!("Scheduler started in detached mode");

                    // Spawn Discord listener concurrently (if discord feature is enabled and configured)
                    #[cfg(feature = "discord")]
                    {
                        if is_discord_configured() {
                            tracing::info!(
                                "Discord configuration detected, starting Discord listener..."
                            );

                            // Create shutdown channel for graceful Discord shutdown
                            let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

                            let discord_handle = tokio::spawn(async move {
                                if let Err(e) =
                                    start_discord_listener_with_shutdown(Some(shutdown_rx), None)
                                        .await
                                {
                                    tracing::warn!("Discord listener failed to start: {}. Scheduler will continue running.", e);
                                }
                            });
                            tracing::info!("Discord listener spawned (optional)");

                            // Wait for shutdown signal
                            #[cfg(unix)]
                            {
                                // On Unix, handle SIGTERM and SIGINT in addition to ctrl_c()
                                let mut sigterm = match signal(SignalKind::terminate()) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        tracing::error!("Failed to create SIGTERM handler: {}", e);
                                        // Signal Discord to shut down
                                        let _ = shutdown_tx.send(());
                                        let _ = discord_handle.await;
                                        return;
                                    }
                                };
                                let mut sigint = match signal(SignalKind::interrupt()) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        tracing::error!("Failed to create SIGINT handler: {}", e);
                                        // Signal Discord to shut down
                                        let _ = shutdown_tx.send(());
                                        let _ = discord_handle.await;
                                        return;
                                    }
                                };

                                tokio::select! {
                                    // Wait for Ctrl+C signal
                                    _ = tokio::signal::ctrl_c() => {
                                        tracing::info!("Received Ctrl+C, stopping scheduler...");
                                    }
                                    // Wait for SIGTERM
                                    _ = sigterm.recv() => {
                                        tracing::info!("Received SIGTERM, stopping scheduler...");
                                    }
                                    // Wait for SIGINT
                                    _ = sigint.recv() => {
                                        tracing::info!("Received SIGINT, stopping scheduler...");
                                    }
                                }

                                // Signal Discord to shut down gracefully
                                let _ = shutdown_tx.send(());

                                // Wait for Discord to finish (with timeout)
                                match tokio::time::timeout(
                                    std::time::Duration::from_secs(5),
                                    discord_handle,
                                )
                                .await
                                {
                                    Ok(_) => {
                                        tracing::info!("Discord listener shut down gracefully")
                                    }
                                    Err(_) => tracing::warn!("Discord listener shutdown timed out"),
                                }
                            }

                            #[cfg(windows)]
                            {
                                // On Windows, only ctrl_c() is available
                                tokio::select! {
                                    // Wait for Ctrl+C signal
                                    _ = tokio::signal::ctrl_c() => {
                                        tracing::info!("Received Ctrl+C, stopping scheduler...");
                                    }
                                }

                                // Signal Discord to shut down gracefully
                                let _ = shutdown_tx.send(());

                                // Wait for Discord to finish (with timeout)
                                match tokio::time::timeout(
                                    std::time::Duration::from_secs(5),
                                    discord_handle,
                                )
                                .await
                                {
                                    Ok(_) => {
                                        tracing::info!("Discord listener shut down gracefully")
                                    }
                                    Err(_) => tracing::warn!("Discord listener shutdown timed out"),
                                }
                            }
                        } else {
                            tracing::debug!(
                                "Discord configuration not found, skipping Discord listener"
                            );

                            // Wait for shutdown signal without Discord
                            #[cfg(unix)]
                            {
                                // On Unix, handle SIGTERM and SIGINT in addition to ctrl_c()
                                let mut sigterm = match signal(SignalKind::terminate()) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        tracing::error!("Failed to create SIGTERM handler: {}", e);
                                        return;
                                    }
                                };
                                let mut sigint = match signal(SignalKind::interrupt()) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        tracing::error!("Failed to create SIGINT handler: {}", e);
                                        return;
                                    }
                                };

                                tokio::select! {
                                    // Wait for Ctrl+C signal
                                    _ = tokio::signal::ctrl_c() => {
                                        tracing::info!("Received Ctrl+C, stopping scheduler...");
                                    }
                                    // Wait for SIGTERM
                                    _ = sigterm.recv() => {
                                        tracing::info!("Received SIGTERM, stopping scheduler...");
                                    }
                                    // Wait for SIGINT
                                    _ = sigint.recv() => {
                                        tracing::info!("Received SIGINT, stopping scheduler...");
                                    }
                                }
                            }

                            #[cfg(windows)]
                            {
                                // On Windows, only ctrl_c() is available
                                tokio::select! {
                                    // Wait for Ctrl+C signal
                                    _ = tokio::signal::ctrl_c() => {
                                        tracing::info!("Received Ctrl+C, stopping scheduler...");
                                    }
                                }
                            }
                        }
                    }
                    #[cfg(not(feature = "discord"))]
                    {
                        // Wait for shutdown signal
                        #[cfg(unix)]
                        {
                            // On Unix, handle SIGTERM and SIGINT in addition to ctrl_c()
                            let mut sigterm = match signal(SignalKind::terminate()) {
                                Ok(s) => s,
                                Err(e) => {
                                    tracing::error!("Failed to create SIGTERM handler: {}", e);
                                    return;
                                }
                            };
                            let mut sigint = match signal(SignalKind::interrupt()) {
                                Ok(s) => s,
                                Err(e) => {
                                    tracing::error!("Failed to create SIGINT handler: {}", e);
                                    return;
                                }
                            };

                            tokio::select! {
                                // Wait for Ctrl+C signal
                                _ = tokio::signal::ctrl_c() => {
                                    tracing::info!("Received Ctrl+C, stopping scheduler...");
                                }
                                // Wait for SIGTERM
                                _ = sigterm.recv() => {
                                    tracing::info!("Received SIGTERM, stopping scheduler...");
                                }
                                // Wait for SIGINT
                                _ = sigint.recv() => {
                                    tracing::info!("Received SIGINT, stopping scheduler...");
                                }
                            }
                        }

                        #[cfg(windows)]
                        {
                            // On Windows, only ctrl_c() is available
                            tokio::select! {
                                // Wait for Ctrl+C signal
                                _ = tokio::signal::ctrl_c() => {
                                    tracing::info!("Received Ctrl+C, stopping scheduler...");
                                }
                            }
                        }
                    }

                    // Stop the scheduler
                    if let Err(e) = sched.stop().await {
                        tracing::error!("Error stopping scheduler: {}", e);
                    } else {
                        tracing::info!("Scheduler stopped gracefully");
                    }

                    // PID file will be cleaned up by the scopeguard when this function exits
                });

                // In detach mode, wait for the task to complete to keep the scheduler running
                // This prevents the main function from returning and shutting down the tokio runtime
                println!("  ✓ Scheduler is running in the background");
                println!("  Process ID: {}", current_pid);
                println!("  Logs directory: {}", log_dir);
                println!("\n  To stop the scheduler:");
                println!("    Press Ctrl+C");
                println!("    switchboard down  (when implemented)");
                println!("    kill {}", current_pid);

                // Await the task to keep the scheduler running indefinitely
                if let Err(e) = task.await {
                    tracing::error!("Scheduler task error: {}", e);
                }
            } else {
                println!("\n⚠ No agents registered, scheduler not started");
            }
        }
    }

    Ok(())
}

/// Handler for the 'run' command - Immediately execute a single agent
///
/// This command executes a single configured agent immediately, bypassing
/// the scheduler. This is useful for:
/// - Testing agent configurations
/// - Running agents on-demand outside of their schedule
/// - Debugging agent behavior
/// - One-off agent executions
///
/// # Functionality
///
/// The function performs the following steps:
/// 1. Loads and validates the configuration file
/// 2. Finds the specified agent by name
/// 3. Resolves the agent's prompt (inline or from file)
/// 4. Builds environment variables from agent configuration
/// 5. Validates the workspace path
/// 6. Creates a Docker container configuration
/// 7. Runs the agent in a Docker container
/// 8. Prints execution summary with container ID, exit code, and duration
///
/// # Arguments
///
/// * `args` - The [`RunCommand`] containing CLI arguments:
///   - `args.agent_name`: The name of the agent to execute
/// * `_config` - Optional path to the configuration file
///   - If `None`, defaults to `./switchboard.toml`
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Configuration parsing or validation fails
/// - The specified agent is not found
/// - Prompt file is not found
/// - Workspace path does not exist
/// - Docker is not available
/// - Agent execution fails
///
/// # Errors
///
/// This function will return an error in the following cases:
///
/// - **Configuration Errors**: Invalid TOML syntax, missing required fields,
///   validation failures
/// - **Agent Not Found**: The specified `agent_name` does not exist in the configuration
/// - **Prompt Errors**: Prompt file not found or cannot be read
/// - **Path Errors**: Workspace path does not exist or is not a directory
/// - **Docker Errors**: Docker daemon is not running or container execution fails
///
/// # Container Execution
///
/// The agent is executed in a Docker container with:
/// - The agent's prompt passed to kilocode
/// - Environment variables from the agent's configuration
/// - Optional timeout enforcement
/// - Optional readonly workspace mounting
/// - Automatic `--auto` flag for non-interactive execution
///
/// # Examples
///
/// ## Execute an agent by name
///
/// ```no_run
/// # use switchboard::cli::RunCommand;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = RunCommand { agent_name: "dev-agent".to_string() };
/// switchboard::cli::run_run(args, None).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Execute with custom config path
///
/// ```no_run
/// # use switchboard::cli::RunCommand;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let args = RunCommand { agent_name: "test-agent".to_string() };
/// switchboard::cli::run_run(args, Some("./custom-config.toml".to_string())).await?;
/// # Ok(())
/// # }
/// ```
///
/// # CLI Usage
///
/// ```text
/// switchboard run <AGENT_NAME>
///
/// Arguments:
///   <AGENT_NAME>  Name of the agent to execute
///
/// Options:
///   -c, --config <PATH>  Path to the configuration file (default: ./switchboard.toml)
///   -h, --help          Print help
/// ```
///
/// # Output
///
/// On successful execution, the following information is printed:
///
/// ```text
/// ✓ Starting agent: dev-agent
/// ✓ Agent execution completed
///   Container ID: abc123def456...
///   Exit Code: 0
///   Duration: 15.32s
/// ```
///
/// # Notes
///
/// - This command does NOT start the scheduler
/// - The agent is executed immediately, not queued
/// - No schedule-based execution occurs
/// - Execution metrics are still recorded and can be viewed with `switchboard metrics`
/// - The workspace path is mounted into the container
pub async fn run_run(
    args: RunCommand,
    _config: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Resolve config path
    let config_path = _config.unwrap_or_else(|| "./switchboard.toml".to_string());
    let path = Path::new(&config_path);

    // Load config
    let config = match Config::from_toml(path) {
        Ok(config) => config,
        Err(e @ ConfigError::ParseError { .. }) => {
            eprintln!("✗ Configuration parsing failed: {}", e);
            return Err(e.into());
        }
        Err(e @ ConfigError::ValidationError { .. }) => {
            eprintln!("✗ Configuration validation failed");
            eprintln!("Error: {}", e);
            return Err(e.into());
        }
        Err(ConfigError::PromptFileNotFound {
            agent_name: _,
            prompt_file,
        }) => {
            eprintln!("✗ Prompt file not found: {}", prompt_file);
            return Err(format!("Prompt file not found: {}", prompt_file).into());
        }
    };

    // Find agent by name
    let available_agents: Vec<&str> = config.agents.iter().map(|a| a.name.as_str()).collect();
    let agent_list = if available_agents.is_empty() {
        "No agents configured".to_string()
    } else {
        available_agents.join(", ")
    };
    let agent = config
        .agents
        .iter()
        .find(|a| a.name == args.agent_name)
        .ok_or_else(|| {
            eprintln!(
                "✗ Agent '{}' not found in configuration. Available agents: {}",
                args.agent_name, agent_list
            );
            Box::new(SchedulerError::AgentNotFound {
                agent_name: args.agent_name.clone(),
                available: agent_list,
            })
        })?;

    // Get effective settings from config (use defaults if not set)
    let settings = config.settings.as_ref();
    let image_name = settings
        .map(|s| s.image_name.clone())
        .unwrap_or_else(|| "switchboard-agent".to_string());
    let image_tag = settings
        .map(|s| s.image_tag.clone())
        .unwrap_or_else(|| "latest".to_string());
    let log_dir = settings
        .map(|s| s.log_dir.clone())
        .unwrap_or_else(|| ".switchboard/logs".to_string());

    // Derive workspace path from config file path (parent directory)
    let workspace_path = Path::new(&config_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    // Validate that workspace path exists and is a directory
    let workspace_path_obj = Path::new(&workspace_path);
    if !workspace_path_obj.exists() || !workspace_path_obj.is_dir() {
        let error_msg = format!("Workspace path '{}' does not exist or is not a directory. Check your switchboard.toml configuration or create the directory.", workspace_path);
        eprintln!("✗ {}", error_msg);
        std::process::exit(1);
    }

    // Resolve the prompt (from agent.prompt or read from agent.prompt_file)
    let prompt: String = match (&agent.prompt, &agent.prompt_file) {
        (Some(inline_prompt), None) => inline_prompt.clone(),
        (None, Some(prompt_file)) => {
            let config_dir = config.config_dir();
            match agent.read_prompt_file(config_dir) {
                Ok(Some(contents)) => contents,
                Ok(None) => {
                    eprintln!("✗ Prompt file not found: {}", prompt_file);
                    return Err(format!("Prompt file not found: {}", prompt_file).into());
                }
                Err(e) => {
                    eprintln!("✗ Failed to read prompt file: {}", e);
                    return Err(e.into());
                }
            }
        }
        _ => {
            eprintln!("✗ Agent must have either 'prompt' or 'prompt_file' specified");
            return Err("Agent must have either 'prompt' or 'prompt_file' specified".into());
        }
    };

    // Build environment variables using agent.env()
    let env_vars = agent.env(settings);

    // Check Docker availability before attempting any Docker operations
    // This ensures consistent error handling across all Docker-dependent commands
    check_docker_available()
        .await
        .map_err(|e| format!("Docker availability check failed: {}", e))?;

    // Create ContainerConfig with all required fields
    let container_config = ContainerConfig {
        agent_name: agent.name.clone(),
        env_vars,
        timeout: agent.timeout.clone(),
        readonly: agent.readonly.unwrap_or(false),
        prompt: prompt.clone(),
        skills: agent.skills.clone(),
    };

    // Create DockerClient using DockerClient::new()
    let docker_client = match DockerClient::new(image_name.clone(), image_tag.clone()).await {
        Ok(client) => Arc::new(client) as Arc<dyn DockerClientTrait>,
        Err(e) => {
            eprintln!("✗ Docker connection failed: {}", e);
            return Err(e.into());
        }
    };

    // Create Logger with log_dir and agent_name, foreground_mode: true
    let log_dir_path = PathBuf::from(log_dir);
    let logger = Logger::new(log_dir_path.clone(), Some(agent.name.clone()), true);

    // Wrap Logger in Arc<Mutex<>>
    let logger = Arc::new(Mutex::new(logger));

    // Create MetricsStore for collecting execution metrics
    let metrics_store = MetricsStore::new(log_dir_path.clone());

    // Load existing metrics before the run (error is logged but doesn't fail the agent run)
    if let Err(e) = metrics_store.load() {
        tracing::warn!("Metrics file not found (first run or missing file): {}", e);
    }

    // Build the full image name
    let image = format!("{}:{}", image_name, image_tag);

    // Run the agent
    eprintln!("✓ Starting agent: {}", agent.name);
    let start_time = std::time::Instant::now();

    // When PROMPT is piped to kilocode, --auto flag is required
    // Kilo Code CLI uses --auto flag to automatically determine the mode based on prompt content
    let cmd_args: Vec<String> = vec!["--auto".to_string(), prompt.clone()];

    let result = match run_agent(
        &workspace_path,
        docker_client,
        &container_config,
        agent.timeout.clone(),
        &image,
        Some(cmd_args.as_slice()),
        Some(logger),
        Some(&metrics_store),
        &agent.name,
        None, // queued_start_time - CLI runs are not queued
    )
    .await
    {
        Ok(result) => result,
        Err(e) => {
            eprintln!("✗ Failed to run agent: {}", e);
            return Err(e.into());
        }
    };

    let duration = start_time.elapsed();

    // Print execution summary
    eprintln!("✓ Agent execution completed");
    eprintln!("  Container ID: {}", result.container_id);
    eprintln!("  Exit Code: {}", result.exit_code);
    eprintln!("  Duration: {:.2}s", duration.as_secs_f64());

    Ok(())
}

/// Handler for the 'build' command - Build or rebuild the agent Docker image
///
/// This command builds or rebuilds the Docker image used to run AI coding agents.
/// The image is built from a Dockerfile in the project directory and includes
/// the necessary dependencies and tools for executing agent prompts.
///
/// # Functionality
///
/// The function performs the following steps:
/// 1. Delegates to the [`BuildCommand::run()`] method
/// 2. Executes the Docker build process
/// 3. Tags the resulting image with the configured name and tag
///
/// # Arguments
///
/// * `args` - The [`BuildCommand`] containing CLI arguments:
///   - See [`BuildCommand`] for available options (if any)
/// * `_config` - Optional path to the configuration file
///   - If `None`, defaults to `./switchboard.toml`
///   - Currently unused by this command but provided for consistency
///
/// # Returns
///
/// Returns `Ok(())` on successful build, or an error if:
/// - Docker is not available
/// - Dockerfile is not found
/// - Build process fails
///
/// # Errors
///
/// This function will return an error in the following cases:
///
/// - **Docker Errors**: Docker daemon is not running or not accessible
/// - **Build Errors**: Dockerfile contains errors or build process fails
/// - **File Errors**: Dockerfile is not found or cannot be read
///
/// # Image Naming
///
/// The built image is tagged according to the configuration in `switchboard.toml`:
///
/// ```toml
/// [settings]
/// image_name = "switchboard-agent"
/// image_tag = "latest"
/// ```
///
/// This results in an image named `switchboard-agent:latest`.
///
/// # Examples
///
/// ## Build the agent image
///
/// Builds a Docker image for agents using the Dockerfile in the workspace.
///
/// # CLI Usage
///
/// ```text
/// switchboard build [OPTIONS]
///
/// Options:
///   -c, --config <PATH>  Path to the configuration file (default: ./switchboard.toml)
///   -h, --help          Print help
/// ```
///
/// # Docker Requirements
///
/// - Docker daemon must be running
/// - Sufficient disk space for the image
/// - Network access to download base images (if needed)
/// - Write permissions to the Docker daemon
///
/// # Notes
///
/// - This command does NOT start the scheduler
/// - The image must be built before using `switchboard up` or `switchboard run`
/// - Existing images with the same name and tag will be overwritten
/// - The build process can take several minutes depending on network speed and system resources
pub async fn run_build(
    args: BuildCommand,
    _config: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    args.run().await
}

/// Handler for the 'list' command
pub fn run_list(_config: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = _config.unwrap_or_else(|| "./switchboard.toml".to_string());
    let path = Path::new(&config_path);

    // Load config
    let config = match Config::from_toml(path) {
        Ok(config) => config,
        Err(e @ ConfigError::ParseError { .. }) => {
            eprintln!("✗ Configuration parsing failed: {}", e);
            return Err(e.into());
        }
        Err(e @ ConfigError::ValidationError { .. }) => {
            eprintln!("✗ Configuration validation failed");
            eprintln!("Error: {}", e);
            return Err(e.into());
        }
        Err(ConfigError::PromptFileNotFound {
            agent_name: _,
            prompt_file,
        }) => {
            eprintln!("✗ Prompt file not found: {}", prompt_file);
            return Err(format!("Prompt file not found: {}", prompt_file).into());
        }
    };

    // Call list_agents and handle errors
    list_agents(&config).map_err(|e| e.into())
}

/// Handler for the 'logs' command
pub async fn run_logs(
    args: LogsCommand,
    _config: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let logs_args = LogsArgs {
        agent_name: args.agent_name,
        follow: args.follow,
        tail: Some(args.tail),
    };
    logs_run(logs_args).await
}

/// Handler for the 'metrics' command
pub fn run_metrics(
    args: MetricsCommand,
    _config: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load config to get log directory and agent schedules
    let config_path = args
        .config
        .unwrap_or_else(|| "./switchboard.toml".to_string());
    let path = Path::new(&config_path);

    let config = match Config::from_toml(path) {
        Ok(config) => config,
        Err(e @ ConfigError::ParseError { .. }) => {
            eprintln!("✗ Configuration parsing failed: {}", e);
            return Err(e.into());
        }
        Err(e @ ConfigError::ValidationError { .. }) => {
            eprintln!("✗ Configuration validation failed");
            eprintln!("Error: {}", e);
            return Err(e.into());
        }
        Err(ConfigError::PromptFileNotFound {
            agent_name: _,
            prompt_file,
        }) => {
            eprintln!("✗ Prompt file not found: {}", prompt_file);
            return Err(format!("Prompt file not found: {}", prompt_file).into());
        }
    };

    // Get log directory from config
    let log_dir = config
        .settings
        .as_ref()
        .map(|s| s.log_dir.clone())
        .unwrap_or_else(|| ".switchboard/logs".to_string());

    // Build agent schedules HashMap
    let mut agent_schedules: HashMap<String, String> = HashMap::new();
    for agent in &config.agents {
        agent_schedules.insert(agent.name.clone(), agent.schedule.clone());
    }

    // Call metrics function with the log directory, agent filter, and schedules
    metrics(
        &log_dir,
        args.detailed,
        args.agent.as_deref(),
        Some(&agent_schedules),
    )
    .map_err(|e| e.into())
}

/// Handler for the 'down' command
pub async fn run_down(
    args: DownCommand,
    _config: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pid_file_path = ".switchboard/scheduler.pid";

    // Check if PID file exists
    if !Path::new(pid_file_path).exists() {
        return Err("Scheduler is not running".into());
    }

    // Read PID file
    let pid_content = fs::read_to_string(pid_file_path)?;

    // Parse PID as u32
    let pid: u32 = pid_content.trim().parse().map_err(|e| {
        eprintln!("Error: Failed to parse PID: {}", e);
        e
    })?;

    // Track if scheduler stop failed
    let mut scheduler_stop_failed = false;

    // Check if the process is still running before sending signal
    if !is_process_running(pid) {
        println!("Scheduler process no longer running");
        // Clean up the stale PID file
        let _ = fs::remove_file(pid_file_path);
        // Continue to check for Docker containers even if scheduler is not running
    } else {
        println!("Stopping scheduler...");

        // Send SIGTERM to the scheduler process
        let executor = default_executor();
        let kill_result = executor.execute("kill", &["-15".to_string(), pid.to_string()]);

        match kill_result {
            Ok(output) if output.status.success() => {
                println!("✓ Scheduler stopped");
            }
            Ok(output) => {
                // Check if the process no longer exists (ESRCH) or permission denied (EPERM)
                let exit_code = output.status.code().unwrap_or(-1);
                if exit_code == 1 {
                    // exit code 1 typically indicates ESRCH (no such process) or EPERM
                    println!("Scheduler process no longer running");
                } else {
                    eprintln!("✗ Failed to stop scheduler (exit code: {})", exit_code);
                    scheduler_stop_failed = true;
                }
            }
            Err(e) => {
                // Check if this is a "no such process" error
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("no such process") || error_msg.contains("esrch") {
                    println!("Scheduler process no longer running");
                } else {
                    eprintln!("✗ Failed to stop scheduler: {}", e);
                    scheduler_stop_failed = true;
                }
            }
        }
    }

    // Docker container listing
    println!("\nChecking for running agent containers...");

    // Check Docker availability before attempting any Docker operations
    // This ensures consistent error handling across all Docker-dependent commands
    // check_docker_available() returns the Docker client for further use
    let docker = match check_docker_available().await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("  ⚠ Warning: Failed to connect to Docker: {}", e);
            println!("No agent containers found");
            return Ok(());
        }
    };

    // List containers with label filter
    let filters = HashMap::from([("label".to_string(), vec!["switchboard.agent".to_string()])]);
    let list_options = Some(ListContainersOptions {
        all: true,
        filters,
        ..Default::default()
    });

    let containers = match docker.list_containers::<String>(list_options).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  ⚠ Warning: Failed to list containers: {}", e);
            println!("No agent containers found");
            return Ok(());
        }
    };

    // Track container failures
    let mut failed_count = 0;

    if containers.is_empty() {
        println!("No agent containers found");
    } else {
        println!("Found {} agent container(s)", containers.len());

        // Stop each container
        let mut stopped_count = 0;

        for container in &containers {
            // Get container ID
            let container_id = match &container.id {
                Some(id) => id.clone(),
                None => {
                    println!("  ⚠ Warning: Container has no ID, skipping");
                    continue;
                }
            };

            println!("  Stopping container {}...", container_id);

            // Stop the container with a 10 second timeout
            let stop_result = docker
                .stop_container(&container_id, Some(StopContainerOptions { t: 10 }))
                .await;

            match stop_result {
                Ok(_) => {
                    // Wait briefly for the container to fully stop
                    sleep(Duration::from_secs(2)).await;
                    println!("  ✓ Stopped container {}", container_id);
                    stopped_count += 1;
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    // Check if container was already stopped
                    if error_msg.contains("already stopped") || error_msg.contains("is not running")
                    {
                        println!("  Container {} was already stopped", container_id);
                        stopped_count += 1;
                    } else {
                        eprintln!("  ✗ Failed to stop container {}: {}", container_id, e);
                        failed_count += 1;
                    }
                }
            }
        }

        if stopped_count > 0 {
            println!("\nAll agent containers stopped");
        }

        if failed_count > 0 {
            eprintln!("  ⚠ Failed to stop {} container(s)", failed_count);
        }
    }

    // PID file cleanup
    println!("\nCleaning up PID file...");
    match fs::remove_file(pid_file_path) {
        Ok(_) => {
            println!("✓ PID file removed");
        }
        Err(e) => {
            eprintln!("✗ Failed to remove PID file: {}", e);
        }
    }

    // Clean up .switchboard directory if --cleanup flag is set
    if args.cleanup {
        println!("\nCleaning up .switchboard directory...");
        let switchboard_dir = Path::new(".switchboard");
        if switchboard_dir.exists() {
            match fs::remove_dir_all(switchboard_dir) {
                Ok(_) => {
                    println!("✓ .switchboard directory removed");
                }
                Err(e) => {
                    eprintln!("✗ Failed to remove .switchboard directory: {}", e);
                }
            }
        } else {
            println!("  .switchboard directory does not exist");
        }
    }

    // Shutdown summary
    println!("\nShutdown complete");
    if scheduler_stop_failed || failed_count > 0 {
        println!("⚠ Shutdown completed with errors");
    } else {
        println!("✓ Scheduler and all containers stopped");
    }

    Ok(())
}

/// Handler for the 'validate' command
pub async fn run_validate(
    _args: ValidateCommand,
    _config: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::path::PathBuf;
    let config_path = _config.unwrap_or_else(|| "./switchboard.toml".to_string());
    _args.run(PathBuf::from(config_path)).await
}

/// Handler for the 'skills' command
pub async fn run_skills(
    args: SkillsCommand,
    config_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::path::Path;
    let config_path_str = config_path.unwrap_or_else(|| "./switchboard.toml".to_string());
    let path = Path::new(&config_path_str);

    let config = match Config::from_toml(path) {
        Ok(config) => config,
        Err(e @ ConfigError::ParseError { .. }) => {
            eprintln!("✗ Configuration parsing failed: {}", e);
            return Err(e.into());
        }
        Err(e @ ConfigError::ValidationError { .. }) => {
            eprintln!("✗ Configuration validation failed");
            eprintln!("Error: {}", e);
            return Err(e.into());
        }
        Err(ConfigError::PromptFileNotFound {
            agent_name: _,
            prompt_file,
        }) => {
            eprintln!("✗ Prompt file not found: {}", prompt_file);
            return Err(format!("Prompt file not found: {}", prompt_file).into());
        }
    };

    let exit_code = crate::commands::skills::run_skills(args, &config).await;
    match exit_code {
        crate::commands::skills::ExitCode::Success => Ok(()),
        crate::commands::skills::ExitCode::Error => Err("Skills command failed".into()),
    }
}

/// Handler for the 'status' command - Check scheduler health and status
///
/// This command reads the heartbeat file and displays the current status of the
/// scheduler, including whether it's running, the last heartbeat time, and
/// information about registered agents.
///
/// # Functionality
///
/// The function performs the following steps:
/// 1. Reads the heartbeat file from `.switchboard/heartbeat.json`
/// 2. Displays scheduler status (running/stopped)
/// 3. Shows last heartbeat time and age
/// 4. Lists registered agents and their schedules
/// 5. Checks if the scheduler process is still running
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Heartbeat file cannot be read
/// - Heartbeat file format is invalid
///
/// # CLI Usage
///
/// ```text
/// switchboard status
///
/// Options:
///   -c, --config <PATH>  Path to the configuration file (default: ./switchboard.toml)
///   -h, --help          Print help
/// ```
///
/// # Output
///
/// When the scheduler is running:
///
/// ```text
/// Scheduler Status: Running
///   Process ID: 12345
///   Last Heartbeat: 2026-02-20T08:00:00Z (30s ago)
///
/// Registered Agents:
///   - dev-agent (schedule: 0 * * * *) - Idle
/// ```
///
/// When the scheduler is not running:
///
/// ```text
/// Scheduler Status: Not running
/// No heartbeat file found - scheduler may not be started
/// ```
///
/// # Notes
///
/// - This command works for both foreground and detached modes
/// - The heartbeat file is updated every 30 seconds
/// - If the heartbeat is older than 2 minutes, the scheduler may be stuck
pub fn run_status(_config: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    use serde::Deserialize;

    // Heartbeat file path
    let heartbeat_path = Path::new(".switchboard/heartbeat.json");

    // Check if heartbeat file exists
    if !heartbeat_path.exists() {
        println!("Scheduler Status: Not running");
        println!("No heartbeat file found - scheduler may not be started");
        return Ok(());
    }

    // Read heartbeat file
    let heartbeat_content = fs::read_to_string(heartbeat_path)
        .map_err(|e| format!("Failed to read heartbeat file: {}", e))?;

    // Parse heartbeat JSON
    #[derive(Debug, Deserialize)]
    struct HeartbeatData {
        pid: u32,
        last_heartbeat: String,
        state: String,
        agents: Vec<AgentHeartbeat>,
    }

    #[derive(Debug, Deserialize)]
    struct AgentHeartbeat {
        name: String,
        schedule: String,
        current_run: Option<String>,
    }

    let heartbeat: HeartbeatData = serde_json::from_str(&heartbeat_content)
        .map_err(|e| format!("Failed to parse heartbeat file: {}", e))?;

    // Check if process is still running
    let process_running = is_process_running(heartbeat.pid);

    // Parse last heartbeat time
    let last_heartbeat_time = chrono::DateTime::parse_from_rfc3339(&heartbeat.last_heartbeat)
        .map_err(|e| format!("Failed to parse heartbeat timestamp: {}", e))?;

    let now = chrono::Utc::now();
    let heartbeat_age = now.signed_duration_since(last_heartbeat_time);

    // Display status
    if heartbeat.state == "running" && process_running {
        println!("Scheduler Status: Running");
        println!("  Process ID: {}", heartbeat.pid);
        println!(
            "  Last Heartbeat: {} ({} ago)",
            last_heartbeat_time.format("%Y-%m-%d %H:%M:%S UTC"),
            format_duration(heartbeat_age)
        );

        // Check if heartbeat is stale (> 2 minutes)
        if heartbeat_age.num_seconds() > 120 {
            println!("  ⚠ Warning: Heartbeat is stale - scheduler may be unresponsive");
        }

        // Display agent information
        if !heartbeat.agents.is_empty() {
            println!("\nRegistered Agents:");
            for agent in &heartbeat.agents {
                let status = if let Some(ref run_info) = agent.current_run {
                    if run_info == "starting" {
                        "Starting".to_string()
                    } else {
                        format!("Running (container: {})", run_info)
                    }
                } else {
                    "Idle".to_string()
                };
                println!(
                    "  - {} (schedule: {}) - {}",
                    agent.name, agent.schedule, status
                );
            }
        }
    } else if heartbeat.state == "running" && !process_running {
        println!("Scheduler Status: Stopped (crashed)");
        println!("  Process ID: {} (no longer running)", heartbeat.pid);
        println!(
            "  Last Heartbeat: {}",
            last_heartbeat_time.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!("\n  Use 'switchboard down' to clean up stale files");
    } else {
        println!("Scheduler Status: {}", heartbeat.state);
        println!(
            "  Last Heartbeat: {}",
            last_heartbeat_time.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }

    Ok(())
}

/// Format a chrono Duration into a human-readable string
fn format_duration(duration: chrono::Duration) -> String {
    let total_secs = duration.num_seconds().abs();
    if total_secs < 60 {
        format!("{}s", total_secs)
    } else if total_secs < 3600 {
        format!("{}m {}s", total_secs / 60, total_secs % 60)
    } else {
        format!("{}h {}m", total_secs / 3600, (total_secs % 3600) / 60)
    }
}
