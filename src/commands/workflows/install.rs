//! Handler for the `workflows install` subcommand.
//!
//! This module provides the `run_workflows_install` function which downloads
//! and installs a workflow from the kkingsbe/switchboard-workflows repository.

use crate::config::Config;
use crate::workflows::github::GitHubClient;
use crate::workflows::WorkflowsError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use super::types::WorkflowsInstall;
use super::ExitCode;

/// Directory where workflows are installed
const WORKFLOWS_DIR: &str = ".switchboard/workflows";
/// Lockfile filename for workflows
const WORKFLOWS_LOCKFILE: &str = "workflows.lock.json";
/// Source repository for workflows
const WORKFLOWS_SOURCE: &str = "kkingsbe/switchboard-workflows";

/// Represents a single workflow entry in the lockfile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowLockEntry {
    /// Name of the workflow
    #[serde(rename = "workflow_name")]
    pub workflow_name: String,
    /// Source repository
    pub source: String,
    /// When the workflow was installed
    #[serde(rename = "installed_at")]
    pub installed_at: String,
}

/// Represents the workflows lockfile structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowsLockfile {
    /// Lockfile version
    pub version: u32,
    /// Map of workflow name to entry
    pub workflows: std::collections::HashMap<String, WorkflowLockEntry>,
}

impl Default for WorkflowsLockfile {
    fn default() -> Self {
        Self {
            version: 1,
            workflows: std::collections::HashMap::new(),
        }
    }
}

/// Reads the workflows lockfile from the workflows directory
fn read_workflows_lockfile(workflows_dir: &PathBuf) -> Result<WorkflowsLockfile, WorkflowsError> {
    let lockfile_path = workflows_dir.join(WORKFLOWS_LOCKFILE);
    
    if !lockfile_path.exists() {
        return Ok(WorkflowsLockfile::default());
    }
    
    let contents = fs::read_to_string(&lockfile_path)?;
    let lockfile: WorkflowsLockfile = serde_json::from_str(&contents)
        .map_err(|e| WorkflowsError::DecodeError(e.to_string()))?;
    
    Ok(lockfile)
}

/// Writes the workflows lockfile to the workflows directory
fn write_workflows_lockfile(
    lockfile: &WorkflowsLockfile,
    workflows_dir: &PathBuf,
) -> Result<(), WorkflowsError> {
    // Ensure the directory exists
    if !workflows_dir.exists() {
        fs::create_dir_all(workflows_dir)?;
    }
    
    let lockfile_path = workflows_dir.join(WORKFLOWS_LOCKFILE);
    let json = serde_json::to_string_pretty(lockfile)
        .map_err(|e| WorkflowsError::DecodeError(e.to_string()))?;
    fs::write(&lockfile_path, json)?;
    
    Ok(())
}

/// Adds a workflow to the lockfile
fn add_workflow_to_lockfile(
    workflows_dir: &PathBuf,
    workflow_name: &str,
) -> Result<(), WorkflowsError> {
    let mut lockfile = read_workflows_lockfile(workflows_dir)?;
    
    let entry = WorkflowLockEntry {
        workflow_name: workflow_name.to_string(),
        source: WORKFLOWS_SOURCE.to_string(),
        installed_at: Utc::now().to_rfc3339(),
    };
    
    lockfile.workflows.insert(workflow_name.to_string(), entry);
    write_workflows_lockfile(&lockfile, workflows_dir)?;
    
    Ok(())
}

/// Run the `switchboard workflows install` command
///
/// This command downloads a workflow from the switchboard-workflows repository
/// and installs it to the local workflows directory.
///
/// # Arguments
///
/// * `args` - The [`WorkflowsInstall`] containing the workflow name and options
/// * `_config` - Reference to the application configuration (unused)
///
/// # Returns
///
/// Returns [`ExitCode::Success`] on successful installation, [`ExitCode::Error`] on failure
pub async fn run_workflows_install(args: WorkflowsInstall, _config: &Config) -> ExitCode {
    let workflow_name = &args.workflow_name;
    
    // Determine the workflows directory
    let workflows_dir = PathBuf::from(WORKFLOWS_DIR);
    let workflow_path = workflows_dir.join(workflow_name);
    
    // Check if destination already exists
    if workflow_path.exists() {
        if !args.yes {
            eprintln!(
                "Error: Workflow '{}' already exists at {}/",
                workflow_name,
                workflow_path.display()
            );
            eprintln!("Use --yes flag to overwrite existing workflow");
            return ExitCode::Error;
        }
        // If --yes is provided, remove existing workflow to allow reinstall
        if let Err(e) = fs::remove_dir_all(&workflow_path) {
            eprintln!("Error: Failed to remove existing workflow: {}", e);
            return ExitCode::Error;
        }
    }
    
    // Create workflows directory if it doesn't exist
    if !workflows_dir.exists() {
        if let Err(e) = fs::create_dir_all(&workflows_dir) {
            eprintln!("Error: Failed to create workflows directory: {}", e);
            return ExitCode::Error;
        }
    }
    
    // Download the workflow from GitHub
    let client = GitHubClient::new();
    
    match client.download_workflow(workflow_name, &workflow_path).await {
        Ok(files_downloaded) => {
            println!(
                "Downloaded {} files for workflow '{}'",
                files_downloaded, workflow_name
            );
        }
        Err(WorkflowsError::NotFound(_)) => {
            eprintln!("Error: Workflow '{}' not found in registry", workflow_name);
            return ExitCode::Error;
        }
        Err(e) => {
            eprintln!("Error: Failed to download workflow: {}", e);
            // Clean up the directory if download failed
            let _ = fs::remove_dir_all(&workflow_path);
            return ExitCode::Error;
        }
    }
    
    // Attempt to download manifest.toml
    let manifest_path = workflow_path.join("manifest.toml");
    match client.download_manifest_to_file(workflow_name, &manifest_path).await {
        Ok(()) => {
            println!("Downloaded manifest.toml for workflow '{}'", workflow_name);
        }
        Err(WorkflowsError::NotFound(_)) => {
            // Backward compatibility: manifest.toml is optional
            println!(
                "Warning: No manifest.toml found for workflow '{}' (optional)",
                workflow_name
            );
        }
        Err(e) => {
            println!(
                "Warning: Failed to download manifest.toml for workflow '{}': {}",
                workflow_name, e
            );
        }
    }
    
    // Update lockfile
    if let Err(e) = add_workflow_to_lockfile(&workflows_dir, workflow_name) {
        eprintln!("Warning: Failed to update lockfile: {}", e);
    }
    
    println!(
        "Successfully installed workflow '{}' to {}/",
        workflow_name,
        workflow_path.display()
    );
    
    ExitCode::Success
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workflows_lockfile_default() {
        let lockfile = WorkflowsLockfile::default();
        assert_eq!(lockfile.version, 1);
        assert!(lockfile.workflows.is_empty());
    }

    #[test]
    fn test_read_write_lockfile() {
        let temp_dir = TempDir::new().unwrap();
        
        // Write a lockfile
        let mut lockfile = WorkflowsLockfile::default();
        lockfile.workflows.insert(
            "test-workflow".to_string(),
            WorkflowLockEntry {
                workflow_name: "test-workflow".to_string(),
                source: WORKFLOWS_SOURCE.to_string(),
                installed_at: "2024-01-01T00:00:00Z".to_string(),
            },
        );
        
        write_workflows_lockfile(&lockfile, temp_dir.path()).unwrap();
        
        // Read it back
        let read_lockfile = read_workflows_lockfile(temp_dir.path()).unwrap();
        assert_eq!(read_lockfile.version, 1);
        assert!(read_lockfile.workflows.contains_key("test-workflow"));
    }

    #[test]
    fn test_read_nonexistent_lockfile() {
        let temp_dir = TempDir::new().unwrap();
        
        // Should return default lockfile if file doesn't exist
        let lockfile = read_workflows_lockfile(temp_dir.path()).unwrap();
        assert_eq!(lockfile.version, 1);
        assert!(lockfile.workflows.is_empty());
    }
}
