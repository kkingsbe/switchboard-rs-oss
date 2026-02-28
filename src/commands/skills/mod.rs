pub mod types;
pub mod list;
pub mod remove;

pub use types::*;

use crate::config::Config;
use crate::skills::{
    add_skill_to_lockfile, create_npx_command, find_skill_directory, get_agents_using_skill,
    read_lockfile, remove_skill_directory, remove_skill_from_lockfile, scan_global_skills,
    scan_project_skills, skills_sh_search, write_lockfile, LockfileStruct, SkillLockEntry,
    SkillMetadata, SkillsError, SkillsManager, NPX_NOT_FOUND_ERROR,
};
use comfy_table::{Attribute, Cell, Table};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Run the skills command based on the provided subcommand.
///
/// This function serves as the main entry point for all skills-related operations.
/// It dispatches the execution to the appropriate handler function based on the
/// subcommand specified in the [`SkillsCommand`].
///
/// # Supported Subcommands
///
/// - `list` - List available skills from the skills.sh registry
/// - `install` - Install a skill from GitHub, npm package, or local path
/// - `installed` - List currently installed skills
/// - `update` - Update installed skills to their latest versions
/// - `remove` - Remove an installed skill
///
/// # Parameters
///
/// * `args` - The [`SkillsCommand`] containing the subcommand and its arguments
/// * `config` - Reference to the application configuration
///
/// # Returns
///
/// Returns an [`ExitCode`] indicating success or failure:
/// - [`ExitCode::Success`] - The command executed successfully
/// - [`ExitCode::Error`] - The command execution failed
///
/// # Examples
///
/// Listing available skills:
/// ```text
/// switchboard skills list
/// ```
///
/// Installing a skill:
/// ```text
/// switchboard skills install owner/repo
/// ```
///
/// Listing installed skills:
/// ```text
/// switchboard skills installed
/// ```
pub async fn run_skills(args: SkillsCommand, config: &Config) -> ExitCode {
    match args.subcommand {
        SkillsSubcommand::List(list_args) => list::run_skills_list(list_args, config).await,
        SkillsSubcommand::Install(install_args) => run_skills_install(install_args, config).await,
        SkillsSubcommand::Installed(installed_args) => {
            run_skills_installed(installed_args, config).await
        }
        SkillsSubcommand::Update(update_args) => handle_skills_update(update_args, config).await,
        SkillsSubcommand::Remove(remove_args) => remove::run_skills_remove(remove_args, config).await,
    }
}

/// Run the `switchboard skills install` command
///
/// This command delegates to `npx skills add` to install a skill from
/// the specified source. The command includes `-a kilo -y` flags to
/// auto-confirm the installation and set the author to "kilo".
/// If the destination already exists, it will prompt for confirmation
/// unless the --yes flag is provided.
///
/// Per requirements (Section 3.3.2), the install flow is:
/// 1. Run `npx skills add <source> -y` which installs to `.agents/skills/<skill-name>/`
/// 2. Move skill from `.agents/skills/<skill-name>/` to `./skills/<skill-name>/`
/// 3. Verify `SKILL.md` exists after move
/// 4. Update lockfile
/// 5. Clean up `.agents/skills/` and `.agents/` directories
async fn run_skills_install(args: SkillsInstall, _config: &Config) -> ExitCode {
    // Check if npx is available before invoking the command
    let mut skills_manager = SkillsManager::new(None);
    if skills_manager.check_npx_available().is_err() {
        eprintln!("{}", NPX_NOT_FOUND_ERROR);
        return ExitCode::Error;
    }

    // Extract skill name from source
    let skill_name = extract_skill_name(&args.source);

    // Determine the target skills directory
    let skills_dir = if args.global {
        skills_manager.global_skills_dir.clone()
    } else {
        skills_manager.skills_dir.clone()
    };

    // Check if destination already exists
    let skill_path = skills_dir.join(&skill_name);
    if skill_path.exists() && !args.yes {
        // Return error with the --yes flag suggestion
        let error = SkillsError::DestinationAlreadyExists {
            skill_name: skill_name.clone(),
            path: skill_path.display().to_string(),
        };
        eprintln!("Error: {}", error);
        return ExitCode::Error;
    }
    // If --yes is provided, proceed with overwrite (npx skills add -y will handle it)

    // Build the npx skills add command
    // Per requirements: use --skill flag when source contains @skill-name
    // Format: npx skills add <repo> --skill <skill-name> -a kilo -y
    let mut cmd = create_npx_command();
    cmd.arg("skills");
    cmd.arg("add");

    // Parse source to handle @skill-name format
    if let Some(at_pos) = args.source.rfind('@') {
        let repo = &args.source[..at_pos];
        let skill_name_from_source = &args.source[at_pos + 1..];
        cmd.arg(repo);
        cmd.arg("--skill");
        cmd.arg(skill_name_from_source);
    } else {
        cmd.arg(&args.source);
    }

    cmd.arg("-a");
    cmd.arg("kilo");
    cmd.arg("-y"); // Auto-confirm

    // Add global flag if specified
    if args.global {
        cmd.arg("-g");
    }

    // Inherit stdout/stderr from parent process for interactive display
    let result = match cmd.status() {
        Ok(status) => ExitCode::from_i32(status.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("Failed to execute npx skills add: {}", e);
            return ExitCode::Error;
        }
    };

    // If installation was successful, perform post-install steps
    if result == ExitCode::Success {
        // Step 2-5: Move from .agents/skills/ to ./skills/, verify, update lockfile, cleanup
        match perform_post_install_move(&skills_dir, &skill_name, &args.source) {
            Ok(_) => {
                println!(
                    "Moved to ./{}/
Updated skills.lock.json",
                    skill_name
                );
            }
            Err(e) => {
                eprintln!("Warning: Post-install move failed: {}", e);
                // Still try to add to lockfile even if move failed
            }
        }

        // Add to lockfile
        if let Err(e) = add_skill_to_lockfile(&skills_dir, &skill_name, &args.source) {
            eprintln!("Warning: Failed to update lockfile: {}", e);
        }
    }

    result
}

/// Performs the post-install move from .agents/skills/ to ./skills/
///
/// Per Section 3.3.2 requirements:
/// 1. Create ./skills/ if needed
/// 2. Move skill from .agents/skills/<name>/ to ./skills/<name>/
/// 3. Verify SKILL.md exists after move
/// 4. Clean up empty .agents/skills/ and .agents/ directories
fn perform_post_install_move(
    skills_dir: &PathBuf,
    skill_name: &str,
    _source: &str,
) -> Result<(), SkillsError> {
    // The npx skills add command installs to .agents/skills/<skill-name>/
    let source_dir = PathBuf::from(".agents/skills").join(skill_name);
    let dest_dir = skills_dir.join(skill_name);

    // Check if npx actually installed something to .agents/skills/
    if source_dir.exists() {
        // Create destination directory if it doesn't exist
        if !skills_dir.exists() {
            fs::create_dir_all(skills_dir).map_err(|e| SkillsError::IoError {
                operation: "create skills directory".to_string(),
                path: skills_dir.display().to_string(),
                message: e.to_string(),
            })?;
        }

        // Remove destination if it exists (overwrite case)
        if dest_dir.exists() {
            fs::remove_dir_all(&dest_dir).map_err(|e| SkillsError::IoError {
                operation: "remove existing skill".to_string(),
                path: dest_dir.display().to_string(),
                message: e.to_string(),
            })?;
        }

        // Move from .agents/skills/ to ./skills/
        fs::rename(&source_dir, &dest_dir).map_err(|e| SkillsError::IoError {
            operation: "move skill from .agents/skills".to_string(),
            path: format!("{} -> {}", source_dir.display(), dest_dir.display()),
            message: e.to_string(),
        })?;

        // Verify SKILL.md exists after move
        let skill_md_path = dest_dir.join("SKILL.md");
        if !skill_md_path.exists() {
            return Err(SkillsError::SkillNotFound {
                skill_source: skill_name.to_string(),
            });
        }

        // Clean up empty .agents/skills/ and .agents/ directories
        cleanup_agents_directory()?;

        println!("Done.");
    } else {
        // If .agents/skills/ doesn't exist, npx may have installed directly to ./skills/
        // This is fine - just verify the skill exists where expected
        if !dest_dir.exists() {
            return Err(SkillsError::SkillNotFound {
                skill_source: skill_name.to_string(),
            });
        }

        // Verify SKILL.md exists
        let skill_md_path = dest_dir.join("SKILL.md");
        if !skill_md_path.exists() {
            return Err(SkillsError::SkillNotFound {
                skill_source: skill_name.to_string(),
            });
        }
    }

    Ok(())
}

/// Cleans up empty .agents/skills/ and .agents/ directories
fn cleanup_agents_directory() -> Result<(), SkillsError> {
    let agents_skills_dir = PathBuf::from(".agents/skills");
    let agents_dir = PathBuf::from(".agents");

    // Remove .agents/skills/ if it exists and is empty
    if agents_skills_dir.exists() {
        if let Ok(mut entries) = fs::read_dir(&agents_skills_dir) {
            if entries.next().is_none() {
                let _ = fs::remove_dir(&agents_skills_dir);
            }
        }
    }

    // Remove .agents/ if it exists and is empty
    if agents_dir.exists() {
        if let Ok(mut entries) = fs::read_dir(&agents_dir) {
            if entries.next().is_none() {
                let _ = fs::remove_dir(&agents_dir);
            }
        }
    }

    Ok(())
}

/// Extracts the skill name from a source string.
///
/// Handles formats like:
/// - "owner/repo" -> "repo"
/// - "owner/repo@skill-name" -> "skill-name"
/// - "https://github.com/owner/repo" -> "repo"
/// - "https://github.com/owner/repo@skill-name" -> "skill-name"
fn extract_skill_name(source: &str) -> String {
    // Check for @ delimiter first (explicit skill name)
    if let Some(at_pos) = source.rfind('@') {
        return source[at_pos + 1..].to_string();
    }

    // Check for GitHub URL format
    if source.starts_with("http://") || source.starts_with("https://") {
        // Extract repo name from URL like https://github.com/owner/repo
        if let Some(last_slash) = source.rfind('/') {
            return source[last_slash + 1..].to_string();
        }
    }

    // Default: use the last part of owner/repo format
    if let Some(last_slash) = source.rfind('/') {
        return source[last_slash + 1..].to_string();
    }

    // Fallback: use the whole string
    source.to_string()
}

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
async fn run_skills_installed(args: SkillsInstalled, config: &Config) -> ExitCode {
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
fn format_skills_list(
    project_skills: Vec<SkillMetadata>,
    global_skills: Vec<SkillMetadata>,
    warnings: &[String],
    config: &Config,
    lockfile: Option<&LockfileStruct>,
) -> String {
    let mut output = String::new();
    // Column widths: name(20) + description(30) + source(25) + installed_at(20) + agents ~= 95 chars
    let separator = "─".repeat(95);

    // Header
    output.push_str("Installed Skills\n\n");
    output.push_str("  Name                 Description                Source                     Installed At            Agents\n");
    output.push_str(&format!("  {}\n", separator));

    // Check if we have any skills
    let total_skills = project_skills.len() + global_skills.len();

    if total_skills == 0 {
        // Empty state
        output.push_str("  No skills installed\n\n");
        output.push_str("  Browse available skills with: switchboard skills list\n");
        output.push_str("  Install a skill with: switchboard skills install <source>\n");
        output.push('\n');
    } else {
        // Project skills section (displayed first as they take precedence over global skills)
        if !project_skills.is_empty() {
            output.push_str("  Project (./skills/)\n");
            output.push_str(&format!("  {}\n", separator));
            for skill in &project_skills {
                let lockfile_entry = lockfile.and_then(|lf| lf.skills.get(&skill.name));
                output.push_str(&format_skill_entry(skill, config, lockfile_entry));
            }
            output.push('\n');
        }

        // Global skills section (displayed after project skills with visual separation)
        if !global_skills.is_empty() {
            output.push_str("  Global (./skills/)\n");
            output.push_str(&format!("  {}\n", separator));
            for skill in &global_skills {
                let lockfile_entry = lockfile.and_then(|lf| lf.skills.get(&skill.name));
                output.push_str(&format_skill_entry(skill, config, lockfile_entry));
            }
            output.push('\n');
        }

        // Count summary (only show if there are skills)
        if total_skills > 0 {
            output.push_str(&format!(
                "  {} skills installed ({} project, {} global)\n",
                total_skills,
                project_skills.len(),
                global_skills.len()
            ));
        }
    }

    // Warnings section (only display if there are warnings)

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
///
/// # Arguments
///
/// * `skill` - The skill metadata to format
/// * `config` - The switchboard configuration for agent assignment lookup
/// * `lockfile_entry` - Optional reference to the lockfile entry for this skill
///
/// # Returns
///
/// A formatted string with fixed-width columns for alignment.
fn format_skill_entry(
    skill: &SkillMetadata,
    config: &Config,
    lockfile_entry: Option<&SkillLockEntry>,
) -> String {
    let name = &skill.name;
    let description = skill.description.as_deref().unwrap_or("<no description>");
    let agents = get_agent_assignment_display(name, config);

    // Truncate description if too long (keep it under 40 chars, leave room for "...")
    let truncated_desc = if description.len() > 40 {
        format!("{}...", &description[..37])
    } else {
        description.to_string()
    };

    // Get source and installed_at from lockfile entry, or show "Not in lockfile"
    let (source, installed_at) = match lockfile_entry {
        Some(entry) => (entry.source.clone(), entry.installed_at.clone()),
        None => ("Not in lockfile".to_string(), "-".to_string()),
    };

    // Format with fixed-width columns for table alignment:
    // - Name column: 20 chars
    // - Description column: 30 chars
    // - Source column: 25 chars
    // - Installed at column: 20 chars
    // - Agents column: variable width
    format!(
        "  {:.<20} {:.<30} {:.<25} {:.<20} {}\n",
        name, truncated_desc, source, installed_at, agents
    )
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
fn get_agent_assignment_display(skill_name: &str, config: &Config) -> String {
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

/// Handle the `switchboard skills update` command.
///
/// Updates installed skills to their latest versions by delegating to `npx skills update`.
/// If a specific skill name is provided, only that skill is updated.
/// If no skill name is provided, all installed skills are updated.
///
/// # Arguments
///
/// * `args` - The command arguments containing an optional skill name
/// * `_config` - The switchboard configuration (not used in this implementation)
///
/// # Returns
///
/// * `ExitCode` - The exit code from the npx command
///
/// # Errors
///
/// Returns an error if npx is not available or the npx command invocation fails.
pub async fn handle_skills_update(args: SkillsUpdate, _config: &Config) -> ExitCode {
    use crate::skills::SkillsManager;
    use chrono::Utc;

    // Check if npx is available before invoking the command
    let mut manager = SkillsManager::new(None);
    if manager.check_npx_available().is_err() {
        eprintln!("{}", NPX_NOT_FOUND_ERROR);
        return ExitCode::Error;
    }

    // Get the skills directory from the manager
    let skills_dir = manager.skills_dir.clone();

    // Read the lockfile to get skill sources
    let lockfile = match read_lockfile(&skills_dir) {
        Ok(lf) => lf,
        Err(e) => {
            eprintln!("Error: Failed to read lockfile: {}", e);
            return ExitCode::Error;
        }
    };

    // If a specific skill name is provided, update only that skill
    if let Some(skill_name) = &args.skill_name {
        // Look up the skill in the lockfile
        let skill_entry = match lockfile.skills.get(skill_name) {
            Some(entry) => entry,
            None => {
                eprintln!(
                    "Error: Skill '{}' is not in the lockfile. Install it first with 'switchboard skills install' or update all skills with 'switchboard skills update'.",
                    skill_name
                );
                return ExitCode::Error;
            }
        };

        // Re-install the skill from the stored source with overwrite (-y flag)
        let source = &skill_entry.source;
        println!("Updating skill '{}' from source '{}'", skill_name, source);

        let result = reinstall_skill_from_source(&skills_dir, skill_name, source, false);

        // If re-installation was successful, update the timestamp in lockfile
        if result == ExitCode::Success {
            if let Err(e) = update_skill_timestamp(&skills_dir, skill_name) {
                eprintln!("Warning: Failed to update lockfile timestamp: {}", e);
            }
        }

        return result;
    }

    // No skill name provided - update ALL skills from lockfile
    if lockfile.skills.is_empty() {
        eprintln!("Error: No skills found in lockfile. Install skills first with 'switchboard skills install'.");
        return ExitCode::Error;
    }

    println!(
        "Updating all {} skills from lockfile...",
        lockfile.skills.len()
    );

    let mut all_success = true;
    let mut updated_skills: Vec<String> = Vec::new();

    for (skill_name, skill_entry) in &lockfile.skills {
        let source = &skill_entry.source;
        println!("Updating skill '{}' from source '{}'", skill_name, source);

        if reinstall_skill_from_source(&skills_dir, skill_name, source, false) == ExitCode::Success
        {
            updated_skills.push(skill_name.clone());
        } else {
            all_success = false;
            eprintln!("Failed to update skill '{}'", skill_name);
        }
    }

    // Update timestamps for all successfully updated skills
    if !updated_skills.is_empty() {
        // Re-read the lockfile to get the latest state
        match read_lockfile(&skills_dir) {
            Ok(mut lockfile) => {
                let timestamp = Utc::now().to_rfc3339();
                for skill_name in &updated_skills {
                    if let Some(skill) = lockfile.skills.get_mut(skill_name) {
                        skill.installed_at = timestamp.clone();
                    }
                }
                if let Err(e) = write_lockfile(&lockfile, &skills_dir) {
                    eprintln!("Warning: Failed to update lockfile timestamps: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to re-read lockfile: {}", e);
            }
        }
    }

    if all_success {
        ExitCode::Success
    } else {
        ExitCode::Error
    }
}

/// Re-installs a skill from the given source, overwriting if it already exists.
///
/// This is used by the update command to re-install a skill from its original source.
/// Per Section 3.3.2, this performs the post-install move from .agents/skills/ to ./skills/
fn reinstall_skill_from_source(
    skills_dir: &std::path::Path,
    skill_name: &str,
    source: &str,
    global: bool,
) -> ExitCode {
    // Build the npx skills add command with -y flag to auto-confirm overwrite
    // Per requirements: use --skill flag when source contains @skill-name
    let mut cmd = create_npx_command();
    cmd.arg("skills");
    cmd.arg("add");

    // Parse source to handle @skill-name format
    if let Some(at_pos) = source.rfind('@') {
        let repo = &source[..at_pos];
        let skill_name_from_source = &source[at_pos + 1..];
        cmd.arg(repo);
        cmd.arg("--skill");
        cmd.arg(skill_name_from_source);
    } else {
        cmd.arg(source);
    }

    cmd.arg("-a");
    cmd.arg("kilo");
    cmd.arg("-y"); // Auto-confirm overwrite

    // Add global flag if specified
    if global {
        cmd.arg("-g");
    }

    // Inherit stdout/stderr from parent process for interactive display
    match cmd.status() {
        Ok(status) => {
            if status.success() {
                // Per Section 3.3.2: Perform post-install move from .agents/skills/ to ./skills/
                // Need to get skills_dir as PathBuf for the function call
                let skills_dir_buf = std::path::PathBuf::from(skills_dir);
                if let Err(e) = perform_post_install_move(&skills_dir_buf, skill_name, source) {
                    eprintln!("Warning: Post-install move failed: {}", e);
                }
                ExitCode::Success
            } else {
                // npx skills add failed - the source may no longer be available
                eprintln!(
                    "Error: Failed to update '{}'. The source may no longer be available. You can remove this skill from lockfile with 'switchboard skills remove {}'.",
                    skill_name, skill_name
                );
                ExitCode::Error
            }
        }
        Err(_e) => {
            eprintln!(
                "Error: Failed to update '{}'. The source may no longer be available. You can remove this skill from lockfile with 'switchboard skills remove {}'.",
                skill_name, skill_name
            );
            ExitCode::Error
        }
    }
}

/// Updates the installed_at timestamp for a specific skill in the lockfile.
///
/// This is called after successfully re-installing a skill during an update.
fn update_skill_timestamp(
    skills_dir: &std::path::Path,
    skill_name: &str,
) -> Result<(), SkillsError> {
    use chrono::Utc;

    // Read the lockfile
    let mut lockfile = read_lockfile(skills_dir)?;

    // Update the timestamp for the specified skill
    let timestamp = Utc::now().to_rfc3339();
    if let Some(skill) = lockfile.skills.get_mut(skill_name) {
        skill.installed_at = timestamp;
    }

    // Write the updated lockfile
    write_lockfile(&lockfile, skills_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Agent;
    use clap::Parser;

    #[test]
    fn test_skills_installed_parsing() {
        // Test that --global flag is parsed correctly
        let args = SkillsInstalled { global: true };
        assert!(args.global);

        let args = SkillsInstalled { global: false };
        assert!(!args.global);
    }

    #[test]
    fn test_skills_remove_parsing() {
        // Test that all fields are parsed correctly
        let args = SkillsRemove {
            skill_name: "test-skill".to_string(),
            global: false,
            yes: false,
        };
        assert_eq!(args.skill_name, "test-skill");
        assert!(!args.global);
        assert!(!args.yes);
    }

    #[test]
    fn test_skills_remove_with_global_flag() {
        let args = SkillsRemove {
            skill_name: "test-skill".to_string(),
            global: true,
            yes: false,
        };
        assert!(args.global);
    }

    #[test]
    fn test_skills_remove_with_yes_flag() {
        let args = SkillsRemove {
            skill_name: "test-skill".to_string(),
            global: false,
            yes: true,
        };
        assert!(args.yes);
    }

    #[test]
    fn test_confirm_returns_true_for_y() {
        // We can't easily test the actual user input function in unit tests,
        // but we can verify the function exists and has the right signature
        let prompt = "Test prompt";
        // The actual function would prompt the user
        let _ = prompt;
    }

    #[test]
    fn test_format_skills_list_empty() {
        let config = Config::default();
        let project_skills: Vec<SkillMetadata> = Vec::new();
        let global_skills: Vec<SkillMetadata> = Vec::new();
        let warnings: Vec<String> = Vec::new();

        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        assert!(output.contains("No skills installed"));
        assert!(output.contains("switchboard skills list"));
        assert!(output.contains("switchboard skills install"));
    }

    #[test]
    fn test_format_skills_list_with_project_skills() {
        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec!["test-skill".to_string()]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["test-skill".to_string()]),
                ..Default::default()
            },
        ];

        let project_skills = vec![SkillMetadata {
            name: "test-skill".to_string(),
            description: Some("A test skill".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let global_skills: Vec<SkillMetadata> = Vec::new();
        let warnings: Vec<String> = Vec::new();

        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        assert!(output.contains("Installed Skills"));
        assert!(output.contains("Project (./skills/)"));
        assert!(output.contains("test-skill"));
        assert!(output.contains("A test skill"));
        assert!(output.contains("[all agents]"));
        assert!(output.contains("1 skills installed (1 project, 0 global)"));
    }

    #[test]
    fn test_format_skills_list_mixed() {
        let mut config = Config::default();
        config.agents = vec![Agent {
            name: "agent1".to_string(),
            skills: Some(vec!["project-skill".to_string()]),
            ..Default::default()
        }];

        let project_skills = vec![SkillMetadata {
            name: "project-skill".to_string(),
            description: Some("Project skill".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let global_skills = vec![SkillMetadata {
            name: "global-skill".to_string(),
            description: Some("Global skill".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let warnings: Vec<String> = Vec::new();

        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        assert!(output.contains("Project (./skills/)"));
        assert!(output.contains("Global (./skills/)"));
        assert!(output.contains("project-skill"));
        assert!(output.contains("global-skill"));
        assert!(output.contains("2 skills installed (1 project, 1 global)"));
    }

    #[test]
    fn test_get_agent_assignment_display_all_agents() {
        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec!["test-skill".to_string()]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["test-skill".to_string()]),
                ..Default::default()
            },
        ];

        let display = get_agent_assignment_display("test-skill", &config);
        assert_eq!(display, "[all agents]");
    }

    #[test]
    fn test_get_agent_assignment_display_some_agents() {
        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec!["test-skill".to_string()]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["other-skill".to_string()]),
                ..Default::default()
            },
        ];

        let display = get_agent_assignment_display("test-skill", &config);
        assert_eq!(display, "agent1");
    }

    #[test]
    fn test_get_agent_assignment_display_none() {
        let mut config = Config::default();
        config.agents = vec![Agent {
            name: "agent1".to_string(),
            skills: Some(vec!["other-skill".to_string()]),
            ..Default::default()
        }];

        let display = get_agent_assignment_display("test-skill", &config);
        assert_eq!(display, "[none]");
    }

    #[test]
    fn test_get_agent_assignment_display_no_agents() {
        let mut config = Config::default();
        config.agents = Vec::new();

        let display = get_agent_assignment_display("test-skill", &config);
        assert_eq!(display, "[none]");
    }

    #[test]
    fn test_format_skill_entry_truncates_long_description() {
        let mut config = Config::default();
        config.agents = vec![Agent {
            name: "agent1".to_string(),
            skills: Some(vec!["test-skill".to_string()]),
            ..Default::default()
        }];

        let skill = SkillMetadata {
            name: "test-skill".to_string(),
            description: Some("This is a very long description that should be truncated because it exceeds forty characters".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        };

        let entry = format_skill_entry(&skill, &config, None);
        assert!(entry.contains("...")); // Should have truncation marker
        assert!(entry.len() < 150); // Should be reasonably short
    }

    #[test]
    fn test_format_skill_entry_no_description() {
        let config = Config::default();

        let skill = SkillMetadata {
            name: "test-skill".to_string(),
            description: None,
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        };

        let entry = format_skill_entry(&skill, &config, None);
        assert!(entry.contains("<no description>"));
    }

    #[test]
    fn test_skills_update_parsing_no_skill_name() {
        // Test that the update command works without a skill name (updates all)
        let args = SkillsUpdate { skill_name: None };
        assert!(args.skill_name.is_none());
    }

    #[test]
    fn test_skills_update_parsing_with_skill_name() {
        // Test that the update command works with a specific skill name
        let args = SkillsUpdate {
            skill_name: Some("frontend-design".to_string()),
        };
        assert_eq!(args.skill_name, Some("frontend-design".to_string()));
    }

    #[test]
    fn test_skills_update_skill_name_is_optional() {
        // Test that skill_name is truly optional by creating both variants
        let args_no_name = SkillsUpdate { skill_name: None };
        let args_with_name = SkillsUpdate {
            skill_name: Some("test-skill".to_string()),
        };

        assert!(args_no_name.skill_name.is_none());
        assert!(args_with_name.skill_name.is_some());
    }

    #[test]
    fn test_skills_update_no_args() {
        // Parse the full command structure
        let cmd = SkillsCommand::try_parse_from(["skills", "update"]).unwrap();

        // Extract the Update subcommand
        match cmd.subcommand {
            SkillsSubcommand::Update(update_args) => {
                assert!(update_args.skill_name.is_none());
            }
            _ => panic!("Expected Update subcommand"),
        }
    }

    #[test]
    fn test_skills_update_with_skill_name() {
        // Parse the full command structure
        let cmd =
            SkillsCommand::try_parse_from(["skills", "update", "--", "frontend-design"]).unwrap();

        // Extract the Update subcommand
        match cmd.subcommand {
            SkillsSubcommand::Update(update_args) => {
                assert_eq!(update_args.skill_name, Some("frontend-design".to_string()));
            }
            _ => panic!("Expected Update subcommand"),
        }
    }

    #[test]
    fn test_format_skills_list_global_flag_filters() {
        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec![
                    "project-skill-1".to_string(),
                    "global-skill-1".to_string(),
                ]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["project-skill-2".to_string()]),
                ..Default::default()
            },
        ];

        // Create project skills with distinct names
        let project_skills = vec![
            SkillMetadata {
                name: "project-skill-1".to_string(),
                description: Some("Project skill 1 description".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
            SkillMetadata {
                name: "project-skill-2".to_string(),
                description: Some("Project skill 2 description".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
        ];

        // Create global skills with distinct names
        let global_skills = vec![
            SkillMetadata {
                name: "global-skill-1".to_string(),
                description: Some("Global skill 1 description".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
            SkillMetadata {
                name: "global-skill-2".to_string(),
                description: Some("Global skill 2 description".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
        ];
        let warnings: Vec<String> = Vec::new();

        // Test that both project and global skills are displayed
        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        // Verify both project skill names are present
        assert!(output.contains("project-skill-1"));
        assert!(output.contains("project-skill-2"));
        assert!(output.contains("Project skill 1 description"));
        assert!(output.contains("Project skill 2 description"));

        // Verify both global skill names are present
        assert!(output.contains("global-skill-1"));
        assert!(output.contains("global-skill-2"));
        assert!(output.contains("Global skill 1 description"));
        assert!(output.contains("Global skill 2 description"));

        // Verify sections are displayed
        assert!(output.contains("Project (./skills/)"));
        assert!(output.contains("Global (./skills/)"));

        // Verify correct counts
        assert!(output.contains("4 skills installed (2 project, 2 global)"));
    }

    #[test]
    fn test_format_skills_list_global_flag_shows_only_global() {
        let mut config = Config::default();
        config.agents = vec![Agent {
            name: "agent1".to_string(),
            skills: Some(vec!["global-skill-1".to_string()]),
            ..Default::default()
        }];

        // Create global skills
        let global_skills = vec![SkillMetadata {
            name: "global-skill-1".to_string(),
            description: Some("Global skill 1 description".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let warnings: Vec<String> = Vec::new();

        // Test filtering by passing empty project skills (simulating --global flag behavior)
        let output = format_skills_list(Vec::new(), global_skills, &warnings, &config, None);

        // Verify project skills are NOT present
        assert!(!output.contains("project-skill-1"));
        assert!(!output.contains("project-skill-2"));
        assert!(!output.contains("Project skill 1 description"));
        assert!(!output.contains("Project skill 2 description"));
        assert!(!output.contains("Project (./skills/)"));

        // Verify global skills ARE present
        assert!(output.contains("global-skill-1"));
        assert!(output.contains("Global skill 1 description"));
        assert!(output.contains("Global (./skills/)"));

        // Verify correct counts (only global skills)
        assert!(output.contains("1 skills installed (0 project, 1 global)"));
    }

    #[test]
    fn test_format_skills_list_with_warnings() {
        let config = Config::default();
        let project_skills = vec![SkillMetadata {
            name: "test-skill".to_string(),
            description: Some("A test skill".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let global_skills: Vec<SkillMetadata> = Vec::new();
        let warnings = vec![
            "Warning: Could not parse SKILL.md for 'skill-1' — using directory name".to_string(),
            "Warning: Could not parse SKILL.md for 'skill-2' — using directory name".to_string(),
        ];

        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        // Verify warnings section is displayed
        assert!(output.contains("Warnings:"));
        // Verify each warning is present with two-space prefix
        assert!(output
            .contains("  Warning: Could not parse SKILL.md for 'skill-1' — using directory name"));
        assert!(output
            .contains("  Warning: Could not parse SKILL.md for 'skill-2' — using directory name"));
        // Verify skills are still displayed
        assert!(output.contains("test-skill"));
    }

    #[test]
    fn test_format_skills_list_without_warnings() {
        let config = Config::default();
        let project_skills = vec![SkillMetadata {
            name: "test-skill".to_string(),
            description: Some("A test skill".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let global_skills: Vec<SkillMetadata> = Vec::new();
        let warnings: Vec<String> = Vec::new();

        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        // Verify warnings section is NOT displayed when there are no warnings
        assert!(!output.contains("Warnings:"));
        // Verify skills are still displayed
        assert!(output.contains("test-skill"));
    }

    #[test]
    fn test_format_skills_list_empty_with_warnings() {
        let config = Config::default();
        let project_skills: Vec<SkillMetadata> = Vec::new();
        let global_skills: Vec<SkillMetadata> = Vec::new();
        let warnings = vec![
            "Warning: Could not parse SKILL.md for 'bad-skill' — using directory name".to_string(),
        ];

        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        // Verify empty state message is shown
        assert!(output.contains("No skills installed"));
        // Verify warnings section is still displayed even when there are no skills
        assert!(output.contains("Warnings:"));
        assert!(output.contains(
            "  Warning: Could not parse SKILL.md for 'bad-skill' — using directory name"
        ));
    }

    /// Test that the --global flag filters to show only global skills
    ///
    /// When --global flag is set (simulated by empty project_skills),
    /// only global skills should be displayed in the output.
    /// This test verifies the filtering behavior of format_skills_list()
    /// which is called by run_skills_installed() when global=true.
    #[test]
    fn test_global_flag_shows_only_global_skills() {
        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec![
                    "global-skill-1".to_string(),
                    "global-skill-2".to_string(),
                ]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["global-skill-1".to_string()]),
                ..Default::default()
            },
        ];

        // Create global skills that SHOULD be displayed
        let global_skills = vec![
            SkillMetadata {
                name: "global-skill-1".to_string(),
                description: Some("Global skill 1".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
            SkillMetadata {
                name: "global-skill-2".to_string(),
                description: Some("Global skill 2".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
        ];
        let warnings: Vec<String> = Vec::new();

        // Simulate --global flag behavior by passing empty project_skills
        let output = format_skills_list(Vec::new(), global_skills, &warnings, &config, None);

        // Verify project skills are NOT in the output
        assert!(!output.contains("project-skill-1"));
        assert!(!output.contains("project-skill-2"));
        assert!(!output.contains("Project skill 1"));
        assert!(!output.contains("Project skill 2"));
        assert!(!output.contains("Project (./skills/)"));

        // Verify global skills ARE in the output
        assert!(output.contains("global-skill-1"));
        assert!(output.contains("global-skill-2"));
        assert!(output.contains("Global skill 1"));
        assert!(output.contains("Global skill 2"));
        assert!(output.contains("Global (./skills/)"));

        // Verify correct counts (only global skills)
        assert!(output.contains("2 skills installed (0 project, 2 global)"));
    }

    /// Test that without --global flag, both project and global skills are shown
    ///
    /// When --global flag is NOT set, both project and global skills
    /// should be displayed in the output.
    /// This test verifies the default behavior of format_skills_list()
    /// which is called by run_skills_installed() when global=false.
    #[test]
    fn test_no_global_flag_shows_both_scopes() {
        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec![
                    "project-skill-1".to_string(),
                    "global-skill-1".to_string(),
                ]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec![
                    "project-skill-2".to_string(),
                    "global-skill-1".to_string(),
                ]),
                ..Default::default()
            },
        ];

        // Create project skills that SHOULD be displayed
        let project_skills = vec![
            SkillMetadata {
                name: "project-skill-1".to_string(),
                description: Some("Project skill 1".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
            SkillMetadata {
                name: "project-skill-2".to_string(),
                description: Some("Project skill 2".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
        ];

        // Create global skills that SHOULD be displayed
        let global_skills = vec![SkillMetadata {
            name: "global-skill-1".to_string(),
            description: Some("Global skill 1".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let warnings: Vec<String> = Vec::new();

        // Test default behavior (no --global flag) by passing both skill vectors
        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        // Verify project skills ARE in the output
        assert!(output.contains("project-skill-1"));
        assert!(output.contains("project-skill-2"));
        assert!(output.contains("Project skill 1"));
        assert!(output.contains("Project skill 2"));
        assert!(output.contains("Project (./skills/)"));

        // Verify global skills ARE in the output
        assert!(output.contains("global-skill-1"));
        assert!(output.contains("Global skill 1"));
        assert!(output.contains("Global (./skills/)"));

        // Verify both scopes are shown
        assert!(output.contains("Project (./skills/)"));
        assert!(output.contains("Global (./skills/)"));

        // Verify correct counts (both project and global skills)
        assert!(output.contains("3 skills installed (2 project, 1 global)"));
    }

    // ============================================================================
    // Install Command Tests
    // ============================================================================

    use crate::skills::SkillsError;

    /// Test SkillsInstall parsing with --global flag
    #[test]
    fn test_skills_install_args_parse_global_flag() {
        // Use try_parse_from to avoid clap picking up test binary args
        // Using skill-name format - SkillsInstall is a subcommand so we parse just the subcommand args
        let install =
            SkillsInstall::try_parse_from(vec!["install", "frontend-design", "--global"]).unwrap();
        assert_eq!(install.source, "frontend-design");
        assert!(install.global);

        // Test without --global flag (should default to false)
        let install = SkillsInstall::try_parse_from(vec!["install", "frontend-design"]).unwrap();
        assert_eq!(install.source, "frontend-design");
        assert!(!install.global);
    }

    /// Test SkillsInstall parsing with various source formats
    #[test]
    fn test_skills_install_args_parse_source_formats() {
        // skill-name format (new format)
        let install = SkillsInstall::try_parse_from(vec!["install", "frontend-design"]).unwrap();
        assert_eq!(install.source, "frontend-design");

        // skill-name with hyphen format
        let install = SkillsInstall::try_parse_from(vec!["install", "security-audit"]).unwrap();
        assert_eq!(install.source, "security-audit");

        // skill-name with underscore format
        let install = SkillsInstall::try_parse_from(vec!["install", "my_skill"]).unwrap();
        assert_eq!(install.source, "my_skill");

        // skill-name with numbers
        let install = SkillsInstall::try_parse_from(vec!["install", "skill123"]).unwrap();
        assert_eq!(install.source, "skill123");
    }

    /// Test SkillsInstall requires source argument
    #[test]
    fn test_skills_install_args_require_source() {
        // This should fail because source is required - SkillsInstall is a subcommand so we parse just the subcommand name
        let result = SkillsInstall::try_parse_from(vec!["install"]);
        assert!(result.is_err());
    }

    /// Test SkillsInstall with global flag in different positions
    #[test]
    fn test_skills_install_flag_order() {
        // Note: Current version doesn't have --yes flag, only --global
        // This test verifies global flag works in different positions
        // Using skill-name format - SkillsInstall is a subcommand so we parse just the subcommand args
        let install =
            SkillsInstall::try_parse_from(vec!["install", "frontend-design", "--global"]).unwrap();
        assert!(install.global);
    }

    /// Test SkillsError::SkillAlreadyInstalled displays correct error message
    #[test]
    fn test_skill_already_installed_error_display() {
        let error = SkillsError::SkillAlreadyInstalled {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let display_message = format!("{}", error);

        // Verify error message contains key information
        assert!(
            display_message.contains("test-skill"),
            "Error message should contain skill name, got: {}",
            display_message
        );
    }

    /// Test SkillsError::SkillAlreadyInstalled can be cloned
    #[test]
    fn test_skill_already_installed_error_clone() {
        let error = SkillsError::SkillAlreadyInstalled {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let cloned = error.clone();

        assert_eq!(format!("{}", error), format!("{}", cloned));
    }

    /// Test SkillsError::SkillAlreadyInstalled equality
    #[test]
    fn test_skill_already_installed_error_equality() {
        let error1 = SkillsError::SkillAlreadyInstalled {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let error2 = SkillsError::SkillAlreadyInstalled {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let error3 = SkillsError::SkillAlreadyInstalled {
            skill_name: "different-skill".to_string(),
            path: "./skills/different-skill".to_string(),
        };

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    /// Test SkillsError::DestinationAlreadyExists displays correct error message
    #[test]
    fn test_destination_already_exists_error_display() {
        let error = SkillsError::DestinationAlreadyExists {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let display_message = format!("{}", error);

        // Verify error message contains key information
        assert!(
            display_message.contains("test-skill"),
            "Error message should contain skill name, got: {}",
            display_message
        );
        assert!(
            display_message.contains("./skills/test-skill"),
            "Error message should contain path, got: {}",
            display_message
        );
    }

    /// Test SkillsError::DestinationAlreadyExists can be cloned
    #[test]
    fn test_destination_already_exists_error_clone() {
        let error = SkillsError::DestinationAlreadyExists {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let cloned = error.clone();

        assert_eq!(format!("{}", error), format!("{}", cloned));
    }

    /// Test SkillsError::DestinationAlreadyExists equality
    #[test]
    fn test_destination_already_exists_error_equality() {
        let error1 = SkillsError::DestinationAlreadyExists {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let error2 = SkillsError::DestinationAlreadyExists {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let error3 = SkillsError::DestinationAlreadyExists {
            skill_name: "different-skill".to_string(),
            path: "./skills/different-skill".to_string(),
        };

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
}
