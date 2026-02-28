//! Implementation of the skills remove subcommand.

use super::types::{ExitCode, SkillsRemove};
use crate::config::Config;
use crate::skills::{
    find_skill_directory, get_agents_using_skill, remove_skill_directory,
    remove_skill_from_lockfile, SkillsManager,
};
use std::io::{self, Write};

/// Removes a skill from the local or global skills directory.
///
/// This function:
/// 1. Finds the skill directory
/// 2. Checks if any agents reference this skill
/// 3. Prompts for confirmation (unless --yes flag is set)
/// 4. Removes the skill directory
/// 5. Updates the lockfile
///
/// # Arguments
///
/// * `args` - The [`SkillsRemove`] arguments containing skill name and options
/// * `config` - Reference to the application [`Config`]
///
/// # Returns
///
/// Returns [`ExitCode`] indicating success or failure
pub async fn run_skills_remove(args: SkillsRemove, config: &Config) -> ExitCode {
    // Find the skill directory
    let skill_path = match find_skill_directory(&args.skill_name, args.global) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::Error;
        }
    };

    // Get list of agents that reference this skill
    let referenced_agents = get_agents_using_skill(&args.skill_name, config);

    // Show warning if skill is referenced in config
    if !referenced_agents.is_empty() {
        let agents_str = referenced_agents.join(", ");
        eprintln!(
            "Warning: Skill '{}' is still referenced in switchboard.toml by: {}",
            args.skill_name, agents_str
        );
    }

    // Prompt for confirmation unless --yes flag is set
    if !args.yes {
        let prompt = format!("Remove skill '{}'? [y/N]", args.skill_name);
        if !confirm(&prompt) {
            println!("Operation cancelled.");
            return ExitCode::Success;
        }
    }

    // Remove the skill directory
    match remove_skill_directory(&skill_path) {
        Ok(()) => {
            // Remove skill from lockfile
            let skills_manager = SkillsManager::new(None);
            let skills_dir = if args.global {
                skills_manager.global_skills_dir.clone()
            } else {
                skills_manager.skills_dir.clone()
            };

            if let Err(e) = remove_skill_from_lockfile(&skills_dir, &args.skill_name) {
                eprintln!("Warning: Failed to update lockfile: {}", e);
            }

            let scope = if args.global { "global " } else { "" };
            println!("Removed skill '{}' ({})", args.skill_name, scope.trim());
            ExitCode::Success
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::Error
        }
    }
}

/// Prompts the user for confirmation and returns their choice.
///
/// # Arguments
///
/// * `prompt` - The confirmation prompt to display
///
/// # Returns
///
/// * `true` - If the user confirms with 'y' or 'Y'
/// * `false` - If the user declines with 'n', 'N', or presses Enter (default)
fn confirm(prompt: &str) -> bool {
    print!("{} ", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let input = input.trim().to_lowercase();
    input == "y"
}
