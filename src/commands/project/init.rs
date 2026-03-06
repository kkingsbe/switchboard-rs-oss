//! Project init command implementation
//!
//! This module provides the functionality to scaffold a new Switchboard project
//! with the standard directory structure and minimal configuration.

use crate::commands::project::types::{ExitCode, ProjectInit};
use std::fs;
use std::path::{Path, PathBuf};

/// Run the `switchboard project init` command
///
/// This command scaffolds a new Switchboard project with the following structure:
/// - switchboard.toml - Main configuration file
/// - .switchboard/ - Local data directory with .gitkeep
/// - skills/ - Project-level skills directory (unless --minimal)
/// - prompts/ - Agent prompt files directory (unless --minimal)
/// - .gitignore - Standard gitignore
/// - README.md - Getting started documentation
///
/// # Arguments
///
/// * `args` - The ProjectInit arguments containing path, name, force, and minimal options
///
/// # Returns
///
/// * `ExitCode::Success` - Project created successfully
/// * `ExitCode::Error` - Project creation failed
pub async fn run_project_init(args: ProjectInit) -> ExitCode {
    // 1. Determine target directory
    let target_dir = if args.path == "." {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        PathBuf::from(&args.path)
    };

    // 2. Resolve to absolute path
    let target_dir = if target_dir.is_absolute() {
        target_dir
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&target_dir)
    };

    // 3. Get project name from args.name or derive from directory path
    let project_name = args.name.clone().unwrap_or_else(|| {
        target_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "switchboard-project".to_string())
    });

    // 4. Validate directory
    if let Err(e) = validate_directory(&target_dir, args.force) {
        eprintln!("Error: {}", e);
        return ExitCode::Error;
    }

    // 5. Create project structure
    if let Err(e) = create_project_structure(&target_dir, &project_name, args.minimal) {
        eprintln!("Error: {}", e);
        return ExitCode::Error;
    }

    println!(
        "Successfully initialized Switchboard project '{}' at {}",
        project_name,
        target_dir.display()
    );

    ExitCode::Success
}

/// Validate the target directory
///
/// Checks if:
/// - Directory exists and is not empty (unless --force)
/// - Directory doesn't already contain a switchboard project (unless --force)
fn validate_directory(target_dir: &Path, force: bool) -> Result<(), String> {
    if !target_dir.exists() {
        // Directory doesn't exist - this is fine, we'll create it
        return Ok(());
    }

    if !target_dir.is_dir() {
        return Err(format!(
            "Path '{}' exists but is not a directory",
            target_dir.display()
        ));
    }

    // Check if directory is empty
    let entries = fs::read_dir(target_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    let entries_vec: Vec<_> = entries.filter_map(|e| e.ok()).collect();

    // If directory is empty or only has hidden files, it's okay
    if entries_vec.is_empty() {
        return Ok(());
    }

    // Check for existing switchboard.toml
    if target_dir.join("switchboard.toml").exists() {
        if force {
            println!("Warning: Overwriting existing Switchboard project");
            return Ok(());
        }
        return Err(
            "Directory already contains a Switchboard project. Use --force to overwrite.".to_string()
        );
    }

    // Check if directory has non-hidden files
    let has_visible_files = entries_vec
        .iter()
        .any(|e| {
            let name = e.file_name();
            !name.to_string_lossy().starts_with('.')
        });

    if has_visible_files && !force {
        return Err(
            "Directory is not empty. Use --force to overwrite.".to_string()
        );
    }

    Ok(())
}

/// Create the project directory structure and files
fn create_project_structure(
    target_dir: &Path,
    project_name: &str,
    minimal: bool,
) -> Result<(), String> {
    // Create main directories
    let mut dirs_to_create = vec![
        target_dir.join(".switchboard"),
        target_dir.join(".switchboard/logs"),
    ];

    if !minimal {
        dirs_to_create.push(target_dir.join("skills"));
        dirs_to_create.push(target_dir.join("prompts"));
    }

    for dir in dirs_to_create {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory '{}': {}", dir.display(), e))?;
    }

    // Create .gitkeep files for empty directories
    let mut gitkeep_files = vec![
        target_dir.join(".switchboard").join(".gitkeep"),
    ];

    if !minimal {
        gitkeep_files.push(target_dir.join("skills").join(".gitkeep"));
        gitkeep_files.push(target_dir.join("prompts").join(".gitkeep"));
    }

    for gitkeep in gitkeep_files {
        if !gitkeep.exists() {
            fs::write(&gitkeep, "").map_err(|e| format!("Failed to create '{}': {}", gitkeep.display(), e))?;
        }
    }

    // Create switchboard.toml
    let config_content = if minimal {
        get_minimal_config(project_name)
    } else {
        get_full_config(project_name)
    };

    let config_path = target_dir.join("switchboard.toml");
    fs::write(&config_path, config_content)
        .map_err(|e| format!("Failed to create '{}': {}", config_path.display(), e))?;

    // Create .gitignore
    let gitignore_path = target_dir.join(".gitignore");
    fs::write(&gitignore_path, get_gitignore())
        .map_err(|e| format!("Failed to create '{}': {}", gitignore_path.display(), e))?;

    // Create README.md
    let readme_path = target_dir.join("README.md");
    fs::write(&readme_path, get_readme(project_name, minimal))
        .map_err(|e| format!("Failed to create '{}': {}", readme_path.display(), e))?;

    Ok(())
}

/// Get the minimal switchboard.toml configuration
fn get_minimal_config(_project_name: &str) -> String {
    r#"# Switchboard Project Configuration
# Generated by switchboard project init

[settings]
# Docker image name for agent containers
image_name = "switchboard-agent"
image_tag = "latest"

# Log directory
log_dir = ".switchboard/logs"

# Timezone for schedules
timezone = "system"

# =============================================================================
# AGENT CONFIGURATION
# =============================================================================
# To add agents, uncomment and customize the following:
# [[agent]]
# name = "example-agent"
# schedule = "0 9 * * *"
# prompt = "Your agent prompt here"
"#.to_string()
}

/// Get the full switchboard.toml configuration (non-minimal)
fn get_full_config(_project_name: &str) -> String {
    r#"# Switchboard Project Configuration
# Generated by switchboard project init

[settings]
# Docker image name for agent containers
image_name = "switchboard-agent"
image_tag = "latest"

# Log directory
log_dir = ".switchboard/logs"

# Timezone for schedules
timezone = "system"

# Overlap mode: skip new runs if agent is already running
overlap_mode = "skip"

# =============================================================================
# EXAMPLE AGENT
# =============================================================================

[[agent]]
name = "example-agent"
schedule = "0 9 * * *"
prompt = """
This is an example agent. Edit this prompt to customize your agent's behavior.

The agent will run daily at 9:00 AM (according to the configured timezone).
"""

# =============================================================================
# GETTING STARTED
# =============================================================================
#
# 1. Edit this configuration file to define your agents
# 2. Add prompts to the prompts/ directory
# 3. Run 'switchboard up' to start the scheduler
# 4. Run 'switchboard list' to see your configured agents
#
"#.to_string()
}

/// Get the .gitignore content
fn get_gitignore() -> String {
    r#"# Switchboard
.switchboard/
*.log

# Editor
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db
"#
    .to_string()
}

/// Get the README.md content
fn get_readme(project_name: &str, minimal: bool) -> String {
    let structure = if minimal {
        r#"./
├── switchboard.toml    # Main configuration file
├── .switchboard/       # Local data (logs, PID files)
├── .gitignore         # Git ignore rules
└── README.md          # This file"#
    } else {
        r#"./
├── switchboard.toml    # Main configuration file
├── .switchboard/       # Local data (logs, PID files)
├── skills/            # Project-level skills
├── prompts/           # Agent prompt files
├── .gitignore         # Git ignore rules
└── README.md          # This file"#
    };

    format!(
        r#"# {}

This is a Switchboard project. It contains AI coding agents that run on scheduled intervals.

## Getting Started

1. Edit `switchboard.toml` to configure your agents
2. Add prompts to the `prompts/` directory
3. Run `switchboard up` to start the scheduler

## Project Structure

```
{}
```

## Available Commands

```bash
# Start the scheduler
switchboard up

# List configured agents
switchboard list

# View logs
switchboard logs

# Stop the scheduler
switchboard down
```

For more information, see the [documentation](https://github.com/switchboard/switchboard-rs-oss/docs).
"#,
        project_name, structure
    )
}
