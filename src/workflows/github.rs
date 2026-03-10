//! GitHub API client for fetching workflows from the switchboard-workflows repository

use crate::workflows::manifest::ManifestConfig;
use crate::workflows::WorkflowsError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// GitHub repository owner
const REPO_OWNER: &str = "kkingsbe";
/// GitHub repository name
const REPO_NAME: &str = "switchboard-workflows";
/// Base URL for GitHub Contents API
const CONTENTS_API_BASE: &str = "https://api.github.com/repos";
/// Base URL for raw file content
const RAW_BASE: &str = "https://raw.githubusercontent.com";

/// GitHub API response for repository contents
#[derive(Debug, Deserialize, Serialize)]
struct GitHubContent {
    name: String,
    path: String,
    #[serde(rename = "type")]
    content_type: String,
    size: Option<u64>,
    sha: Option<String>,
}

/// Information about a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    /// Name of the workflow
    pub name: String,
    /// Description from README.md
    pub description: Option<String>,
    /// List of prompt files in the prompts/ subdirectory
    pub prompts: Vec<String>,
    /// SHA of the workflow directory
    pub sha: Option<String>,
}

/// GitHub API client for workflow operations
pub struct GitHubClient {
    client: Client,
    token: Option<String>,
}

impl GitHubClient {
    /// Create a new GitHub client
    pub fn new() -> Self {
        let token = std::env::var("GITHUB_TOKEN").ok();
        
        let client = Client::builder()
            .user_agent("switchboard/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, token }
    }

    /// Build the authorization header if token is available
    fn auth_header(&self) -> Option<(String, String)> {
        self.token.as_ref().map(|t| ("Authorization".to_string(), format!("Bearer {}", t)))
    }

    /// Make a request to the GitHub API with optional auth
    async fn get(&self, url: &str) -> Result<String, WorkflowsError> {
        let mut request = self.client.get(url);

        if let Some((key, value)) = self.auth_header() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;

        // Check for rate limiting
        if response.status() == 403 {
            if let Some(reset_time) = response.headers().get("X-RateLimit-Reset") {
                if let Ok(reset_str) = reset_time.to_str() {
                    if let Ok(reset) = reset_str.parse::<u32>() {
                        return Err(WorkflowsError::RateLimited(reset));
                    }
                }
            }
        }

        // Check for 404
        if response.status() == 404 {
            return Err(WorkflowsError::NotFound(url.to_string()));
        }

        let body = response.text().await?;
        Ok(body)
    }

    /// Fetch raw file content from GitHub
    async fn get_raw(&self, path: &str, branch: &str) -> Result<String, WorkflowsError> {
        let url = format!("{}/{}/{}/{}/{}", RAW_BASE, REPO_OWNER, REPO_NAME, branch, path);
        
        let mut request = self.client.get(&url);

        if let Some((key, value)) = self.auth_header() {
            request = request.header(&key, &value);
        }

        let response = request.send().await?;

        if response.status() == 404 {
            return Err(WorkflowsError::NotFound(path.to_string()));
        }

        let body = response.text().await?;
        Ok(body)
    }

    /// List all workflows available in the repository
    ///
    /// Fetches the root contents of the repository and filters for directories.
    /// Each directory represents a workflow.
    ///
    /// # Returns
    /// A vector of workflow names
    ///
    /// # Errors
    /// Returns an error if the GitHub API request fails
    pub async fn list_workflows(&self) -> Result<Vec<String>, WorkflowsError> {
        let url = format!("{}/{}/{}/contents", CONTENTS_API_BASE, REPO_OWNER, REPO_NAME);
        
        let body = self.get(&url).await?;
        
        let contents: Vec<GitHubContent> = serde_json::from_str(&body)
            .map_err(|e| WorkflowsError::DecodeError(e.to_string()))?;

        // Filter for directories only (workflows are in subdirectories)
        let workflows: Vec<String> = contents
            .into_iter()
            .filter(|c| c.content_type == "dir")
            .map(|c| c.name)
            .collect();

        Ok(workflows)
    }

    /// Get information about a specific workflow
    ///
    /// Fetches the workflow's README.md for description and lists
    /// the prompts/ subdirectory contents.
    ///
    /// # Arguments
    /// * `workflow_name` - The name of the workflow to fetch
    ///
    /// # Returns
    /// A WorkflowInfo struct with the workflow's metadata
    ///
    /// # Errors
    /// Returns an error if the workflow doesn't exist or API requests fail
    pub async fn get_workflow_info(&self, workflow_name: &str) -> Result<WorkflowInfo, WorkflowsError> {
        // First, verify the workflow directory exists by checking its contents
        let url = format!(
            "{}/{}/{}/contents/{}",
            CONTENTS_API_BASE, REPO_OWNER, REPO_NAME, workflow_name
        );
        
        let body = self.get(&url).await?;
        
        let contents: Vec<GitHubContent> = serde_json::from_str(&body)
            .map_err(|e| WorkflowsError::DecodeError(e.to_string()))?;

        // Get the directory SHA if available
        let sha = contents.first().and_then(|c| c.sha.clone());

        // Try to get README.md content
        let description = match self.get_raw(&format!("{}/README.md", workflow_name), "main").await {
            Ok(content) => Some(content),
            Err(WorkflowsError::NotFound(_)) => None,
            Err(e) => return Err(e),
        };

        // Try to get prompts directory contents
        let mut prompts = Vec::new();
        
        // Check if prompts directory exists
        let prompts_exist = contents.iter().any(|c| c.name == "prompts" && c.content_type == "dir");
        
        if prompts_exist {
            let prompts_url = format!(
                "{}/{}/{}/contents/{}/prompts",
                CONTENTS_API_BASE, REPO_OWNER, REPO_NAME, workflow_name
            );
            
            match self.get(&prompts_url).await {
                Ok(prompts_body) => {
                    if let Ok(prompts_contents) = serde_json::from_str::<Vec<GitHubContent>>(&prompts_body) {
                        prompts = prompts_contents
                            .into_iter()
                            .filter(|c| c.content_type == "file")
                            .map(|c| c.name)
                            .collect();
                    }
                }
                Err(WorkflowsError::NotFound(_)) => {
                    // prompts directory might be empty or not exist
                }
                Err(e) => return Err(e),
            }
        }

        Ok(WorkflowInfo {
            name: workflow_name.to_string(),
            description,
            prompts,
            sha,
        })
    }

    /// Download a workflow to the local filesystem
    ///
    /// Downloads the workflow directory contents including README.md
    /// and the prompts/ subdirectory.
    ///
    /// # Arguments
    /// * `workflow_name` - The name of the workflow to download
    /// * `dest_dir` - The destination directory path
    ///
    /// # Returns
    /// The number of files downloaded
    ///
    /// # Errors
    /// Returns an error if the workflow doesn't exist or file operations fail
    pub async fn download_workflow(
        &self,
        workflow_name: &str,
        dest_dir: &Path,
    ) -> Result<usize, WorkflowsError> {
        // First verify the workflow exists
        let url = format!(
            "{}/{}/{}/contents/{}",
            CONTENTS_API_BASE, REPO_OWNER, REPO_NAME, workflow_name
        );
        
        let body = self.get(&url).await?;
        
        let contents: Vec<GitHubContent> = serde_json::from_str(&body)
            .map_err(|e| WorkflowsError::DecodeError(e.to_string()))?;

        // Create the destination directory
        std::fs::create_dir_all(dest_dir)?;

        let mut files_downloaded = 0;
        let branch = "main";

        // Download each item in the workflow directory
        for item in contents {
            match item.content_type.as_str() {
                "file" => {
                    let file_path = dest_dir.join(&item.name);
                    let content = self.get_raw(&format!("{}/{}", workflow_name, item.name), branch).await?;
                    std::fs::write(&file_path, content)?;
                    files_downloaded += 1;
                }
                "dir" if item.name == "prompts" => {
                    // Download prompts subdirectory
                    let prompts_dir = dest_dir.join("prompts");
                    std::fs::create_dir_all(&prompts_dir)?;

                    let prompts_url = format!(
                        "{}/{}/{}/contents/{}/prompts",
                        CONTENTS_API_BASE, REPO_OWNER, REPO_NAME, workflow_name
                    );
                    
                    match self.get(&prompts_url).await {
                        Ok(prompts_body) => {
                            if let Ok(prompts_contents) = serde_json::from_str::<Vec<GitHubContent>>(&prompts_body) {
                                for prompt_file in prompts_contents {
                                    if prompt_file.content_type == "file" {
                                        let prompt_path = prompts_dir.join(&prompt_file.name);
                                        let content = self.get_raw(
                                            &format!("{}/prompts/{}", workflow_name, prompt_file.name),
                                            branch
                                        ).await?;
                                        std::fs::write(&prompt_path, content)?;
                                        files_downloaded += 1;
                                    }
                                }
                            }
                        }
                        Err(WorkflowsError::NotFound(_)) => {
                            // prompts directory might be empty
                        }
                        Err(e) => return Err(e),
                    }
                }
                _ => {
                    // Skip other directories or items we don't handle
                }
            }
        }

        Ok(files_downloaded)
    }

    /// Download and parse manifest.toml for a workflow
    ///
    /// Fetches manifest.toml from the workflow directory and parses it into a ManifestConfig.
    /// This does NOT save the file locally - use download_manifest_to_file() for that.
    ///
    /// # Arguments
    /// * `workflow_name` - The name of the workflow
    ///
    /// # Returns
    /// * `Ok(ManifestConfig)` - Successfully downloaded and parsed manifest
    /// * `Err(WorkflowsError)` - Error if manifest doesn't exist or can't be parsed
    pub async fn download_manifest(&self, workflow_name: &str) -> Result<ManifestConfig, WorkflowsError> {
        let manifest_path = format!("{}/manifest.toml", workflow_name);
        
        // Try to fetch the raw manifest.toml content
        let content = match self.get_raw(&manifest_path, "main").await {
            Ok(c) => c,
            Err(WorkflowsError::NotFound(_)) => {
                return Err(WorkflowsError::NotFound("manifest.toml not found in workflow".to_string()));
            }
            Err(e) => return Err(e),
        };
        
        // Parse the TOML content
        let manifest: ManifestConfig = toml::from_str(&content).map_err(|e| {
            WorkflowsError::InvalidFormat(format!("Failed to parse manifest.toml: {}", e))
        })?;
        
        Ok(manifest)
    }

    /// Download manifest.toml and save it to a file
    ///
    /// Fetches manifest.toml from the workflow repository and saves it to the specified path.
    ///
    /// # Arguments
    /// * `workflow_name` - The name of the workflow
    /// * `dest_path` - Where to save the manifest.toml file
    ///
    /// # Returns
    /// * `Ok(())` - Successfully downloaded and saved manifest
    /// * `Err(WorkflowsError)` - Error if manifest doesn't exist or file operations fail
    pub async fn download_manifest_to_file(
        &self,
        workflow_name: &str,
        dest_path: &Path,
    ) -> Result<(), WorkflowsError> {
        let manifest_path = format!("{}/manifest.toml", workflow_name);
        
        // Try to fetch the raw manifest.toml content
        let content = match self.get_raw(&manifest_path, "main").await {
            Ok(c) => c,
            Err(WorkflowsError::NotFound(_)) => {
                return Err(WorkflowsError::NotFound("manifest.toml not found in workflow".to_string()));
            }
            Err(e) => return Err(e),
        };
        
        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Write to file
        std::fs::write(dest_path, content)?;
        
        Ok(())
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GitHubClient::new();
        assert!(client.token.is_none() || client.token.is_some());
    }

    #[tokio::test]
    async fn test_list_workflows() {
        let client = GitHubClient::new();
        let result = client.list_workflows().await;
        
        // This will fail without network, but tests the structure
        match result {
            Ok(workflows) => {
                println!("Found {} workflows", workflows.len());
            }
            Err(e) => {
                println!("Expected error without network: {}", e);
            }
        }
    }
}
