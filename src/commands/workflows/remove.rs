//! Implementation of the workflows remove subcommand.
//!
//! This module provides the `run_workflows_remove` function which removes
//! an installed workflow from the local workflows directory.

use crate::config::Config;
use crate::workflows::WorkflowsError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::types::WorkflowsRemove;
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

/// Removes a workflow from the lockfile
fn remove_workflow_from_lockfile(
    workflows_dir: &Path,
    workflow_name: &str,
) -> Result<(), WorkflowsError> {
    let mut lockfile = read_workflows_lockfile(workflows_dir)?;

    // Remove the workflow entry if it exists
    lockfile.workflows.remove(workflow_name);

    write_workflows_lockfile(&lockfile, workflows_dir)?;

    Ok(())
}

/// Run the `switchboard workflows remove` command
///
/// This command removes an installed workflow from the local workflows directory.
///
/// # Arguments
///
/// * `args` - The [`WorkflowsRemove`] containing the workflow name and options
/// * `_config` - Reference to the application configuration (unused)
///
/// # Returns
///
/// Returns [`ExitCode::Success`] on successful removal, [`ExitCode::Error`] on failure
pub async fn run_workflows_remove(args: WorkflowsRemove, _config: &Config) -> ExitCode {
    let workflow_name = &args.workflow_name;

    // Determine the workflows directory and workflow path
    let workflows_dir = PathBuf::from(WORKFLOWS_DIR);
    let workflow_path = workflows_dir.join(workflow_name);

    // Check if workflow exists
    if !workflow_path.exists() {
        eprintln!(
            "Error: Workflow '{}' not found at {}/",
            workflow_name,
            workflow_path.display()
        );
        eprintln!("Use 'switchboard workflows installed' to see installed workflows");
        return ExitCode::Error;
    }

    // Prompt for confirmation unless --yes flag is set
    if !args.yes {
        let prompt = format!("Remove workflow '{}'? [y/N]", workflow_name);
        if !confirm(&prompt) {
            println!("Operation cancelled.");
            return ExitCode::Success;
        }
    }

    // Remove the workflow directory
    match fs::remove_dir_all(&workflow_path) {
        Ok(()) => {
            // Remove workflow from lockfile
            if let Err(e) = remove_workflow_from_lockfile(&workflows_dir, workflow_name) {
                eprintln!("Warning: Failed to update lockfile: {}", e);
            }

            println!("Removed workflow '{}'", workflow_name);
            ExitCode::Success
        }
        Err(e) => {
            eprintln!("Error: Failed to remove workflow directory: {}", e);
            ExitCode::Error
        }
    }
}

/// Prompts the user for confirmation and returns their choice.
///
/// # Arguments
///
/// * `prompt` - The confirmation prompt to display
///
/// # Returns
///
/// * `true` - If the user confirms with 'y' or 'Y'
/// * `false` - If the user declines with 'n', 'N', or presses Enter (default)
fn confirm(prompt: &str) -> bool {
    print!("{} ", prompt);
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let input = input.trim().to_lowercase();
    input == "y"
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
                source: "kkingsbe/switchboard-workflows".to_string(),
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

    #[test]
    fn test_remove_workflow_from_lockfile() {
        let temp_dir = TempDir::new().unwrap();

        // Create and write a lockfile with a workflow
        let mut lockfile = WorkflowsLockfile::default();
        lockfile.workflows.insert(
            "test-workflow".to_string(),
            WorkflowLockEntry {
                workflow_name: "test-workflow".to_string(),
                source: "kkingsbe/switchboard-workflows".to_string(),
                installed_at: "2024-01-01T00:00:00Z".to_string(),
            },
        );
        write_workflows_lockfile(&lockfile, temp_dir.path()).unwrap();

        // Remove the workflow
        remove_workflow_from_lockfile(temp_dir.path(), "test-workflow").unwrap();

        // Verify it's removed
        let read_lockfile = read_workflows_lockfile(temp_dir.path()).unwrap();
        assert!(!read_lockfile.workflows.contains_key("test-workflow"));
    }
}
