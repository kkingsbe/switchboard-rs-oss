//! Workflow metadata handling - parsing, loading, and scanning workflow metadata
//!
//! This module provides functionality for loading workflow metadata from local
//! workflow directories, including reading README.md descriptions and scanning
//! for prompt files.

use crate::workflows::{WorkflowsError, WORKFLOWS_SOURCE};
use std::fs;
use std::path::Path;

/// Metadata for a workflow directory.
///
/// This struct represents the metadata extracted from a workflow directory,
/// including the workflow name, description, source, prompts, and optional version.
///
/// # Directory Structure
///
/// A workflow directory should have the following structure:
///
/// ```text
/// workflow-name/
///   README.md           # Description of the workflow
///   prompts/            # Subdirectory containing agent prompt files
///     ARCHITECT.md
///     CODE_REVIEWER.md
///     ...
/// ```
///
/// # Fields
///
/// * `name` - The workflow name derived from the directory name
/// * `description` - Content from README.md (or empty string if not found)
/// * `source` - Source repository (hardcoded to "kkingsbe/switchboard-workflows")
/// * `prompts` - List of prompt file names from the prompts/ subdirectory
/// * `version` - Optional version string
///
/// # Examples
///
/// ```rust
/// use switchboard::workflows::metadata::WorkflowMetadata;
///
/// let metadata = WorkflowMetadata {
///     name: "bmad".to_string(),
///     description: "BMAD workflow description".to_string(),
///     source: "kkingsbe/switchboard-workflows".to_string(),
///     prompts: vec!["ARCHITECT.md".to_string(), "CODE_REVIEWER.md".to_string()],
///     version: Some("1.0.0".to_string()),
/// };
/// println!("Workflow: {}", metadata.name);
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowMetadata {
    /// Name of the workflow (derived from directory name).
    ///
    /// This is the unique identifier for the workflow, typically matching
    /// the directory name.
    pub name: String,

    /// Description of the workflow from README.md.
    ///
    /// This is the content of the README.md file in the workflow directory.
    /// If README.md doesn't exist, this will be an empty string.
    pub description: String,

    /// Source repository for the workflow.
    ///
    /// This is hardcoded to "kkingsbe/switchboard-workflows" indicating
    /// where the workflow originates from.
    pub source: String,

    /// List of prompt files in the prompts/ subdirectory.
    ///
    /// Each entry is the filename (not full path) of a prompt file.
    /// Only .md files are included.
    pub prompts: Vec<String>,

    /// Optional version string for the workflow.
    ///
    /// This can be used to track the version of the workflow.
    #[serde(default)]
    pub version: Option<String>,
}

/// Loads workflow metadata from a workflow directory.
///
/// This function parses a workflow directory to extract metadata:
/// - Reads README.md for the description (falls back to empty string)
/// - Scans prompts/ subdirectory for .md files
/// - Uses directory name as the workflow name
///
/// # Arguments
///
/// * `path` - Path to the workflow directory
///
/// # Returns
///
/// * `Ok(WorkflowMetadata)` - The extracted workflow metadata
/// * `Err(WorkflowsError::IoError)` - If the directory cannot be read
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::workflows::metadata::load_workflow_metadata;
/// use std::path::Path;
///
/// let metadata = load_workflow_metadata(Path::new(".switchboard/workflows/bmad"))?;
/// println!("Workflow: {}", metadata.name);
/// println!("Description: {}", metadata.description);
/// println!("Prompts: {:?}", metadata.prompts);
/// # Ok::<(), switchboard::workflows::WorkflowsError>(())
/// ```
pub fn load_workflow_metadata(path: &Path) -> Result<WorkflowMetadata, WorkflowsError> {
    // Get workflow name from directory name
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Read README.md for description (fallback to empty string if not found)
    let readme_path = path.join("README.md");
    let description = match fs::read_to_string(&readme_path) {
        Ok(content) => content,
        Err(_) => String::new(), // Fallback to empty string if README.md doesn't exist
    };

    // Scan prompts/ subdirectory for .md files
    let prompts_dir = path.join("prompts");
    let prompts = scan_prompts_directory(&prompts_dir);

    // Create metadata with hardcoded source
    Ok(WorkflowMetadata {
        name,
        description,
        source: WORKFLOWS_SOURCE.to_string(),
        prompts,
        version: None,
    })
}

/// Scans the prompts/ subdirectory for markdown files.
///
/// This helper function lists all .md files in the prompts directory.
/// If the directory doesn't exist or cannot be read, it returns an empty vector.
///
/// # Arguments
///
/// * `prompts_dir` - Path to the prompts directory
///
/// # Returns
///
/// A vector of prompt filenames (not full paths).
fn scan_prompts_directory(prompts_dir: &Path) -> Vec<String> {
    // Check if prompts directory exists and is a directory
    if !prompts_dir.is_dir() {
        return Vec::new();
    }

    // Read directory and filter for .md files
    match fs::read_dir(prompts_dir) {
        Ok(entries) => entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                
                // Only include files (not directories) with .md extension
                if path.is_file() {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .filter(|name| name.ends_with(".md"))
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => Vec::new(), // Fallback to empty vector on error
    }
}

/// Scans a directory for all installed workflows.
///
/// This function searches the specified directory for workflow subdirectories
/// and loads metadata for each one found. Results are sorted by workflow name.
///
/// # Arguments
///
/// * `dir` - Path to the workflows directory to scan
///
/// # Returns
///
/// * `Ok(Vec<WorkflowMetadata>)` - List of workflow metadata, sorted by name
/// * `Err(WorkflowsError::IoError)` - If the directory cannot be read
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::workflows::metadata::scan_workflows_directory;
/// use std::path::Path;
///
/// let workflows = scan_workflows_directory(Path::new(".switchboard/workflows"))?;
/// for workflow in &workflows {
///     println!("- {}: {}", workflow.name, workflow.description.lines().next().unwrap_or(""));
/// }
/// # Ok::<(), switchboard::workflows::WorkflowsError>(())
/// ```
pub fn scan_workflows_directory(dir: &Path) -> Result<Vec<WorkflowMetadata>, WorkflowsError> {
    // Check if directory exists
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    // Read directory and collect workflow directories
    let mut workflows = Vec::new();
    
    let entries = fs::read_dir(dir).map_err(|e| WorkflowsError::IoError(e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        
        // Only process directories
        if path.is_dir() {
            // Try to load metadata for each workflow directory
            match load_workflow_metadata(&path) {
                Ok(metadata) => workflows.push(metadata),
                Err(_) => {
                    // Skip directories that can't be parsed as workflows
                    // (e.g., hidden directories, non-workflow folders)
                }
            }
        }
    }

    // Sort by workflow name
    workflows.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(workflows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_workflow_metadata_fields() {
        let metadata = WorkflowMetadata {
            name: "test-workflow".to_string(),
            description: "A test workflow".to_string(),
            source: "kkingsbe/switchboard-workflows".to_string(),
            prompts: vec!["ARCHITECT.md".to_string()],
            version: Some("1.0.0".to_string()),
        };

        assert_eq!(metadata.name, "test-workflow");
        assert_eq!(metadata.description, "A test workflow");
        assert_eq!(metadata.source, "kkingsbe/switchboard-workflows");
        assert_eq!(metadata.prompts.len(), 1);
        assert_eq!(metadata.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_workflow_metadata_serialize() {
        let metadata = WorkflowMetadata {
            name: "bmad".to_string(),
            description: "BMAD workflow".to_string(),
            source: "kkingsbe/switchboard-workflows".to_string(),
            prompts: vec!["ARCHITECT.md".to_string()],
            version: None,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("bmad"));
        assert!(json.contains("BMAD workflow"));
    }

    #[test]
    fn test_scan_prompts_directory_nonexistent() {
        let prompts_dir = Path::new("/nonexistent/path/prompts");
        let prompts = scan_prompts_directory(prompts_dir);
        assert!(prompts.is_empty());
    }

    #[test]
    fn test_scan_prompts_directory_with_files() {
        // Create a temporary directory with prompt files
        let temp_dir = TempDir::new().unwrap();
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir(&prompts_dir).unwrap();
        
        // Create some prompt files
        fs::write(prompts_dir.join("ARCHITECT.md"), "# Architect").unwrap();
        fs::write(prompts_dir.join("CODE_REVIEWER.md"), "# Code Reviewer").unwrap();
        fs::write(prompts_dir.join("README.md"), "# Readme").unwrap(); // Included as .md

        let prompts = scan_prompts_directory(&prompts_dir);
        
        assert_eq!(prompts.len(), 3);
        assert!(prompts.contains(&"ARCHITECT.md".to_string()));
        assert!(prompts.contains(&"CODE_REVIEWER.md".to_string()));
        assert!(prompts.contains(&"README.md".to_string()));
    }

    #[test]
    fn test_load_workflow_metadata_basic() {
        // Create a temporary workflow directory
        let temp_dir = TempDir::new().unwrap();
        let workflow_dir = temp_dir.path().join("test-workflow");
        fs::create_dir(&workflow_dir).unwrap();
        
        // Create README.md
        fs::write(workflow_dir.join("README.md"), "# Test Workflow\n\nThis is a test.").unwrap();

        // Create prompts directory with files
        let prompts_dir = workflow_dir.join("prompts");
        fs::create_dir(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("ARCHITECT.md"), "# Architect").unwrap();

        let metadata = load_workflow_metadata(&workflow_dir).unwrap();

        assert_eq!(metadata.name, "test-workflow");
        assert!(metadata.description.contains("Test Workflow"));
        assert_eq!(metadata.source, "kkingsbe/switchboard-workflows");
        assert!(metadata.prompts.contains(&"ARCHITECT.md".to_string()));
    }

    #[test]
    fn test_load_workflow_metadata_no_readme() {
        // Create a temporary workflow directory without README.md
        let temp_dir = TempDir::new().unwrap();
        let workflow_dir = temp_dir.path().join("no-readme-workflow");
        fs::create_dir(&workflow_dir).unwrap();

        let metadata = load_workflow_metadata(&workflow_dir).unwrap();

        assert_eq!(metadata.name, "no-readme-workflow");
        assert_eq!(metadata.description, ""); // Empty string fallback
    }

    #[test]
    fn test_load_workflow_metadata_no_prompts() {
        // Create a temporary workflow directory without prompts directory
        let temp_dir = TempDir::new().unwrap();
        let workflow_dir = temp_dir.path().join("no-prompts-workflow");
        fs::create_dir(&workflow_dir).unwrap();
        fs::write(workflow_dir.join("README.md"), "# No Prompts").unwrap();

        let metadata = load_workflow_metadata(&workflow_dir).unwrap();

        assert_eq!(metadata.prompts, Vec::<String>::new());
    }

    #[test]
    fn test_scan_workflows_directory_empty() {
        let temp_dir = TempDir::new().unwrap();
        
        let workflows = scan_workflows_directory(temp_dir.path()).unwrap();
        assert!(workflows.is_empty());
    }

    #[test]
    fn test_scan_workflows_directory_with_workflows() {
        // Create a temporary workflows directory with subdirectories
        let temp_dir = TempDir::new().unwrap();
        let workflows_dir = temp_dir.path();
        
        // Create first workflow
        let workflow1_dir = workflows_dir.join("zzz-workflow");
        fs::create_dir(&workflow1_dir).unwrap();
        fs::write(workflow1_dir.join("README.md"), "# ZZZ Workflow").unwrap();

        // Create second workflow
        let workflow2_dir = workflows_dir.join("aaa-workflow");
        fs::create_dir(&workflow2_dir).unwrap();
        fs::write(workflow2_dir.join("README.md"), "# AAA Workflow").unwrap();

        let workflows = scan_workflows_directory(workflows_dir).unwrap();

        assert_eq!(workflows.len(), 2);
        // Should be sorted alphabetically
        assert_eq!(workflows[0].name, "aaa-workflow");
        assert_eq!(workflows[1].name, "zzz-workflow");
    }

    #[test]
    fn test_workflow_metadata_clone() {
        let metadata = WorkflowMetadata {
            name: "clone-test".to_string(),
            description: "Testing clone".to_string(),
            source: "kkingsbe/switchboard-workflows".to_string(),
            prompts: vec!["TEST.md".to_string()],
            version: Some("0.1.0".to_string()),
        };

        let cloned = metadata.clone();
        assert_eq!(metadata, cloned);
    }
}
