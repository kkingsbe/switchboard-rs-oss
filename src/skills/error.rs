//! Skills module error types
//!
//! This module defines error types for the skills functionality,
//! covering all error scenarios from npx invocation to skill installation.

use std::fmt;

/// Errors that can occur during skills operations.
///
/// This enum represents all possible errors that can occur when managing
/// skills through the Switchboard CLI, including npx invocation issues,
/// skill installation failures, and configuration validation errors.
///
/// # Examples
///
/// ```rust
/// use switchboard::skills::SkillsError;
///
/// let result: Result<(), SkillsError> = Err(SkillsError::NpxNotFound);
/// match result {
///     Err(SkillsError::NpxNotFound) => {
///         eprintln!("Error: npx is required for this command. Install Node.js from https://nodejs.org");
///     }
///     Err(SkillsError::SkillNotFound { skill_source }) => {
///         eprintln!("Skill '{}' not found", skill_source);
///     }
///     Err(SkillsError::MalformedSkillMetadata { skill_name, reason, .. }) => {
///         eprintln!("Malformed SKILL.md for '{}': {}", skill_name, reason);
///     }
///     Ok(_) => println!("Operation successful"),
///     _ => todo!("Handle other error variants"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum SkillsError {
    /// npx is not available on the host system.
    ///
    /// This error occurs when the user tries to run a skills command
    /// (list, install, update) but npx is not found on the host.
    /// Node.js must be installed for these operations.
    ///
    /// # User Action
    ///
    /// Install Node.js from <https://nodejs.org>
    NpxNotFound,

    /// npx skills command failed with a non-zero exit code.
    ///
    /// This error wraps failures from npx skills subcommands (add, find, update).
    /// Switchboard forwards the exit code and stderr from npx skills directly.
    ///
    /// # Fields
    ///
    /// * `command` - The npx skills command that failed
    /// * `exit_code` - The non-zero exit code returned
    /// * `stderr` - Standard error output from npx skills
    ///
    /// # Common Causes
    ///
    /// - Network connectivity issues
    /// - Invalid skill repository or package name
    /// - npx skills CLI internal errors
    /// - Permission issues accessing skills directories
    NpxCommandFailed {
        /// The npx skills command that failed
        command: String,
        /// The non-zero exit code
        exit_code: i32,
        /// Standard error output
        stderr: String,
    },

    /// Skill not found in the specified repository.
    ///
    /// This error occurs when npx skills cannot locate the requested skill.
    /// This may be due to a non-existent repository, a typo in the skill name,
    /// or the skill being removed from the registry.
    ///
    /// # Fields
    ///
    /// * `skill_source` - The skill source that was not found (e.g., "owner/repo@skill-name")
    ///
    /// # User Action
    ///
    /// Verify the skill source is correct and exists in the skills.sh registry.
    SkillNotFound {
        /// The skill source that was not found
        skill_source: String,
    },

    /// Malformed SKILL.md file frontmatter.
    ///
    /// This error occurs when parsing the YAML frontmatter from a SKILL.md file
    /// fails during the `installed` command. The skill may be present but not
    /// parseable.
    ///
    /// # Fields
    ///
    /// * `skill_name` - Name of the skill with the malformed metadata
    /// * `path` - Path to the SKILL.md file
    /// * `reason` - Description of why the metadata is malformed
    ///
    /// # Common Causes
    ///
    /// - Missing YAML frontmatter (--- delimiters)
    /// - Invalid YAML syntax
    /// - Missing required fields (name, description)
    /// - Invalid encoding or non-UTF-8 characters
    MalformedSkillMetadata {
        /// Name of the skill
        skill_name: String,
        /// Path to the SKILL.md file
        path: String,
        /// Reason for the parsing failure
        reason: String,
    },

    /// Network unavailable during a remote operation.
    ///
    /// This error occurs when npx skills cannot reach the skills.sh registry
    /// or GitHub due to network connectivity issues.
    ///
    /// # Fields
    ///
    /// * `operation` - The operation that failed (e.g., "list", "install", "update")
    /// * `message` - Details about the network error
    NetworkUnavailable {
        /// The operation that failed
        operation: String,
        /// Network error details
        message: String,
    },

    /// Skill name collision between project and global skills.
    ///
    /// This error (or warning) occurs when the same skill name exists in both
    /// the project-level and global skills directories. Project-level takes
    /// precedence, but the user is warned.
    ///
    /// # Fields
    ///
    /// * `skill_name` - Name of the colliding skill
    ///
    /// # Note
    ///
    /// This is a non-fatal warning; the project-level skill takes precedence.
    SkillNameCollision {
        /// Name of the colliding skill
        skill_name: String,
    },

    /// Skill installation failed inside a container.
    ///
    /// This error occurs when npx skills add fails during container startup
    /// skill installation. The container will exit with a non-zero code and
    /// the failure is logged with distinction from agent execution failures.
    ///
    /// # Fields
    ///
    /// * `skill_source` - The skill source that failed to install
    /// * `agent_name` - Name of the agent attempting to install the skill
    /// * `exit_code` - Exit code from npx skills
    /// * `stderr` - Standard error output
    ContainerInstallFailed {
        /// The skill source that failed
        skill_source: String,
        /// Name of the agent
        agent_name: String,
        /// Exit code from npx skills
        exit_code: i32,
        /// Standard error output
        stderr: String,
    },

    /// Script generation failed for container entrypoint.
    ///
    /// This error occurs when the generate_entrypoint_script function fails
    /// to create the entrypoint script for a container. This can happen due
    /// to various issues during script template rendering or file writing.
    ///
    /// # Fields
    ///
    /// * `agent_name` - Name of the agent whose script failed to generate
    /// * `reason` - Description of why script generation failed
    ScriptGenerationFailed {
        /// Name of the agent
        agent_name: String,
        /// Description of the failure
        reason: String,
    },

    /// Agent has an empty skills list in configuration.
    ///
    /// This is a validation warning that occurs when an agent has a `skills`
    /// field that is present but empty (`skills = []`). This is valid but
    /// unusual, so `switchboard validate` warns about it.
    ///
    /// # Fields
    ///
    /// * `agent_name` - Name of the agent with empty skills
    ///
    /// # User Action
    ///
    /// Either remove the `skills` field entirely or add skills to the list.
    EmptySkillsList {
        /// Name of the agent
        agent_name: String,
    },

    /// Invalid skill entry format in configuration.
    ///
    /// This error occurs when a `skills` entry in switchboard.toml does not match
    /// the required `owner/repo` or `owner/repo@skill-name` format.
    ///
    /// # Fields
    ///
    /// * `entry` - The invalid entry string
    /// * `agent_name` - Name of the agent with the invalid entry
    /// * `reason` - Description of why the format is invalid
    ///
    /// # Valid Formats
    ///
    /// - `owner/repo` - installs all skills from the repo
    /// - `owner/repo@skill-name` - installs a specific skill
    /// - Full GitHub or GitLab URL
    ///
    /// # User Action
    ///
    /// Correct the skills entry to match one of the valid formats.
    InvalidSkillsEntryFormat {
        /// The invalid entry
        entry: String,
        /// Name of the agent
        agent_name: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Skills directory not found.
    ///
    /// This error occurs when attempting to read from a skills directory
    /// that does not exist.
    ///
    /// # Fields
    ///
    /// * `path` - Path to the skills directory
    ///
    /// # User Action
    ///
    /// This is typically a non-fatal state during `installed` commands;
    /// it simply means no skills are installed yet.
    SkillsDirectoryNotFound {
        /// Path to the skills directory
        path: String,
    },

    /// IO error during skills operations.
    ///
    /// This error wraps standard IO errors that may occur during
    /// file operations on skills directories.
    ///
    /// # Fields
    ///
    /// * `operation` - The operation being performed (e.g., "read", "write", "delete")
    /// * `path` - Path being operated on
    /// * `message` - IO error details
    IoError {
        /// The operation that failed
        operation: String,
        /// Path being operated on
        path: String,
        /// Error details
        message: String,
    },

    /// Missing YAML frontmatter block in SKILL.md file.
    ///
    /// This error occurs when a SKILL.md file exists but does not contain
    /// the expected YAML frontmatter block delimited by `---` markers.
    ///
    /// # Fields
    ///
    /// * `path` - Path to the SKILL.md file without frontmatter
    ///
    /// # Common Causes
    ///
    /// - SKILL.md file was created without frontmatter
    /// - Frontmatter delimiters were incorrectly formatted
    MissingFrontmatter {
        /// Path to the SKILL.md file
        path: String,
    },

    /// Invalid skill directory.
    ///
    /// This error occurs when a directory is expected to contain a skill
    /// (SKILL.md file) but does not have one. This can happen when
    /// scanning directories for installed skills.
    ///
    /// # Fields
    ///
    /// * `path` - Path to the invalid skill directory
    ///
    /// # Common Causes
    ///
    /// - Directory was created but SKILL.md was not added
    /// - SKILL.md file was deleted or moved
    InvalidSkillDirectory {
        /// Path to the invalid skill directory
        path: String,
    },

    /// Required field missing from skill frontmatter.
    ///
    /// This error occurs when parsing YAML frontmatter and a required field
    /// is not present. The `name` field is currently the only required field.
    ///
    /// # Fields
    ///
    /// * `field_name` - Name of the missing required field
    /// * `path` - Path to the SKILL.md file with missing field
    FieldMissing {
        /// Name of the missing field
        field_name: String,
        /// Path to the SKILL.md file
        path: String,
    },

    /// Skill not found in project or global directory.
    ///
    /// This error occurs when attempting to remove a skill that does not exist
    /// in either the project-level (./skills/) or global (./skills/)
    /// skills directories.
    ///
    /// # Fields
    ///
    /// * `skill_name` - Name of the skill that was not found
    ///
    /// # User Action
    ///
    /// Verify the skill name is correct and that the skill is installed.
    /// Use `switchboard skills installed` to list all installed skills.
    SkillDirectoryNotFound {
        /// Name of the skill that was not found
        skill_name: String,
    },

    /// Skill directory removal failed.
    ///
    /// This error occurs when removing a skill directory fails due to
    /// filesystem permissions, active locks, or other IO errors.
    ///
    /// # Fields
    ///
    /// * `skill_name` - Name of the skill that failed to remove
    /// * `reason` - Description of why the removal failed
    ///
    /// # Common Causes
    ///
    /// - Permission denied - insufficient rights to delete the directory
    /// - Directory is in use by another process
    /// - Filesystem is read-only
    /// - Symbolic link issues
    RemoveFailed {
        /// Name of the skill that failed to remove
        skill_name: String,
        /// Description of why removal failed
        reason: String,
    },

    /// Invalid skill source format.
    ///
    /// This error occurs when a skill source string does not conform to the
    /// expected format for parsing and validation. This can happen during
    /// skill source string validation before attempting to install or reference
    /// a skill.
    ///
    /// # Fields
    ///
    /// * `skill_source` - The malformed skill source string that failed validation
    /// * `reason` - Explanation of why the format is invalid
    ///
    /// # Common Causes
    ///
    /// - Missing required delimiters (e.g., / or @)
    /// - Invalid characters in skill source
    /// - Empty components in the skill source
    ///
    /// # Valid Formats
    ///
    /// - `owner/repo` - installs all skills from the repo
    /// - `owner/repo@skill-name` - installs a specific skill
    /// - Full GitHub or GitLab URL
    ///
    /// # User Action
    ///
    /// Verify the skill source format matches one of the valid patterns.
    InvalidSkillFormat {
        /// The malformed skill source string
        skill_source: String,
        /// Reason for invalidity
        reason: String,
    },

    /// Invalid search query.
    ///
    /// This error occurs when the search query for skills.sh API is invalid.
    /// The query must be at least 2 characters long.
    ///
    /// # Fields
    ///
    /// * `query` - The query that was provided
    /// * `reason` - Explanation of why the query is invalid
    InvalidQuery {
        /// The invalid query
        query: String,
        /// Reason for invalidity
        reason: String,
    },

    /// HTTP error from skills.sh API.
    ///
    /// This error occurs when the skills.sh API returns an error HTTP status code.
    ///
    /// # Fields
    ///
    /// * `status` - The HTTP status code
    /// * `message` - Error message from the API
    HttpError {
        /// The HTTP status code
        status: u16,
        /// Error message
        message: String,
    },

    /// JSON parsing error from skills.sh API response.
    ///
    /// This error occurs when the response from skills.sh API cannot be parsed as JSON.
    ///
    /// # Fields
    ///
    /// * `message` - Details about the parsing error
    JsonParseError {
        /// Error message
        message: String,
    },

    /// Failed to read the lockfile from disk.
    ///
    /// This error occurs when attempting to read the skills.lock.json file fails.
    ///
    /// # Fields
    ///
    /// * `path` - Path to the lockfile
    /// * `message` - Details about the read failure
    LockfileReadError {
        /// Path to the lockfile
        path: String,
        /// Error details
        message: String,
    },

    /// Failed to write the lockfile to disk.
    ///
    /// This error occurs when attempting to write the skills.lock.json file fails.
    ///
    /// # Fields
    ///
    /// * `path` - Path to the lockfile
    /// * `message` - Details about the write failure
    LockfileWriteError {
        /// Path to the lockfile
        path: String,
        /// Error details
        message: String,
    },

    /// Failed to parse the lockfile JSON.
    ///
    /// This error occurs when the skills.lock.json file exists but contains invalid JSON.
    ///
    /// # Fields
    ///
    /// * `path` - Path to the lockfile
    /// * `message` - Details about the parse failure
    LockfileParseError {
        /// Path to the lockfile
        path: String,
        /// Error details
        message: String,
    },

    /// Lockfile does not exist.
    ///
    /// This error occurs when the skills.lock.json file is expected but not found.
    /// This is normal when no skills have been installed yet.
    ///
    /// # Fields
    ///
    /// * `path` - Path to the expected lockfile location
    LockfileNotFound {
        /// Path to the expected lockfile
        path: String,
    },

    /// Destination directory already exists during skill installation.
    ///
    /// This error occurs when attempting to install a skill where the destination
    /// directory already exists. The user can either confirm to overwrite or use
    /// the --yes flag to bypass this prompt.
    ///
    /// # Fields
    ///
    /// * `skill_name` - Name of the skill being installed
    /// * `path` - Path to the existing directory
    DestinationAlreadyExists {
        /// Name of the skill
        skill_name: String,
        /// Path to the existing directory
        path: String,
    },

    /// Skill is already installed.
    ///
    /// This error occurs when attempting to install a skill that is already present
    /// in the skills directory. The user should either remove the existing skill
    /// or use the --yes flag to overwrite.
    ///
    /// # Fields
    ///
    /// * `skill_name` - Name of the skill that is already installed
    /// * `path` - Path to the existing skill directory
    SkillAlreadyInstalled {
        /// Name of the skill
        skill_name: String,
        /// Path to the existing skill directory
        path: String,
    },
}

impl fmt::Display for SkillsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkillsError::NpxNotFound => {
                write!(
                    f,
                    "npx is required for this command. Install Node.js from https://nodejs.org"
                )
            }
            SkillsError::NpxCommandFailed {
                command,
                exit_code,
                stderr,
            } => {
                // Provide a clear message with the exit code and stderr details
                // The stderr often contains helpful diagnostic information from npx
                write!(
                    f,
                    "npx skills command failed with exit code {}: {}. Run '{}' manually for detailed error output.",
                    exit_code,
                    stderr.trim(),
                    command
                )
            }
            SkillsError::SkillNotFound { skill_source } => {
                // Suggest checking the skills list to verify the skill name/format
                write!(
                    f,
                    "Skill not found: '{}'. Verify the skill exists with 'switchboard skills list' or check the format: owner/repo@skill-name",
                    skill_source
                )
            }
            SkillsError::MalformedSkillMetadata {
                skill_name,
                path,
                reason,
            } => {
                write!(
                    f,
                    "Malformed SKILL.md for '{}': {} ({})",
                    skill_name, reason, path
                )
            }
            SkillsError::NetworkUnavailable { operation, message } => {
                // Network errors are often transient; suggest checking connectivity
                write!(
                    f,
                    "Network unavailable during {}: {}. Check your internet connection and try again.",
                    operation, message
                )
            }
            SkillsError::SkillNameCollision { skill_name } => {
                write!(
                    f,
                    "Skill name collision: '{}' exists in both project and global skills. Using project-level.",
                    skill_name
                )
            }
            SkillsError::ContainerInstallFailed {
                skill_source,
                agent_name,
                exit_code,
                stderr,
            } => {
                // Provide actionable information for container skill installation failures
                write!(
                    f,
                    "[SKILL INSTALL] Skill installation failed for agent '{}': '{}' exited with code {}.
    - Verify the skill exists: switchboard skills list
    - Check your network connection (npx needs internet access)
    - stderr: {}",
                    agent_name,
                    skill_source,
                    exit_code,
                    stderr.trim()
                )
            }
            SkillsError::ScriptGenerationFailed { agent_name, reason } => {
                write!(
                    f,
                    "Failed to generate entrypoint script for agent '{}': {}",
                    agent_name, reason
                )
            }
            SkillsError::EmptySkillsList { agent_name } => {
                write!(
                    f,
                    "Agent '{}' has an empty skills list. Either remove the 'skills' field or add skills.",
                    agent_name
                )
            }
            SkillsError::InvalidSkillsEntryFormat {
                entry,
                agent_name,
                reason,
            } => {
                write!(
                    f,
                    "Invalid skills entry '{}' for agent '{}': {}. Valid formats: owner/repo or owner/repo@skill-name",
                    entry, agent_name, reason
                )
            }
            SkillsError::SkillsDirectoryNotFound { path } => {
                // This is typically non-fatal during 'installed' commands - no skills installed yet
                write!(
                    f,
                    "Skills directory not found: {}. No skills are currently installed. Use 'switchboard skills install <skill>' to install a skill.",
                    path
                )
            }
            SkillsError::IoError {
                operation,
                path,
                message,
            } => {
                write!(f, "IO error during {} on {}: {}", operation, path, message)
            }
            SkillsError::MissingFrontmatter { path } => {
                write!(f, "Missing YAML frontmatter in SKILL.md file: {}", path)
            }
            SkillsError::InvalidSkillDirectory { path } => {
                // Directory exists but lacks SKILL.md - may be an incomplete installation
                write!(
                    f,
                    "Invalid skill directory (no SKILL.md found): {}. This directory may contain an incomplete skill installation.",
                    path
                )
            }
            SkillsError::FieldMissing { field_name, path } => {
                write!(
                    f,
                    "Required field '{}' missing from frontmatter in: {}",
                    field_name, path
                )
            }
            SkillsError::SkillDirectoryNotFound { skill_name } => {
                write!(
                    f,
                    "Skill '{}' not found in project or global directory. Use 'switchboard skills installed' to list available skills.",
                    skill_name
                )
            }
            SkillsError::RemoveFailed { skill_name, reason } => {
                write!(f, "Failed to remove skill '{}': {}", skill_name, reason)
            }
            SkillsError::InvalidSkillFormat {
                skill_source,
                reason,
            } => {
                write!(
                    f,
                    "Invalid skill format '{}': {}. Valid formats: owner/repo or owner/repo@skill-name",
                    skill_source, reason
                )
            }
            SkillsError::InvalidQuery { query, reason } => {
                write!(f, "Invalid query '{}': {}", query, reason)
            }
            SkillsError::HttpError { status, message } => {
                write!(f, "HTTP error {}: {}", status, message)
            }
            SkillsError::JsonParseError { message } => {
                write!(f, "Failed to parse skills.sh API response: {}", message)
            }
            SkillsError::LockfileReadError { path, message } => {
                write!(f, "Failed to read lockfile at {}: {}", path, message)
            }
            SkillsError::LockfileWriteError { path, message } => {
                write!(f, "Failed to write lockfile at {}: {}", path, message)
            }
            SkillsError::LockfileParseError { path, message } => {
                write!(f, "Failed to parse lockfile at {}: {}", path, message)
            }
            SkillsError::LockfileNotFound { path } => {
                write!(f, "Lockfile not found at: {}", path)
            }
            SkillsError::DestinationAlreadyExists { skill_name, path } => {
                write!(
                    f,
                    "Skill '{}' is already installed at '{}'. Use --yes to overwrite or remove the existing skill first.",
                    skill_name, path
                )
            }
            SkillsError::SkillAlreadyInstalled { skill_name, path } => {
                write!(
                    f,
                    "Skill '{}' is already installed at '{}'. Remove the existing skill first or use 'switchboard skills remove' to uninstall it before reinstalling.",
                    skill_name, path
                )
            }
        }
    }
}

impl std::error::Error for SkillsError {}
