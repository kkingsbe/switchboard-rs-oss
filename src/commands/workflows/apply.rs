//! Handler for the `workflows apply` subcommand.
//!
//! This module provides the `run_workflows_apply` function which generates
//! switchboard.toml entries from a workflow's manifest.toml file.

use crate::config::{Agent, Config};
use crate::workflows::manifest::{ManifestAgent, ManifestConfig, ManifestDefaults};
use crate::workflows::WORKFLOWS_DIR;

use super::types::{ExitCode, WorkflowsApply};

use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn merge_agents_into_toml(existing_content: &str, agents_to_add: &[Agent]) -> Result<String, Box<dyn std::error::Error>> {
    let trimmed = existing_content.trim();
    if trimmed.is_empty() {
        let mut config = Config::default();
        config.agents = agents_to_add.to_vec();
        return Ok(toml::to_string_pretty(&config)?);
    }

    let mut document: toml::Value = toml::from_str(existing_content)?;
    let agent_values: Vec<toml::Value> = agents_to_add
        .iter()
        .cloned()
        .map(toml::Value::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    match document.get_mut("agent") {
        Some(toml::Value::Array(existing_agents)) => existing_agents.extend(agent_values),
        Some(_) => unreachable!("validated switchboard.toml should deserialize 'agent' as an array"),
        None => {
            if let Some(table) = document.as_table_mut() {
                table.insert("agent".to_string(), toml::Value::Array(agent_values));
            }
        }
    }

    Ok(toml::to_string_pretty(&document)?)
}

/// Run the `switchboard workflows apply` command
///
/// This command generates switchboard.toml configuration entries from a workflow's
/// manifest.toml file. It creates agent configurations for each agent defined in
/// the manifest, applying default values from the manifest where not specified.
///
/// # Arguments
///
/// * `args` - The [`WorkflowsApply`] containing the workflow name and options
/// * `_config` - Reference to the application configuration (unused)
///
/// # Returns
///
/// Returns [`ExitCode::Success`] if the operation completes successfully,
/// [`ExitCode::Error`] on failure
pub async fn run_workflows_apply(args: WorkflowsApply, _config: &Config) -> ExitCode {
    let workflow_name = &args.workflow_name;
    let prefix = args.prefix.as_deref().unwrap_or(workflow_name);
    let output_path = args.output.as_deref().unwrap_or("switchboard.toml");

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
        eprintln!("Run: switchboard workflows install {}", workflow_name);
        return ExitCode::Error;
    }

    // Check if manifest.toml exists
    let manifest_path = workflow_path.join("manifest.toml");
    if !manifest_path.exists() {
        eprintln!(
            "Error: manifest.toml not found for workflow '{}'",
            workflow_name
        );
        eprintln!(
            "The workflow may be outdated. Try updating it:\nswitchboard workflows update {}",
            workflow_name
        );
        return ExitCode::Error;
    }

    // Load and parse manifest.toml
    let manifest = match ManifestConfig::from_path(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: Failed to parse manifest.toml:");
            eprintln!("  {}", e);
            return ExitCode::Error;
        }
    };

    // Get the defaults (or create empty defaults if not specified)
    let defaults = manifest.defaults.clone().unwrap_or_else(|| ManifestDefaults {
        schedule: None,
        timeout: None,
        readonly: None,
        overlap_mode: None,
        max_queue_size: None,
        env: None,
        skills: None,
    });

    // Convert manifest agents to Config agents
    let new_agents: Vec<Agent> = manifest
        .agents
        .iter()
        .map(|agent: &ManifestAgent| agent.to_agent(prefix, &defaults))
        .collect();

    if new_agents.is_empty() {
        eprintln!(
            "Warning: Workflow '{}' has no agents defined in manifest.toml",
            workflow_name
        );
        return ExitCode::Error;
    }

    // Handle the config file
    let output = Path::new(output_path);
    let mut config = if output.exists() {
        // Load existing config in append mode
        match Config::from_toml(output) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: Failed to load existing config at '{}':", output_path);
                eprintln!("  {}", e);
                return ExitCode::Error;
            }
        }
    } else {
        // Create new config
        Config::default()
    };

    // Check for name conflicts
    let existing_names: HashSet<String> = config.agents.iter().map(|a| a.name.clone()).collect();
    let mut agents_to_add: Vec<Agent> = Vec::new();

    for agent in new_agents {
        if existing_names.contains(&agent.name) {
            eprintln!(
                "Warning: Agent '{}' already exists in config. Use --prefix to avoid conflicts.",
                agent.name
            );
        } else {
            agents_to_add.push(agent);
        }
    }

    if agents_to_add.is_empty() {
        eprintln!("No new agents to add (all agents already exist with that prefix).");
        return ExitCode::Error;
    }

    // Show preview if --dry-run
    if args.dry_run {
        println!("=== Preview: switchboard.toml would contain ===\n");
        println!("# New agents to be added from workflow '{}':\n", workflow_name);
        for agent in &agents_to_add {
            println!("[[agent]]");
            println!("name = \"{}\"", agent.name);
            if let Some(pf) = &agent.prompt_file {
                println!("prompt_file = \"{}\"", pf);
            }
            println!("schedule = \"{}\"", agent.schedule);
            if let Some(t) = &agent.timeout {
                println!("timeout = \"{}\"", t);
            }
            if let Some(ro) = agent.readonly {
                println!("readonly = {}", ro);
            }
            if let Some(om) = &agent.overlap_mode {
                println!("overlap_mode = \"{:?}\"", om);
            }
            if let Some(mqs) = agent.max_queue_size {
                println!("max_queue_size = {}", mqs);
            }
            if let Some(skills) = &agent.skills {
                println!("skills = {:?}", skills);
            }
            println!();
        }
        return ExitCode::Success;
    }

    // Ask for confirmation unless --yes
    if !args.yes {
        println!(
            "This will add {} agent(s) from workflow '{}' to '{}'.",
            agents_to_add.len(),
            workflow_name,
            output_path
        );

        if output.exists() {
            println!("The agents will be appended to the existing configuration.");
        } else {
            println!("A new configuration file will be created.");
        }

        print!("\nDo you want to continue? [y/N] ");
        if let Err(e) = io::stdout().flush() {
            eprintln!("Error flushing stdout: {}", e);
            return ExitCode::Error;
        }

        let mut response = String::new();
        if let Err(e) = io::stdin().read_line(&mut response) {
            eprintln!("Error reading input: {}", e);
            return ExitCode::Error;
        }

        let response = response.trim().to_lowercase();
        if response != "y" && response != "yes" {
            println!("Aborted.");
            return ExitCode::Error;
        }
    }

    // Add new agents to config
    let agents_count = agents_to_add.len();
    config.agents.extend(agents_to_add);

    // Write the config to file
    let toml_content = if output.exists() {
        let existing_content = match fs::read_to_string(output) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error: Failed to read existing config at '{}':", output_path);
                eprintln!("  {}", e);
                return ExitCode::Error;
            }
        };

        match merge_agents_into_toml(&existing_content, &config.agents[config.agents.len() - agents_count..]) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: Failed to serialize merged config:");
                eprintln!("  {}", e);
                return ExitCode::Error;
            }
        }
    } else {
        match toml::to_string_pretty(&config) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: Failed to serialize config:");
                eprintln!("  {}", e);
                return ExitCode::Error;
            }
        }
    };

    if let Err(e) = fs::write(output, &toml_content) {
        eprintln!("Error: Failed to write config to '{}':", output_path);
        eprintln!("  {}", e);
        return ExitCode::Error;
    }

    println!(
        "Successfully added {} agent(s) to '{}'",
        agents_count,
        output_path
    );
    println!("Configuration saved.");

    ExitCode::Success
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_requires_workflow() {
        // This test would require mocking the workflow directory
        // For now, just test the basic structure
        let args = WorkflowsApply {
            workflow_name: "nonexistent".to_string(),
            prefix: None,
            output: None,
            append: false,
            yes: true,
            dry_run: true,
        };
        // The function will return Error because workflow doesn't exist
        // This is expected behavior
    }

    #[test]
    fn test_apply_struct_fields() {
        let args = WorkflowsApply {
            workflow_name: "test-workflow".to_string(),
            prefix: Some("custom".to_string()),
            output: Some("custom.toml".to_string()),
            append: true,
            yes: false,
            dry_run: true,
        };

        assert_eq!(args.workflow_name, "test-workflow");
        assert_eq!(args.prefix, Some("custom".to_string()));
        assert_eq!(args.output, Some("custom.toml".to_string()));
        assert!(args.append);
        assert!(!args.yes);
        assert!(args.dry_run);
    }

    #[test]
    fn test_merge_agents_into_toml_preserves_unrelated_sections() {
        let existing = r#"
[settings]
timezone = "America/New_York"

[api]
enabled = true
port = 8080

[[agent]]
name = "existing-agent"
prompt = "Keep me"
schedule = "0 * * * *"
"#;

        let new_agents = vec![Agent {
            name: "workflow-agent".to_string(),
            prompt: None,
            prompt_file: Some(".switchboard/workflows/test/prompts/WORKFLOW.md".to_string()),
            schedule: "*/5 * * * *".to_string(),
            timeout: Some("10m".to_string()),
            readonly: Some(true),
            overlap_mode: None,
            max_queue_size: None,
            env: None,
            skills: Some(vec!["frontend-design".to_string()]),
            silent_timeout: None,
            gpu: None,
        }];

        let merged = merge_agents_into_toml(existing, &new_agents).unwrap();

        assert!(merged.contains("[settings]"));
        assert!(merged.contains("timezone = \"America/New_York\""));
        assert!(merged.contains("[api]"));
        assert!(merged.contains("port = 8080"));
        assert!(merged.contains("name = \"existing-agent\""));
        assert!(merged.contains("name = \"workflow-agent\""));
    }

    #[test]
    fn test_merge_agents_into_empty_toml_creates_agent_config() {
        let new_agents = vec![Agent {
            name: "workflow-agent".to_string(),
            prompt: Some("Hello".to_string()),
            prompt_file: None,
            schedule: "*/5 * * * *".to_string(),
            timeout: None,
            readonly: None,
            overlap_mode: None,
            max_queue_size: None,
            env: None,
            skills: None,
            silent_timeout: None,
            gpu: None,
        }];

        let merged = merge_agents_into_toml("", &new_agents).unwrap();

        assert!(merged.contains("[[agent]]"));
        assert!(merged.contains("name = \"workflow-agent\""));
        assert!(merged.contains("prompt = \"Hello\""));
    }

    #[test]
    fn test_merge_agents_into_toml_preserves_canonical_agent_array() {
        let existing_content = r#"
[[agent]]
name = "existing"
schedule = "0 9 * * *"
prompt_file = "existing.md"
"#;

        let agents_to_add = vec![Agent {
            name: "added".to_string(),
            prompt: None,
            prompt_file: Some("added.md".to_string()),
            schedule: "0 10 * * *".to_string(),
            timeout: None,
            readonly: None,
            overlap_mode: None,
            max_queue_size: None,
            env: None,
            skills: None,
            silent_timeout: None,
            gpu: None,
        }];

        let merged = merge_agents_into_toml(existing_content, &agents_to_add).unwrap();

        assert!(merged.contains("[[agent]]"));
        assert!(!merged.contains("[[agents]]"));
        assert!(merged.contains("name = \"existing\""));
        assert!(merged.contains("name = \"added\""));
    }
}
