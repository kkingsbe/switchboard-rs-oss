//! Handler for the 'list' command
//!
//! This command lists all configured agents from the switchboard.toml file.

use crate::commands::list_agents;
use crate::config::{Config, ConfigError};
use std::path::Path;

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
