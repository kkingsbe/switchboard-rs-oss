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
use crate::commands::{metrics, BuildCommand, SkillsCommand, ValidateCommand};
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

pub mod commands;
pub mod discord;
pub mod process;
pub use discord::{
    is_discord_configured, load_discord_config_from_toml, DiscordFullConfig, DiscordTomlSection,
};
pub use process::*;

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
        Commands::List => commands::list::run_list(cli.config),
        Commands::Logs(args) => run_logs(args, cli.config).await,
        Commands::Metrics(args) => run_metrics(args, cli.config),
        Commands::Down(args) => run_down(args, cli.config).await,
        Commands::Validate(args) => run_validate(args, cli.config).await,
        Commands::Skills(args) => commands::skills::run_skills(args, cli.config).await,
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
    crate::cli::commands::up::run_up(args, config_path).await
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
