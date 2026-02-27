//! Skill validation functions
//!
//! This module contains pure validation functions for skill source strings.
//! These functions validate that skill strings conform to the expected format
//! (either `owner/repo` or `owner/repo@skill-name`).

use crate::skills::SkillsError;

/// Extracts the skill name from a skill source string.
///
/// This function parses skill strings in two formats:
/// - `owner/repo@skill-name` - returns the skill-name portion
/// - `owner/repo` - returns the repo portion as the skill name
///
/// # Arguments
///
/// * `skill` - The skill source string to parse
///
/// # Returns
///
/// * `Ok(String)` - The extracted skill name
/// * `Err(SkillsError)` - If the skill format is invalid
pub fn extract_skill_name(skill: &str) -> Result<String, SkillsError> {
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
/// * `Ok(())` - If the skill string is valid
/// * `Err(SkillsError::InvalidSkillFormat)` - If the skill string is invalid
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
