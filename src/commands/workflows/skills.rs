//! Skills installation service for workflows
//!
//! This module provides functions for installing, updating, and checking
//! skills required by workflow manifests.

use crate::commands::skills::install::{
    cleanup_agents_directory, extract_skill_name, perform_post_install_move,
};
use crate::skills::{
    add_skill_to_lockfile, create_npx_command, scan_project_skills, SkillsManager, NPX_NOT_FOUND_ERROR,
};
use crate::workflows::WorkflowsError;

/// Installs all skills required by a workflow manifest.
///
/// This function checks if npx is available, then for each skill source:
/// - Checks if already installed (skips if yes)
/// - Runs `npx skills add skill-source -a kilo -y`
/// - Handles post-install move
/// - Adds to lockfile
///
/// Returns a list of installed skill names. Fails completely if any skill fails to install.
///
/// # Arguments
/// * `skills` - List of skill sources to install
/// * `yes` - Whether to skip confirmation prompts
///
/// # Returns
/// * `Ok(Vec<String>)` - List of installed skill names
/// * `Err(WorkflowsError)` - If any skill fails to install
pub fn install_workflow_skills(skills: &[String], _yes: bool) -> Result<Vec<String>, WorkflowsError> {
    // Check if npx is available
    let mut skills_manager = SkillsManager::new(None);
    skills_manager
        .check_npx_available()
        .map_err(|e| WorkflowsError::SkillError(e.to_string()))?;

    if !skills_manager.npx_available {
        return Err(WorkflowsError::SkillError(NPX_NOT_FOUND_ERROR.to_string()));
    }

    let mut installed_skills = Vec::new();
    let skills_dir = skills_manager.skills_dir.clone();

    for skill_source in skills {
        let skill_name = extract_skill_name(skill_source);
        let skill_path = skills_dir.join(&skill_name);

        // Check if already installed
        if skill_path.exists() {
            println!("Skill '{}' already installed, skipping", skill_name);
            installed_skills.push(skill_name);
            continue;
        }

        // Install the skill
        println!("Installing skill '{}' from {}...", skill_name, skill_source);

        let mut cmd = create_npx_command();
        cmd.arg("skills");
        cmd.arg("add");

        // Parse source to handle @skill-name format
        if let Some(at_pos) = skill_source.rfind('@') {
            let repo = &skill_source[..at_pos];
            let skill_name_from_source = &skill_source[at_pos + 1..];
            cmd.arg(repo);
            cmd.arg("--skill");
            cmd.arg(skill_name_from_source);
        } else {
            cmd.arg(skill_source);
        }

        cmd.arg("-a");
        cmd.arg("kilo");
        cmd.arg("-y"); // Auto-confirm

        let result = cmd.status().map_err(|e| {
            WorkflowsError::SkillError(format!("Failed to execute npx skills add: {}", e))
        })?;

        if !result.success() {
            return Err(WorkflowsError::SkillError(format!(
                "Failed to install skill '{}' from {} (exit code: {:?})",
                skill_name,
                skill_source,
                result.code()
            )));
        }

        // Handle post-install move
        if let Err(e) = perform_post_install_move(&skills_dir, &skill_name, skill_source) {
            return Err(WorkflowsError::SkillError(format!(
                "Post-install move failed for skill '{}': {}",
                skill_name, e
            )));
        }

        // Add to lockfile
        if let Err(e) = add_skill_to_lockfile(&skills_dir, &skill_name, skill_source) {
            return Err(WorkflowsError::SkillError(format!(
                "Failed to add skill '{}' to lockfile: {}",
                skill_name, e
            )));
        }

        // Clean up .agents directory
        if let Err(e) = cleanup_agents_directory() {
            eprintln!("Warning: Failed to cleanup .agents directory: {}", e);
        }

        println!("Successfully installed skill '{}'", skill_name);
        installed_skills.push(skill_name);
    }

    Ok(installed_skills)
}

/// Updates all skills required by a workflow.
///
/// This function tries `npx skills update` first, and falls back to
/// fresh install if update fails.
///
/// # Arguments
/// * `skills` - List of skill sources to update
///
/// # Returns
/// * `Ok(Vec<String>)` - List of updated skill names
/// * `Err(WorkflowsError)` - If any skill fails to update
pub fn update_workflow_skills(skills: &[String]) -> Result<Vec<String>, WorkflowsError> {
    // Check if npx is available
    let mut skills_manager = SkillsManager::new(None);
    skills_manager
        .check_npx_available()
        .map_err(|e| WorkflowsError::SkillError(e.to_string()))?;

    if !skills_manager.npx_available {
        return Err(WorkflowsError::SkillError(NPX_NOT_FOUND_ERROR.to_string()));
    }

    let mut updated_skills = Vec::new();
    let skills_dir = skills_manager.skills_dir.clone();

    for skill_source in skills {
        let skill_name = extract_skill_name(skill_source);
        let skill_path = skills_dir.join(&skill_name);

        // Check if skill is installed
        if !skill_path.exists() {
            println!(
                "Skill '{}' not installed, installing fresh...",
                skill_name
            );
            // Try to install fresh
            match install_single_skill(&skills_manager, skill_source, &skill_name) {
                Ok(_) => {
                    updated_skills.push(skill_name);
                }
                Err(e) => {
                    return Err(WorkflowsError::SkillError(format!(
                        "Failed to install skill '{}': {}",
                        skill_name, e
                    )));
                }
            }
            continue;
        }

        // Try npx skills update first
        println!("Updating skill '{}'...", skill_name);

        let mut cmd = create_npx_command();
        cmd.arg("skills");
        cmd.arg("update");
        cmd.arg(&skill_name);

        let result = cmd.status();

        match result {
            Ok(status) if status.success() => {
                println!("Successfully updated skill '{}'", skill_name);
                updated_skills.push(skill_name);
            }
            _ => {
                // Fall back to fresh install
                println!(
                    "Update failed for '{}', trying fresh install...",
                    skill_name
                );

                // Remove existing skill directory
                if skill_path.exists() {
                    std::fs::remove_dir_all(&skill_path).map_err(|e| {
                        WorkflowsError::SkillError(format!(
                            "Failed to remove existing skill '{}': {}",
                            skill_name, e
                        ))
                    })?;
                }

                match install_single_skill(&skills_manager, skill_source, &skill_name) {
                    Ok(_) => {
                        println!(
                            "Successfully reinstalled skill '{}'",
                            skill_name
                        );
                        updated_skills.push(skill_name);
                    }
                    Err(e) => {
                        return Err(WorkflowsError::SkillError(format!(
                            "Failed to update skill '{}': {}",
                            skill_name, e
                        )));
                    }
                }
            }
        }
    }

    Ok(updated_skills)
}

/// Installs a single skill (helper function for update).
fn install_single_skill(
    skills_manager: &SkillsManager,
    skill_source: &str,
    skill_name: &str,
) -> Result<(), WorkflowsError> {
    let skills_dir = skills_manager.skills_dir.clone();

    let mut cmd = create_npx_command();
    cmd.arg("skills");
    cmd.arg("add");

    // Parse source to handle @skill-name format
    if let Some(at_pos) = skill_source.rfind('@') {
        let repo = &skill_source[..at_pos];
        let skill_name_from_source = &skill_source[at_pos + 1..];
        cmd.arg(repo);
        cmd.arg("--skill");
        cmd.arg(skill_name_from_source);
    } else {
        cmd.arg(skill_source);
    }

    cmd.arg("-a");
    cmd.arg("kilo");
    cmd.arg("-y");

    let result = cmd.status().map_err(|e| {
        WorkflowsError::SkillError(format!("Failed to execute npx skills add: {}", e))
    })?;

    if !result.success() {
        return Err(WorkflowsError::SkillError(format!(
            "Failed to install skill '{}' (exit code: {:?})",
            skill_name,
            result.code()
        )));
    }

    // Handle post-install move
    if let Err(e) = perform_post_install_move(&skills_dir, skill_name, skill_source) {
        return Err(WorkflowsError::SkillError(format!(
            "Post-install move failed: {}",
            e
        )));
    }

    // Add to lockfile
    if let Err(e) = add_skill_to_lockfile(&skills_dir, skill_name, skill_source) {
        return Err(WorkflowsError::SkillError(format!(
            "Failed to add to lockfile: {}",
            e
        )));
    }

    // Cleanup
    if let Err(e) = cleanup_agents_directory() {
        eprintln!("Warning: Failed to cleanup .agents directory: {}", e);
    }

    Ok(())
}

/// Checks if all required skills are installed.
///
/// # Arguments
/// * `skills` - List of skill sources to check
///
/// # Returns
/// * `Ok(Vec<String>)` - List of missing skill names (empty if all installed)
/// * `Err(WorkflowsError)` - If there's an error checking skills
pub fn check_skills_installed(skills: &[String]) -> Result<Vec<String>, WorkflowsError> {
    // Scan installed skills (uses default ./skills directory)
    let (installed_skills, _warnings) = scan_project_skills()
        .map_err(|e| WorkflowsError::SkillError(format!("Failed to scan skills: {}", e)))?;

    let installed_names: Vec<String> = installed_skills
        .iter()
        .map(|s| s.name.clone())
        .collect();

    let mut missing_skills = Vec::new();

    for skill_source in skills {
        let skill_name = extract_skill_name(skill_source);

        if !installed_names.contains(&skill_name) {
            missing_skills.push(skill_name);
        }
    }

    Ok(missing_skills)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_skill_name_from_source() {
        // Test various skill source formats
        assert_eq!(
            extract_skill_name("owner/repo"),
            "repo"
        );
        assert_eq!(
            extract_skill_name("owner/repo@skill-name"),
            "skill-name"
        );
        assert_eq!(
            extract_skill_name("https://github.com/owner/repo"),
            "repo"
        );
        assert_eq!(
            extract_skill_name("https://github.com/owner/repo@skill-name"),
            "skill-name"
        );
    }

    #[test]
    fn test_check_skills_installed_empty_list() {
        // Test with empty skills list
        let result = check_skills_installed(&[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_check_skills_installed_nonexistent_skills() {
        // Test checking for skills that don't exist
        let skills = vec!["nonexistent/skill1".to_string(), "nonexistent/skill2".to_string()];
        let result = check_skills_installed(&skills);
        
        // This should return the skill names as missing
        // Note: This test may pass or fail depending on whether skills exist
        assert!(result.is_ok());
        let missing = result.unwrap();
        // The skills don't exist in the test environment, so they should be missing
        assert_eq!(missing.len(), 2);
    }

    #[test]
    fn test_extract_skill_name_git_at_format() {
        // Test git@github.com format
        assert_eq!(
            extract_skill_name("git@github.com:owner/repo"),
            "repo"
        );
        assert_eq!(
            extract_skill_name("git@github.com:owner/repo@skill-name"),
            "skill-name"
        );
    }

    #[test]
    fn test_extract_skill_name_various_formats() {
        // Test more edge cases
        assert_eq!(
            extract_skill_name("org"),
            "org"
        );
        assert_eq!(
            extract_skill_name("my-skill-name"),
            "my-skill-name"
        );
    }
}
