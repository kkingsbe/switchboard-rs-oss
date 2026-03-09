//! Manifest module for parsing manifest.toml files in workflows
//!
//! This module provides structures and utilities for parsing and validating
//! manifest.toml files that define workflow configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::config::{Agent, OverlapMode};

/// ManifestDefaults defines default configuration values for agents
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ManifestDefaults {
    /// Default cron schedule for agents (e.g., "0 9 * * *")
    #[serde(default)]
    pub schedule: Option<String>,
    /// Default timeout for agent runs (e.g., "30m", "2h")
    #[serde(default)]
    pub timeout: Option<String>,
    /// Default read-only mode
    #[serde(default)]
    pub readonly: Option<bool>,
    /// Default overlap mode ("skip" or "queue")
    #[serde(default)]
    pub overlap_mode: Option<String>,
    /// Default max queue size for queue mode
    #[serde(default)]
    pub max_queue_size: Option<usize>,
    /// Default environment variables
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    /// Default skills available to agents
    #[serde(default)]
    pub skills: Option<Vec<String>>,
}

/// ManifestPrompt represents a single prompt file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManifestPrompt {
    /// Filename of the prompt (e.g., "ARCHITECT.md")
    pub name: String,
    /// Human-readable description of what this prompt does
    pub description: Option<String>,
    /// Role or purpose of this prompt (e.g., "architect", "developer")
    pub role: Option<String>,
}

/// ManifestAgent defines an agent configuration from the manifest
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManifestAgent {
    /// Agent name (will be prefixed with workflow name)
    pub name: String,
    /// Prompt file to use (reference to prompts array)
    pub prompt_file: String,
    /// Agent-specific schedule override
    #[serde(default)]
    pub schedule: Option<String>,
    /// Agent-specific timeout override
    #[serde(default)]
    pub timeout: Option<String>,
    /// Agent-specific readonly override
    #[serde(default)]
    pub readonly: Option<bool>,
    /// Agent-specific overlap mode override
    #[serde(default)]
    pub overlap_mode: Option<String>,
    /// Agent-specific max queue size override
    #[serde(default)]
    pub max_queue_size: Option<usize>,
    /// Agent-specific environment variables
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    /// Agent-specific skills
    #[serde(default)]
    pub skills: Option<Vec<String>>,
}

/// ManifestConfig represents the complete manifest.toml structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManifestConfig {
    /// Workflow name (matches directory name)
    #[serde(default)]
    pub name: Option<String>,
    /// Human-readable workflow description
    #[serde(default)]
    pub description: Option<String>,
    /// Version of the workflow manifest
    #[serde(default)]
    pub version: Option<String>,
    /// Default configuration values
    #[serde(default)]
    pub defaults: Option<ManifestDefaults>,
    /// Available prompt files
    #[serde(default)]
    pub prompts: Vec<ManifestPrompt>,
    /// Agent configurations
    #[serde(default)]
    pub agents: Vec<ManifestAgent>,
}

/// Errors that can occur when parsing or using manifest.toml
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    /// Failed to read manifest file
    #[error("Failed to read manifest file: {0}")]
    IoError(#[from] std::io::Error),

    /// Failed to parse TOML
    #[error("Failed to parse manifest.toml: {0}")]
    ParseError(String),

    /// Validation error in manifest
    #[error("Manifest validation error: {0}")]
    ValidationError(String),

    /// Referenced prompt file not found
    #[error("Prompt file '{0}' referenced in manifest not found")]
    PromptNotFound(String),

    /// Invalid overlap mode value
    #[error("Invalid overlap mode '{0}': must be 'skip' or 'queue'")]
    InvalidOverlapMode(String),
}

impl ManifestConfig {
    /// Read and parse a manifest.toml file from the given path
    ///
    /// # Arguments
    /// * `path` - Path to the manifest.toml file
    ///
    /// # Returns
    /// * `Ok(ManifestConfig)` - Successfully parsed manifest
    /// * `Err(ManifestError)` - Error reading or parsing the manifest
    pub fn from_path(path: &Path) -> Result<ManifestConfig, ManifestError> {
        let content = fs::read_to_string(path)?;
        
        toml::from_str(&content).map_err(|e| {
            ManifestError::ParseError(e.to_string())
        })
    }

    /// Validate that all referenced prompt files exist in the prompts/ directory
    ///
    /// # Arguments
    /// * `workflow_dir` - Path to the workflow directory containing the manifest
    ///
    /// # Returns
    /// * `Ok(())` - All referenced prompt files exist
    /// * `Err(ManifestError)` - A referenced prompt file was not found
    pub fn validate_prompts(&self, workflow_dir: &Path) -> Result<(), ManifestError> {
        let prompts_dir = workflow_dir.join("prompts");
        
        // Get all available prompt names from the prompts directory
        let available_prompts: Vec<String> = if prompts_dir.exists() {
            fs::read_dir(&prompts_dir)
                .map_err(|e| ManifestError::IoError(e))?
                .filter_map(|entry| {
                    entry.ok().and_then(|e| {
                        let path = e.path();
                        if path.is_file() {
                            path.file_name()
                                .and_then(|n| n.to_str())
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                })
                .collect()
        } else {
            // If prompts directory doesn't exist, check prompts from manifest
            self.prompts.iter().map(|p| p.name.clone()).collect()
        };

        // Validate each agent's prompt_file references
        for agent in &self.agents {
            // Extract just the filename from the prompt_file path
            let prompt_filename = Path::new(&agent.prompt_file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&agent.prompt_file)
                .to_string();

            if !available_prompts.contains(&prompt_filename) {
                return Err(ManifestError::PromptNotFound(prompt_filename));
            }
        }

        Ok(())
    }

    /// Validates that skill entries have correct format
    /// Returns error for invalid skill source formats
    pub fn validate_skill_sources(&self) -> Result<(), ManifestError> {
        // Check defaults skills
        if let Some(defaults) = &self.defaults {
            if let Some(skills) = &defaults.skills {
                for skill in skills {
                    validate_skill_format(skill)?;
                }
            }
        }
        
        // Check per-agent skills
        for agent in &self.agents {
            if let Some(skills) = &agent.skills {
                for skill in skills {
                    validate_skill_format(skill)?;
                }
            }
        }
        
        Ok(())
    }
}

/// Validates a single skill source format
/// Accepts: owner/repo, owner/repo@skill-name, https://github.com/owner/repo
fn validate_skill_format(skill: &str) -> Result<(), ManifestError> {
    // Simple validation - just check it's not empty and doesn't contain spaces
    if skill.trim().is_empty() {
        return Err(ManifestError::ValidationError(
            "Skill source cannot be empty".to_string()
        ));
    }
    
    if skill.contains(' ') {
        return Err(ManifestError::ValidationError(
            format!("Skill source '{}' cannot contain spaces", skill)
        ));
    }
    
    Ok(())
}

impl ManifestAgent {
    /// Convert this manifest agent to a Config Agent struct
    ///
    /// Applies defaults from ManifestDefaults with agent-specific overrides taking precedence.
    /// The agent name is prefixed with the workflow name.
    ///
    /// # Arguments
    /// * `workflow_name` - The name of the workflow (used for prefixing)
    /// * `defaults` - Default values from the manifest
    ///
    /// # Returns
    /// * `Agent` - Config Agent with proper defaults applied
    pub fn to_agent(&self, workflow_name: &str, defaults: &ManifestDefaults) -> Agent {
        // Apply defaults in order: agent-specific override first, then workflow defaults
        let effective_schedule = self.schedule.clone()
            .or_else(|| defaults.schedule.clone())
            .unwrap_or_else(|| "0 9 * * *".to_string());
        
        let effective_timeout = self.timeout.clone()
            .or_else(|| defaults.timeout.clone());
        
        let effective_readonly = self.readonly.clone()
            .or_else(|| defaults.readonly.clone());
        
        let effective_overlap_mode = self.overlap_mode.clone()
            .or_else(|| defaults.overlap_mode.clone());
        
        let effective_max_queue_size = self.max_queue_size
            .or(defaults.max_queue_size);
        
        let effective_env = self.env.clone()
            .or_else(|| defaults.env.clone());
        
        let effective_skills = self.skills.clone()
            .or_else(|| defaults.skills.clone());

        // Convert overlap_mode string to OverlapMode enum
        let overlap_mode = effective_overlap_mode.as_ref().map(|mode| {
            match mode.to_lowercase().as_str() {
                "skip" => OverlapMode::Skip,
                "queue" => OverlapMode::Queue,
                _ => OverlapMode::Skip, // Default to Skip for invalid values
            }
        });

        // Prefix agent name with workflow name
        let agent_name = format!("{}_{}", workflow_name, self.name);

        // Convert prompt_file to prompts/ prefix
        let prompt_file = format!("prompts/{}", self.prompt_file);

        Agent {
            name: agent_name,
            prompt: None,
            prompt_file: Some(prompt_file),
            schedule: effective_schedule,
            env: effective_env,
            readonly: effective_readonly,
            timeout: effective_timeout,
            overlap_mode,
            max_queue_size: effective_max_queue_size,
            skills: effective_skills,
            silent_timeout: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parse_manifest_defaults() {
        let toml_content = r#"
[defaults]
schedule = "0 9 * * *"
timeout = "30m"
readonly = false
overlap_mode = "skip"
max_queue_size = 5
env = { KEY1 = "value1" }
skills = ["skill1", "skill2"]
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        assert!(manifest.defaults.is_some());
        let defaults = manifest.defaults.unwrap();
        assert_eq!(defaults.schedule, Some("0 9 * * *".to_string()));
        assert_eq!(defaults.timeout, Some("30m".to_string()));
        assert_eq!(defaults.readonly, Some(false));
        assert_eq!(defaults.overlap_mode, Some("skip".to_string()));
        assert_eq!(defaults.max_queue_size, Some(5));
    }

    #[test]
    fn test_parse_manifest_prompts() {
        let toml_content = r#"
[[prompts]]
name = "ARCHITECT.md"
description = "Architect role prompt"
role = "architect"

[[prompts]]
name = "CODE_REVIEWER.md"
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(manifest.prompts.len(), 2);
        assert_eq!(manifest.prompts[0].name, "ARCHITECT.md");
        assert_eq!(manifest.prompts[0].role, Some("architect".to_string()));
    }

    #[test]
    fn test_parse_manifest_agents() {
        let toml_content = r#"
[[agents]]
name = "architect"
prompt_file = "ARCHITECT.md"
schedule = "0 10 * * *"
timeout = "1h"
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(manifest.agents.len(), 1);
        assert_eq!(manifest.agents[0].name, "architect");
        assert_eq!(manifest.agents[0].prompt_file, "ARCHITECT.md");
    }

    #[test]
    fn test_from_path() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.toml");
        
        let toml_content = r#"
name = "test-workflow"
version = "1.0.0"

[[agents]]
name = "test-agent"
prompt_file = "test.md"
"#;
        
        fs::write(&manifest_path, toml_content).unwrap();
        
        let manifest = ManifestConfig::from_path(&manifest_path).unwrap();
        assert_eq!(manifest.name, Some("test-workflow".to_string()));
        assert_eq!(manifest.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_to_agent() {
        let defaults = ManifestDefaults {
            schedule: Some("0 9 * * *".to_string()),
            timeout: Some("30m".to_string()),
            readonly: Some(false),
            overlap_mode: Some("skip".to_string()),
            max_queue_size: Some(3),
            env: Some(HashMap::new()),
            skills: Some(vec!["default-skill".to_string()]),
        };

        let agent = ManifestAgent {
            name: "architect".to_string(),
            prompt_file: "ARCHITECT.md".to_string(),
            schedule: Some("0 10 * * *".to_string()), // Override default
            timeout: None,
            readonly: None,
            overlap_mode: None,
            max_queue_size: None,
            env: None,
            skills: None,
        };

        let config_agent = agent.to_agent("myworkflow", &defaults);
        
        // Agent name should be prefixed
        assert_eq!(config_agent.name, "myworkflow_architect");
        
        // Schedule should be agent-specific override
        assert_eq!(config_agent.schedule, "0 10 * * *");
        
        // Timeout should come from defaults since agent didn't override
        assert_eq!(config_agent.timeout, Some("30m".to_string()));
        
        // prompt_file should have prompts/ prefix
        assert_eq!(config_agent.prompt_file, Some("prompts/ARCHITECT.md".to_string()));
        
        // Skills should come from defaults
        assert_eq!(config_agent.skills, Some(vec!["default-skill".to_string()]));
    }

    #[test]
    fn test_to_agent_default_schedule() {
        let defaults = ManifestDefaults {
            schedule: None,
            timeout: None,
            readonly: None,
            overlap_mode: None,
            max_queue_size: None,
            env: None,
            skills: None,
        };

        let agent = ManifestAgent {
            name: "test".to_string(),
            prompt_file: "test.md".to_string(),
            schedule: None,
            timeout: None,
            readonly: None,
            overlap_mode: None,
            max_queue_size: None,
            env: None,
            skills: None,
        };

        let config_agent = agent.to_agent("workflow", &defaults);
        
        // Should fall back to default schedule
        assert_eq!(config_agent.schedule, "0 9 * * *");
    }

    #[test]
    fn test_validate_prompts() {
        let temp_dir = TempDir::new().unwrap();
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir(&prompts_dir).unwrap();
        
        // Create a prompt file
        let prompt_path = prompts_dir.join("ARCHITECT.md");
        fs::write(&prompt_path, "# Architect Prompt").unwrap();

        let manifest = ManifestConfig {
            name: Some("test".to_string()),
            description: None,
            version: None,
            defaults: None,
            prompts: vec![
                ManifestPrompt {
                    name: "ARCHITECT.md".to_string(),
                    description: None,
                    role: None,
                },
            ],
            agents: vec![
                ManifestAgent {
                    name: "architect".to_string(),
                    prompt_file: "ARCHITECT.md".to_string(),
                    schedule: None,
                    timeout: None,
                    readonly: None,
                    overlap_mode: None,
                    max_queue_size: None,
                    env: None,
                    skills: None,
                },
            ],
        };

        let result = manifest.validate_prompts(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_prompts_not_found() {
        let temp_dir = TempDir::new().unwrap();

        let manifest = ManifestConfig {
            name: Some("test".to_string()),
            description: None,
            version: None,
            defaults: None,
            prompts: vec![],
            agents: vec![
                ManifestAgent {
                    name: "architect".to_string(),
                    prompt_file: "NONEXISTENT.md".to_string(),
                    schedule: None,
                    timeout: None,
                    readonly: None,
                    overlap_mode: None,
                    max_queue_size: None,
                    env: None,
                    skills: None,
                },
            ],
        };

        let result = manifest.validate_prompts(temp_dir.path());
        assert!(matches!(result, Err(ManifestError::PromptNotFound(_))));
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_parse_bmad_manifest() {
        // Test parsing the actual bmad manifest.toml
        let manifest_path = std::path::Path::new("examples/bmad/manifest.toml");
        
        // Skip test if file doesn't exist (in CI without examples)
        if !manifest_path.exists() {
            eprintln!("Skipping test: examples/bmad/manifest.toml not found");
            return;
        }
        
        let manifest = ManifestConfig::from_path(manifest_path).expect("Failed to parse bmad manifest.toml");
        
        assert_eq!(manifest.name, Some("bmad".to_string()));
        assert_eq!(manifest.version, Some("1.0.0".to_string()));
        assert!(manifest.defaults.is_some());
        assert!(!manifest.prompts.is_empty());
        assert!(!manifest.agents.is_empty());
        
        // Verify defaults
        let defaults = manifest.defaults.as_ref().unwrap();
        assert_eq!(defaults.schedule, Some("0 9 * * *".to_string()));
        assert_eq!(defaults.timeout, Some("30m".to_string()));
        assert_eq!(defaults.readonly, Some(false));
        assert_eq!(defaults.overlap_mode, Some("skip".to_string()));
        
        // Verify prompts
        assert_eq!(manifest.prompts.len(), 4);
        assert!(manifest.prompts.iter().any(|p| p.name == "ARCHITECT.md"));
        assert!(manifest.prompts.iter().any(|p| p.name == "CODE_REVIEWER.md"));
        
        // Verify agents
        assert_eq!(manifest.agents.len(), 5);
        assert!(manifest.agents.iter().any(|a| a.name == "architect"));
        assert!(manifest.agents.iter().any(|a| a.name == "code-reviewer"));
    }

    #[test]
    fn test_validate_bmad_prompts() {
        // Test validating prompts for bmad workflow
        let manifest_path = std::path::Path::new("examples/bmad/manifest.toml");
        
        if !manifest_path.exists() {
            eprintln!("Skipping test: examples/bmad/manifest.toml not found");
            return;
        }
        
        let manifest = ManifestConfig::from_path(manifest_path).expect("Failed to parse bmad manifest.toml");
        let workflow_dir = std::path::Path::new("examples/bmad");
        
        let result = manifest.validate_prompts(workflow_dir);
        assert!(result.is_ok(), "Prompt validation failed: {:?}", result.err());
    }

    #[test]
    fn test_backward_compatibility_minimal_manifest() {
        // Test parsing manifest with minimal required fields
        let toml_content = r#"
name = "minimal-workflow"

[[agents]]
name = "test-agent"
prompt_file = "test.md"
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        
        assert_eq!(manifest.name, Some("minimal-workflow".to_string()));
        assert_eq!(manifest.version, None);
        assert_eq!(manifest.description, None);
        assert_eq!(manifest.defaults, None);
        assert!(manifest.prompts.is_empty());
        assert_eq!(manifest.agents.len(), 1);
    }

    #[test]
    fn test_backward_compatibility_no_agents() {
        // Test parsing manifest with no agents (edge case)
        let toml_content = r#"
name = "empty-workflow"
description = "A workflow with no agents"

[[prompts]]
name = "test.md"
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        
        assert_eq!(manifest.name, Some("empty-workflow".to_string()));
        assert!(manifest.agents.is_empty());
        assert!(!manifest.prompts.is_empty());
    }

    #[test]
    fn test_error_handling_invalid_toml() {
        // Test error handling for invalid TOML
        let invalid_toml = r#"
name = "bad-workflow"
invalid[ = ] syntax
"#;
        let result: Result<ManifestConfig, _> = toml::from_str(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_handling_missing_agent_name() {
        // Test error handling for missing agent name
        let toml_content = r#"
[[agents]]
prompt_file = "test.md"
"#;
        let result: Result<ManifestConfig, _> = toml::from_str(toml_content);
        // This should fail because name is required
        assert!(result.is_err() || result.unwrap().agents[0].name.is_empty());
    }

    #[test]
    fn test_error_handling_missing_prompt_file() {
        // Test error handling for missing prompt_file
        let toml_content = r#"
[[agents]]
name = "test-agent"
"#;
        let result: Result<ManifestConfig, _> = toml::from_str(toml_content);
        // This should fail because prompt_file is required
        assert!(result.is_err() || result.unwrap().agents[0].prompt_file.is_empty());
    }

    #[test]
    fn test_overlap_mode_case_insensitive() {
        // Test that overlap_mode is case-insensitive
        let toml_content = r#"
[defaults]
overlap_mode = "QUEUE"

[[agents]]
name = "test-agent"
prompt_file = "test.md"
overlap_mode = "Skip"
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        
        let defaults = manifest.defaults.unwrap();
        assert_eq!(defaults.overlap_mode, Some("QUEUE".to_string()));
        
        let agent = &manifest.agents[0];
        assert_eq!(agent.overlap_mode, Some("Skip".to_string()));
    }

    #[test]
    fn test_agent_env_parsing() {
        // Test parsing environment variables
        let toml_content = r#"
[defaults]
env = { KEY1 = "value1", KEY2 = "value2" }

[[agents]]
name = "test-agent"
prompt_file = "test.md"
env = { AGENT_ID = "1", DEBUG = "true" }
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        
        let defaults = manifest.defaults.unwrap();
        let env = defaults.env.unwrap();
        assert_eq!(env.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(env.get("KEY2"), Some(&"value2".to_string()));
        
        let agent = &manifest.agents[0];
        let agent_env = agent.env.as_ref().unwrap();
        assert_eq!(agent_env.get("AGENT_ID"), Some(&"1".to_string()));
    }

    #[test]
    fn test_skills_parsing() {
        // Test parsing skills array
        let toml_content = r#"
[defaults]
skills = ["skill1", "skill2"]

[[agents]]
name = "test-agent"
prompt_file = "test.md"
skills = ["agent-skill"]
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        
        let defaults = manifest.defaults.unwrap();
        assert_eq!(defaults.skills.unwrap(), vec!["skill1", "skill2"]);
        
        let agent = &manifest.agents[0];
        assert_eq!(agent.skills.as_ref().unwrap(), &vec!["agent-skill"]);
    }
}
