//! Handler for the `skills list` subcommand.
//!
//! This module provides the `run_skills_list` function which lists available
//! skills from the skills.sh registry.

use crate::config::Config;
use crate::skills::{
    skills_sh_search, NPX_NOT_FOUND_ERROR, SkillsManager,
};
use comfy_table::{Attribute, Cell, Table};

use super::types::SkillsList;
use super::ExitCode;

/// Run the `switchboard skills list` command
///
/// This command uses the skills.sh API to list available skills.
/// If --search or positional query is provided, passes the query to filter results.
pub async fn run_skills_list(args: SkillsList, _config: &Config) -> ExitCode {
    // Check if npx is available before invoking the command
    let mut skills_manager = SkillsManager::new(None);
    if skills_manager.check_npx_available().is_err() {
        eprintln!("{}", NPX_NOT_FOUND_ERROR);
        return ExitCode::Error;
    }

    // Use search query if provided, otherwise use positional query
    // Default to "ai" to get popular skills if neither is provided
    // Note: Query must be at least 2 characters per requirements
    let query = args
        .search
        .unwrap_or_else(|| args.query.unwrap_or_else(|| "ai".to_string()));

    // Validate query length (minimum 2 characters per requirements)
    if query.len() < 2 {
        eprintln!("Query must be at least 2 characters");
        return ExitCode::Error;
    }

    // Per requirements: default limit is 10, not 20
    let limit = args.limit.unwrap_or(10);

    // Call the skills.sh API
    match skills_sh_search(&query, Some(limit)).await {
        Ok(results) => {
            if results.is_empty() {
                println!("No skills found");
                return ExitCode::Success;
            }

            // Build table with results
            let mut table = Table::new();
            table
                .load_preset(comfy_table::presets::UTF8_FULL)
                .set_header(vec![
                    Cell::new("Name").add_attribute(Attribute::Bold),
                    Cell::new("ID").add_attribute(Attribute::Bold),
                    Cell::new("Installs").add_attribute(Attribute::Bold),
                    Cell::new("Source").add_attribute(Attribute::Bold),
                ]);

            for skill in results {
                // Display installable name as source@name (e.g., apollographql/skills@rust-best-practices)
                // This shows users the correct format to use with switchboard skills install
                let installable_name = format!("{}@{}", skill.source, skill.name);
                table.add_row(vec![
                    Cell::new(&installable_name),
                    Cell::new(&skill.id),
                    Cell::new(skill.installs.to_string()),
                    Cell::new(&skill.source),
                ]);
            }

            println!("{}", table);
            ExitCode::Success
        }
        Err(e) => {
            eprintln!("Failed to search skills: {}", e);
            ExitCode::Error
        }
    }
}
