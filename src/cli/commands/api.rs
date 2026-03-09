//! CLI API command implementation
//!
//! This module contains the API subcommand handler for starting
//! the REST API server.

use crate::api::registry::{
    derive_instance_id_from_config, InstanceRegistration, InstanceRegistry, InstanceStatus,
};
use crate::config::{ApiConfig, Config};
use clap::{Parser, Subcommand};
use std::net::TcpListener;
use std::path::Path;
use std::sync::Mutex;
use thiserror::Error;
use tracing::{error, info, warn};

/// Global instance registry for cleanup on shutdown
static INSTANCE_REGISTRY: Mutex<Option<InstanceRegistry>> = Mutex::new(None);

/// Register an instance with the registry.
///
/// # Arguments
///
/// * `instance_id` - The instance identifier.
/// * `port` - The API server port.
/// * `host` - The host address.
/// * `config_path` - Path to the config file.
///
/// # Returns
///
/// * `Result<(), ApiCommandError>` - Success or error.
fn register_instance(
    instance_id: &str,
    port: u16,
    host: &str,
    config_path: &str,
) -> Result<(), ApiCommandError> {
    let mut registry = InstanceRegistry::load().map_err(|e| {
        ApiCommandError::RegistryError(format!("Failed to load registry: {}", e))
    })?;

    let registration = InstanceRegistration::new(
        instance_id.to_string(),
        port,
        host.to_string(),
        config_path.to_string(),
    );

    registry.register(registration).map_err(|e| {
        ApiCommandError::RegistryError(format!("Failed to register instance: {}", e))
    })?;

    // Store registry globally for cleanup
    let mut global_registry = INSTANCE_REGISTRY.lock().unwrap();
    *global_registry = Some(registry);

    info!("Registered instance: {} on port {}", instance_id, port);
    Ok(())
}

/// Unregister an instance from the registry.
///
/// # Arguments
///
/// * `instance_id` - The instance identifier.
fn unregister_instance(instance_id: &str) {
    if let Ok(mut registry) = InstanceRegistry::load() {
        if let Err(e) = registry.update_status(instance_id, InstanceStatus::Stopped) {
            warn!("Failed to update instance status: {}", e);
        }
        if let Err(e) = registry.unregister(instance_id) {
            warn!("Failed to unregister instance: {}", e);
        }
    }
    info!("Unregistered instance: {}", instance_id);
}

/// Error types for API command operations.
#[derive(Debug, Error)]
pub enum ApiCommandError {
    /// Failed to load configuration.
    #[error("Failed to load configuration: {0}")]
    ConfigError(#[from] crate::config::ConfigError),

    /// Failed to start the API server.
    #[error("Failed to start API server: {0}")]
    ServerError(#[from] crate::api::ApiServerError),

    /// Invalid configuration path.
    #[error("Invalid configuration path: {0}")]
    InvalidPath(String),

    /// Port allocation failed.
    #[error("Failed to find available port in range 18500-18699")]
    PortAllocationFailed,

    /// API is not enabled in configuration.
    #[error("API is not enabled. Set enabled = true in [api] section of config file")]
    NotEnabled,

    /// Registry error.
    #[error("Registry error: {0}")]
    RegistryError(String),
}

/// API subcommand.
#[derive(Parser)]
#[command(name = "api", about = "Start the REST API server")]
pub struct ApiCommand {
    #[command(subcommand)]
    pub subcommand: ApiSubcommand,
}

/// API subcommand variants.
#[derive(Subcommand)]
pub enum ApiSubcommand {
    /// Start the API server
    Start(ApiStartArgs),
}

/// Arguments for the API start command.
#[derive(Parser)]
#[command(about = "Start the REST API server")]
pub struct ApiStartArgs {
    /// Path to the configuration file (default: switchboard.toml)
    #[arg(short, long, value_name = "PATH", default_value = "switchboard.toml")]
    pub config: String,

    /// Override the API port from configuration
    #[arg(long, value_name = "PORT")]
    pub port: Option<u16>,

    /// Override the API host from configuration
    #[arg(long, value_name = "HOST")]
    pub host: Option<String>,

    /// Enable API server (override config)
    #[arg(long)]
    pub enabled: Option<bool>,
}

/// Load API configuration from switchboard.toml
///
/// This function loads the main config file and extracts the API section,
/// then applies environment variable overrides.
///
/// # Arguments
///
/// * `config_path` - Path to the configuration file
/// * `port_override` - Optional port override from CLI
/// * `host_override` - Optional host override from CLI
/// * `enabled_override` - Optional enabled override from CLI
///
/// # Returns
///
/// Returns the configured ApiConfig or an error.
fn load_api_config(
    config_path: &str,
    port_override: Option<u16>,
    host_override: Option<String>,
    enabled_override: Option<bool>,
) -> Result<ApiConfig, ApiCommandError> {
    // Check if config file exists
    let path = Path::new(config_path);
    if !path.exists() {
        return Err(ApiCommandError::InvalidPath(format!(
            "Configuration file not found: {}",
            config_path
        )));
    }

    // Load main config
    tracing::info!("Loading configuration from: {}", config_path);
    let config = Config::from_toml(Path::new(config_path))?;

    // Get API config or use defaults
    let mut api_config = config.api.unwrap_or_default();

    // Apply environment variable overrides
    if let Ok(env_port) = std::env::var("SWITCHBOARD_API_PORT") {
        if let Ok(port) = env_port.parse::<u16>() {
            tracing::info!("Overriding API port from environment: {}", port);
            api_config.port = port;
        }
    }

    if let Ok(env_host) = std::env::var("SWITCHBOARD_API_HOST") {
        tracing::info!("Overriding API host from environment: {}", env_host);
        api_config.host = env_host;
    }

    if let Ok(env_enabled) = std::env::var("SWITCHBOARD_API_ENABLED") {
        let enabled = env_enabled.to_lowercase() == "true" || env_enabled == "1";
        tracing::info!("Overriding API enabled from environment: {}", enabled);
        api_config.enabled = enabled;
    }

    if let Ok(instance_id) = std::env::var("SWITCHBOARD_INSTANCE_ID") {
        tracing::info!("Overriding instance ID from environment: {}", instance_id);
        api_config.instance_id = Some(instance_id);
    }

    // If instance_id is not set, derive from config file path
    if api_config.instance_id.is_none() {
        let derived_id = derive_instance_id_from_config(config_path);
        tracing::info!("Derived instance ID from config path: {}", derived_id);
        api_config.instance_id = Some(derived_id);
    }

    // Apply CLI argument overrides (highest priority)
    if let Some(port) = port_override {
        api_config.port = port;
    }

    if let Some(host) = host_override {
        api_config.host = host;
    }

    if let Some(enabled) = enabled_override {
        api_config.enabled = enabled;
    }

    Ok(api_config)
}

/// Find an available port starting from the given port
///
/// This function checks if the port is available, and if auto_port is enabled,
/// it will try subsequent ports until it finds one that's available.
///
/// # Arguments
///
/// * `start_port` - The port to start checking from
/// * `auto_port` - Whether to automatically find the next available port
///
/// # Returns
///
/// Returns the available port or an error if no ports are available.
fn find_available_port(start_port: u16, auto_port: bool) -> Result<u16, ApiCommandError> {
    // Try ports in range 18500-18699
    let max_port = 18699u16;
    let mut port = start_port;

    while port <= max_port {
        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(_) => {
                tracing::debug!("Port {} is available", port);
                return Ok(port);
            }
            Err(e) => {
                tracing::debug!("Port {} is in use: {}", port, e);
            }
        }

        if !auto_port {
            // If auto_port is false, don't try other ports
            break;
        }

        port += 1;
    }

    // If we get here, no ports are available
    if auto_port {
        Err(ApiCommandError::PortAllocationFailed)
    } else {
        // If auto_port is false and the start port is in use, return an error
        Err(ApiCommandError::PortAllocationFailed)
    }
}

/// Run the API command.
pub async fn run_api(args: ApiCommand) -> Result<(), Box<dyn std::error::Error>> {
    match args.subcommand {
        ApiSubcommand::Start(start_args) => run_api_start(start_args).await,
    }
}

/// Run the API start subcommand.
async fn run_api_start(args: ApiStartArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Load API configuration
    let api_config = load_api_config(
        &args.config,
        args.port,
        args.host,
        args.enabled,
    )?;

    // Check if API is enabled
    if !api_config.enabled {
        return Err(ApiCommandError::NotEnabled.into());
    }

    // Get the instance_id (already derived in load_api_config)
    let instance_id = api_config.instance_id.clone().unwrap_or_else(|| {
        derive_instance_id_from_config(&args.config)
    });

    // Handle port allocation
    let port = if api_config.auto_port {
        find_available_port(api_config.port, true)?
    } else {
        // Try to bind to the specified port
        match find_available_port(api_config.port, false) {
            Ok(p) => p,
            Err(_) => {
                // Port is in use and auto_port is false
                return Err(ApiCommandError::PortAllocationFailed.into());
            }
        }
    };

    // Create the final config with the resolved port
    let mut final_config = api_config.clone();
    final_config.port = port;

    // Register the instance
    if let Err(e) = register_instance(
        &instance_id,
        port,
        &final_config.host,
        &args.config,
    ) {
        warn!("Failed to register instance: {}", e);
        // Continue anyway - not a fatal error
    }

    // Print server information
    let server_url = format!("http://{}:{}", final_config.host, final_config.port);
    println!("Starting Switchboard API server...");
    println!("Instance ID: {}", instance_id);
    println!("Server URL: {}", server_url);
    println!("Health endpoint: {}/health", server_url);
    println!("API version: v1");
    println!();
    println!("Press Ctrl+C to stop the server");

    // Set up panic hook for cleanup
    let instance_id_for_cleanup = instance_id.clone();
    std::panic::set_hook(Box::new(move |_panic_info| {
        error!("API server panicked, cleaning up instance: {}", instance_id_for_cleanup);
        unregister_instance(&instance_id_for_cleanup);
    }));

    // Start the API server
    let result = crate::api::serve_with_config(final_config, Some(&args.config)).await;

    // Unregister the instance on shutdown
    unregister_instance(&instance_id);

    match result {
        Ok(_) => {
            tracing::info!("API server stopped");
            Ok(())
        }
        Err(e) => {
            error!("API server error: {}", e);
            Err(Box::new(e))
        }
    }
}
