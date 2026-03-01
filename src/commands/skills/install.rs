use crate::commands::skills::SkillsInstall;
use crate::config::Config;
use crate::skills::{
    add_skill_to_lockfile, create_npx_command,
    SkillsError, SkillsManager, NPX_NOT_FOUND_ERROR,
};
use crate::traits::ExitCode;
use std::fs;
use std::path::PathBuf;

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
pub fn perform_post_install_move(
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
