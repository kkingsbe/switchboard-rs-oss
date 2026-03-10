//! Handler for the `workflows list` subcommand.
//!
//! This module provides the `run_workflows_list` function which lists available
//! workflows from the kkingsbe/switchboard-workflows repository.

use crate::config::Config;
use crate::workflows::github::GitHubClient;
use crate::workflows::WorkflowsError;
use comfy_table::{Attribute, Cell, Table};

use super::types::WorkflowsList;
use super::ExitCode;

/// Run the `switchboard workflows list` command
///
/// This command uses the GitHub API to list available workflows from the
/// switchboard-workflows repository. If --search is provided, filters results
/// by name (case-insensitive). If --limit is provided, limits the number of results.
pub async fn run_workflows_list(args: WorkflowsList, _config: &Config) -> ExitCode {
    // Create GitHub client
    let client = GitHubClient::new();

    // Get list of workflow names from GitHub
    let workflow_names = match client.list_workflows().await {
        Ok(names) => names,
        Err(e) => {
            eprintln!("Failed to fetch workflows: {}", e);
            return ExitCode::Error;
        }
    };

    // If no workflows found, print message and return success
    if workflow_names.is_empty() {
        println!("No workflows found");
        return ExitCode::Success;
    }

    // Fetch info for each workflow
    let mut workflows = Vec::new();
    for name in workflow_names {
        match client.get_workflow_info(&name).await {
            Ok(info) => workflows.push(info),
            Err(WorkflowsError::NotFound(_)) => {
                // Skip workflows that don't have info
            }
            Err(e) => {
                eprintln!("Warning: Failed to get info for workflow '{}': {}", name, e);
            }
        }
    }

    // Filter by search query if provided (case-insensitive)
    if let Some(ref query) = args.search {
        let query_lower = query.to_lowercase();
        workflows.retain(|w| w.name.to_lowercase().contains(&query_lower));
    }

    // Limit results if limit is provided
    if let Some(limit) = args.limit {
        let limit_usize = limit as usize;
        if workflows.len() > limit_usize {
            workflows.truncate(limit_usize);
        }
    }

    // If no workflows match criteria, print message
    if workflows.is_empty() {
        println!("No workflows found");
        return ExitCode::Success;
    }

    // Build table with results
    let mut table = Table::new();
    table
        .load_preset(comfy_table::presets::UTF8_FULL)
        .set_header(vec![
            Cell::new("Name").add_attribute(Attribute::Bold),
            Cell::new("Description").add_attribute(Attribute::Bold),
            Cell::new("Prompts").add_attribute(Attribute::Bold),
        ]);

    for workflow in workflows {
        // Get first line of description, truncated if too long
        let description = workflow
            .description
            .map(|desc| {
                let first_line = desc.lines().next().unwrap_or("");
                if first_line.len() > 60 {
                    format!("{}...", &first_line[..57])
                } else {
                    first_line.to_string()
                }
            })
            .unwrap_or_else(|| "".to_string());

        // Count prompts
        let prompts_count = workflow.prompts.len().to_string();

        table.add_row(vec![
            Cell::new(&workflow.name),
            Cell::new(&description),
            Cell::new(&prompts_count),
        ]);
    }

    println!("{}", table);
    ExitCode::Success
}
