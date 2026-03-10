//! Implements the `switchboard skills installed` command.
//!
//! This module provides functionality to list all currently installed skills
//! in both project and global scopes, showing skill name, description,
//! and which agents have each skill assigned.

use crate::config::Config;
use crate::skills::{
    get_agents_using_skill, read_lockfile, scan_global_skills, scan_project_skills, LockfileStruct,
    SkillLockEntry, SkillMetadata, SkillsError,
};
use comfy_table::{Attribute, Cell, Table};
use std::path::PathBuf;

use super::{ExitCode, SkillsInstalled};

/// Run the `switchboard skills installed` command
///
/// Lists all currently installed skills in both project and global scopes.
/// Shows skill name, description, and which agents have each skill assigned.
///
/// # Arguments
///
/// * `args` - The command arguments containing the `--global` flag
/// * `config` - The switchboard configuration for agent assignment lookup
///
/// # Returns
///
/// * `ExitCode::Success` - If skills were listed successfully
/// * `ExitCode::Error` - If there was an error scanning skills directories
///
/// # Behavior
///
/// - When `--global` flag is set, only shows global skills from `./skills/`
/// - Without `--global` flag, shows both project skills (`./skills/`) and global skills
/// - Displays skill name, description (truncated if too long), and agent assignments
/// - Shows a summary count of total skills, separated by project and global counts
///
/// # Examples
///
/// List all installed skills (project + global):
/// ```text
/// switchboard skills installed
/// ```
///
/// List only global skills:
/// ```text
/// switchboard skills installed --global
/// ```
pub async fn run_skills_installed(args: SkillsInstalled, config: &Config) -> ExitCode {
    // Scan for skills based on the --global flag
    let mut project_skills = Vec::new();
    let mut warnings = Vec::new();

    if !args.global {
        match scan_project_skills() {
            Ok((skills, scan_warnings)) => {
                project_skills = skills;
                warnings.extend(scan_warnings);
            }
            Err(e) => {
                eprintln!("Error scanning project skills: {}", e);
                return ExitCode::Error;
            }
        }
    }

    let global_skills = match scan_global_skills() {
        Ok((skills, scan_warnings)) => {
            warnings.extend(scan_warnings);
            skills
        }
        Err(e) => {
            eprintln!("Error scanning global skills: {}", e);
            return ExitCode::Error;
        }
    };

    // Load the lockfile for cross-referencing
    let lockfile = match read_lockfile(&PathBuf::from("./skills")) {
        Ok(lf) => Some(lf),
        Err(SkillsError::LockfileNotFound { .. }) => {
            // No lockfile exists yet, that's okay
            None
        }
        Err(e) => {
            eprintln!("Warning: Could not load lockfile: {}", e);
            None
        }
    };

    // Generate warnings for skills that exist on disk but are not in the lockfile
    if let Some(ref lf) = lockfile {
        for skill in &project_skills {
            if !lf.skills.contains_key(&skill.name) {
                warnings.push(format!(
                    "Warning: Skill '{}' exists on disk but is not in lockfile",
                    skill.name
                ));
            }
        }
        for skill in &global_skills {
            if !lf.skills.contains_key(&skill.name) {
                warnings.push(format!(
                    "Warning: Skill '{}' exists on disk but is not in lockfile",
                    skill.name
                ));
            }
        }
    } else if project_skills.is_empty() && global_skills.is_empty() {
        // No lockfile and no skills, no warnings needed
    } else {
        // Lockfile doesn't exist but skills exist on disk
        for skill in &project_skills {
            warnings.push(format!(
                "Warning: Skill '{}' exists on disk but is not in lockfile",
                skill.name
            ));
        }
        for skill in &global_skills {
            warnings.push(format!(
                "Warning: Skill '{}' exists on disk but is not in lockfile",
                skill.name
            ));
        }
    }

    // Format and display the output
    let output = format_skills_list(
        project_skills,
        global_skills,
        &warnings,
        config,
        lockfile.as_ref(),
    );
    println!("{}", output);

    ExitCode::Success
}

/// Formats the list of installed skills with sections for project and global skills
///
/// This function formats a display of all installed skills, separating them into
/// project and global scopes. Each skill shows its name, description (truncated if long),
/// source, installed_at timestamp, and which agents have it assigned.
///
/// # Arguments
///
/// * `project_skills` - Vector of project-level skill metadata
/// * `global_skills` - Vector of global skill metadata
/// * `warnings` - Vector of warning messages collected during skill scanning
/// * `config` - The switchboard configuration for agent assignment lookup
/// * `lockfile` - Optional reference to the lockfile for cross-referencing
///
/// # Returns
///
/// A formatted string ready to be printed to stdout.
pub fn format_skills_list(
    project_skills: Vec<SkillMetadata>,
    global_skills: Vec<SkillMetadata>,
    warnings: &[String],
    config: &Config,
    lockfile: Option<&LockfileStruct>,
) -> String {
    let mut output = String::new();

    // Check if we have any skills
    let total_skills = project_skills.len() + global_skills.len();

    if total_skills == 0 {
        // Empty state
        output.push_str("No skills installed\n\n");
        output.push_str("  Browse available skills with: switchboard skills list\n");
        output.push_str("  Install a skill with: switchboard skills install <source>\n");
        output.push('\n');
    } else {
        // Build table with results
        let mut table = Table::new();
        table
            .load_preset(comfy_table::presets::UTF8_FULL)
            .set_header(vec![
                Cell::new("Name").add_attribute(Attribute::Bold),
                Cell::new("Description").add_attribute(Attribute::Bold),
                Cell::new("Source").add_attribute(Attribute::Bold),
                Cell::new("Installed At").add_attribute(Attribute::Bold),
                Cell::new("Agents").add_attribute(Attribute::Bold),
            ]);

        // Project skills section (displayed first as they take precedence over global skills)
        if !project_skills.is_empty() {
            output.push_str("Project (./skills/)\n");
            for skill in &project_skills {
                let lockfile_entry = lockfile.and_then(|lf| lf.skills.get(&skill.name));
                let (source, installed_at, name_with_scope) =
                    format_skill_entry_table(skill, lockfile_entry, "project");
                table.add_row(vec![
                    Cell::new(&name_with_scope),
                    Cell::new(&skill.description.clone().unwrap_or_else(|| "<no description>".to_string())),
                    Cell::new(&source),
                    Cell::new(&installed_at),
                    Cell::new(&get_agent_assignment_display(&skill.name, config)),
                ]);
            }
            output.push_str(&table.to_string());
            output.push('\n');
            // Reset table for global skills
            table = Table::new();
            table
                .load_preset(comfy_table::presets::UTF8_FULL)
                .set_header(vec![
                    Cell::new("Name").add_attribute(Attribute::Bold),
                    Cell::new("Description").add_attribute(Attribute::Bold),
                    Cell::new("Source").add_attribute(Attribute::Bold),
                    Cell::new("Installed At").add_attribute(Attribute::Bold),
                    Cell::new("Agents").add_attribute(Attribute::Bold),
                ]);
        }

        // Global skills section (displayed after project skills with visual separation)
        if !global_skills.is_empty() {
            output.push_str("Global (./skills/)\n");
            for skill in &global_skills {
                let lockfile_entry = lockfile.and_then(|lf| lf.skills.get(&skill.name));
                let (source, installed_at, name_with_scope) =
                    format_skill_entry_table(skill, lockfile_entry, "global");
                table.add_row(vec![
                    Cell::new(&name_with_scope),
                    Cell::new(&skill.description.clone().unwrap_or_else(|| "<no description>".to_string())),
                    Cell::new(&source),
                    Cell::new(&installed_at),
                    Cell::new(&get_agent_assignment_display(&skill.name, config)),
                ]);
            }
            output.push_str(&table.to_string());
            output.push('\n');
        }

        // Count summary (only show if there are skills)
        if total_skills > 0 {
            output.push_str(&format!(
                "{} skills installed ({} project, {} global)\n",
                total_skills,
                project_skills.len(),
                global_skills.len()
            ));
        }
    }

    // Warnings section (only display if there are warnings)
    if !warnings.is_empty() {
        output.push('\n');
        output.push_str("Warnings:\n");
        for warning in warnings {
            output.push_str(&format!("  {}\n", warning));
        }
    }

    output
}

/// Formats a single skill entry with name, description, source, installed_at, and agent assignments
/// Returns (source, installed_at, name_with_scope) for use with comfy_table
///
/// # Arguments
///
/// * `skill` - The skill metadata to format
/// * `lockfile_entry` - Optional reference to the lockfile entry for this skill
/// * `scope` - The scope of the skill ("project" or "global")
///
/// # Returns
///
/// A tuple of (source, installed_at, name_with_scope) strings.
pub fn format_skill_entry_table(
    skill: &SkillMetadata,
    lockfile_entry: Option<&SkillLockEntry>,
    scope: &str,
) -> (String, String, String) {
    let name_with_scope = format!("[{}] {}", scope, skill.name);

    // Get source and installed_at from lockfile entry, or show "Not in lockfile"
    let (source, installed_at) = match lockfile_entry {
        Some(entry) => (entry.source.clone(), entry.installed_at.clone()),
        None => ("Not in lockfile".to_string(), "-".to_string()),
    };

    (source, installed_at, name_with_scope)
}

/// Gets a display string showing which agents have a skill assigned
///
/// # Arguments
///
/// * `skill_name` - The name of the skill to look up
/// * `config` - The switchboard configuration
///
/// # Returns
///
/// A string representing the agent assignments:
/// - "[all agents]" if every agent references the skill
/// - Comma-separated list of agent names if some agents use it
/// - "[none]" if no agents reference the skill
pub fn get_agent_assignment_display(skill_name: &str, config: &Config) -> String {
    // Get all agents using this skill
    let agents = get_agents_using_skill(skill_name, config);

    if agents.is_empty() {
        return "[none]".to_string();
    }

    // Edge case: no agents configured in config, so cannot determine if "all agents"
    if config.agents.is_empty() {
        return "[none]".to_string();
    }

    // Check if all agents use this skill (saves space in display vs listing all agent names)
    if agents.len() == config.agents.len() {
        return "[all agents]".to_string();
    }

    // Return comma-separated list for partial assignment
    agents.join(", ")
}
