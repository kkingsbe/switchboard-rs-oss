//! Handler for the `workflows installed` subcommand.
//!
//! This module provides the `run_workflows_installed` function which lists
//! all locally installed workflows in the project scope.

use crate::config::Config;
use crate::workflows::metadata::{scan_workflows_directory, WorkflowMetadata};
use comfy_table::{Attribute, Cell, Table};
use std::path::PathBuf;

use super::types::WorkflowsInstalled;
use super::ExitCode;

/// Directory where workflows are installed
const WORKFLOWS_DIR: &str = ".switchboard/workflows";
/// Lockfile filename for workflows
const WORKFLOWS_LOCKFILE: &str = "workflows.lock.json";

/// Run the `switchboard workflows installed` command
///
/// Lists all currently installed workflows in the project scope.
/// Shows workflow name, description, prompts count, and installation timestamp.
///
/// # Arguments
///
/// * `args` - The [`WorkflowsInstalled`] command arguments (unused, kept for consistency)
/// * `_config` - Reference to the application configuration (unused)
///
/// # Returns
///
/// * `ExitCode::Success` - If workflows were listed successfully
/// * `ExitCode::Error` - If there was an error scanning the workflows directory
///
/// # Behavior
///
/// - Scans `.switchboard/workflows/` directory for installed workflows
/// - Loads metadata for each workflow (name, description, prompts)
/// - Reads lockfile to get installation timestamps
/// - Displays results in a formatted table
/// - Shows helpful message when no workflows are installed
pub async fn run_workflows_installed(_args: WorkflowsInstalled, _config: &Config) -> ExitCode {
    // Scan the workflows directory
    let workflows_dir = PathBuf::from(WORKFLOWS_DIR);

    let workflows = match scan_workflows_directory(&workflows_dir) {
        Ok(workflows) => workflows,
        Err(e) => {
            eprintln!("Error scanning workflows directory: {}", e);
            return ExitCode::Error;
        }
    };

    // Read the lockfile to get installation timestamps
    let lockfile = read_workflows_lockfile(&workflows_dir);

    // Format and display the output
    let output = format_workflows_list(workflows, lockfile.as_ref());
    println!("{}", output);

    ExitCode::Success
}

/// Reads the workflows lockfile from the workflows directory
fn read_workflows_lockfile(workflows_dir: &PathBuf) -> Option<WorkflowsLockfile> {
    let lockfile_path = workflows_dir.join(WORKFLOWS_LOCKFILE);

    if !lockfile_path.exists() {
        return None;
    }

    match std::fs::read_to_string(&lockfile_path) {
        Ok(contents) => serde_json::from_str(&contents).ok(),
        Err(_) => None,
    }
}

/// Represents a single workflow entry in the lockfile
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct WorkflowsLockfile {
    /// Lockfile version
    pub version: u32,
    /// Map of workflow name to entry
    pub workflows: std::collections::HashMap<String, WorkflowLockEntry>,
}

/// Formats the list of installed workflows
///
/// This function formats a display of all installed workflows, showing
/// name, description (truncated), prompts count, and installation timestamp.
///
/// # Arguments
///
/// * `workflows` - Vector of workflow metadata
/// * `lockfile` - Optional reference to the lockfile for installation timestamps
///
/// # Returns
///
/// A formatted string ready to be printed to stdout.
pub fn format_workflows_list(
    workflows: Vec<WorkflowMetadata>,
    lockfile: Option<&WorkflowsLockfile>,
) -> String {
    // Check if we have any workflows
    if workflows.is_empty() {
        return format!(
            "No workflows installed\n\n\
             Browse available workflows with: switchboard workflows list\n\
             Install a workflow with: switchboard workflows install <name>\n"
        );
    }

    // Build table with results
    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_header(vec![
            Cell::new("Name").add_attribute(Attribute::Bold),
            Cell::new("Description").add_attribute(Attribute::Bold),
            Cell::new("Prompts").add_attribute(Attribute::Bold),
            Cell::new("Installed At").add_attribute(Attribute::Bold),
        ]);

    let workflow_count = workflows.len();

    for workflow in &workflows {
        // Get first line of description, truncated if too long
        let description = if workflow.description.is_empty() {
            "<no description>".to_string()
        } else {
            let first_line = workflow.description.lines().next().unwrap_or("");
            if first_line.len() > 50 {
                format!("{}...", &first_line[..47])
            } else {
                first_line.to_string()
            }
        };

        // Count prompts
        let prompts_count = workflow.prompts.len().to_string();

        // Get installed_at from lockfile
        let installed_at = lockfile
            .and_then(|lf| lf.workflows.get(&workflow.name))
            .map(|entry| entry.installed_at.clone())
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            Cell::new(&workflow.name),
            Cell::new(&description),
            Cell::new(&prompts_count),
            Cell::new(&installed_at),
        ]);
    }

    // Add summary count
    let mut output = table.to_string();
    output.push_str(&format!("\n{} workflows installed\n", workflow_count));

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_format_workflows_list_empty() {
        let result = format_workflows_list(vec![], None);
        assert!(result.contains("No workflows installed"));
        assert!(result.contains("switchboard workflows list"));
        assert!(result.contains("switchboard workflows install"));
    }

    #[test]
    fn test_format_workflows_list_with_workflows() {
        let workflows = vec![
            WorkflowMetadata {
                name: "bmad".to_string(),
                description: "BMAD workflow description".to_string(),
                source: "kkingsbe/switchboard-workflows".to_string(),
                prompts: vec!["ARCHITECT.md".to_string(), "CODE_REVIEWER.md".to_string()],
                version: Some("1.0.0".to_string()),
            },
            WorkflowMetadata {
                name: "test-workflow".to_string(),
                description: "Test workflow".to_string(),
                source: "kkingsbe/switchboard-workflows".to_string(),
                prompts: vec!["PROMPT.md".to_string()],
                version: None,
            },
        ];

        let result = format_workflows_list(workflows, None);
        assert!(result.contains("bmad"));
        assert!(result.contains("test-workflow"));
        assert!(result.contains("2 workflows installed"));
    }

    #[test]
    fn test_format_workflows_list_with_lockfile() {
        let workflows = vec![WorkflowMetadata {
            name: "bmad".to_string(),
            description: "BMAD workflow".to_string(),
            source: "kkingsbe/switchboard-workflows".to_string(),
            prompts: vec![],
            version: None,
        }];

        let mut lockfile = WorkflowsLockfile::default();
        lockfile.workflows.insert(
            "bmad".to_string(),
            WorkflowLockEntry {
                workflow_name: "bmad".to_string(),
                source: "kkingsbe/switchboard-workflows".to_string(),
                installed_at: "2024-01-15T10:30:00Z".to_string(),
            },
        );

        let result = format_workflows_list(workflows, Some(&lockfile));
        assert!(result.contains("bmad"));
        assert!(result.contains("2024-01-15T10:30:00Z"));
    }

    #[test]
    fn test_read_workflows_lockfile_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let result = read_workflows_lockfile(&PathBuf::from(temp_dir.path()));
        assert!(result.is_none());
    }
}
