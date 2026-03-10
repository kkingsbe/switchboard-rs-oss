//! Handler for the 'workflows' command
//!
//! This command manages Switchboard workflows - listing, installing, and removing workflows.

use crate::commands::WorkflowsCommand;
use crate::config::{Config, ConfigError};

/// Handler for the 'workflows' command
pub async fn run_workflows(
    args: WorkflowsCommand,
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

    let exit_code = crate::commands::workflows::run_workflows(args, &config).await;
    match exit_code {
        crate::commands::workflows::ExitCode::Success => Ok(()),
        crate::commands::workflows::ExitCode::Error => Err("Workflows command failed".into()),
    }
}
