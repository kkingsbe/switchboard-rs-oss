//! CLI up command implementation
//!
//! This module contains the run_up command handler and its helper functions.

use crate::config::{Config, ConfigError};
use crate::docker::{check_docker_available, DockerClient};
use crate::logging::init_logging;
use crate::scheduler::Scheduler;
use crate::scheduler::SchedulerError;
use crate::traits::{DockerClientTrait, ProcessExecutorTrait, RealProcessExecutor};

#[cfg(feature = "discord")]
use crate::config::env::resolve_config_value;
#[cfg(feature = "discord")]
use crate::discord::config::LlmConfig;
#[cfg(feature = "discord")]
use tokio::sync::broadcast;

use bollard::container::ListContainersOptions;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::time::Duration;

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

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
                    std::env::set_var("DISCORD_TOKEN", &token);
                }

                // Set channel ID (use as-is from TOML, or resolve if it has ${VAR} syntax)
                let channel_id = resolve_config_value(&full_config.channel_id);
                if !channel_id.is_empty() {
                    std::env::set_var("DISCORD_CHANNEL_ID", &channel_id);
                }

                // Handle LLM API key
                let api_key = if let Some(llm_config) = &full_config.llm {
                    // Use resolve_config_value to handle ${VAR} syntax
                    resolve_config_value(&llm_config.api_key_env)
                } else {
                    String::new()
                };

                if !api_key.is_empty() {
                    std::env::set_var("OPENROUTER_API_KEY", &api_key);
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

/// Create a default process executor

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

/// Check if the Docker image exists locally
///
/// # Arguments
/// * `client` - The Docker client to use
/// * `image_name` - The name of the image
/// * `image_tag` - The tag of the image
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

/// Run the 'up' command - Start the scheduler
///
/// This command starts the Switchboard scheduler which manages automated
/// agent executions based on their configured schedules.
///
/// # Functionality
///
/// The function performs the following steps:
/// 1. Loads and validates the configuration file
/// 2. Checks for and cleans up stale PID files from previous runs
/// 3. Verifies no other scheduler is already running
/// 4. Initializes logging
/// 5. Validates the workspace path exists
/// 6. Prints configuration summary
/// 7. Creates scheduler and registers all configured agents
/// 8. Checks Docker availability and image existence
/// 9. Starts the scheduler in either foreground or detached mode
///
/// # Arguments
///
/// * `args` - The [`UpCommand`] containing CLI arguments:
///   - `args.detach`: Run in background mode
/// * `config_path` - Optional path to the configuration file
///   - If `None`, defaults to `./switchboard.toml`
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Configuration parsing or validation fails
/// - Another scheduler is already running
/// - Workspace path does not exist
/// - Docker is not available
/// - Scheduler fails to start
///
/// # Modes
///
/// ## Foreground Mode (default)
///
/// In foreground mode, the scheduler runs in the current terminal:
/// - The scheduler starts and runs indefinitely
/// - Press Ctrl+C to stop the scheduler
/// - All agent output is streamed to the console
/// - Discord listener is started automatically if configured
///
/// ## Detached Mode
///
/// In detached mode, the scheduler runs in the background:
/// - The scheduler runs as a background process
/// - Control returns to the terminal immediately
/// - Logs are written to the configured log directory
/// - Use `switchboard list` to check scheduler status
///
/// # PID File Behavior
///
/// A PID file is used to track the running scheduler:
/// - Location: `.switchboard/scheduler.pid`
/// - Contains the process ID of the running scheduler
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
    args: crate::cli::UpCommand,
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
                            if let Err(e) = crate::discord::start_discord_listener_with_shutdown(
                                Some(shutdown_rx),
                                None,
                            )
                            .await
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
                                    crate::discord::start_discord_listener_with_shutdown(
                                        Some(shutdown_rx),
                                        None,
                                    )
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
