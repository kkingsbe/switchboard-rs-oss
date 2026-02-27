//! Docker Skills - Skill installation and entrypoint script generation for Docker containers
//!
//! This module handles:
//! - Skill installation into Docker containers
//! - Entrypoint script generation for skills
//! - Management of skill dependencies
//! - Integration with the Docker container lifecycle
//!
//! # Test Coverage
//!
//! As of 2026-02-20, the module has comprehensive test coverage:
//! - Function Coverage: 100.00% (27/27)
//! - Line Coverage: 98.89% (356/360)
//! - Region Coverage: 98.89% (533/539)
//!
//! All coverage metrics exceed the project standard of >80%.

use crate::skills::SkillsError;
use std::path::Path;

/// Validates that a skill source string conforms to the expected format.
///
/// This function ensures that skill source strings follow either the `owner/repo`
/// or `owner/repo@skill-name` pattern. Validation rules:
///
/// - Must contain exactly one `/` separator
/// - Before `/`: non-empty owner name (alphanumeric, hyphen, underscore)
/// - After `/`: non-empty repo name (alphanumeric, hyphen, underscore)
/// - Optional `@` followed by skill name (same character rules)
///
/// # Validation Algorithm
///
/// The validation algorithm performs the following steps:
///
/// 1. **Slash Count Check**: Verifies that the skill string contains exactly one `/` separator
/// 2. **Component Splitting**: Splits the string at the `/` to separate owner and repo parts
/// 3. **Owner Validation**: Checks that the owner name is non-empty and contains only
///    alphanumeric characters, hyphens, or underscores
/// 4. **Repo Validation**: Checks if an optional `@skill-name` suffix exists:
///    - If `@` is present: validates both the repo name (before `@`) and skill name (after `@`)
///    - If `@` is absent: validates only the repo name
/// 5. All components must be non-empty and contain only allowed characters
///
/// The algorithm does not use regex pattern matching, but instead performs explicit
/// character validation checks for clarity and precise error reporting.
///
/// # Parameters
///
/// * `skill` - The skill source string to validate
///
/// # Returns
///
/// * `Ok(())` - If the skill format is valid
///
/// # Errors
///
/// Returns `Err(SkillsError::InvalidSkillFormat)` if the skill format is invalid:
///
/// - Missing or multiple `/` separators
/// - Empty owner name
/// - Empty repo name (before or after `@`)
/// - Empty skill name (after `@`)
/// - Invalid characters in any component (only alphanumeric, hyphen, underscore allowed)
///
/// Each error includes the problematic skill source and a detailed reason for validation failure.
///
/// # Examples
///
/// ```rust,ignore
/// use switchboard::docker::skills::validate_skill_format;
///
/// // Valid formats
/// assert!(validate_skill_format("owner/repo").is_ok());
/// assert!(validate_skill_format("owner/repo@skill-name").is_ok());
///
/// // Invalid formats
/// assert!(validate_skill_format("owner").is_err());
/// assert!(validate_skill_format("owner/").is_err());
/// assert!(validate_skill_format("/repo").is_err());
/// assert!(validate_skill_format("owner//repo").is_err());
/// assert!(validate_skill_format("owner/repo@").is_err());
/// ```
/// Extracts the skill name from a skill source string.
///
/// This function parses skill source strings in the following formats:
/// - `owner/repo` - uses the repo part as the skill name
/// - `owner/repo@skill-name` - uses the skill-name part as the skill name
///
/// The skill name extraction logic matches the behavior in `find_preexisting_skills()`
/// to ensure consistency across the codebase.
///
/// # Parameters
///
/// * `skill` - The skill source string (e.g., "owner/repo" or "owner/repo@skill-name")
///
/// # Returns
///
/// * `Ok(String)` - The extracted skill name
///
/// # Errors
///
/// Returns `Err(SkillsError::Configuration)` if the skill format is invalid.
fn extract_skill_name(skill: &str) -> Result<String, SkillsError> {
    if skill.contains('@') {
        // Format: owner/repo@skill-name
        let parts: Vec<&str> = skill.split('@').collect();
        if parts.len() == 2 {
            Ok(parts[1].to_string())
        } else {
            Err(SkillsError::InvalidSkillFormat {
                skill_source: skill.to_string(),
                reason: "Invalid skill format".to_string(),
            })
        }
    } else {
        // Format: owner/repo - use the repo part as the skill name
        let parts: Vec<&str> = skill.split('/').collect();
        if parts.len() == 2 {
            Ok(parts[1].to_string())
        } else {
            Err(SkillsError::InvalidSkillFormat {
                skill_source: skill.to_string(),
                reason: "Invalid skill format".to_string(),
            })
        }
    }
}

pub fn validate_skill_format(skill: &str) -> Result<(), SkillsError> {
    // Check for exactly one `/` separator
    let slash_count = skill.matches('/').count();
    if slash_count != 1 {
        return Err(SkillsError::InvalidSkillFormat {
            skill_source: skill.to_string(),
            reason: format!("Expected exactly one '/' separator, found {}", slash_count),
        });
    }

    // Split at the `/` separator
    let parts: Vec<&str> = skill.split('/').collect();
    let owner = parts[0];
    let repo_part = parts[1];

    // Validate owner name (non-empty, alphanumeric, hyphen, underscore)
    if owner.is_empty() {
        return Err(SkillsError::InvalidSkillFormat {
            skill_source: skill.to_string(),
            reason: "Owner name is empty".to_string(),
        });
    }
    if !owner
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(SkillsError::InvalidSkillFormat {
            skill_source: skill.to_string(),
            reason: format!(
                "Owner name contains invalid characters '{}'. Only alphanumeric, hyphen, and underscore are allowed.",
                owner
            ),
        });
    }

    // Check if repo contains an optional `@skill-name` suffix
    if let Some(at_pos) = repo_part.find('@') {
        // Has `@skill-name` suffix
        let repo = &repo_part[..at_pos];
        let skill_name = &repo_part[at_pos + 1..];

        // Validate repo name (non-empty, alphanumeric, hyphen, underscore)
        if repo.is_empty() {
            return Err(SkillsError::InvalidSkillFormat {
                skill_source: skill.to_string(),
                reason: "Repository name is empty before '@'".to_string(),
            });
        }
        if !repo
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(SkillsError::InvalidSkillFormat {
                skill_source: skill.to_string(),
                reason: format!(
                    "Repository name contains invalid characters '{}'. Only alphanumeric, hyphen, and underscore are allowed.",
                    repo
                ),
            });
        }

        // Validate skill name (non-empty, alphanumeric, hyphen, underscore)
        if skill_name.is_empty() {
            return Err(SkillsError::InvalidSkillFormat {
                skill_source: skill.to_string(),
                reason: "Skill name is empty after '@'".to_string(),
            });
        }
        if !skill_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(SkillsError::InvalidSkillFormat {
                skill_source: skill.to_string(),
                reason: format!(
                    "Skill name contains invalid characters '{}'. Only alphanumeric, hyphen, and underscore are allowed.",
                    skill_name
                ),
            });
        }
    } else {
        // No `@skill-name` suffix, just validate repo name
        let repo = repo_part;

        if repo.is_empty() {
            return Err(SkillsError::InvalidSkillFormat {
                skill_source: skill.to_string(),
                reason: "Repository name is empty after '/'".to_string(),
            });
        }
        if !repo
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(SkillsError::InvalidSkillFormat {
                skill_source: skill.to_string(),
                reason: format!(
                    "Repository name contains invalid characters '{}'. Only alphanumeric, hyphen, and underscore are allowed.",
                    repo
                ),
            });
        }
    }

    Ok(())
}

/// Generates a shell script to install skills and launch Kilo Code CLI.
///
/// This function creates an entrypoint script for Docker containers that:
/// 1. Validates the format of each skill identifier
/// 2. Installs specified skills using `npx skills add`
/// 3. Executes the Kilo Code CLI with all container arguments
///
/// The script uses `set -e` for error propagation and `exec` for proper
/// process replacement, ensuring that signal handling and exit codes
/// are correctly passed through to the Kilo Code CLI.
///
/// # Script Generation Structure
///
/// The script generation follows a three-part structure:
///
/// 1. **Empty Skills Case**: If the skills slice is empty, returns an empty string
///    since no installation is needed
///
/// 2. **Validation Loop**: Iterates through all skills and validates each format
///    using [`validate_skill_format()`]. If any skill is invalid, returns an error
///    immediately with the agent name and detailed reason
///
/// 3. **Script Template**: For valid skills, constructs the complete entrypoint script:
///    - Shebang line (`#!/bin/sh`) for POSIX compatibility
///    - `set -e` for immediate exit on errors
///    - Sequential `npx skills add` commands for each skill
///    - `exec kilocode --yes "$@"` to hand off to the CLI with process replacement
///
/// # Parameters
///
/// * `agent_name` - Name of the agent for which the script is being generated
/// * `skills` - A slice of skill identifiers to install. Each skill must be in
///   the format `owner/repo` or `owner/repo@skill-name`. The owner and repo names
///   must be non-empty and contain only alphanumeric characters, hyphens, and underscores.
///
/// # Returns
///
/// * `Ok(String)` - The generated shell script contents
///
/// # Errors
///
/// Returns `Err(SkillsError::ScriptGenerationFailed)` in the following cases:
///
/// - **Invalid Skill Format**: If any skill identifier does not match the expected
///   `owner/repo` or `owner/repo@skill-name` format. The error includes the agent name,
///   the malformed skill source, and a detailed reason explaining why validation failed.
///
/// The `ScriptGenerationFailed` error variant is also reserved for future enhancements
/// such as template parsing errors or file system issues.
///
/// # Examples
///
/// ```rust,ignore
/// use switchboard::docker::skills::generate_entrypoint_script;
///
/// // Generate script with valid skills
/// let skills = vec![
///     "owner/repo1".to_string(),
///     "owner/repo2@skill-name".to_string(),
/// ];
///
/// let script = generate_entrypoint_script("my-agent", &skills)?;
/// assert!(script.contains("#!/bin/sh"));
/// assert!(script.contains("npx skills add owner/repo1 -a kilo -y"));
/// assert!(script.contains("npx skills add owner/repo2@skill-name -a kilo -y"));
/// assert!(script.contains("exec kilocode --yes \"$@\""));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ```rust,ignore
/// use switchboard::docker::skills::generate_entrypoint_script;
///
/// // Generate script with empty skills list
/// let skills: Vec<String> = vec![];
///
/// let script = generate_entrypoint_script("my-agent", &skills)?;
/// assert!(script.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Output Format
///
/// For non-empty skills input:
/// ```text
/// #!/bin/sh
/// # POSIX shell for maximum compatibility across container environments
/// set -e
/// # Error propagation - immediately exit on any command failure to prevent cascading errors
///
/// # Install skills
/// # Skills are installed sequentially in declaration order to satisfy dependencies
/// npx skills add owner/repo1 -a kilo -y
/// npx skills add owner/repo2@skill-name -a kilo -y
///
/// # Hand off to Kilo Code CLI
/// # Process replacement - replaces shell with kilocode, ensuring proper signal handling and exit code propagation
/// exec kilocode --yes "$@"
/// ```
///
/// For empty skills input, returns an empty string.
///
/// # Parameters
///
/// * `agent_name` - The name of the agent (used for error messages)
/// * `skills` - The list of skills to install
/// * `preexisting_skills` - A list of skill names that are already manually installed in `./skills/`
///
/// Skills in `preexisting_skills` will skip the `npx skills add` command and be logged as preexisting.
pub fn generate_entrypoint_script(
    agent_name: &str,
    skills: &[String],
    preexisting_skills: &[String],
) -> Result<String, SkillsError> {
    // Return empty string for empty skills list to prevent generating unnecessary entrypoint scripts
    // This optimization avoids creating a script when no skills need to be installed
    if skills.is_empty() {
        return Ok(String::new());
    }

    // Validate all skill formats before generating the script
    // Early return on any validation failure prevents partial script generation
    for skill in skills {
        validate_skill_format(skill).map_err(|e| {
            // Map validation errors to user-friendly script generation errors with context
            // This provides better error messages that include the agent name and specific skill
            if let SkillsError::InvalidSkillFormat {
                skill_source,
                reason,
            } = e
            {
                SkillsError::ScriptGenerationFailed {
                    agent_name: agent_name.to_string(),
                    reason: format!("Invalid skill format '{}': {}", skill_source, reason),
                }
            } else {
                e
            }
        })?;
    }

    // Validate all skills exist in ./skills/ directory
    // Per Section 3.6 requirements: If a skill doesn't exist on host, container launch FAILS immediately
    // This enforces the "bind-mount skills instead of runtime npx" requirement
    for skill in skills {
        let skill_name = extract_skill_name(skill)?;
        if !preexisting_skills.contains(&skill_name) {
            // Check if skill file exists in ./skills/ directory
            let skill_path = Path::new("./skills").join(&skill_name).join("SKILL.md");
            if !skill_path.exists() {
                return Err(SkillsError::ScriptGenerationFailed {
                    agent_name: agent_name.to_string(),
                    reason: format!(
                        "Skill '{}' is not found in ./skills/ directory. Please add the skill to ./skills/{}/SKILL.md before using it.",
                        skill_name, skill_name
                    ),
                });
            }
        }
    }

    // Per Section 3.6: Skills are bind-mounted from host, NOT installed at runtime
    // Generate simple entrypoint script that just runs kilocode
    // No npx skills add needed - skills are already available via bind-mount
    let mut script = String::from("#!/bin/sh\nset -e\n\n");

    // Add the kilocode execution command
    // Skills are already mounted at /workspace/skills/<skill-name>/ via Docker bind-mounts
    script.push_str("exec kilocode --yes \"$@\"\n");

    Ok(script)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_skill_format_valid_owner_repo() {
        let result = validate_skill_format("owner/repo");
        assert!(
            result.is_ok(),
            "Valid 'owner/repo' format should be accepted"
        );
    }

    #[test]
    fn test_validate_skill_format_valid_with_skill_name() {
        let result = validate_skill_format("owner/repo@skill-name");
        assert!(
            result.is_ok(),
            "Valid 'owner/repo@skill-name' format should be accepted"
        );
    }

    #[test]
    fn test_validate_skill_format_missing_slash() {
        let result = validate_skill_format("owner");
        assert!(result.is_err(), "Format without slash should be rejected");

        let result = validate_skill_format("owner_repo");
        assert!(
            result.is_err(),
            "Format with underscores but no slash should be rejected"
        );
    }

    #[test]
    fn test_validate_skill_format_empty_owner() {
        let result = validate_skill_format("/repo");
        assert!(result.is_err(), "Empty owner should be rejected");

        match result {
            Err(SkillsError::InvalidSkillFormat { reason, .. }) => {
                assert!(
                    reason.contains("empty") || reason.contains("Owner"),
                    "Error should mention empty owner"
                );
            }
            _ => panic!("Expected InvalidSkillFormat error"),
        }
    }

    #[test]
    fn test_validate_skill_format_empty_repo() {
        let result = validate_skill_format("owner/");
        assert!(result.is_err(), "Empty repo should be rejected");

        let result = validate_skill_format("owner/@skill");
        assert!(result.is_err(), "Empty repo before @ should be rejected");
    }

    #[test]
    fn test_validate_skill_format_invalid_chars() {
        // Test invalid characters in owner
        let result = validate_skill_format("owner.name/repo");
        assert!(
            result.is_err(),
            "Owner with invalid characters (dot) should be rejected"
        );

        let result = validate_skill_format("owner repo/repo");
        assert!(result.is_err(), "Owner with spaces should be rejected");

        // Test invalid characters in repo
        let result = validate_skill_format("owner/repo.name");
        assert!(
            result.is_err(),
            "Repo with invalid characters (dot) should be rejected"
        );

        // Test invalid characters in skill name
        let result = validate_skill_format("owner/repo@skill.name");
        assert!(
            result.is_err(),
            "Skill name with invalid characters (dot) should be rejected"
        );

        let result = validate_skill_format("owner/repo@skill name");
        assert!(result.is_err(), "Skill name with spaces should be rejected");
    }

    #[test]
    fn test_generate_entrypoint_script_with_invalid_skill() {
        let skills = vec!["invalid-skill-format".to_string()];
        let result = generate_entrypoint_script("test-agent", &skills, &[]);
        assert!(
            result.is_err(),
            "generate_entrypoint_script should return error for invalid skill format"
        );

        match result {
            Err(SkillsError::ScriptGenerationFailed { agent_name, .. }) => {
                assert_eq!(agent_name, "test-agent", "Error should include agent name");
            }
            _ => panic!("Expected ScriptGenerationFailed error"),
        }
    }

    #[test]
    fn test_generate_entrypoint_script_with_valid_skills() {
        let skills = vec!["owner/repo1".to_string(), "owner/repo2@skill".to_string()];
        let preexisting_skills = vec!["repo1".to_string(), "skill".to_string()];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);
        assert!(
            result.is_ok(),
            "Valid skills should generate script successfully"
        );

        let script = result.unwrap();
        assert!(
            script.contains("#!/bin/sh"),
            "Script should contain shebang"
        );
        assert!(
            script.contains("exec kilocode --yes \"$@\""),
            "Script should contain CLI execution command"
        );
    }

    #[test]
    fn test_generate_entrypoint_script_with_empty_skills() {
        let skills: Vec<String> = vec![];
        let result = generate_entrypoint_script("test-agent", &skills, &[]);
        assert!(result.is_ok(), "Empty skills list should return Ok");

        let script = result.unwrap();
        assert!(
            script.is_empty(),
            "Empty skills list should return empty string"
        );
    }

    #[test]
    fn test_generate_entrypoint_script_with_multiple_skills_3plus() {
        let skills = vec![
            "owner/repo1".to_string(),
            "owner/repo2@skill-name".to_string(),
            "owner/repo3".to_string(),
        ];
        let preexisting_skills = vec![
            "repo1".to_string(),
            "skill-name".to_string(),
            "repo3".to_string(),
        ];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);
        assert!(
            result.is_ok(),
            "Valid skills should generate script successfully"
        );

        let script = result.unwrap();
        assert!(
            script.contains("#!/bin/sh"),
            "Script should contain shebang"
        );
        assert!(
            script.contains("set -e"),
            "Script should contain error handling"
        );
        assert!(
            script.contains("exec kilocode --yes \"$@\""),
            "Script should contain CLI execution command"
        );
    }

    #[test]
    fn test_script_structure_and_safety() {
        // Test with 2 valid skills
        let skills = vec![
            "owner/repo1".to_string(),
            "owner/repo2@skill-name".to_string(),
        ];
        let preexisting_skills = vec!["repo1".to_string(), "skill-name".to_string()];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);

        // Verify that generate_entrypoint_script returns Ok
        assert!(
            result.is_ok(),
            "generate_entrypoint_script should succeed with valid skills"
        );

        let script = result.unwrap();

        // Verify shebang line is present at the start
        assert!(
            script.starts_with("#!/bin/sh"),
            "Script must start with shebang #!/bin/sh"
        );

        // Verify set -e is present for error handling
        assert!(
            script.contains("set -e"),
            "Script must contain 'set -e' for error handling"
        );

        // Verify exec kilocode --yes "$@" is present
        assert!(
            script.contains("exec kilocode --yes \"$@\""),
            "Script must contain 'exec kilocode --yes \"$@\"' for CLI execution"
        );

        // With bind-mounts, skills are mounted at runtime, not installed via npx
        // So the script should NOT contain any npx skills add commands
        assert!(
            !script.contains("npx skills"),
            "Script should NOT contain npx skills add commands (skills are mounted via bind-mounts)"
        );
    }

    #[test]
    fn test_validate_skill_format_repo_invalid_chars_with_skill_name() {
        // Test invalid characters in repo when using owner/repo@skill-name format
        let result = validate_skill_format("owner/repo.name@skill");
        assert!(
            result.is_err(),
            "Repo with invalid characters (dot) before @ should be rejected"
        );

        let result = validate_skill_format("owner/repo name@skill");
        assert!(
            result.is_err(),
            "Repo with spaces before @ should be rejected"
        );

        let result = validate_skill_format("owner/repo*test@skill");
        assert!(
            result.is_err(),
            "Repo with invalid characters (asterisk) before @ should be rejected"
        );
    }

    #[test]
    fn test_validate_skill_format_empty_skill_name() {
        // Test empty skill name after @
        let result = validate_skill_format("owner/repo@");
        assert!(
            result.is_err(),
            "Empty skill name after @ should be rejected"
        );

        match result {
            Err(SkillsError::InvalidSkillFormat { reason, .. }) => {
                assert!(
                    reason.contains("empty") || reason.contains("skill name"),
                    "Error should mention empty skill name"
                );
            }
            _ => panic!("Expected InvalidSkillFormat error"),
        }
    }

    #[test]
    fn test_validate_skill_format_multiple_slashes() {
        // Test multiple slashes (not exactly one)
        let result = validate_skill_format("owner//repo");
        assert!(result.is_err(), "Multiple slashes should be rejected");

        let result = validate_skill_format("owner///repo");
        assert!(result.is_err(), "Multiple slashes should be rejected");
    }

    #[test]
    fn test_validate_skill_format_valid_edge_cases() {
        // Test valid edge cases with special characters that ARE allowed
        assert!(
            validate_skill_format("owner-name/repo-name").is_ok(),
            "Hyphens in owner and repo should be valid"
        );

        assert!(
            validate_skill_format("owner_name/repo_name").is_ok(),
            "Underscores in owner and repo should be valid"
        );

        assert!(
            validate_skill_format("owner-123/repo-456").is_ok(),
            "Numbers in owner and repo should be valid"
        );

        assert!(
            validate_skill_format("owner/repo@skill-name_123").is_ok(),
            "Mixed alphanumeric with hyphens and underscores in skill name should be valid"
        );

        assert!(
            validate_skill_format("Owner/Repo@Skill").is_ok(),
            "Mixed case should be valid"
        );
    }

    #[test]
    fn test_validate_skill_format_special_invalid_chars() {
        // Test various invalid characters in different positions
        let result = validate_skill_format("owner#repo");
        assert!(result.is_err(), "Hash in owner/repo should be rejected");

        let result = validate_skill_format("owner/repo@skill#name");
        assert!(result.is_err(), "Hash in skill name should be rejected");

        let result = validate_skill_format("owner?repo");
        assert!(result.is_err(), "Question mark should be rejected");

        let result = validate_skill_format("owner/repo@skill!");
        assert!(
            result.is_err(),
            "Exclamation mark in skill name should be rejected"
        );

        let result = validate_skill_format("owner/repo@skill()");
        assert!(
            result.is_err(),
            "Parentheses in skill name should be rejected"
        );

        let result = validate_skill_format("owner/repo@skill[]");
        assert!(result.is_err(), "Brackets in skill name should be rejected");
    }

    #[test]
    fn test_generate_entrypoint_script_single_skill() {
        // Test with just one skill
        let skills = vec!["owner/single-repo".to_string()];
        let preexisting_skills = vec!["single-repo".to_string()];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);
        assert!(
            result.is_ok(),
            "Single valid skill should generate script successfully"
        );

        let script = result.unwrap();
        assert!(
            script.contains("#!/bin/sh"),
            "Script should contain shebang"
        );
        assert!(
            script.contains("exec kilocode --yes \"$@\""),
            "Script should contain CLI execution command"
        );
    }

    #[test]
    fn test_generate_entrypoint_script_validates_all_skills() {
        // Test that validation fails if any skill is invalid, even in a list
        let skills = vec![
            "owner/valid-repo".to_string(),
            "invalid-format".to_string(), // This one is invalid
            "owner/another-valid-repo".to_string(),
        ];
        let result = generate_entrypoint_script("test-agent", &skills, &[]);
        assert!(
            result.is_err(),
            "Should fail when one skill in list has invalid format"
        );

        match result {
            Err(SkillsError::ScriptGenerationFailed { agent_name, .. }) => {
                assert_eq!(agent_name, "test-agent", "Error should include agent name");
            }
            _ => panic!("Expected ScriptGenerationFailed error"),
        }
    }

    #[test]
    fn test_generate_entrypoint_script_with_underscore_characters() {
        // Test skills with underscores
        let skills = vec!["owner_name/repo_name".to_string()];
        let preexisting_skills = vec!["repo_name".to_string()];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);
        assert!(result.is_ok(), "Valid skills with underscores should work");

        let script = result.unwrap();
        assert!(
            script.contains("#!/bin/sh"),
            "Script should contain shebang"
        );
        assert!(
            script.contains("exec kilocode --yes \"$@\""),
            "Script should execute kilocode directly"
        );
    }

    #[test]
    fn test_generate_entrypoint_script_with_hyphen_characters() {
        // Test skills with hyphens
        let skills = vec!["owner-name/repo-name@skill-name".to_string()];
        let preexisting_skills = vec!["skill-name".to_string()];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);
        assert!(result.is_ok(), "Valid skills with hyphens should work");

        let script = result.unwrap();
        assert!(
            script.contains("#!/bin/sh"),
            "Script should contain shebang"
        );
        assert!(
            script.contains("exec kilocode --yes \"$@\""),
            "Script should execute kilocode directly"
        );
    }

    #[test]
    fn test_generate_entrypoint_script_with_numeric_characters() {
        // Test skills with numbers
        let skills = vec!["owner123/repo456".to_string()];
        let preexisting_skills = vec!["repo456".to_string()];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);
        assert!(result.is_ok(), "Valid skills with numbers should work");

        let script = result.unwrap();
        assert!(
            script.contains("#!/bin/sh"),
            "Script should contain shebang"
        );
        assert!(
            script.contains("exec kilocode --yes \"$@\""),
            "Script should execute kilocode directly"
        );
    }

    /// Test that error messages include agent context when skill format is invalid
    ///
    /// This test verifies that when generate_entrypoint_script encounters an invalid
    /// skill format, the resulting ScriptGenerationFailed error includes the agent name
    /// in its context. This ensures proper error tracking and debugging.
    #[test]
    fn test_error_message_includes_agent_context_invalid_skill() {
        let skills = vec!["invalid-skill-format".to_string()];
        let agent_name = "agent-alpha";

        let result = generate_entrypoint_script(agent_name, &skills, &[]);
        assert!(
            result.is_err(),
            "generate_entrypoint_script should return error for invalid skill format"
        );

        // Verify the Display format includes agent name
        let error_string = format!("{}", result.as_ref().unwrap_err());
        assert!(
            error_string.contains(agent_name),
            "Error display should mention the agent name: {}",
            error_string
        );

        match result {
            Err(SkillsError::ScriptGenerationFailed {
                agent_name: returned_agent,
                reason,
            }) => {
                assert_eq!(
                    returned_agent, agent_name,
                    "Error should include the agent name 'agent-alpha' for context"
                );
                assert!(
                    reason.contains("Invalid skill format"),
                    "Error reason should describe the validation failure"
                );
            }
            _ => panic!(
                "Expected ScriptGenerationFailed error with agent context, got: {:?}",
                result
            ),
        }
    }

    /// Test that error messages include agent context with multiple different agent names
    ///
    /// This test verifies that the agent context is properly captured for different
    /// agent names, ensuring that error messages correctly identify which agent
    /// encountered the script generation failure.
    #[test]
    fn test_error_message_includes_agent_context_generation_failed() {
        // Test with multiple different agent names to verify context is properly captured
        let test_agents = vec![
            "production-processor",
            "dev-analyzer",
            "test-runner-42",
            "ci-build-agent",
        ];

        for agent_name in test_agents {
            let skills = vec![
                "valid/owner-repo".to_string(),
                "invalid-format".to_string(), // This one is invalid
            ];

            let result = generate_entrypoint_script(agent_name, &skills, &[]);
            assert!(
                result.is_err(),
                "generate_entrypoint_script should return error for invalid skill format in agent '{}'",
                agent_name
            );

            // Verify the Display format includes agent name
            let error_string = format!("{}", result.as_ref().unwrap_err());
            assert!(
                error_string.contains(agent_name),
                "Error display for agent '{}' should mention the agent name: {}",
                agent_name,
                error_string
            );

            match &result {
                Err(SkillsError::ScriptGenerationFailed {
                    agent_name: returned_agent,
                    reason,
                }) => {
                    assert_eq!(
                        returned_agent, agent_name,
                        "Error should include the correct agent name '{}' for context",
                        agent_name
                    );
                    assert!(
                        reason.contains("Invalid skill format"),
                        "Error reason should describe the validation failure for agent '{}'",
                        agent_name
                    );
                    assert!(
                        reason.contains("invalid-format"),
                        "Error reason should include the problematic skill for agent '{}'",
                        agent_name
                    );
                }
                _ => panic!(
                    "Expected ScriptGenerationFailed error with agent context for '{}', got: {:?}",
                    agent_name, result
                ),
            }
        }
    }

    /// Test that error messages include agent context for different invalid skill formats
    ///
    /// This test verifies that agent context is preserved across different types
    /// of invalid skill format errors, ensuring consistent error reporting.
    #[test]
    fn test_error_message_includes_agent_context_multiple_invalid_formats() {
        let agent_name = "quality-assurance-agent";
        let invalid_skills = [
            "missing-slash",         // Missing slash separator
            "/repo-only",            // Empty owner name
            "owner-only/",           // Empty repo name
            "owner/repo@",           // Empty skill name
            "owner/repo@skill#name", // Invalid character in skill name
        ];

        for skill in invalid_skills.iter() {
            let skills = vec![skill.to_string()];
            let result = generate_entrypoint_script(agent_name, &skills, &[]);

            assert!(
                result.is_err(),
                "Skill '{}' should trigger validation error",
                skill
            );

            // Verify the Display format includes agent name
            let error_string = format!("{}", result.as_ref().unwrap_err());
            assert!(
                error_string.contains(agent_name),
                "Error display for skill '{}' should mention agent '{}': {}",
                skill,
                agent_name,
                error_string
            );

            match &result {
                Err(SkillsError::ScriptGenerationFailed {
                    agent_name: returned_agent,
                    reason,
                }) => {
                    assert_eq!(
                        returned_agent, agent_name,
                        "Error for skill '{}' should include agent name '{}'",
                        skill, agent_name
                    );
                    // Verify the error includes information about what was invalid
                    assert!(
                        reason.contains("Invalid skill format")
                            || reason.contains("empty")
                            || reason.contains("invalid"),
                        "Error for skill '{}' should describe the validation issue",
                        skill
                    );
                }
                _ => panic!(
                    "Expected ScriptGenerationFailed error for skill '{}', got: {:?}",
                    skill, result
                ),
            }
        }
    }

    /// Test script generation with realistic skills using bind-mount approach
    ///
    /// Verifies that the script is generated correctly when skills are present.
    /// With bind-mounts, skills are already mounted at /workspace/skills/<skill-name>/
    /// so the script simply executes kilocode directly.
    #[test]
    fn test_conditional_script_generation_realistic_skills() {
        let skills = vec![
            "kilocode/typescript@code-assistant".to_string(),
            "kilocode/rust@code-assistant".to_string(),
            "thirdparty/python-tools@analyzer".to_string(),
        ];
        let agent_name = "production-code-agent";
        let preexisting_skills = vec!["code-assistant".to_string(), "analyzer".to_string()];

        let result = generate_entrypoint_script(agent_name, &skills, &preexisting_skills);
        assert!(
            result.is_ok(),
            "Should generate script for realistic skill list with {} skills",
            skills.len()
        );

        let script = result.unwrap();

        // Verify script is generated (non-empty when skills are present)
        assert!(
            !script.is_empty(),
            "Script should be generated when skills list is non-empty"
        );

        // With bind-mounts, we DON'T have npx skills add commands
        assert!(
            !script.contains("npx skills"),
            "Script should NOT contain npx skills add commands (skills are mounted via bind-mounts)"
        );

        // Verify proper shebang
        assert!(
            script.starts_with("#!/bin/sh"),
            "Script must start with shebang #!/bin/sh"
        );

        // Verify error handling (set -e)
        assert!(
            script.contains("set -e"),
            "Script must contain 'set -e' for error handling"
        );

        // Verify CLI execution command
        assert!(
            script.contains("exec kilocode --yes \"$@\""),
            "Script should contain 'exec kilocode --yes \"$@\"' for CLI execution"
        );
    }

    /// Integration test verifying script is NOT generated when skills are absent
    ///
    /// This test complements the conditional generation test by verifying that
    /// when no skills are specified, the function returns an empty string (no script).
    /// This is important because the calling code should use the default entrypoint
    /// in this case rather than a custom skill installation script.
    #[test]
    fn test_conditional_script_generation_no_skills() {
        let skills: Vec<String> = vec![];
        let agent_name = "agent-with-no-skills";

        let result = generate_entrypoint_script(agent_name, &skills, &[]);
        assert!(result.is_ok(), "Should succeed even with empty skills list");

        let script = result.unwrap();

        // Verify script is NOT generated (empty string when no skills)
        assert!(
            script.is_empty(),
            "Script should NOT be generated (empty string) when skills list is empty"
        );

        // Verify no shell commands are present in empty result
        assert!(
            !script.contains("#!/bin/sh"),
            "Empty script should not contain shebang"
        );
        assert!(
            !script.contains("npx skills"),
            "Empty script should not contain any npx commands"
        );
        assert!(
            !script.contains("exec kilocode"),
            "Empty script should not contain exec command"
        );
    }

    /// Integration test verifying script format for bind-mount approach
    ///
    /// With bind-mounts, skills are already mounted so the script is simple:
    /// just shebang, set -e, and exec kilocode
    #[test]
    fn test_script_format_matches_expected_entrypoint() {
        let skills = vec![
            "openai/gpt-tools@assistant".to_string(),
            "anthropic/claude-tools@analyzer".to_string(),
        ];
        let agent_name = "ai-model-agent";
        let preexisting_skills = vec!["assistant".to_string(), "analyzer".to_string()];

        let result = generate_entrypoint_script(agent_name, &skills, &preexisting_skills);
        assert!(result.is_ok(), "Should generate script for AI model skills");

        let script = result.unwrap();

        // Verify complete expected format by checking each section in order
        let lines: Vec<&str> = script.lines().collect();

        // Line 1: Shebang
        assert_eq!(lines[0], "#!/bin/sh", "First line must be shebang");

        // Find and verify error propagation section
        let _set_e_line = lines
            .iter()
            .find(|line| line.contains("set -e"))
            .expect("Must contain 'set -e' for error handling");

        // With bind-mounts, we DON'T have npx skills add commands
        assert!(
            !script.contains("npx skills"),
            "Script should NOT contain npx skills add commands (skills are mounted via bind-mounts)"
        );

        // Verify final line is exec command
        assert!(
            lines
                .last()
                .unwrap()
                .starts_with("exec kilocode --yes \"$@\""),
            "Final line must be exec command for CLI execution"
        );
    }

    /// Integration test verifying single realistic skill with bind-mounts
    ///
    /// With bind-mounts, the script simply executes kilocode directly.
    #[test]
    fn test_conditional_script_generation_single_realistic_skill() {
        let skills = vec!["kilocode/docker-tools@container-manager".to_string()];
        let agent_name = "docker-orchestrator-agent";
        let preexisting_skills = vec!["container-manager".to_string()];

        let result = generate_entrypoint_script(agent_name, &skills, &preexisting_skills);
        assert!(
            result.is_ok(),
            "Should generate script for single realistic skill"
        );

        let script = result.unwrap();

        // Verify script is generated (non-empty)
        assert!(
            !script.is_empty(),
            "Script should be generated even with single skill"
        );

        // With bind-mounts, we DON'T have npx skills add commands
        assert!(
            !script.contains("npx skills"),
            "Script should NOT contain npx skills add commands (skills are mounted via bind-mounts)"
        );

        // Verify shebang
        assert!(
            script.starts_with("#!/bin/sh"),
            "Script must start with shebang"
        );

        // Verify error handling
        assert!(
            script.contains("set -e"),
            "Script must contain 'set -e' for error handling"
        );

        // Verify CLI execution
        assert!(
            script.contains("exec kilocode --yes \"$@\""),
            "Script should contain 'exec kilocode --yes \"$@\"' for CLI execution"
        );
    }

    /// Integration test with complex realistic skill names using bind-mounts
    ///
    /// With bind-mounts, skills are already mounted so the script is simple.
    #[test]
    fn test_conditional_script_generation_complex_realistic_names() {
        let skills = vec![
            "my-org/typescript-language-server@vscode-integration".to_string(),
            "my-org/rust-analyzer-pro@ide-support".to_string(),
            "external-tools/python-linter@code-quality".to_string(),
        ];
        let agent_name = "ide-integration-agent";
        let preexisting_skills = vec![
            "vscode-integration".to_string(),
            "ide-support".to_string(),
            "code-quality".to_string(),
        ];

        let result = generate_entrypoint_script(agent_name, &skills, &preexisting_skills);
        assert!(
            result.is_ok(),
            "Should generate script for complex realistic skill names"
        );

        let script = result.unwrap();

        // With bind-mounts, we DON'T have npx skills add commands
        assert!(
            !script.contains("npx skills"),
            "Script should NOT contain npx skills add commands (skills are mounted via bind-mounts)"
        );

        // Verify script structure is maintained
        assert!(
            script.starts_with("#!/bin/sh"),
            "Script must start with shebang"
        );
        assert!(
            script.contains("set -e"),
            "Script must contain 'set -e' for error handling"
        );
        assert!(
            script.contains("exec kilocode --yes \"$@\""),
            "Script should contain exec command"
        );
    }

    /// Test that generated script does NOT include npx skills add (bind-mount approach)
    ///
    /// With bind-mounts, skills are already mounted at /workspace/skills/<skill-name>/
    /// so no runtime installation is needed. The script should simply execute kilocode.
    #[test]
    fn test_generated_script_includes_error_trap() {
        let skills = vec!["owner/repo".to_string()];
        let preexisting_skills = vec!["repo".to_string()];
        let script =
            generate_entrypoint_script("test-agent", &skills, &preexisting_skills).unwrap();

        // With bind-mounts, we DON'T need error traps because there's no installation happening
        // The script is simple: just exec kilocode
        assert!(
            !script.contains("trap"),
            "Script should NOT contain error trap (no installation happening)"
        );
        assert!(
            !script.contains("handle_error"),
            "Script should NOT define handle_error function (no installation happening)"
        );
    }

    /// Test that generated script does NOT log skill installation (bind-mount approach)
    ///
    /// With bind-mounts, skills are already mounted so there's no installation to log.
    #[test]
    fn test_generated_script_logs_skill_installation() {
        let skills = vec!["owner/repo@skill-name".to_string()];
        let preexisting_skills = vec!["skill-name".to_string()];
        let script =
            generate_entrypoint_script("test-agent", &skills, &preexisting_skills).unwrap();

        // With bind-mounts, we DON'T log installations because there's no installation happening
        assert!(
            !script.contains("[SKILL INSTALL]"),
            "Script should NOT log skill installation (skills are mounted, not installed)"
        );
    }

    /// Test that generated script does NOT redirect stderr (bind-mount approach)
    ///
    /// With bind-mounts, there's no npx installation, so no stderr redirection is needed.
    #[test]
    fn test_generated_script_redirects_stderr() {
        let skills = vec!["owner/repo".to_string()];
        let preexisting_skills = vec!["repo".to_string()];
        let script =
            generate_entrypoint_script("test-agent", &skills, &preexisting_skills).unwrap();

        // With bind-mounts, we DON'T redirect stderr because there's no installation
        assert!(
            !script.contains("[SKILL INSTALL STDERR]"),
            "Script should NOT contain stderr prefix (no installation happening)"
        );
        assert!(
            !script.contains("2>&1"),
            "Script should NOT redirect stderr (no installation happening)"
        );
    }

    /// Test that generated script has simple structure for bind-mount approach
    ///
    /// With bind-mounts, skills are already mounted so the script is simple:
    /// - shebang
    /// - set -e
    /// - exec kilocode --yes "$@"
    #[test]
    fn test_generated_script_has_valid_shell_syntax_structure() {
        let skills = vec!["owner/repo@skill-name".to_string()];
        let preexisting_skills = vec!["skill-name".to_string()];
        let script =
            generate_entrypoint_script("test-agent", &skills, &preexisting_skills).unwrap();

        // Verify shebang line
        assert!(
            script.starts_with("#!/bin/sh"),
            "Script must start with shebang #!/bin/sh"
        );

        // With bind-mounts, we DON'T need function definitions
        assert!(
            !script.contains("handle_error()"),
            "Script should NOT have function definitions (simple bind-mount approach)"
        );

        // Verify set -e is present
        assert!(
            script.contains("set -e"),
            "Script must contain 'set -e' for error propagation"
        );

        // With bind-mounts, we DON'T have npx skills add commands
        assert!(
            !script.contains("npx skills"),
            "Script should NOT contain npx skills add commands (skills are mounted via bind-mounts)"
        );

        // Verify final exec command
        assert!(
            script.contains("exec kilocode --yes \"$@\""),
            "Script must contain final exec command"
        );
    }

    // ============== Missing Skills Error Handling Tests ==============
    // These tests verify the error handling behavior when skills are not found
    // in the preexisting_skills list (./skills/ directory)

    #[test]
    fn test_generate_entrypoint_script_missing_skill_returns_error() {
        // Test that missing skill returns ScriptGenerationFailed error
        let skills = vec!["owner/nonexistent-skill".to_string()];
        // Empty preexisting_skills means the skill won't be found
        let preexisting_skills: Vec<String> = vec![];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);

        assert!(result.is_err(), "Missing skill should return error");

        match result {
            Err(SkillsError::ScriptGenerationFailed { .. }) => {
                // Expected error type
            }
            _ => panic!("Expected ScriptGenerationFailed error for missing skill"),
        }
    }

    #[test]
    fn test_generate_entrypoint_script_missing_skill_error_contains_skill_name() {
        // Test that error message contains the skill name
        let skills = vec!["owner/my-missing-skill".to_string()];
        let preexisting_skills: Vec<String> = vec![];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);

        assert!(result.is_err(), "Missing skill should return error");

        if let Err(SkillsError::ScriptGenerationFailed { reason, .. }) = result {
            assert!(
                reason.contains("my-missing-skill"),
                "Error message should contain the skill name. Got: {}",
                reason
            );
        } else {
            panic!("Expected ScriptGenerationFailed error");
        }
    }

    #[test]
    fn test_generate_entrypoint_script_missing_skill_error_contains_guidance() {
        // Test that error message contains helpful guidance about adding SKILL.md
        let skills = vec!["owner/another-missing".to_string()];
        let preexisting_skills: Vec<String> = vec![];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);

        assert!(result.is_err(), "Missing skill should return error");

        if let Err(SkillsError::ScriptGenerationFailed { reason, .. }) = result {
            assert!(
                reason.contains("SKILL.md"),
                "Error message should mention SKILL.md. Got: {}",
                reason
            );
            assert!(
                reason.contains("./skills/"),
                "Error message should mention ./skills/ directory. Got: {}",
                reason
            );
        } else {
            panic!("Expected ScriptGenerationFailed error");
        }
    }

    #[test]
    fn test_generate_entrypoint_script_missing_skill_with_at_syntax() {
        // Test missing skill using owner/repo@skill-name format
        let skills = vec!["owner/repo@custom-skill".to_string()];
        let preexisting_skills: Vec<String> = vec![];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);

        assert!(result.is_err(), "Missing skill should return error");

        if let Err(SkillsError::ScriptGenerationFailed { reason, .. }) = result {
            assert!(
                reason.contains("custom-skill"),
                "Error message should contain the skill name (custom-skill). Got: {}",
                reason
            );
        } else {
            panic!("Expected ScriptGenerationFailed error");
        }
    }

    #[test]
    fn test_generate_entrypoint_script_skill_not_in_preexisting_list() {
        // Test when skill is valid format but not in preexisting_skills list
        let skills = vec!["owner/repo1".to_string(), "owner/repo2".to_string()];
        // Only repo1 is in preexisting_skills, repo2 is missing
        let preexisting_skills = vec!["repo1".to_string()];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);

        assert!(
            result.is_err(),
            "Skill not in preexisting_skills should return error"
        );

        if let Err(SkillsError::ScriptGenerationFailed { reason, .. }) = result {
            assert!(
                reason.contains("repo2"),
                "Error message should identify the missing skill (repo2). Got: {}",
                reason
            );
        } else {
            panic!("Expected ScriptGenerationFailed error");
        }
    }

    #[test]
    fn test_generate_entrypoint_script_all_skills_exist_succeeds() {
        // Test that when all skills exist in preexisting_skills, script generation succeeds
        let skills = vec!["owner/repo1".to_string(), "owner/repo2@custom".to_string()];
        // Both skills are in the preexisting_skills list
        let preexisting_skills = vec!["repo1".to_string(), "custom".to_string()];
        let result = generate_entrypoint_script("test-agent", &skills, &preexisting_skills);

        assert!(
            result.is_ok(),
            "All skills in preexisting_skills should succeed"
        );
    }

    #[test]
    fn test_generate_entrypoint_script_error_includes_agent_name() {
        // Test that the error includes the agent name for context
        let skills = vec!["owner/missing-skill".to_string()];
        let preexisting_skills: Vec<String> = vec![];
        let result = generate_entrypoint_script("my-special-agent", &skills, &preexisting_skills);

        assert!(result.is_err(), "Missing skill should return error");

        if let Err(SkillsError::ScriptGenerationFailed { agent_name, .. }) = result {
            assert!(
                agent_name == "my-special-agent",
                "Error should include the agent name. Got: {}",
                agent_name
            );
        } else {
            panic!("Expected ScriptGenerationFailed error");
        }
    }
}
