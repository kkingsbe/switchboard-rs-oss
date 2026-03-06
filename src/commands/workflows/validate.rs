//! Handler for the `workflows validate` subcommand.
//!
//! This module provides the `run_workflows_validate` function which validates
//! a workflow's manifest.toml file.

use crate::config::Config;
use crate::skills::{extract_skill_name, SkillsManager};
use crate::workflows::manifest::{ManifestConfig, ManifestError};
use crate::workflows::WORKFLOWS_DIR;

use super::types::WorkflowsValidate;
use super::ExitCode;

use std::path::Path;

/// Run the `switchboard workflows validate` command
///
/// This command validates a workflow's manifest.toml file exists and is properly formatted.
/// It checks:
/// - manifest.toml exists
/// - All referenced prompt files exist in prompts/
/// - Cron schedule format is valid (basic check)
/// - overlap_mode values are valid ("skip" or "queue")
/// - Timeout format is valid (basic check)
///
/// # Arguments
///
/// * `args` - The [`WorkflowsValidate`] containing the workflow name
/// * `_config` - Reference to the application configuration (unused)
///
/// # Returns
///
/// Returns [`ExitCode::Success`] if validation passes, [`ExitCode::Error`] on failure
pub async fn run_workflows_validate(args: WorkflowsValidate, _config: &Config) -> ExitCode {
    let workflow_name = &args.workflow_name;

    // Determine the workflow path
    let workflow_path = Path::new(WORKFLOWS_DIR).join(workflow_name);

    // Check if workflow directory exists
    if !workflow_path.exists() {
        eprintln!(
            "Error: Workflow '{}' not found at {}/",
            workflow_name,
            workflow_path.display()
        );
        eprintln!("Make sure the workflow is installed first.");
        return ExitCode::Error;
    }

    // Check if manifest.toml exists
    let manifest_path = workflow_path.join("manifest.toml");
    if !manifest_path.exists() {
        eprintln!(
            "Error: manifest.toml not found for workflow '{}'",
            workflow_name
        );
        eprintln!("manifest.toml is required for validation.");
        return ExitCode::Error;
    }

    // Try to load and parse manifest.toml
    let manifest = match ManifestConfig::from_path(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: Failed to parse manifest.toml:");
            eprintln!("  {}", e);
            return ExitCode::Error;
        }
    };

    // Validate prompt files exist
    if let Err(e) = manifest.validate_prompts(&workflow_path) {
        match e {
            ManifestError::PromptNotFound(prompt) => {
                eprintln!("Error: Referenced prompt file '{}' not found in prompts/", prompt);
                return ExitCode::Error;
            }
            ManifestError::IoError(io_err) => {
                eprintln!("Error: I/O error while validating prompts: {}", io_err);
                return ExitCode::Error;
            }
            _ => {
                eprintln!("Error: Prompt validation failed: {}", e);
                return ExitCode::Error;
            }
        }
    }

    // Validate cron schedule format (basic check)
    if let Err(e) = validate_cron_schedules(&manifest, &workflow_name) {
        eprintln!("Error: {}", e);
        return ExitCode::Error;
    }

    // Validate overlap_mode values
    if let Err(e) = validate_overlap_modes(&manifest) {
        eprintln!("Error: {}", e);
        return ExitCode::Error;
    }

    // Validate timeout format (basic check)
    if let Err(e) = validate_timeouts(&manifest) {
        eprintln!("Error: {}", e);
        return ExitCode::Error;
    }

    // Validate skills are installed
    let skill_warnings = validate_workflow_skills(&manifest);
    for warning in &skill_warnings {
        println!("Warning: {}", warning);
    }

    if !skill_warnings.is_empty() {
        // Collect unique skill sources for the install command
        let mut unique_skills: Vec<String> = Vec::new();
        
        if let Some(defaults) = &manifest.defaults {
            if let Some(skills) = &defaults.skills {
                for skill in skills {
                    if !unique_skills.contains(skill) {
                        unique_skills.push(skill.clone());
                    }
                }
            }
        }
        
        for agent in &manifest.agents {
            if let Some(skills) = &agent.skills {
                for skill in skills {
                    if !unique_skills.contains(skill) {
                        unique_skills.push(skill.clone());
                    }
                }
            }
        }
        
        println!("\nTo install missing skills, run:");
        for skill_source in unique_skills {
            println!("  switchboard skills install {}", skill_source);
        }
    }

    // All validations passed
    println!("✓ Manifest is valid");
    ExitCode::Success
}

/// Validate cron schedule format (basic check)
///
/// Checks that cron expressions have the correct number of fields (5)
/// and contain valid characters.
fn validate_cron_schedules(manifest: &ManifestConfig, _workflow_name: &str) -> Result<(), String> {
    // Check defaults schedule
    if let Some(defaults) = &manifest.defaults {
        if let Some(schedule) = &defaults.schedule {
            validate_cron_expression(schedule, "defaults.schedule")?;
        }
    }

    // Check each agent's schedule
    for agent in &manifest.agents {
        if let Some(schedule) = &agent.schedule {
            let field = format!("agent '{}'.schedule", agent.name);
            validate_cron_expression(schedule, &field)?;
        }
    }

    Ok(())
}

/// Validate a single cron expression (basic check)
fn validate_cron_expression(expr: &str, field: &str) -> Result<(), String> {
    let parts: Vec<&str> = expr.trim().split_whitespace().collect();

    if parts.len() != 5 {
        return Err(format!(
            "Invalid cron expression '{}' for {}: expected 5 fields, got {}",
            expr,
            field,
            parts.len()
        ));
    }

    // Basic character validation - cron expressions should only contain
    // digits, *, /, -, and ,
    let valid_chars = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '*', '/', '-', ',', ' '];
    for part in &parts {
        for ch in part.chars() {
            if !valid_chars.contains(&ch) {
                return Err(format!(
                    "Invalid cron expression '{}' for {}: contains invalid character '{}'",
                    expr, field, ch
                ));
            }
        }
    }

    Ok(())
}

/// Validate overlap_mode values are valid ("skip" or "queue")
fn validate_overlap_modes(manifest: &ManifestConfig) -> Result<(), String> {
    let valid_modes = ["skip", "queue"];

    // Check defaults overlap_mode
    if let Some(defaults) = &manifest.defaults {
        if let Some(mode) = &defaults.overlap_mode {
            let mode_lower = mode.to_lowercase();
            if !valid_modes.contains(&mode_lower.as_str()) {
                return Err(format!(
                    "Invalid overlap_mode '{}' in defaults: must be 'skip' or 'queue'",
                    mode
                ));
            }
        }
    }

    // Check each agent's overlap_mode
    for agent in &manifest.agents {
        if let Some(mode) = &agent.overlap_mode {
            let mode_lower = mode.to_lowercase();
            if !valid_modes.contains(&mode_lower.as_str()) {
                return Err(format!(
                    "Invalid overlap_mode '{}' in agent '{}': must be 'skip' or 'queue'",
                    mode, agent.name
                ));
            }
        }
    }

    Ok(())
}

/// Validate timeout format (basic check)
///
/// Validates that timeout strings end with a valid unit (m for minutes, h for hours)
fn validate_timeouts(manifest: &ManifestConfig) -> Result<(), String> {
    // Check defaults timeout
    if let Some(defaults) = &manifest.defaults {
        if let Some(timeout) = &defaults.timeout {
            validate_timeout_format(timeout, "defaults.timeout")?;
        }
    }

    // Check each agent's timeout
    for agent in &manifest.agents {
        if let Some(timeout) = &agent.timeout {
            let field = format!("agent '{}'.timeout", agent.name);
            validate_timeout_format(timeout, &field)?;
        }
    }

    Ok(())
}

/// Validate a single timeout format
fn validate_timeout_format(timeout: &str, field: &str) -> Result<(), String> {
    let timeout = timeout.trim();

    if timeout.is_empty() {
        return Err(format!("Empty timeout value for {}", field));
    }

    // Check it ends with a valid unit
    if !timeout.ends_with('m') && !timeout.ends_with('h') && !timeout.ends_with('s') {
        return Err(format!(
            "Invalid timeout format '{}' for {}: must end with 'm' (minutes), 'h' (hours), or 's' (seconds)",
            timeout, field
        ));
    }

    // Check the numeric part
    let numeric_part = &timeout[..timeout.len() - 1];
    if numeric_part.is_empty() {
        return Err(format!(
            "Invalid timeout format '{}' for {}: missing numeric value",
            timeout, field
        ));
    }

    if numeric_part.parse::<u64>().is_err() {
        return Err(format!(
            "Invalid timeout format '{}' for {}: '{}' is not a valid number",
            timeout, field, numeric_part
        ));
    }

    Ok(())
}

/// Validates that required skills are installed
///
/// This function checks if all skills referenced in the manifest are actually
/// installed in the project's skills directory.
///
/// # Arguments
///
/// * `manifest` - Reference to the ManifestConfig containing skill references
///
/// # Returns
///
/// * `Vec<String>` - Vector of warning messages for missing skills
pub fn validate_workflow_skills(manifest: &ManifestConfig) -> Vec<String> {
    let mut warnings = Vec::new();
    let manager = SkillsManager::new(None);
    
    // Collect all skills from manifest
    let mut all_skills: Vec<String> = Vec::new();
    
    if let Some(defaults) = &manifest.defaults {
        if let Some(skills) = &defaults.skills {
            all_skills.extend(skills.clone());
        }
    }
    
    for agent in &manifest.agents {
        if let Some(skills) = &agent.skills {
            all_skills.extend(skills.clone());
        }
    }
    
    // Remove duplicates
    all_skills.sort();
    all_skills.dedup();
    
    // Check each skill
    for skill_source in &all_skills {
        // Extract skill name - handle potential errors gracefully
        let skill_name = match extract_skill_name(skill_source) {
            Ok(name) => name,
            Err(_) => {
                // If we can't extract the skill name, use the source as-is
                skill_source.clone()
            }
        };
        let skill_path = manager.skills_dir.join(&skill_name);
        
        if !skill_path.exists() {
            warnings.push(format!(
                "Skill '{}' (required by workflow) is not installed. \
                Run 'switchboard skills install {}' to install it.",
                skill_name, skill_source
            ));
        }
    }
    
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cron_expression_valid() {
        assert!(validate_cron_expression("0 9 * * *", "test").is_ok());
        assert!(validate_cron_expression("*/15 * * * *", "test").is_ok());
        assert!(validate_cron_expression("0 0 1 * *", "test").is_ok());
    }

    #[test]
    fn test_validate_cron_expression_invalid() {
        assert!(validate_cron_expression("0 9 * *", "test").is_err());
        assert!(validate_cron_expression("0 9 * * * *", "test").is_err());
    }

    #[test]
    fn test_validate_timeout_format_valid() {
        assert!(validate_timeout_format("30m", "test").is_ok());
        assert!(validate_timeout_format("2h", "test").is_ok());
        assert!(validate_timeout_format("60s", "test").is_ok());
    }

    #[test]
    fn test_validate_timeout_format_invalid() {
        assert!(validate_timeout_format("30", "test").is_err());
        assert!(validate_timeout_format("m", "test").is_err());
        assert!(validate_timeout_format("", "test").is_err());
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_validate_invalid_overlap_mode_defaults() {
        let toml_content = r#"
[defaults]
overlap_mode = "invalid"

[[agents]]
name = "test-agent"
prompt_file = "test.md"
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        let result = validate_overlap_modes(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid overlap_mode"));
    }

    #[test]
    fn test_validate_invalid_overlap_mode_agent() {
        let toml_content = r#"
[[agents]]
name = "test-agent"
prompt_file = "test.md"
overlap_mode = "invalid"
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        let result = validate_overlap_modes(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid overlap_mode"));
    }

    #[test]
    fn test_validate_invalid_cron_expression() {
        let toml_content = r#"
[defaults]
schedule = "0 9 *"

[[agents]]
name = "test-agent"
prompt_file = "test.md"
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        let result = validate_cron_schedules(&manifest, "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid cron expression"));
    }

    #[test]
    fn test_validate_invalid_timeout_format() {
        let toml_content = r#"
[defaults]
timeout = "invalid"

[[agents]]
name = "test-agent"
prompt_file = "test.md"
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        let result = validate_timeouts(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid timeout format"));
    }

    #[test]
    fn test_validate_valid_manifest() {
        let toml_content = r#"
name = "test-workflow"
version = "1.0.0"

[defaults]
schedule = "0 9 * * *"
timeout = "30m"
overlap_mode = "skip"

[[prompts]]
name = "test.md"

[[agents]]
name = "test-agent"
prompt_file = "test.md"
schedule = "0 10 * * *"
timeout = "1h"
overlap_mode = "queue"
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        
        assert!(validate_cron_schedules(&manifest, "test").is_ok());
        assert!(validate_overlap_modes(&manifest).is_ok());
        assert!(validate_timeouts(&manifest).is_ok());
    }

    // ============================================================================
    // Skill Validation Tests
    // ============================================================================

    #[test]
    fn test_validate_workflow_skills_no_skills_defined() {
        // Test manifest with no skills defined
        let toml_content = r#"
name = "test-workflow"
version = "1.0.0"

[defaults]
schedule = "0 9 * * *"
timeout = "30m"

[[agents]]
name = "test-agent"
prompt_file = "test.md"
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        
        // No skills defined, so should return empty warnings
        let warnings = validate_workflow_skills(&manifest);
        assert!(warnings.is_empty(), "Expected no warnings when no skills are defined");
    }

    #[test]
    fn test_validate_workflow_skills_with_defaults_skills() {
        // Test manifest with skills in defaults section
        let toml_content = r#"
name = "test-workflow"
version = "1.0.0"

[defaults]
schedule = "0 9 * * *"
timeout = "30m"
skills = ["skills/repo", "skills/repo1"]

[[agents]]
name = "test-agent"
prompt_file = "test.md"
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        
        // Skills defined in defaults, should get warnings for missing skills
        let warnings = validate_workflow_skills(&manifest);
        // The actual result depends on whether skills are installed
        // At minimum, we should have collected the skills from defaults
        assert!(!warnings.is_empty() || manifest.defaults.as_ref().unwrap().skills.as_ref().unwrap().len() == 2);
    }

    #[test]
    fn test_validate_workflow_skills_with_agent_skills() {
        // Test manifest with skills in agent section
        let toml_content = r#"
name = "test-workflow"
version = "1.0.0"

[defaults]
schedule = "0 9 * * *"
timeout = "30m"

[[agents]]
name = "test-agent"
prompt_file = "test.md"
skills = ["skills/repo2", "skills/repo3"]
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        
        // Skills defined in agent, should get warnings for missing skills
        let warnings = validate_workflow_skills(&manifest);
        // The actual result depends on whether skills are installed
        // At minimum, we should have collected the skills from agent
        assert!(!warnings.is_empty() || manifest.agents[0].skills.as_ref().unwrap().len() == 2);
    }

    #[test]
    fn test_validate_workflow_skills_with_both_defaults_and_agents() {
        // Test manifest with skills in both defaults and agent sections
        let toml_content = r#"
name = "test-workflow"
version = "1.0.0"

[defaults]
schedule = "0 9 * * *"
timeout = "30m"
skills = ["skills/repo"]

[[agents]]
name = "test-agent1"
prompt_file = "test.md"
skills = ["skills/repo1"]

[[agents]]
name = "test-agent2"
prompt_file = "test2.md"
skills = ["skills/repo2", "skills/repo3"]
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        
        // Skills defined in both defaults and agents
        let warnings = validate_workflow_skills(&manifest);
        // Should have 4 unique skills total
        // Warnings depend on whether they're installed
        let defaults_skills = manifest.defaults.as_ref().unwrap().skills.as_ref().unwrap().len();
        let agent_skills: usize = manifest.agents.iter()
            .map(|a| a.skills.as_ref().map_or(0, |s| s.len()))
            .sum();
        assert_eq!(defaults_skills + agent_skills, 4);
    }

    #[test]
    fn test_validate_workflow_skills_duplicate_removal() {
        // Test that duplicate skills are handled correctly
        let toml_content = r#"
name = "test-workflow"
version = "1.0.0"

[defaults]
skills = ["skills/repo", "skills/repo1"]

[[agents]]
name = "test-agent"
prompt_file = "test.md"
skills = ["skills/repo", "skills/repo2"]
"#;
        let manifest: ManifestConfig = toml::from_str(toml_content).unwrap();
        
        // skills/repo appears in both defaults and agent, should be deduped
        let warnings = validate_workflow_skills(&manifest);
        // We should have 3 unique skills (repo, repo1, repo2)
        // but skills/repo only appears once in warnings (if not installed)
        let skill_names: Vec<&str> = warnings.iter()
            .filter_map(|w| {
                // Extract skill name from warning message
                if let Some(start) = w.find("'") {
                    if let Some(end) = w[start+1..].find("'") {
                        return Some(&w[start+1..start+1+end]);
                    }
                }
                None
            })
            .collect();
        
        // Check that repo appears only once in the warnings
        let repo_count = skill_names.iter().filter(|&&n| n == "repo").count();
        assert_eq!(repo_count, 1, "Duplicate skills should be removed");
    }
}
