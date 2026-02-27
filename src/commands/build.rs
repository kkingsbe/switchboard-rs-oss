//! Build command - Build Docker images for agents defined in switchboard.toml
//!
//! This module provides:
//! - CLI argument parsing for the build command
//! - Full implementation for building Docker images

use crate::config::{Config, Settings};
use crate::docker::{check_docker_available, DockerClient};
use crate::ui::colors::{color_error, color_info, color_success};
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
        // Step 1: Load the config file
        println!("[1/5] {}", color_info("Loading configuration..."));
        let config = Config::from_toml(&self.config)?;

        // Get the image name and tag from settings (or defaults)
        let default_settings = Settings::default();
        let settings = config.settings.as_ref().unwrap_or(&default_settings);
        let image_name = &settings.image_name;
        let image_tag = &settings.image_tag;

        println!(
            "{}: image_name={}, image_tag={}",
            color_info("Config loaded"),
            image_name,
            image_tag
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

        // Step 2: Verify .kilocode directory exists
        let kilocode_dir = build_context.join(".kilocode");
        if !kilocode_dir.exists() || !kilocode_dir.is_dir() {
            return Err(Box::from(format!(
                "{}: Required .kilocode directory not found in: {}\n\n\n\
                The .kilocode directory contains API keys, model configuration, and MCP server\n\
                definitions needed by Kilo Code CLI. Please configure .kilocode/ in the Switchboard\n\
                repo with your API keys before building the agent image.",
                color_error("Configuration error"),
                build_context.display()
            )));
        }

        // Step 3: Check Docker availability
        println!("[2/5] {}", color_info("Checking Docker availability..."));
        check_docker_available().await.map_err(|e| {
            format!(
                "{}: Docker availability check failed: {}",
                color_error("Error"),
                e
            )
        })?;

        // Step 4: Read Dockerfile
        println!("[3/5] {}", color_info("Reading Dockerfile..."));
        let dockerfile_content = std::fs::read_to_string(&dockerfile_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!(
                    "{}: Dockerfile not found at: {}",
                    color_error("Error"),
                    dockerfile_path.display()
                )
            } else {
                format!("{}: Failed to read Dockerfile: {}", color_error("Error"), e)
            }
        })?;

        println!(
            "{}: {} bytes",
            color_info("Dockerfile read"),
            dockerfile_content.len()
        );

        println!("[4/5] {}", color_info("Building Docker image..."));

        // Create DockerClient instance
        let docker_client = DockerClient::new(image_name.clone(), image_tag.clone()).await
            .map_err(|e| {
                if e.to_string().contains("Docker connection error") ||
                   e.to_string().contains("connection refused") ||
                   e.to_string().contains("No such file") {
                    format!("{}: Docker daemon is not running or not available. Please start Docker and try again.", color_error("Error"))
                } else {
                    format!("{}: {}", color_error("Error"), e)
                }
            })?;

        // Build the agent image
        docker_client
            .build_agent_image(
                &dockerfile_content,
                build_context,
                image_name,
                image_tag,
                self.no_cache,
            )
            .await
            .map_err(|e| format!("{}: {}", color_error("Build failed"), e))?;

        // Step 5: Print success message
        println!(
            "[5/5] {}",
            color_success(&format!(
                "Image built successfully: {}:{}",
                image_name, image_tag
            ))
        );

        Ok(())
    }
}
