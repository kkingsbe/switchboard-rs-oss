//! Build command - Build Docker images for agents defined in switchboard.toml
//!
//! This module provides:
//! - CLI argument parsing for the build command
//! - Full implementation for building Docker images

use crate::config::{Config, Settings};
use crate::docker::{check_docker_available, DockerClient};
use clap::Parser;
use std::path::PathBuf;

/// Build Docker images for agents defined in switchboard.toml
///
/// This command builds a Docker image for KiloCode agents using the configuration
/// specified in the switchboard.toml file. The image name and tag are read from the
/// settings section of the configuration.
///
/// # Fields
///
/// * `config` - Path to the configuration file (default: `./switchboard.toml`)
/// * `no_cache` - Whether to build without using Docker cache layers
///
/// # Examples
///
/// Build with default settings:
/// ```bash
/// switchboard build
/// ```
///
/// Build with a custom config file:
/// ```bash
/// switchboard build --config /path/to/config.toml
/// ```
///
/// Build without cache:
/// ```bash
/// switchboard build --no-cache
/// ```
///
/// # Notes
///
/// - The Dockerfile must be located in the same directory as the config file
/// - Docker daemon must be running and accessible
/// - The build context is the parent directory of the config file
/// - Image name and tag are read from `[settings.image_name]` and `[settings.image_tag]`
#[derive(Parser, Debug)]
#[command(about = "Build Docker images for agents defined in switchboard.toml")]
pub struct BuildCommand {
    /// Path to config file (default: ./switchboard.toml)
    #[arg(short, long, default_value = "./switchboard.toml")]
    pub config: PathBuf,

    /// Build without using cache
    #[arg(long)]
    pub no_cache: bool,
}

impl BuildCommand {
    /// Execute the build command
    ///
    /// This method loads the configuration, reads the Dockerfile, and builds
    /// the Docker image using the specified settings.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully built the Docker image
    /// * `Err(Box<dyn std::error::Error>)` - Error occurred during build:
    ///   - Configuration file not found or invalid
    ///   - Dockerfile not found
    ///   - Docker daemon not running or inaccessible
    ///   - Docker build failed
    ///
    /// # Notes
    ///
    /// - Prints progress messages to stdout
    /// - Uses the parent directory of the config file as the build context
    /// - Handles Docker connection errors with user-friendly messages
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // Load the config file
        let config = Config::from_toml(&self.config)?;

        // Get the image name and tag from settings (or defaults)
        let default_settings = Settings::default();
        let settings = config.settings.as_ref().unwrap_or(&default_settings);
        let image_name = &settings.image_name;
        let image_tag = &settings.image_tag;

        println!(
            "Config loaded: image_name={}, image_tag={}",
            image_name, image_tag
        );

        // Determine project root from config path and read Dockerfile
        let dockerfile_path = self
            .config
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("Dockerfile");

        // Determine the build context (parent directory of the config file)
        let build_context = self
            .config
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        // Verify .kilocode directory exists (before any Docker operations)
        // This check happens at command entry point per PRD §9
        let kilocode_dir = build_context.join(".kilocode");
        if !kilocode_dir.exists() || !kilocode_dir.is_dir() {
            return Err(Box::from(format!(
                "Required .kilocode directory not found in: {}\n\n\
                The .kilocode directory contains API keys, model configuration, and MCP server\n\
                definitions needed by Kilo Code CLI. Please configure .kilocode/ in the Switchboard\n\
                repo with your API keys before building the agent image.",
                build_context.display()
            )));
        }

        // Check Docker availability before attempting any Docker operations
        // This ensures consistent error handling across all Docker-dependent commands
        check_docker_available()
            .await
            .map_err(|e| format!("Docker availability check failed: {}", e))?;

        let dockerfile_content = std::fs::read_to_string(&dockerfile_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!("Dockerfile not found at: {}", dockerfile_path.display())
            } else {
                format!("Failed to read Dockerfile: {}", e)
            }
        })?;

        println!("Dockerfile read: {} bytes", dockerfile_content.len());

        println!("DEBUG: Creating DockerClient...");

        // Create DockerClient instance
        let docker_client = DockerClient::new(image_name.clone(), image_tag.clone()).await
            .map_err(|e| {
                if e.to_string().contains("Docker connection error") ||
                   e.to_string().contains("connection refused") ||
                   e.to_string().contains("No such file") {
                    "Error: Docker daemon is not running or not available. Please start Docker and try again.".to_string()
                } else {
                    format!("Error: {}", e)
                }
            })?;

        // Build the agent image
        eprintln!("DEBUG: About to call build_agent_image...");
        docker_client
            .build_agent_image(
                &dockerfile_content,
                build_context,
                image_name,
                image_tag,
                self.no_cache,
            )
            .await
            .map_err(|e| format!("Build error: {}", e))?;

        // Print success message
        println!("Successfully built image: {}:{}", image_name, image_tag);

        Ok(())
    }
}
