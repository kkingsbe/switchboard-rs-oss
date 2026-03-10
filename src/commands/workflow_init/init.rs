//! Workflow init command implementation
//!
//! This module provides the functionality to scaffold a new Switchboard workflow
//! with the standard directory structure and configuration.

use crate::commands::workflow_init::types::{ExitCode, WorkflowInit};
use std::fs;
use std::path::{Path, PathBuf};

/// Default agents when none are specified
const DEFAULT_AGENTS: &[&str] = &["ARCHITECT", "DEVELOPER", "REVIEWER"];

/// Run the `switchboard workflow init` command
///
/// This command scaffolds a new Switchboard workflow with the following structure:
/// - manifest.toml - Workflow definition with manifest_version, name, description, agents, schedule
/// - prompts/<AGENT>.md - Prompt templates for each agent
///
/// # Arguments
///
/// * `args` - The WorkflowInit arguments containing name, agents, schedule, and path
///
/// # Returns
///
/// * `ExitCode::Success` - Workflow created successfully
/// * `ExitCode::Error` - Workflow creation failed
pub async fn run_workflow_init(args: WorkflowInit) -> ExitCode {
    // 1. Validate workflow name (must be valid identifier)
    if let Err(e) = validate_workflow_name(&args.name) {
        eprintln!("Error: {}", e);
        return ExitCode::Error;
    }

    // 2. Validate cron schedule if provided
    if let Some(ref schedule) = args.schedule {
        if let Err(e) = validate_cron_schedule(schedule) {
            eprintln!("Error: {}", e);
            return ExitCode::Error;
        }
    }

    // 3. Determine and validate target directory
    let target_dir = resolve_target_directory(&args.path, &args.name);
    
    if let Err(e) = validate_target_directory(&target_dir) {
        eprintln!("Error: {}", e);
        return ExitCode::Error;
    }

    // 4. Determine agents list
    let agents = parse_agents(args.agents.as_deref());

    // 5. Create workflow structure
    if let Err(e) = create_workflow_structure(&target_dir, &args.name, &args.schedule, &agents) {
        eprintln!("Error: {}", e);
        return ExitCode::Error;
    }

    println!(
        "Successfully initialized workflow '{}' at {}",
        args.name,
        target_dir.display()
    );

    ExitCode::Success
}

/// Validate the workflow name
///
/// Validates that the name is a valid identifier:
/// - Alphanumeric characters and underscores only
/// - Must start with a letter or underscore
/// - Cannot be empty
fn validate_workflow_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Workflow name cannot be empty".to_string());
    }

    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return Err(format!(
            "Workflow name '{}' must start with a letter or underscore, not '{}'",
            name, first_char
        ));
    }

    for (i, c) in name.chars().enumerate() {
        if !c.is_alphanumeric() && c != '_' {
            return Err(format!(
                "Workflow name '{}' contains invalid character '{}' at position {}",
                name, c, i
            ));
        }
    }

    Ok(())
}

/// Validate the cron schedule expression
///
/// Basic validation: checks for 5 fields (minute, hour, day, month, weekday)
fn validate_cron_schedule(schedule: &str) -> Result<(), String> {
    let parts: Vec<&str> = schedule.split_whitespace().collect();
    
    if parts.len() != 5 {
        return Err(format!(
            "Cron schedule '{}' must have exactly 5 fields (minute hour day month weekday), got {}",
            schedule,
            parts.len()
        ));
    }

    // Validate each field has valid characters
    let valid_chars = |field: &str| -> bool {
        field.chars().all(|c| c.is_numeric() || c == '*' || c == '/' || c == '-' || c == ',')
    };

    for (i, part) in parts.iter().enumerate() {
        if !valid_chars(part) {
            return Err(format!(
                "Cron schedule '{}' has invalid character in field {}: '{}'",
                schedule, i + 1, part
            ));
        }
    }

    Ok(())
}

/// Resolve the target directory from path and workflow name
fn resolve_target_directory(path: &str, name: &str) -> PathBuf {
    let target = if path == "." {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    } else {
        PathBuf::from(path)
    };

    // Resolve to absolute path
    if target.is_absolute() {
        target.join(name)
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&target)
            .join(name)
    }
}

/// Validate the target directory
///
/// Checks if:
/// - Directory exists and is a directory
/// - Directory is empty (workflows must be created in empty directories)
fn validate_target_directory(target_dir: &Path) -> Result<(), String> {
    if target_dir.exists() {
        if !target_dir.is_dir() {
            return Err(format!(
                "Path '{}' exists but is not a directory",
                target_dir.display()
            ));
        }

        // Check if directory is empty
        let entries = fs::read_dir(target_dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        let entries_vec: Vec<_> = entries.filter_map(|e| e.ok()).collect();

        // If directory has visible files, it's not valid
        let has_visible_files = entries_vec
            .iter()
            .any(|e| {
                let name = e.file_name();
                !name.to_string_lossy().starts_with('.')
            });

        if has_visible_files {
            return Err(format!(
                "Directory '{}' is not empty. Use a different path or create an empty directory.",
                target_dir.display()
            ));
        }
    }

    Ok(())
}

/// Parse agents from comma-separated string or use defaults
fn parse_agents(agents_arg: Option<&str>) -> Vec<String> {
    match agents_arg {
        Some(agents_str) => {
            agents_str
                .split(',')
                .map(|s| s.trim().to_uppercase())
                .filter(|s| !s.is_empty())
                .collect()
        }
        None => DEFAULT_AGENTS.iter().map(|s| s.to_string()).collect(),
    }
}

/// Create the workflow directory structure and files
fn create_workflow_structure(
    target_dir: &Path,
    workflow_name: &str,
    schedule: &Option<String>,
    agents: &[String],
) -> Result<(), String> {
    // Create main workflow directory
    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Failed to create directory '{}': {}", target_dir.display(), e))?;

    // Create prompts directory
    let prompts_dir = target_dir.join("prompts");
    fs::create_dir_all(&prompts_dir)
        .map_err(|e| format!("Failed to create prompts directory: {}", e))?;

    // Create manifest.toml
    let manifest_content = get_manifest_content(workflow_name, schedule, agents);
    let manifest_path = target_dir.join("manifest.toml");
    fs::write(&manifest_path, manifest_content)
        .map_err(|e| format!("Failed to create '{}': {}", manifest_path.display(), e))?;

    // Create prompt files for each agent
    for agent in agents {
        let prompt_content = get_agent_prompt(agent, workflow_name);
        let prompt_path = prompts_dir.join(format!("{}.md", agent));
        fs::write(&prompt_path, prompt_content)
            .map_err(|e| format!("Failed to create '{}': {}", prompt_path.display(), e))?;
    }

    // Create .gitkeep for prompts directory
    let gitkeep_path = prompts_dir.join(".gitkeep");
    if !gitkeep_path.exists() {
        fs::write(&gitkeep_path, "").map_err(|e| format!("Failed to create '{}': {}", gitkeep_path.display(), e))?;
    }

    Ok(())
}

/// Generate manifest.toml content
fn get_manifest_content(workflow_name: &str, schedule: &Option<String>, agents: &[String]) -> String {
    let schedule_section = schedule
        .as_ref()
        .map(|s| format!("cron = \"{}\"", s))
        .unwrap_or_else(|| "# cron = \"0 0 * * *\"  # Uncomment to enable scheduled runs".to_string());

    let agents_list: String = agents
        .iter()
        .map(|agent| format!("# - {}", agent.to_lowercase()))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"# Switchboard Workflow Manifest
# Generated by switchboard workflow init

manifest_version = "0.1"
name = "{name}"
description = "A new Switchboard workflow"

[agents]
# List of agents in this workflow
{agents_list}

[schedule]
# Workflow schedule configuration
{schedule}
"#,
        name = workflow_name,
        agents_list = agents_list,
        schedule = schedule_section
    )
}

/// Generate agent prompt template content
fn get_agent_prompt(agent_name: &str, workflow_name: &str) -> String {
    let role = match agent_name.to_uppercase().as_str() {
        "ARCHITECT" => "Architect",
        "DEVELOPER" => "Developer",
        "REVIEWER" => "Code Reviewer",
        "SCRUM_MASTER" => "Scrum Master",
        _ => agent_name,
    };

    let responsibilities = match agent_name.to_uppercase().as_str() {
        "ARCHITECT" => r##"- Analyze requirements and design system architecture
- Make key technical decisions
- Provide architectural guidance
- Plan sprint work and distribute to developers"##,
        "DEVELOPER" => r##"- Implement features and fixes
- Write clean, maintainable code
- Write tests for new functionality
- Follow project conventions and patterns"##,
        "REVIEWER" => r##"- Review code implementations
- Verify acceptance criteria are met
- Ensure code quality and consistency
- Provide constructive feedback"##,
        "SCRUM_MASTER" => r##"- Facilitate sprint planning
- Remove impediments for the team
- Track progress and maintain velocity
- Coordinate between agents"##,
        _ => "- Collaborate with other agents to achieve workflow goals",
    };

    let guidelines = match agent_name.to_uppercase().as_str() {
        "ARCHITECT" => r##"- Consider scalability, maintainability, and security
- Document architectural decisions in ADRs
- Balance technical debt with feature delivery
- Coordinate with other agents for dependencies"##,
        "DEVELOPER" => r##"- Follow existing code patterns and conventions
- Write self-documenting code with clear names
- Include error handling and edge cases
- Test thoroughly before marking complete"##,
        "REVIEWER" => r##"- Be thorough but constructive in feedback
- Verify all acceptance criteria are met
- Check for security vulnerabilities
- Ensure test coverage is adequate"##,
        "SCRUM_MASTER" => r##"- Keep the team focused on sprint goals
- Identify and remove blockers
- Foster collaboration between agents
- Maintain transparent progress tracking"##,
        _ => "- Work effectively to achieve the workflow objectives",
    };

    format!(
        r#"# {role} Agent - {workflow_name}

You are the **{role}** agent for the **{workflow_name}** workflow.

## Your Role

{responsibilities}

## Guidelines

{guidelines}

## Workflow Context

This workflow operates within the Switchboard framework. You have access to:
- The workflow manifest at `manifest.toml`
- Agent prompts in the `prompts/` directory
- Project skills in the `skills/` directory (if available)
- State files in `.switchboard/state/` directory

## Getting Started

1. Read the workflow manifest to understand the overall structure
2. Review any existing agent prompts for context
3. Coordinate with other agents as needed
4. Complete your designated tasks efficiently

---
*Generated by switchboard workflow init*
"#,
        role = role,
        workflow_name = workflow_name,
        responsibilities = responsibilities,
        guidelines = guidelines
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_workflow_name_valid() {
        assert!(validate_workflow_name("my-workflow").is_ok());
        assert!(validate_workflow_name("workflow123").is_ok());
        assert!(validate_workflow_name("_private").is_ok());
        assert!(validate_workflow_name("ARCHITECT").is_ok());
    }

    #[test]
    fn test_validate_workflow_name_invalid() {
        assert!(validate_workflow_name("").is_err());
        assert!(validate_workflow_name("123-workflow").is_err());
        assert!(validate_workflow_name("my-workflow!").is_err());
        assert!(validate_workflow_name("my-workflow-name").is_err());
    }

    #[test]
    fn test_validate_cron_schedule_valid() {
        assert!(validate_cron_schedule("0 0 * * *").is_ok());
        assert!(validate_cron_schedule("*/15 * * * *").is_ok());
        assert!(validate_cron_schedule("0 9 * * 1-5").is_ok());
        assert!(validate_cron_schedule("0,30 * * * *").is_ok());
    }

    #[test]
    fn test_validate_cron_schedule_invalid() {
        assert!(validate_cron_schedule("0 0 * *").is_err());
        assert!(validate_cron_schedule("0 0 * * * *").is_err());
        assert!(validate_cron_schedule("invalid").is_err());
    }

    #[test]
    fn test_parse_agents() {
        assert_eq!(parse_agents(None), vec!["ARCHITECT", "DEVELOPER", "REVIEWER"]);
        assert_eq!(parse_agents(Some("ARCHITECT,DEVELOPER")), vec!["ARCHITECT", "DEVELOPER"]);
        assert_eq!(parse_agents(Some(" architect , developer , reviewer ")), vec!["ARCHITECT", "DEVELOPER", "REVIEWER"]);
    }
}
