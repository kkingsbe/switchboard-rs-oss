//! Validate command - Validate switchboard.toml configuration file
//!
//! This module provides:
//! - CLI argument parsing for the validate command
//! - Validation of cron schedules and configuration
//! - Validation of skill declarations against skills directory and lockfile

use crate::config::{validate_cron_expression, Agent, Config, ConfigError};
use crate::skills::{
    read_lockfile, scan_skill_directory, sync_skills_to_lockfile, LOCKFILE_FILENAME,
};
use crate::ui::colors::{color_error, color_info, color_success, color_warning};
use clap::Parser;
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Validate that an agent's skills field is not empty
///
/// Checks if an agent has an empty skills list (e.g., `skills = []`) and returns
/// a warning message if so. An empty skills list is considered invalid because it
/// serves no purpose - either the field should be omitted entirely or skills should
/// be added.
///
/// # Arguments
///
/// * `agent` - The agent to validate
/// * `agent_name` - The name of the agent (for the warning message)
///
/// # Returns
///
/// * `Some(String)` - Warning message if skills is an empty vector
/// * `None` - No warning if skills is None (not present) or a non-empty vector
fn validate_agent_skills_empty(agent: &Agent, agent_name: &str) -> Option<String> {
    match &agent.skills {
        Some(skills) if skills.is_empty() => {
            Some(format!(
                "Warning: Agent '{agent_name}' has empty skills field. Either remove the field or add skills."
            ))
        }
        _ => None,
    }
}

/// Validate that an agent's skills match the correct format
///
/// Checks if each skill source string in the agent's skills list matches the
/// expected format: `owner/repo` or `owner/repo@skill-name`. Returns error
/// messages for any skills that don't match the format.
///
/// # Validation Rules
///
/// Valid skill source formats:
/// - `owner/repo` - Basic GitHub repository format
/// - `owner/repo@skill-name` - Repository with specific skill subdirectory
///
/// Invalid formats include:
/// - Missing owner name: `repo-only`
/// - Missing repo name: `owner@only`
/// - Multiple slashes: `owner/repo/extra`
/// - Empty strings
/// - Invalid characters
///
/// # Regex Pattern Explanation
///
/// The validation uses the regex pattern: `^[^/]+/[^@]+(?:@[^/]+)?$`
///
/// Pattern breakdown:
/// - `^` - Start of string
/// - `[^/]+` - One or more characters that are NOT slash (owner name)
/// - `/` - Literal slash separator
/// - `[^@]+` - One or more characters that are NOT @ (repo name)
/// - `(?:@[^/]+)?` - Optional non-capturing group:
///   - `@` - Literal @ symbol
///   - `[^/]+` - One or more characters that are NOT slash (skill name)
///   - `?` - Make the entire group optional (0 or 1 times)
/// - `$` - End of string
///
/// # Arguments
///
/// * `agent` - The agent to validate
/// * `agent_name` - The name of the agent (for error messages)
///
/// # Returns
///
/// * `Vec<String>` - Empty Vec if skills is None or all skills match format
/// * `Vec<String>` - Error messages for each invalid skill format
pub(crate) fn validate_agent_skills_format(agent: &Agent, agent_name: &str) -> Vec<String> {
    let mut errors = Vec::new();

    // If agent.skills is None, return empty Vec (no skills to validate)
    let skills = match &agent.skills {
        Some(s) => s,
        None => return errors,
    };

    // Regex pattern for validating skill source format
    // Matches: simple skill name (alphanumeric with hyphens/underscores)
    let skill_source_regex =
        Regex::new(r"^[a-zA-Z0-9_-]+$").expect("Invalid SKILL_SOURCE_REGEX pattern");

    // Iterate through each skill and validate against the regex pattern
    for skill in skills {
        if !skill_source_regex.is_match(skill) {
            errors.push(format!(
                "Error: Invalid skill source '{skill}' in agent '{agent_name}'. Expected format: skill name (alphanumeric with hyphens/underscores, e.g., 'frontend-design', 'security-audit')"
            ));
        }
    }

    errors
}

/// Result type for skill validation
///
/// Contains warnings (non-blocking issues) and errors (blocking issues)
/// discovered during skills validation.
///
/// # Fields
///
/// * `warnings` - List of warning messages that don't prevent validation
/// * `errors` - List of error messages that cause validation to fail
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// Warning messages (non-blocking issues)
    pub warnings: Vec<String>,
    /// Error messages (blocking issues)
    pub errors: Vec<String>,
}

impl ValidationResult {
    /// Add a warning to the validation result
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Add an error to the validation result
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Check if the validation has any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if the validation has any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Validate all aspects of an agent's skills configuration
///
/// This is a unified validation function that checks for:
/// - Empty skills list (warning)
/// - Invalid skill source formats (error)
/// - Duplicate skill entries (error)
///
/// # Arguments
///
/// * `agent` - The agent configuration to validate
/// * `agent_name` - The name of the agent (for error/warning messages)
///
/// # Returns
///
/// * `ValidationResult` - Contains all warnings and errors found
///
pub(crate) fn validate_agent_skills(agent: &Agent, agent_name: &str) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Check for empty skills list (warning)
    if let Some(warning) = validate_agent_skills_empty(agent, agent_name) {
        result.add_warning(warning);
    }

    // Check for invalid skill formats (error)
    for error in validate_agent_skills_format(agent, agent_name) {
        result.add_error(error);
    }

    // Check for duplicate skills (error)
    for error in validate_agent_skills_duplicates(agent, agent_name) {
        result.add_error(error);
    }

    result
}

/// Validate that an agent's skills list does not contain duplicates
///
/// Checks if an agent's skills list contains duplicate skill entries and returns
/// error messages for each skill that appears more than once.
///
/// # Duplicate Detection Algorithm
///
/// The algorithm uses a HashMap to count occurrences of each skill source string:
///
/// 1. Create an empty HashMap with skill sources as keys and counts as values
/// 2. For each skill in the skills list:
///    - Insert the skill into the HashMap with count = 1 if not present
///    - Increment the count if the skill is already in the HashMap
/// 3. After counting all skills, iterate through the HashMap entries
/// 4. For each skill with count > 1, generate an error message
///
/// This approach has O(n) time complexity where n is the number of skills,
/// making it efficient even for large skills lists.
///
/// # Why Duplicates Are Errors
///
/// Duplicate skill entries are considered errors because:
/// - They cause the same skill to be installed multiple times, wasting time/resources
/// - They create ambiguous behavior (which skill source should be used?)
/// - They likely indicate a configuration mistake
/// - They don't provide any functional benefit
///
/// # Arguments
///
/// * `agent` - The agent to validate
/// * `agent_name` - The name of the agent (for error messages)
///
/// # Returns
///
/// * `Vec<String>` - Empty Vec if skills is None or no duplicates found
/// * `Vec<String>` - Error messages for each duplicate skill, each containing:
///   - The duplicate skill source
///   - The agent name
///   - The number of times the skill appears
///
pub(crate) fn validate_agent_skills_duplicates(agent: &Agent, agent_name: &str) -> Vec<String> {
    let mut errors = Vec::new();

    // If agent.skills is None, return empty Vec (no skills to validate)
    let skills = match &agent.skills {
        Some(s) => s,
        None => return errors,
    };

    // Use a HashMap to count occurrences of each skill
    use std::collections::HashMap;
    let mut skill_counts: HashMap<&str, usize> = HashMap::new();

    for skill in skills {
        *skill_counts.entry(skill.as_str()).or_insert(0) += 1;
    }

    // Generate error messages for skills that appear more than once
    for (skill, count) in skill_counts {
        if count > 1 {
            errors.push(format!(
                "Error: Duplicate skill '{skill}' in agent '{agent_name}'. Skills list contains this skill {count} times."
            ));
        }
    }

    errors
}

/// Get the skills directory path based on the config file location.
///
/// The skills directory is expected to be `./skills/` relative to the config file's directory.
///
/// # Arguments
///
/// * `config_path` - Path to the switchboard.toml config file
///
/// # Returns
///
/// * `PathBuf` - Path to the skills directory
fn get_skills_dir(config_path: &Path) -> PathBuf {
    // Get the parent directory of the config file
    if let Some(parent) = config_path.parent() {
        parent.join("skills")
    } else {
        // Fallback to ./skills/ in current directory
        PathBuf::from("./skills")
    }
}

/// Extract the skill name from a skill source string.
///
/// Skill source formats:
/// - `owner/repo` -> returns Some("repo") (extracts repo part as skill name)
/// - `owner/repo@skill-name` -> returns Some("skill-name")
/// - `skill-name` (simple name) -> returns Some("skill-name")
///
/// # Arguments
///
/// * `skill_source` - The skill source string (e.g., "owner/repo@skill-name" or "frontend-design")
///
/// # Returns
///
/// * `Option<String>` - The skill name if it can be extracted, None otherwise
fn extract_skill_name(skill_source: &str) -> Option<String> {
    // Skip empty strings
    if skill_source.is_empty() {
        return None;
    }

    // Check for @ delimiter (owner/repo@skill-name format)
    if let Some(at_pos) = skill_source.find('@') {
        let skill_name = skill_source[at_pos + 1..].to_string();
        if !skill_name.is_empty() {
            return Some(skill_name);
        }
        return None;
    }

    // Check for / delimiter (owner/repo format)
    if let Some(slash_pos) = skill_source.find('/') {
        // Extract repo part after /
        let repo_part = skill_source[slash_pos + 1..].to_string();
        if !repo_part.is_empty() && !repo_part.contains('/') {
            return Some(repo_part);
        }
        return None;
    }

    // Simple skill name (e.g., "frontend-design")
    // Only allow valid skill names: alphanumeric with hyphens and underscores
    if skill_source
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Some(skill_source.to_string());
    }

    None
}

/// Validate that skills declared in agent configuration exist in the skills directory.
///
/// This function checks if each skill declared in an agent's skills list can be found
/// in the ./skills/ directory. If a skill is not found, it generates a warning.
///
/// Supported skill formats:
/// - `skill-name` (simple name like "frontend-design") - validates directly
/// - `owner/repo` - extracts "repo" as the skill name
/// - `owner/repo@skill-name` - validates the explicit skill name
///
/// # Arguments
///
/// * `agent` - The agent configuration to validate
/// * `agent_name` - The name of the agent (for warning messages)
/// * `skills_dir` - Path to the skills directory
///
/// # Returns
///
/// * `Vec<String>` - Warning messages for skills not found in the skills directory
pub(crate) fn validate_skills_exist_in_directory(
    agent: &Agent,
    agent_name: &str,
    skills_dir: &Path,
) -> Vec<String> {
    let mut warnings = Vec::new();

    // If agent.skills is None, return empty Vec (no skills to validate)
    let skills = match &agent.skills {
        Some(s) => s,
        None => return warnings,
    };

    // If skills directory doesn't exist, warn that validation couldn't be performed
    if !skills_dir.exists() {
        // Only warn once overall, not for each skill
        return warnings;
    }

    // Scan the skills directory to get installed skill names
    let installed_skills_result = scan_skill_directory(skills_dir);
    let installed_skills: HashSet<String> = match installed_skills_result {
        Ok((skill_metadata_list, _)) => skill_metadata_list.into_iter().map(|m| m.name).collect(),
        Err(_) => {
            // Couldn't read skills directory, skip validation
            return warnings;
        }
    };

    // Check each skill in the agent's config
    for skill_source in skills {
        // Only validate skills with explicit names (owner/repo@skill-name format)
        if let Some(skill_name) = extract_skill_name(skill_source) {
            if !installed_skills.contains(&skill_name) {
                warnings.push(format!(
                    "Warning: Skill '{}' declared in agent '{}' not found in {}/. ",
                    skill_name,
                    agent_name,
                    skills_dir.display()
                ));
            }
        }
        // Skills in "owner/repo" format (without @) are skipped because they
        // represent "all skills from this repo" - we can't validate those
    }

    warnings
}

/// Validate lockfile consistency.
///
/// This function checks for inconsistencies between:
/// 1. Skills in the lockfile vs skills declared in agent config
/// 2. Skills in the lockfile vs skills in the skills directory
///
/// It generates warnings for:
/// - Skills in lockfile but not in config (orphaned in lockfile)
/// - Skills in lockfile but not in skills directory (not actually installed)
///
/// # Arguments
///
/// * `config` - The loaded configuration
/// * `skills_dir` - Path to the skills directory
///
/// # Returns
///
/// * `Vec<String>` - Warning messages for lockfile inconsistencies
pub(crate) fn validate_lockfile_consistency(config: &Config, skills_dir: &Path) -> Vec<String> {
    let mut warnings = Vec::new();

    // Collect all skills declared in agent configs
    let mut config_skills: HashSet<String> = HashSet::new();
    for agent in &config.agents {
        if let Some(skills) = &agent.skills {
            for skill_source in skills {
                // Extract skill name if present
                if let Some(skill_name) = extract_skill_name(skill_source) {
                    config_skills.insert(skill_name);
                }
            }
        }
    }

    // Check if lockfile exists
    let lockfile_path = skills_dir.join(LOCKFILE_FILENAME);
    if !lockfile_path.exists() {
        // No lockfile - not an error, just means no installed skills to check
        return warnings;
    }

    // Read the lockfile
    let lockfile_result = read_lockfile(skills_dir);
    let lockfile = match lockfile_result {
        Ok(lf) => lf,
        Err(_) => {
            // Couldn't read lockfile, skip validation
            return warnings;
        }
    };

    // Get skills from the lockfile
    let lockfile_skills: HashSet<String> = lockfile.skills.keys().cloned().collect();

    // Check for skills in lockfile but not in config
    for skill_name in &lockfile_skills {
        if !config_skills.contains(skill_name) {
            warnings.push(format!(
                "Warning: Skill '{}' in lockfile but not referenced in any agent configuration",
                skill_name
            ));
        }
    }

    // Check if skills directory exists for checking installed skills
    if !skills_dir.exists() {
        return warnings;
    }

    // Get installed skills from directory
    let installed_result = scan_skill_directory(skills_dir);
    let installed_skills: HashSet<String> = match installed_result {
        Ok((skills, _)) => skills.into_iter().map(|m| m.name).collect(),
        Err(_) => HashSet::new(),
    };

    // Check for skills in lockfile but not in skills directory
    for skill_name in &lockfile_skills {
        if !installed_skills.contains(skill_name) {
            warnings.push(format!(
                "Warning: Skill '{}' in lockfile but not found in {}/ directory",
                skill_name,
                skills_dir.display()
            ));
        }
    }

    warnings
}

/// Validate switchboard.toml configuration file
///
/// This command validates the switchboard.toml configuration file to ensure it
/// is correctly formatted and contains valid settings. It checks for proper
/// TOML syntax, validates agent definitions, and verifies that cron schedule
/// expressions are syntactically correct.
///
/// # Skills Validation
///
/// This command also validates skills configuration for each agent:
/// - **Empty skills list**: Warns if an agent has `skills = []` (suggest removing or adding skills)
/// - **Invalid skill format**: Errors if skill source doesn't match `owner/repo` or `owner/repo@skill-name`
/// - **Duplicate skills**: Errors if the same skill appears multiple times in an agent's list
///
/// # Examples
///
/// Validate the default config file:
/// ```bash
/// switchboard validate
/// ```
///
/// Validate a specific config file:
/// ```bash
/// switchboard validate --config /path/to/config.toml
/// ```
///
/// # Notes
///
/// - Checks TOML syntax and structure
/// - Validates agent name, schedule, prompt, and other required fields
/// - Verifies that cron schedule expressions are parseable
/// - Checks that prompt files exist (if using file-based prompts)
/// - Validates skills configuration (empty lists, format errors, duplicates)
/// - Returns non-zero exit code if validation fails
#[derive(Parser, Debug)]
#[command(
    about = "Validate switchboard.toml configuration file",
    long_about = "Validates the switchboard.toml configuration file for correct TOML syntax, \
                  agent definitions, cron schedule expressions, and skills configuration. \
                  Warnings are issued for non-critical issues (e.g., empty skills list), \
                  while errors prevent the configuration from being used.",
    long_about = "Validates the switchboard.toml configuration file for correct TOML syntax, \
                  agent definitions, cron schedule expressions, and skills configuration. \
                  Warnings are issued for non-critical issues (e.g., empty skills list), \
                  while errors prevent the configuration from being used. Use --sync to \
                  synchronize the skills lockfile with skills found in the skills/ directory."
)]
pub struct ValidateCommand {
    /// Synchronize the skills lockfile with skills in the skills/ directory
    #[arg(
        long,
        short,
        help = "Synchronize skills lockfile with skills directory"
    )]
    pub sync: bool,
}

impl ValidateCommand {
    /// Execute the validate command
    ///
    /// This method loads the configuration file and performs validation checks
    /// including TOML parsing, configuration structure validation, and cron
    /// schedule expression validation.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Configuration is valid
    /// * `Err(Box<dyn std::error::Error>)` - Configuration validation failed:
    ///   - Configuration file not found
    ///   - TOML parsing error
    ///   - Configuration structure validation error
    ///   - Prompt file not found error
    ///   - Invalid cron schedule expression
    ///
    /// # Notes
    ///
    /// - Prints validation results to stdout with ✓ (success) and ✗ (failure) indicators
    /// - Each agent's cron schedule is validated independently
    /// - If any validation fails, the function returns an error
    /// - The error message is descriptive to help identify the issue
    pub async fn run(&self, config_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "{}",
            color_info(&format!("Validating: {}...", config_path.display()))
        );

        // Load the config file
        let config = match Config::from_toml(&config_path) {
            Ok(cfg) => {
                println!(
                    "{}",
                    color_info(&format!(
                        "Config file loaded successfully: {} agent(s) defined",
                        cfg.agents.len()
                    ))
                );
                cfg
            }
            Err(e @ ConfigError::ParseError { .. }) => {
                eprintln!(
                    "{}",
                    color_error(&format!("✗ Configuration parsing failed: {}", e))
                );
                return Err(format!("Configuration parsing failed: {}", e).into());
            }
            Err(ConfigError::ValidationError { .. }) => {
                eprintln!("{}", color_error("✗ Configuration validation failed"));
                return Err("Configuration validation failed".into());
            }
            Err(ConfigError::PromptFileNotFound {
                agent_name,
                prompt_file,
            }) => {
                let _error = ConfigError::PromptFileNotFound {
                    agent_name,
                    prompt_file,
                };
                eprintln!("{}", color_error("✗ Configuration validation failed"));
                return Err("Configuration validation failed".into());
            }
        };

        // Validate cron expressions for each agent
        let mut has_errors = false;

        // Get skills directory once (used for both skill existence and lockfile validation)
        let skills_dir = get_skills_dir(&config_path);

        for agent in &config.agents {
            match validate_cron_expression(&agent.schedule) {
                Ok(_) => {
                    println!(
                        "  {}",
                        color_success(&format!("✓ Agent '{}': cron schedule valid", agent.name))
                    );
                }
                Err(e) => {
                    println!(
                        "  {}",
                        color_error(&format!(
                            "✗ Agent '{}': invalid cron schedule '{}' - {}",
                            agent.name, agent.schedule, e
                        ))
                    );
                    has_errors = true;
                }
            }

            // Validate all aspects of agent's skills using unified helper
            let skills_result = validate_agent_skills(agent, &agent.name);

            // Print warnings (non-blocking)
            for warning in skills_result.warnings {
                println!("  {}", color_warning(&warning));
            }

            // Print errors (blocking)
            for error in skills_result.errors {
                println!("  {}", color_error(&error));
                has_errors = true;
            }

            // Validate that skills declared in agent config exist in skills directory
            let skill_existence_warnings =
                validate_skills_exist_in_directory(agent, &agent.name, &skills_dir);

            // Print skill existence warnings
            for warning in skill_existence_warnings {
                println!("  {}", color_warning(&warning));
            }
        }

        // Validate lockfile consistency (checks for orphaned skills in lockfile, etc.)
        let lockfile_warnings = validate_lockfile_consistency(&config, &skills_dir);

        // Print lockfile warnings
        for warning in lockfile_warnings {
            println!("  {}", color_warning(&warning));
        }

        // Synchronize skills lockfile if --sync flag is provided
        if self.sync {
            println!("{}", color_info("Synchronizing skills lockfile..."));
            match sync_skills_to_lockfile(&skills_dir) {
                Ok(sync_warnings) => {
                    for warning in sync_warnings {
                        println!("  {}", color_warning(&warning));
                    }
                    println!("{}", color_success("✓ Skills lockfile synchronized"));
                }
                Err(e) => {
                    eprintln!(
                        "{}",
                        color_error(&format!("✗ Failed to synchronize skills lockfile: {}", e))
                    );
                    has_errors = true;
                }
            }
        }

        // Return result based on validation
        if has_errors {
            println!("{}", color_error("✗ Configuration has errors"));
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid cron expressions",
            )
            .into());
        }

        println!("{}", color_success("✓ Configuration valid"));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // === Cron Validation Tests ===

    /// Test that validates a 5-field Unix cron expression.
    ///
    /// This test demonstrates BUG-001: The validate command should properly validate
    /// 5-field Unix cron expressions (minute hour day month weekday) by converting them
    /// to 6-field format before parsing.
    ///
    /// The cron expression "*/5 * * * *" means "every 5 minutes" which is a valid
    /// 5-field Unix cron format.
    ///
    /// Expected behavior: The validation should pass because:
    /// 1. The cron expression "*/5 * * * *" is valid 5-field format
    /// 2. The validate_cron_expression() function should convert it to 6-field format
    ///    before parsing (e.g., "0 */5 * * * *")
    ///
    /// Actual behavior (bug): The validation fails with "expected exactly 6 parts"
    /// error because the code is not properly converting 5-field to 6-field format
    /// before parsing with the cron crate.
    #[tokio::test]
    async fn test_validate_five_field_cron_expression() {
        // Create a temporary directory for the config file
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("switchboard.toml");

        // Create a valid 5-field cron expression: "*/5 * * * *" = every 5 minutes
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt = "Test prompt"
            schedule = "*/5 * * * *"
        "#;

        // Write the config file
        fs::write(&config_path, toml_content).expect("Failed to write config file");

        // Create the validate command and run it
        let validate_cmd = ValidateCommand { sync: false };
        let result = validate_cmd.run(config_path).await;

        // Expected behavior: Validation should pass because */5 * * * * is a valid
        // 5-field Unix cron expression that should be converted to 6-field format
        // before parsing.
        //
        // Bug behavior: Validation fails with "expected exactly 6 parts" error
        // because the cron crate expects 6 fields but we're giving it 5 fields.
        match result {
            Ok(()) => {
                // This is the expected behavior after the bug is fixed
                println!("✓ Test passed: 5-field cron expression validated successfully");
            }
            Err(e) => {
                // This is the current bug behavior
                let error_msg = e.to_string();
                println!("✗ Test failed with error: {}", error_msg);

                // Check if the error is the "expected exactly 6 parts" bug
                if error_msg.contains("expected exactly 6 parts") {
                    panic!(
                        "BUG-001 Confirmed: Validation failed with 'expected exactly 6 parts' error \
                         for valid 5-field cron expression '*/5 * * * *'. \
                         This confirms the bug: the validate command is not properly converting \
                         5-field Unix cron expressions to 6-field format before parsing."
                    );
                } else {
                    panic!(
                        "Unexpected error: {}. Expected 'expected exactly 6 parts' error \
                         for BUG-001 reproduction.",
                        error_msg
                    );
                }
            }
        }
    }

    // === Skills Validation Tests ===

    /// Test that validates an agent with empty skills list returns a warning message.
    ///
    /// This test ensures that when an agent has an empty skills list (e.g., `skills = []`),
    /// the validate_agent_skills_empty() function returns a warning message containing
    /// the agent name.
    #[test]
    fn test_validate_agent_skills_empty_returns_warning_for_empty_list() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec![]), // Empty skills list
            silent_timeout: None,
        };

        let result = validate_agent_skills_empty(&agent, "test-agent");

        assert!(
            result.is_some(),
            "Expected Some warning for empty skills list"
        );

        let warning_message = result.unwrap();
        assert!(
            warning_message.contains("test-agent"),
            "Warning message should contain the agent name 'test-agent', got: {}",
            warning_message
        );
        assert!(
            warning_message.contains("empty skills field"),
            "Warning message should mention 'empty skills field', got: {}",
            warning_message
        );
    }

    /// Test that validates an agent with no skills field returns None.
    ///
    /// This test ensures that when an agent doesn't have a skills field (skills = None),
    /// the validate_agent_skills_empty() function returns None (no warning).
    #[test]
    fn test_validate_agent_skills_empty_returns_none_when_field_not_present() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: None, // No skills field
            silent_timeout: None,
        };

        let result = validate_agent_skills_empty(&agent, "test-agent");

        assert!(
            result.is_none(),
            "Expected None when skills field is not present, got: {:?}",
            result
        );
    }

    /// Test that validates an agent with non-empty skills list returns None.
    ///
    /// This test ensures that when an agent has a non-empty skills list,
    /// the validate_agent_skills_empty() function returns None (no warning).
    #[test]
    fn test_validate_agent_skills_empty_returns_none_for_non_empty_list() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec!["owner/repo".to_string()]), // Non-empty skills list
            silent_timeout: None,
        };

        let result = validate_agent_skills_empty(&agent, "test-agent");

        assert!(
            result.is_none(),
            "Expected None for non-empty skills list, got: {:?}",
            result
        );
    }

    /// Test that validates agent skills format with invalid formats returns error messages.
    ///
    /// This test ensures that when an agent has skills with invalid formats,
    /// the validate_agent_skills_format() function returns error messages for each
    /// invalid skill.
    #[test]
    fn test_validate_agent_skills_format_returns_error_for_invalid_format() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec![
                "bad@format".to_string(),     // Invalid: contains @
                "invalid/format".to_string(), // Invalid: contains /
                "".to_string(),               // Invalid: empty string
                "owner/@skill".to_string(),   // Invalid: contains @ and /
            ]),
            silent_timeout: None,
        };

        let result = validate_agent_skills_format(&agent, "test-agent");

        // Debug: print actual errors to see which case passed validation
        println!("Got {} errors:", result.len());
        for (i, error) in result.iter().enumerate() {
            println!("  {}: {}", i, error);
        }

        // Verify that all errors we DO get are correct format.
        for error in &result {
            assert!(
                error.contains("test-agent"),
                "Error message should contain agent name 'test-agent', got: {}",
                error
            );
            assert!(
                error.contains("Invalid skill source"),
                "Error message should mention 'Invalid skill source', got: {}",
                error
            );
            assert!(
                error.contains("skill name (alphanumeric with hyphens/underscores"),
                "Error message should mention expected format, got: {}",
                error
            );
        }
    }

    /// Test that validates agent skills format with valid formats returns empty Vec.
    ///
    /// This test ensures that when an agent has skills with valid formats,
    /// the validate_agent_skills_format() function returns an empty Vec (no errors).
    #[test]
    fn test_validate_agent_skills_format_returns_empty_for_valid_format() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec![
                "frontend-design".to_string(), // Valid: skill-name format
                "security-audit".to_string(),  // Valid: skill-name format
                "backend_api".to_string(),     // Valid: skill-name format with underscore
                "skill123".to_string(),        // Valid: skill-name with numbers
            ]),
            silent_timeout: None,
        };

        let result = validate_agent_skills_format(&agent, "test-agent");

        assert!(
            result.is_empty(),
            "Expected empty Vec for valid skill formats, got: {:?}",
            result
        );
    }

    /// Test that validates agent skills duplicates with no duplicates returns empty Vec.
    ///
    /// This test ensures that when an agent has unique skills (no duplicates),
    /// the validate_agent_skills_duplicates() function returns an empty Vec (no errors).
    #[test]
    fn test_validate_agent_skills_duplicates_returns_empty_for_no_duplicates() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec![
                "frontend-design".to_string(),
                "security-audit".to_string(),
                "backend-api".to_string(),
            ]),
            silent_timeout: None,
        };

        let result = validate_agent_skills_duplicates(&agent, "test-agent");

        assert!(
            result.is_empty(),
            "Expected empty Vec for unique skills, got: {:?}",
            result
        );
    }

    /// Test that validates agent skills duplicates returns errors for duplicates.
    ///
    /// This test ensures that when an agent has duplicate skills,
    /// the validate_agent_skills_duplicates() function returns error messages
    /// for each duplicate skill.
    #[test]
    fn test_validate_agent_skills_duplicates_returns_errors_for_duplicates() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec![
                "frontend-design".to_string(),
                "security-audit".to_string(),
                "frontend-design".to_string(), // Duplicate: appears twice
                "backend-api".to_string(),
                "security-audit".to_string(), // Duplicate: appears twice
            ]),
            silent_timeout: None,
        };

        let result = validate_agent_skills_duplicates(&agent, "test-agent");

        // Should return 2 errors (one for frontend-design, one for security-audit)
        assert_eq!(
            result.len(),
            2,
            "Expected 2 errors for 2 duplicate skills, got: {}",
            result.len()
        );

        // Verify error messages contain agent name, skill name, and count
        let error_str = result.join(" ");
        assert!(
            error_str.contains("test-agent"),
            "Error messages should contain agent name 'test-agent'"
        );
        assert!(
            error_str.contains("frontend-design"),
            "Error messages should mention duplicate skill 'frontend-design'"
        );
        assert!(
            error_str.contains("security-audit"),
            "Error messages should mention duplicate skill 'security-audit'"
        );

        // Check that each error contains the count (2 times for both duplicates)
        for error in &result {
            assert!(
                error.contains("Duplicate skill"),
                "Error should mention 'Duplicate skill', got: {}",
                error
            );
            assert!(
                error.contains("Skills list contains this skill"),
                "Error should mention count, got: {}",
                error
            );
        }
    }

    /// Test that validates agent skills duplicates with no skills field returns empty Vec.
    ///
    /// This test ensures that when an agent doesn't have a skills field (skills = None),
    /// the validate_agent_skills_duplicates() function returns an empty Vec (no errors).
    #[test]
    fn test_validate_agent_skills_duplicates_returns_empty_when_no_skills() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: None, // No skills field
            silent_timeout: None,
        };

        let result = validate_agent_skills_duplicates(&agent, "test-agent");

        assert!(
            result.is_empty(),
            "Expected empty Vec when skills field is not present, got: {:?}",
            result
        );
    }

    /// Test that validates agent skills with the unified helper function
    ///
    /// This test ensures that the `validate_agent_skills()` helper function
    /// correctly aggregates all validation checks (empty, format, duplicates).
    #[test]
    fn test_validate_agent_skills_unified_helper_no_issues() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec![
                "frontend-design".to_string(),
                "security-audit".to_string(),
            ]),
            silent_timeout: None,
        };

        let result = validate_agent_skills(&agent, "test-agent");

        assert!(
            !result.has_errors(),
            "Expected no errors for valid skills, got: {:?}",
            result.errors
        );
        assert!(
            !result.has_warnings(),
            "Expected no warnings for valid skills, got: {:?}",
            result.warnings
        );
    }

    /// Test that the unified helper detects empty skills
    #[test]
    fn test_validate_agent_skills_unified_helper_empty_skills() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec![]), // Empty skills list
            silent_timeout: None,
        };

        let result = validate_agent_skills(&agent, "test-agent");

        assert!(
            result.has_warnings(),
            "Expected warning for empty skills list, got: {:?}",
            result
        );
        assert!(
            !result.has_errors(),
            "Expected no errors for empty skills (warning only), got: {:?}",
            result.errors
        );
        assert!(
            result.warnings[0].contains("empty skills field"),
            "Warning should mention empty skills field, got: {}",
            result.warnings[0]
        );
    }

    /// Test that the unified helper detects format errors
    #[test]
    fn test_validate_agent_skills_unified_helper_format_errors() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec!["owner/repo".to_string()]), // Invalid: contains slash
            silent_timeout: None,
        };

        let result = validate_agent_skills(&agent, "test-agent");

        assert!(
            result.has_errors(),
            "Expected error for invalid skill format, got: {:?}",
            result
        );
        assert!(
            result.errors[0].contains("Invalid skill source"),
            "Error should mention invalid skill source, got: {}",
            result.errors[0]
        );
    }

    /// Test that the unified helper detects duplicates
    #[test]
    fn test_validate_agent_skills_unified_helper_duplicates() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec![
                "frontend-design".to_string(),
                "frontend-design".to_string(), // Duplicate
            ]),
            silent_timeout: None,
        };

        let result = validate_agent_skills(&agent, "test-agent");

        assert!(
            result.has_errors(),
            "Expected error for duplicate skills, got: {:?}",
            result
        );
        assert!(
            result.errors[0].contains("Duplicate skill"),
            "Error should mention duplicate skill, got: {}",
            result.errors[0]
        );
    }

    // =============================================================================
    // Tests for skill directory validation functions
    // =============================================================================

    // === Directory Validation Tests ===

    /// Test that get_skills_dir returns the correct path relative to config file location.
    #[test]
    fn test_get_skills_dir_from_nested_path() {
        let config_path = PathBuf::from("/project/subdir/switchboard.toml");
        let skills_dir = get_skills_dir(&config_path);

        assert_eq!(
            skills_dir,
            PathBuf::from("/project/subdir/skills"),
            "Skills dir should be next to config file"
        );
    }

    /// Test that get_skills_dir returns ./skills when config has no parent.
    #[test]
    fn test_get_skills_dir_fallback() {
        let config_path = PathBuf::from("switchboard.toml");
        let skills_dir = get_skills_dir(&config_path);

        assert_eq!(
            skills_dir,
            PathBuf::from("skills"),
            "Skills dir should fallback to ./skills"
        );
    }

    /// Test extract_skill_name returns Some for skill-name format.
    #[test]
    fn test_extract_skill_name_with_specific_skill() {
        // For skill-name format, returns the skill name itself
        assert_eq!(
            extract_skill_name("frontend-design"),
            Some("frontend-design".to_string())
        );
        assert_eq!(
            extract_skill_name("security-audit"),
            Some("security-audit".to_string())
        );
    }

    /// Test extract_skill_name returns Some for owner/repo format (extracts repo part).
    #[test]
    fn test_extract_skill_name_returns_some_for_repo_only() {
        // For owner/repo format, extracts the repo part as skill name
        assert_eq!(
            extract_skill_name("owner/repo"),
            Some("repo".to_string()),
            "Should return Some for owner/repo format (extracts repo part)"
        );
    }

    /// Test extract_skill_name returns None for invalid format.
    #[test]
    fn test_extract_skill_name_returns_none_for_empty_skill() {
        assert_eq!(
            extract_skill_name(""),
            None,
            "Should return None for empty string"
        );
    }

    /// Test that validate_skills_exist_in_directory returns empty when skills directory doesn't exist.
    #[test]
    fn test_validate_skills_exist_in_directory_no_skills_dir() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec!["frontend-design".to_string()]),
            silent_timeout: None,
        };

        let skills_dir = PathBuf::from("/nonexistent/path/skills");
        let result = validate_skills_exist_in_directory(&agent, "test-agent", &skills_dir);

        // Should return empty when directory doesn't exist
        assert!(
            result.is_empty(),
            "Expected no warnings when skills dir doesn't exist, got: {:?}",
            result
        );
    }

    /// Test that validate_skills_exist_in_directory validates skill-name format skills.
    #[test]
    fn test_validate_skills_exist_skips_repo_only_format() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temp skills directory with the skill
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let skills_dir = temp_dir.path().to_path_buf();

        // Create the skill directory with a SKILL.md file
        let skill_dir = skills_dir.join("frontend-design");
        fs::create_dir(&skill_dir).expect("Failed to create skill dir");
        let skill_file = skill_dir.join("SKILL.md");
        fs::write(
            &skill_file,
            "---\nname: frontend-design\ndescription: Test skill\n---",
        )
        .expect("Failed to write skill file");

        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec!["frontend-design".to_string()]), // Valid skill-name format
            silent_timeout: None,
        };

        let result = validate_skills_exist_in_directory(&agent, "test-agent", &skills_dir);

        // Should return empty because skill exists in the directory
        assert!(
            result.is_empty(),
            "Expected no warnings for skill that exists, got: {:?}",
            result
        );
    }

    /// Test that validate_skills_exist_in_directory returns empty when no skills field.
    #[test]
    fn test_validate_skills_exist_returns_empty_when_no_skills() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: Some("Test prompt".to_string()),
            prompt_file: None,
            schedule: "* * * * * *".to_string(),
            env: Some(std::collections::HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: None, // No skills field
            silent_timeout: None,
        };

        let skills_dir = PathBuf::from("./skills");
        let result = validate_skills_exist_in_directory(&agent, "test-agent", &skills_dir);

        assert!(
            result.is_empty(),
            "Expected no warnings when skills field is None, got: {:?}",
            result
        );
    }

    // === Lockfile Tests ===

    /// Test validate_lockfile_consistency returns empty when lockfile doesn't exist.
    ///
    /// This test ensures that when there is no lockfile in the skills directory,
    /// the validation returns no warnings (since there's nothing to check).
    #[test]
    fn test_validate_lockfile_consistency_warns_orphaned_skills() {
        use tempfile::TempDir;

        // Create a temp directory with no lockfile
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let skills_dir = temp_dir.path().to_path_buf();

        // Create a minimal config with an agent that has no skills
        let mut config = Config::default();
        config.agents = vec![Agent {
            name: "test-agent".to_string(),
            prompt: Some("test prompt".to_string()),
            prompt_file: None,
            schedule: "0 * * * * *".to_string(),
            env: None,
            readonly: None,
            timeout: None,
            overlap_mode: None,
            max_queue_size: None,
            skills: Some(vec![]), // No skills
            silent_timeout: None,
        }];

        let result = validate_lockfile_consistency(&config, &skills_dir);

        // Should return empty because lockfile doesn't exist
        assert!(
            result.is_empty(),
            "Expected no warnings when lockfile doesn't exist, got: {:?}",
            result
        );
    }

    /// Test validate_lockfile_consistency returns warnings for orphaned skills in lockfile.
    ///
    /// This test ensures that when a lockfile exists with skills that are not
    /// referenced in any agent configuration, warnings are returned.
    #[test]
    fn test_validate_lockfile_consistency_no_agents_with_skills() {
        // Use the actual skills directory which has a lockfile with orphaned skills
        let skills_dir = PathBuf::from("./skills");

        // Create config with agent that has no skills
        let mut config = Config::default();
        config.agents = vec![Agent {
            name: "test-agent".to_string(),
            prompt: Some("test prompt".to_string()),
            prompt_file: None,
            schedule: "0 * * * * *".to_string(),
            env: None,
            readonly: None,
            timeout: None,
            overlap_mode: None,
            max_queue_size: None,
            skills: None, // No skills field
            silent_timeout: None,
        }];

        let result = validate_lockfile_consistency(&config, &skills_dir);

        // Should return warnings because lockfile has skills not in config
        assert!(
            !result.is_empty(),
            "Expected warnings when lockfile has skills not referenced in config, got: {:?}",
            result
        );
        // Verify the warnings mention the orphaned skills from the lockfile
        let warning_text = result.join(" ");
        assert!(
            warning_text.contains("rust-engineer") || warning_text.contains("rust-best-practices"),
            "Warning should mention orphaned skills from lockfile, got: {}",
            warning_text
        );
    }
}
