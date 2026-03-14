use crate::commands::skills::SkillsInstall;
use crate::config::Config;
use crate::skills::{
    add_skill_to_lockfile, create_npx_command, SkillsError, SkillsManager, NPX_NOT_FOUND_ERROR,
};
use crate::traits::ExitCode;
use std::fs;
use std::path::{Path, PathBuf};

/// Run the `switchboard skills install` command
///
/// This command delegates to `npx skills add` to install a skill from
/// the specified source. The command includes `-a kilo -y` flags to
/// auto-confirm the installation and set the author to "kilo".
/// If the destination already exists, it will prompt for confirmation
/// unless the --yes flag is provided.
///
/// Per requirements (Section 3.3.2), the install flow is:
/// 1. Run `npx skills add <source> -y` which installs to the Kilo-local skills directory
/// 2. Verify the skill exists in the resolved skills directory
/// 3. Verify `SKILL.md` exists after install
/// 4. Update lockfile
pub async fn run_skills_install(args: SkillsInstall, _config: &Config) -> ExitCode {
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
        // Step 2-4: Verify post-install location, verify skill contents, update lockfile
        match perform_post_install_move(&skills_dir, &skill_name, &args.source) {
            Ok(_) => {
                println!(
                    "Moved to ./{}/
Updated skills.lock.json",
                    skill_name
                );
            }
            Err(e) => {
                eprintln!("Warning: Post-install verification failed: {}", e);
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

/// Reconciles a skill installed by Kilo into Switchboard's canonical skills directory.
///
/// The external installer writes to `.kilocode/skills/<skill-name>`, while
/// Switchboard discovers project skills from `skills/<skill-name>`. This helper
/// copies the installer output into the requested destination and verifies that
/// `SKILL.md` is present afterward.
pub fn perform_post_install_move(
    skills_dir: &Path,
    skill_name: &str,
    _source: &str,
) -> Result<(), SkillsError> {
    let installer_dir = PathBuf::from(".kilocode").join("skills").join(skill_name);
    let dest_dir = skills_dir.join(skill_name);

    if !installer_dir.exists() {
        return Err(SkillsError::SkillNotFound {
            skill_source: skill_name.to_string(),
        });
    }

    copy_skill_directory(&installer_dir, &dest_dir)?;

    if !dest_dir.exists() {
        return Err(SkillsError::SkillNotFound {
            skill_source: skill_name.to_string(),
        });
    }

    let skill_md_path = dest_dir.join("SKILL.md");
    if !skill_md_path.exists() {
        return Err(SkillsError::SkillNotFound {
            skill_source: skill_name.to_string(),
        });
    }

    println!("Done.");

    Ok(())
}

fn copy_skill_directory(src: &Path, dest: &Path) -> Result<(), SkillsError> {
    if !src.is_dir() {
        return Err(SkillsError::IoError {
            operation: "read installer skill directory".to_string(),
            path: src.display().to_string(),
            message: format!("Skill source directory not found: {}", src.display()),
        });
    }

    if dest.exists() {
        fs::remove_dir_all(dest).map_err(|e| SkillsError::IoError {
            operation: "remove existing reconciled skill directory".to_string(),
            path: dest.display().to_string(),
            message: e.to_string(),
        })?;
    }

    fs::create_dir_all(dest).map_err(|e| SkillsError::IoError {
        operation: "create reconciled skill directory".to_string(),
        path: dest.display().to_string(),
        message: e.to_string(),
    })?;

    for entry in fs::read_dir(src).map_err(|e| SkillsError::IoError {
        operation: "read installer skill directory".to_string(),
        path: src.display().to_string(),
        message: e.to_string(),
    })? {
        let entry = entry.map_err(|e| SkillsError::IoError {
            operation: "read installer skill directory entry".to_string(),
            path: src.display().to_string(),
            message: e.to_string(),
        })?;
        let entry_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if entry_path.is_dir() {
            copy_skill_directory(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path).map_err(|e| SkillsError::IoError {
                operation: "copy reconciled skill file".to_string(),
                path: dest_path.display().to_string(),
                message: e.to_string(),
            })?;
        }
    }

    Ok(())
}

/// Cleans up empty legacy `.agents/skills/` and `.agents/` directories.
///
/// This remains available for legacy cleanup paths but is not required for
/// current Kilo-local post-install handling.
pub fn cleanup_agents_directory() -> Result<(), SkillsError> {
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
pub fn extract_skill_name(source: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::perform_post_install_move;
    use std::{env, fs};

    #[test]
    fn test_perform_post_install_move_copies_from_kilocode_to_project_skills() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let original_dir = env::current_dir().expect("cwd");
        env::set_current_dir(temp_dir.path()).expect("set cwd");

        let test_result = (|| {
            let installer_skill_dir = temp_dir.path().join(".kilocode/skills/copied-skill/nested");
            let project_skills_dir = temp_dir.path().join("skills");

            fs::create_dir_all(&installer_skill_dir).expect("create installer dir");
            fs::write(temp_dir.path().join(".kilocode/skills/copied-skill/SKILL.md"), "# Skill")
                .expect("write skill");
            fs::write(installer_skill_dir.join("info.txt"), "hello").expect("write nested");

            perform_post_install_move(&project_skills_dir, "copied-skill", "owner/repo@copied-skill")
                .expect("reconcile skill");

            assert!(project_skills_dir.join("copied-skill/SKILL.md").exists());
            assert_eq!(
                fs::read_to_string(project_skills_dir.join("copied-skill/nested/info.txt"))
                    .expect("read nested copied file"),
                "hello"
            );
        })();

        env::set_current_dir(original_dir).expect("restore cwd");
        test_result;
    }

    #[test]
    fn test_perform_post_install_move_fails_when_kilocode_skill_missing() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let original_dir = env::current_dir().expect("cwd");
        env::set_current_dir(temp_dir.path()).expect("set cwd");

        let result = perform_post_install_move(
            &temp_dir.path().join("skills"),
            "missing-skill",
            "owner/repo@missing-skill",
        );

        env::set_current_dir(original_dir).expect("restore cwd");

        assert!(result.is_err());
    }
}
