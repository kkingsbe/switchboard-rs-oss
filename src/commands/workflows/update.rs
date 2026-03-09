//! Handler for the `workflows update` subcommand.
//!
//! This module provides the `handle_workflows_update` function which updates
//! installed workflows to their latest versions by re-downloading from
//! the kkingsbe/switchboard-workflows repository.

use crate::commands::workflows::skills::update_workflow_skills;
use crate::config::Config;
use crate::workflows::github::GitHubClient;
use crate::workflows::manifest::ManifestConfig;
use crate::workflows::WorkflowsError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::types::WorkflowsUpdate;
use super::ExitCode;

/// Directory where workflows are installed
const WORKFLOWS_DIR: &str = ".switchboard/workflows";
/// Lockfile filename for workflows
const WORKFLOWS_LOCKFILE: &str = "workflows.lock.json";

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
    pub workflows: HashMap<String, WorkflowLockEntry>,
}

impl Default for WorkflowsLockfile {
    fn default() -> Self {
        Self {
            version: 1,
            workflows: HashMap::new(),
        }
    }
}

/// Reads the workflows lockfile from the workflows directory
fn read_workflows_lockfile(workflows_dir: &Path) -> Result<WorkflowsLockfile, WorkflowsError> {
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
    workflows_dir: &Path,
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

/// Updates the installed_at timestamp for a specific workflow in the lockfile.
///
/// This is called after successfully re-downloading a workflow during an update.
fn update_workflow_timestamp(
    workflows_dir: &PathBuf,
    workflow_name: &str,
) -> Result<(), WorkflowsError> {
    // Read the lockfile
    let mut lockfile = read_workflows_lockfile(workflows_dir)?;

    // Update the timestamp for the specified workflow
    let timestamp = Utc::now().to_rfc3339();
    if let Some(workflow) = lockfile.workflows.get_mut(workflow_name) {
        workflow.installed_at = timestamp;
    }

    // Write the updated lockfile
    write_workflows_lockfile(&lockfile, workflows_dir)
}

/// Handle the `switchboard workflows update` command.
///
/// Updates installed workflows to their latest versions by re-downloading from GitHub.
/// If a specific workflow name is provided, only that workflow is updated.
/// If no workflow name is provided, all installed workflows are updated.
///
/// # Arguments
///
/// * `args` - The command arguments containing an optional workflow name
/// * `_config` - The switchboard configuration (not used in this implementation)
///
/// # Returns
///
/// * `ExitCode` - The exit code indicating success or failure
///
/// # Errors
///
/// Returns an error if:
/// - A specific workflow is requested but not found in the lockfile
/// - No workflow name is provided and the lockfile has no entries
/// - The GitHub API request fails
pub async fn handle_workflows_update(args: WorkflowsUpdate, _config: &Config) -> ExitCode {
    // Determine the workflows directory
    let workflows_dir = PathBuf::from(WORKFLOWS_DIR);

    // Read the lockfile to get workflow sources
    let lockfile = match read_workflows_lockfile(&workflows_dir) {
        Ok(lf) => lf,
        Err(e) => {
            eprintln!("Error: Failed to read lockfile: {}", e);
            return ExitCode::Error;
        }
    };

    // If a specific workflow name is provided, update only that workflow
    if let Some(workflow_name) = &args.workflow_name {
        // Look up the workflow in the lockfile
        let workflow_entry = match lockfile.workflows.get(workflow_name) {
            Some(entry) => entry,
            None => {
                eprintln!(
                    "Error: Workflow '{}' is not in the lockfile. Install it first with 'switchboard workflows install' or update all workflows with 'switchboard workflows update'.",
                    workflow_name
                );
                return ExitCode::Error;
            }
        };

        // Re-download the workflow from GitHub
        let result = update_single_workflow(&workflows_dir, workflow_name, &workflow_entry.source).await;

        // If update was successful, update the timestamp in lockfile
        if result == ExitCode::Success {
            if let Err(e) = update_workflow_timestamp(&workflows_dir, workflow_name) {
                eprintln!("Warning: Failed to update lockfile timestamp: {}", e);
            }
        }

        return result;
    }

    // No workflow name provided - update ALL workflows from lockfile
    if lockfile.workflows.is_empty() {
        eprintln!("Error: No workflows found in lockfile. Install workflows first with 'switchboard workflows install'.");
        return ExitCode::Error;
    }

    let total_workflows = lockfile.workflows.len();
    println!(
        "Updating all {} workflows from lockfile...",
        total_workflows
    );

    let mut updated_count = 0;
    let mut failed_workflows: Vec<String> = Vec::new();

    for (workflow_name, workflow_entry) in &lockfile.workflows {
        println!("Updating workflow '{}'...", workflow_name);

        let result = update_single_workflow(&workflows_dir, workflow_name, &workflow_entry.source).await;

        if result == ExitCode::Success {
            updated_count += 1;
            println!("Successfully updated workflow '{}'", workflow_name);

            // Update timestamp for successfully updated workflow
            if let Err(e) = update_workflow_timestamp(&workflows_dir, workflow_name) {
                eprintln!("Warning: Failed to update lockfile timestamp: {}", e);
            }
        } else {
            eprintln!("Failed to update workflow '{}'", workflow_name);
            failed_workflows.push(workflow_name.clone());
        }
    }

    // Print summary
    println!("Updated {} of {} workflows", updated_count, total_workflows);

    if !failed_workflows.is_empty() {
        eprintln!("Failed to update {} workflow(s): {}", failed_workflows.len(), failed_workflows.join(", "));
    }

    if updated_count == total_workflows {
        ExitCode::Success
    } else {
        ExitCode::Error
    }
}

/// Updates a single workflow by re-downloading from GitHub.
///
/// This function removes the existing workflow directory and re-downloads
/// from the source repository.
///
/// # Arguments
///
/// * `workflows_dir` - The base workflows directory
/// * `workflow_name` - The name of the workflow to update
/// * `_source` - The source repository (unused, source is hardcoded)
///
/// # Returns
///
/// * `ExitCode::Success` - If the workflow was successfully updated
/// * `ExitCode::Error` - If the update failed
async fn update_single_workflow(
    workflows_dir: &PathBuf,
    workflow_name: &str,
    _source: &str,
) -> ExitCode {
    let workflow_path = workflows_dir.join(workflow_name);

    // Remove existing workflow directory to allow re-download
    if workflow_path.exists() {
        if let Err(e) = fs::remove_dir_all(&workflow_path) {
            eprintln!("Error: Failed to remove existing workflow: {}", e);
            return ExitCode::Error;
        }
    }

    // Create workflows directory if it doesn't exist
    if !workflows_dir.exists() {
        if let Err(e) = fs::create_dir_all(workflows_dir) {
            eprintln!("Error: Failed to create workflows directory: {}", e);
            return ExitCode::Error;
        }
    }

    // Download the workflow from GitHub
    let client = GitHubClient::new();

    let download_result: Result<(), WorkflowsError> = match client.download_workflow(workflow_name, &workflow_path).await {
        Ok(files_downloaded) => {
            println!(
                "Downloaded {} files for workflow '{}'",
                files_downloaded, workflow_name
            );
            Ok(())
        }
        Err(WorkflowsError::NotFound(_)) => {
            eprintln!(
                "Error: Workflow '{}' not found in registry. It may have been removed from the source repository.",
                workflow_name
            );
            return ExitCode::Error;
        }
        Err(e) => {
            eprintln!("Error: Failed to download workflow '{}': {}", workflow_name, e);
            return ExitCode::Error;
        }
    };

    // Download manifest.toml (optional but try anyway)
    let manifest_path = workflow_path.join("manifest.toml");
    let _ = client.download_manifest_to_file(workflow_name, &manifest_path).await;

    // Check if manifest has required skills and update them
    let mut skills_to_update: Vec<String> = Vec::new();

    if manifest_path.exists() {
        match ManifestConfig::from_path(&manifest_path) {
            Ok(manifest) => {
                // Collect skills from defaults
                if let Some(defaults) = &manifest.defaults {
                    if let Some(skills) = &defaults.skills {
                        skills_to_update.extend(skills.clone());
                    }
                }

                // Collect skills from each agent
                for agent in &manifest.agents {
                    if let Some(skills) = &agent.skills {
                        skills_to_update.extend(skills.clone());
                    }
                }

                // Remove duplicates
                skills_to_update.sort();
                skills_to_update.dedup();
            }
            Err(e) => {
                println!(
                    "Warning: Failed to parse manifest.toml: {}",
                    e
                );
            }
        }
    }

    // Update required skills if any are specified
    if !skills_to_update.is_empty() {
        println!(
            "\nUpdating {} required skill(s) for workflow...",
            skills_to_update.len()
        );

        match update_workflow_skills(&skills_to_update) {
            Ok(updated) => {
                println!(
                    "Successfully updated {} skill(s): {}",
                    updated.len(),
                    updated.join(", ")
                );
            }
            Err(e) => {
                // Print warning but continue - workflow update was successful
                eprintln!("Warning: Failed to update required skills: {}", e);
            }
        }
    }

    download_result.map(|_| ExitCode::Success).unwrap_or(ExitCode::Error)
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
                source: crate::commands::workflows::install::WORKFLOWS_SOURCE.to_string(),
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
