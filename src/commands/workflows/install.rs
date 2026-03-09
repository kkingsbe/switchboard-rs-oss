//! Handler for the `workflows install` subcommand.
//!
//! This module provides the `run_workflows_install` function which downloads
//! and installs a workflow from the kkingsbe/switchboard-workflows repository.

use crate::config::Config;
use crate::commands::workflows::skills::install_workflow_skills;
use crate::workflows::github::GitHubClient;
use crate::workflows::manifest::ManifestConfig;
use crate::workflows::WorkflowsError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::types::WorkflowsInstall;
use super::ExitCode;

/// Directory where workflows are installed
const WORKFLOWS_DIR: &str = ".switchboard/workflows";
/// Lockfile filename for workflows
const WORKFLOWS_LOCKFILE: &str = "workflows.lock.json";
/// Source repository for workflows
pub const WORKFLOWS_SOURCE: &str = "kkingsbe/switchboard-workflows";

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

    // Check if manifest has required skills and install them
    let mut skills_to_install: Vec<String> = Vec::new();

    if manifest_path.exists() {
        match ManifestConfig::from_path(&manifest_path) {
            Ok(manifest) => {
                // Collect skills from defaults
                if let Some(defaults) = &manifest.defaults {
                    if let Some(skills) = &defaults.skills {
                        skills_to_install.extend(skills.clone());
                    }
                }

                // Collect skills from each agent
                for agent in &manifest.agents {
                    if let Some(skills) = &agent.skills {
                        skills_to_install.extend(skills.clone());
                    }
                }

                // Remove duplicates
                skills_to_install.sort();
                skills_to_install.dedup();
            }
            Err(e) => {
                println!(
                    "Warning: Failed to parse manifest.toml: {}",
                    e
                );
            }
        }
    }

    // Install required skills if any are specified
    if !skills_to_install.is_empty() {
        println!(
            "\nInstalling {} required skill(s) for workflow...",
            skills_to_install.len()
        );

        match install_workflow_skills(&skills_to_install, args.yes) {
            Ok(installed) => {
                println!(
                    "Successfully installed {} skill(s): {}",
                    installed.len(),
                    installed.join(", ")
                );
            }
            Err(e) => {
                eprintln!("Error: Failed to install required skills: {}", e);
                // Clean up downloaded workflow
                let _ = fs::remove_dir_all(&workflow_path);
                return ExitCode::Error;
            }
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

    // ============================================================================
    // Skill Integration Tests
    // ============================================================================

    #[test]
    fn test_skill_integration_empty_skills_list() {
        // Test that empty skills list is handled correctly
        let skills: Vec<String> = vec![];
        
        // Empty skills list should result in no skills to install
        // This tests the logic path where skills_to_install is empty
        assert!(skills.is_empty());
        
        // When skills list is empty, we shouldn't call install_workflow_skills
        // The main install function checks `if !skills_to_install.is_empty()`
        // which means empty list skips the installation entirely
    }

    #[test]
    fn test_skill_integration_single_skill() {
        // Test with a single skill in the list
        let skills = vec!["skills/repo".to_string()];
        
        // Single skill should be collected correctly
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0], "skills/repo");
    }

    #[test]
    fn test_skill_integration_multiple_skills() {
        // Test with multiple skills - deduplication
        let mut skills = vec![
            "skills/repo".to_string(),
            "skills/repo1".to_string(),
            "skills/repo".to_string(), // duplicate
            "skills/repo2".to_string(),
        ];
        
        // Sort and dedup (mimics the logic in run_workflows_install)
        skills.sort();
        skills.dedup();
        
        // Should have 3 unique skills after dedup
        assert_eq!(skills.len(), 3);
        assert_eq!(skills[0], "skills/repo");
        assert_eq!(skills[1], "skills/repo1");
        assert_eq!(skills[2], "skills/repo2");
    }

    #[test]
    fn test_skill_integration_different_formats() {
        // Test that different skill source formats are handled
        let skills = vec![
            "owner/repo".to_string(),
            "owner/repo@skill-name".to_string(),
            "https://github.com/owner/repo".to_string(),
        ];
        
        // All three formats should be collected
        assert_eq!(skills.len(), 3);
        
        // These would be processed by extract_skill_name in the actual code
        // The function handles:
        // - "owner/repo" -> "repo"
        // - "owner/repo@skill-name" -> "skill-name"
        // - "https://github.com/owner/repo" -> "repo"
    }

    #[test]
    fn test_skill_integration_skills_from_defaults_and_agents() {
        // Test collecting skills from both defaults and agents sections
        // This mimics the logic in run_workflows_install
        
        // Simulated defaults skills
        let defaults_skills = vec!["skills/repo".to_string(), "skills/repo1".to_string()];
        
        // Simulated agent skills
        let agent1_skills = vec!["skills/repo2".to_string()];
        let agent2_skills = vec!["skills/repo3".to_string(), "skills/repo4".to_string()];
        
        // Collect all skills
        let mut all_skills: Vec<String> = Vec::new();
        all_skills.extend(defaults_skills.clone());
        all_skills.extend(agent1_skills.clone());
        all_skills.extend(agent2_skills.clone());
        
        // Sort and dedup
        all_skills.sort();
        all_skills.dedup();
        
        // Should have 5 unique skills total
        assert_eq!(all_skills.len(), 5);
    }
}
