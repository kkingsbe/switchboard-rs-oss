use super::types::ExitCode;
use crate::commands::skills::install::perform_post_install_move;
use crate::config::Config;
use crate::skills::{
    create_npx_command, read_lockfile, write_lockfile, SkillsError, NPX_NOT_FOUND_ERROR,
};
use std::path::Path;

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
pub async fn handle_skills_update(args: super::types::SkillsUpdate, _config: &Config) -> ExitCode {
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
pub fn reinstall_skill_from_source(
    skills_dir: &Path,
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
pub fn update_skill_timestamp(
    skills_dir: &Path,
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
