//! Workflows module for managing Switchboard workflows

pub mod github;
pub mod manifest;
pub mod metadata;

use thiserror::Error;

/// The default source repository for workflows
pub const WORKFLOWS_SOURCE: &str = "kkingsbe/switchboard-workflows";

/// Directory where workflows are installed
pub const WORKFLOWS_DIR: &str = ".switchboard/workflows";

/// Error type for workflow operations
#[derive(Debug, Error)]
pub enum WorkflowsError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// GitHub API returned an error
    #[error("GitHub API error: {0}")]
    ApiError(String),

    /// Workflow not found
    #[error("Workflow not found: {0}")]
    NotFound(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded. Retry after: {0} seconds")]
    RateLimited(u32),

    /// File system operation failed
    #[error("File system error: {0}")]
    IoError(#[from] std::io::Error),

    /// Decoding error (base64, JSON, etc.)
    #[error("Failed to decode content: {0}")]
    DecodeError(String),

    /// Invalid workflow format
    #[error("Invalid workflow format: {0}")]
    InvalidFormat(String),

    /// Manifest error
    #[error("Manifest error: {0}")]
    ManifestError(String),

    /// Skill installation error
    #[error("Skill installation error: {0}")]
    SkillError(String),
}

impl From<crate::workflows::manifest::ManifestError> for WorkflowsError {
    fn from(err: crate::workflows::manifest::ManifestError) -> Self {
        WorkflowsError::ManifestError(err.to_string())
    }
}

impl From<crate::skills::SkillsError> for WorkflowsError {
    fn from(err: crate::skills::SkillsError) -> Self {
        WorkflowsError::SkillError(err.to_string())
    }
}

/// Load a workflow's manifest.toml from the installed workflow directory
///
/// This function looks for manifest.toml in the workflow's directory and parses it.
/// Returns None if the manifest.toml doesn't exist or can't be parsed.
///
/// # Arguments
/// * `workflow_name` - The name of the workflow to load the manifest for
///
/// # Returns
/// * `Some(ManifestConfig)` - If manifest.toml exists and is valid
/// * `None` - If manifest.toml doesn't exist or can't be parsed
pub fn load_workflow_manifest(workflow_name: &str) -> Option<crate::workflows::manifest::ManifestConfig> {
    let workflow_path = std::path::Path::new(WORKFLOWS_DIR).join(workflow_name).join("manifest.toml");
    
    if !workflow_path.exists() {
        return None;
    }
    
    match crate::workflows::manifest::ManifestConfig::from_path(&workflow_path) {
        Ok(manifest) => Some(manifest),
        Err(e) => {
            eprintln!("Warning: Failed to parse manifest.toml for workflow '{}': {}", workflow_name, e);
            None
        }
    }
}
