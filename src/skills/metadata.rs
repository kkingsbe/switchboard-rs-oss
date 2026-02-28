//! Skill metadata handling - parsing, loading, and scanning skill metadata

use crate::config::Config;
use crate::skills::SkillsError;
use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::{Path, PathBuf};

/// Metadata extracted from a SKILL.md file.
///
/// This struct represents the parsed YAML frontmatter from a skill's SKILL.md file.
/// It contains the core information needed to identify, describe, and manage a skill.
///
/// # SKILL.md File Format
///
/// SKILL.md files use YAML frontmatter delimited by `---` markers at the beginning
/// of the file. The frontmatter contains structured metadata about the skill,
/// followed by the skill's documentation content.
///
/// ```markdown
/// ---
/// name: discli
/// description: Discord Notifications Tool
/// version: 0.1.0
/// authors: ["John Doe <john@example.com>"]
/// dependencies: ["discord-api"]
/// compatible_agents: ["architect", "code", "ask"]
/// source: https://github.com/owner/repo
/// ---
///
/// # Discord Notifications Tool
///
/// This skill enables AI agents to send Discord notifications...
/// ```
///
/// # Fields
///
/// * `name` - The name of the skill (required)
/// * `description` - Optional description of what the skill does
/// * `version` - Optional version string for the skill (e.g., "0.1.0", "1.2.3")
/// * `authors` - Optional list of author names or contact information
/// * `dependencies` - Optional list of other skills this skill depends on
/// * `compatible_agents` - Optional list of agent types that can use this skill
/// * `source` - Optional source/origin of the skill (e.g., GitHub URL)
///
/// # Field Details
///
/// **`name`** (required): The unique identifier for this skill. This should be a
/// lowercase string with hyphens separating words (e.g., "discord-notifications").
///
/// **`description`** (optional): A human-readable description of what the skill does.
/// Should be concise but informative, typically 1-3 sentences.
///
/// **`version`** (optional): Semantic version string following [SemVer](https://semver.org/)
/// conventions (e.g., "1.2.3", "2.0.0-beta.1"). Used for tracking skill versions.
///
/// **`authors`** (optional): Array of author information. Each entry can be:
/// - A simple name: `"Jane Doe"`
/// - Name with email: `"Jane Doe <jane@example.com>"`
/// - Full attribution: `"Jane Doe (Organization) <jane@example.com>"`
///
/// **`dependencies`** (optional): List of skill names that must be available for this
/// skill to function. Skills are referenced by their `name` field.
///
/// **`compatible_agents`** (optional): List of agent types that can use this skill.
/// Common agent types include: "architect", "code", "ask", "debug", "orchestrator".
/// If omitted, the skill is assumed to be compatible with all agent types.
///
/// **`source`** (optional): URL or identifier indicating where the skill came from.
/// Common formats:
/// - GitHub repo: `https://github.com/owner/repo`
/// - npm package: `owner/package-name`
/// - GitLab URL: `https://gitlab.com/owner/repo`
///
/// # Examples
///
/// A typical SKILL.md file might look like:
/// ```markdown
/// ---
/// name: discli
/// description: Discord Notifications Tool
/// source: https://github.com/owner/repo
/// version: 0.1.0
/// authors: ["Jane Doe <jane@example.com>"]
/// dependencies: []
/// compatible_agents: ["architect", "code"]
/// ---
///
/// # Discord Notifications Tool
///
/// This skill enables AI agents to send Discord notifications...
/// ```
///
/// This would be parsed into a `SkillMetadata` struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillMetadata {
    /// Name of the skill (required field)
    ///
    /// The unique identifier for this skill. Should be lowercase with hyphens.
    /// Example: "discord-notifications", "file-operations"
    pub name: String,

    /// Optional description of what the skill does
    ///
    /// Should be concise but informative, typically 1-3 sentences.
    pub description: Option<String>,

    /// Optional version string following SemVer conventions
    ///
    /// Examples: "0.1.0", "1.2.3", "2.0.0-beta.1"
    #[serde(default)]
    pub version: Option<String>,

    /// Optional list of author names or contact information
    ///
    /// Each entry can be:
    /// - A simple name: "Jane Doe"
    /// - Name with email: "Jane Doe <jane@example.com>"
    /// - Full attribution: "Jane Doe (Organization) <jane@example.com>"
    #[serde(default)]
    pub authors: Vec<String>,

    /// Optional list of other skills this skill depends on
    ///
    /// Skills are referenced by their `name` field. This allows for
    /// dependency resolution when installing skills.
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Optional list of agent types that can use this skill
    ///
    /// Common agent types include: "architect", "code", "ask", "debug",
    /// "orchestrator", "task-discovery", "html-reporter", "interviewer".
    ///
    /// If omitted, the skill is assumed to be compatible with all agent types.
    #[serde(default)]
    pub compatible_agents: Vec<String>,

    /// Optional source/origin of the skill
    ///
    /// Common formats:
    /// - GitHub repo: "https://github.com/owner/repo"
    /// - npm package: "owner/package-name"
    /// - GitLab URL: "https://gitlab.com/owner/repo"
    #[serde(default)]
    pub source: Option<String>,
}

/// Result from skills.sh API search.
///
/// This struct represents a skill returned from the skills.sh search API.
/// It contains the core information needed to identify and install a skill.
///
/// # Fields
///
/// * `id` - Unique identifier for the skill
/// * `name` - Name of the skill
/// * `installs` - Number of installs/downloads
/// * `source` - GitHub owner/repo format
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillSearchResult {
    /// Unique identifier for the skill
    pub id: String,
    /// Name of the skill
    pub name: String,
    /// Number of installs/downloads
    pub installs: u64,
    /// GitHub owner/repo format
    pub source: String,
}

/// Wrapper struct for skills.sh API search response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillsSearchResponse {
    pub skills: Vec<SkillSearchResult>,
}

/// Searches for skills on skills.sh API.
///
/// This function makes an HTTP GET request to the skills.sh search API
/// to find skills matching the provided query.
///
/// # Arguments
///
/// * `query` - Search query (must be at least 2 characters)
/// * `limit` - Maximum number of results to return (optional, defaults to 10)
///
/// # Returns
///
/// * `Ok(Vec<SkillSearchResult>)` - List of matching skills
/// * `Err(SkillsError::InvalidQuery)` - If query is less than 2 characters
/// * `Err(SkillsError::NetworkUnavailable)` - If network is unreachable
/// * `Err(SkillsError::HttpError)` - If API returns HTTP error
/// * `Err(SkillsError::JsonParseError)` - If response cannot be parsed
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::skills::skills_sh_search;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let results = skills_sh_search("discord", None).await?;
///     for skill in results {
///         println!("{}: {} ({} installs) - {}", skill.id, skill.name, skill.installs, skill.source);
///     }
///     Ok(())
/// }
/// ```
#[allow(dead_code)]
pub async fn skills_sh_search(
    query: &str,
    limit: Option<u32>,
) -> Result<Vec<SkillSearchResult>, SkillsError> {
    // Validate query length
    if query.len() < 2 {
        return Err(SkillsError::InvalidQuery {
            query: query.to_string(),
            reason: "Query must be at least 2 characters".to_string(),
        });
    }

    // Build the URL
    let limit_value = limit.unwrap_or(10);
    // Simple URL encoding for the query
    let encoded_query = query
        .chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '!' => "%21".to_string(),
            '"' => "%22".to_string(),
            '#' => "%23".to_string(),
            '$' => "%24".to_string(),
            '%' => "%25".to_string(),
            '&' => "%26".to_string(),
            '+' => "%2B".to_string(),
            '=' => "%3D".to_string(),
            '?' => "%3F".to_string(),
            _ => c.to_string(),
        })
        .collect::<String>();

    let url = format!(
        "https://skills.sh/api/search?q={}&limit={}",
        encoded_query, limit_value
    );

    // Make HTTP request
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| SkillsError::NetworkUnavailable {
            operation: "skills.sh search".to_string(),
            message: e.to_string(),
        })?;

    // Check HTTP status
    let status = response.status();
    if !status.is_success() {
        return Err(SkillsError::HttpError {
            status: status.as_u16(),
            message: status
                .canonical_reason()
                .unwrap_or("Unknown error")
                .to_string(),
        });
    }

    // Parse JSON response
    let results = response
        .json::<SkillsSearchResponse>()
        .await
        .map_err(|e| SkillsError::JsonParseError {
            message: e.to_string(),
        })?
        .skills;

    Ok(results)
}

/// Reads a SKILL.md file from the given path.
///
/// This function reads the contents of a SKILL.md file and returns them as a string.
/// It handles common IO errors such as file not found, permission denied, and UTF-8 decoding errors.
///
/// # Arguments
///
/// * `path` - Path to the SKILL.md file
///
/// # Returns
///
/// * `Ok(String)` - The file contents as a UTF-8 string
/// * `Err(SkillsError::IoError)` - If the file cannot be read
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::skills::read_skill_file;
/// use std::path::Path;
///
/// let content = read_skill_file(Path::new("skills/example/SKILL.md"))?;
/// println!("Skill file contents: {}", content);
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
pub fn read_skill_file(path: &Path) -> Result<String, SkillsError> {
    fs::read_to_string(path).map_err(|e| SkillsError::IoError {
        operation: "read SKILL.md".to_string(),
        path: path.display().to_string(),
        message: e.to_string(),
    })
}

/// Parses YAML frontmatter from SKILL.md content.
///
/// This function extracts and parses the YAML frontmatter block from a SKILL.md file.
/// The frontmatter is delimited by `---` markers at the beginning of the file.
///
/// # Arguments
///
/// * `content` - The contents of the SKILL.md file as a string
///
/// # Returns
///
/// * `Ok(SkillMetadata)` - The parsed skill metadata
/// * `Err(SkillsError::MissingFrontmatter)` - If no frontmatter block is found
/// * `Err(SkillsError::MalformedSkillMetadata)` - If the YAML is invalid
///
/// # Examples
///
/// ```rust
/// use switchboard::skills::parse_skill_frontmatter;
///
/// let content = "\
/// ---\n\
/// name: test-skill\n\
/// description: A test skill\n\
/// ---\n\
/// ";
///
/// let result = parse_skill_frontmatter(content);
/// assert!(result.is_ok());
/// if let Ok(metadata) = result {
///     assert_eq!(metadata.name, "test-skill");
/// }
/// ```
pub fn parse_skill_frontmatter(content: &str) -> Result<SkillMetadata, SkillsError> {
    // Find the frontmatter block between --- delimiters
    let lines: Vec<&str> = content.lines().collect();

    // SKILL.md files must have YAML frontmatter delimited by ---
    if lines.is_empty() || !lines[0].starts_with("---") {
        return Err(SkillsError::MissingFrontmatter {
            path: "unknown".to_string(),
        });
    }

    // Find the closing --- that marks end of frontmatter block
    let mut frontmatter_end = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.starts_with("---") {
            frontmatter_end = Some(i);
            break;
        }
    }

    // If no closing --- found, frontmatter is malformed
    let frontmatter_end = frontmatter_end.ok_or_else(|| SkillsError::MissingFrontmatter {
        path: "unknown".to_string(),
    })?;

    // Extract frontmatter content between the delimiters
    let frontmatter_str: String = lines[1..frontmatter_end].join("\n").trim().to_string();

    // Parse YAML and convert YAML errors to SkillsError
    let yaml: YamlValue = serde_yaml::from_str(&frontmatter_str).map_err(|e| {
        SkillsError::MalformedSkillMetadata {
            skill_name: "unknown".to_string(),
            path: "unknown".to_string(),
            reason: format!("Invalid YAML: {}", e),
        }
    })?;

    // Extract fields from YAML
    let name =
        yaml.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SkillsError::FieldMissing {
                field_name: "name".to_string(),
                path: "unknown".to_string(),
            })?;

    let description = yaml
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let version = yaml
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Extract optional array fields
    let authors = yaml
        .get("authors")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let dependencies = yaml
        .get("dependencies")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let compatible_agents = yaml
        .get("compatible_agents")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let source = yaml
        .get("source")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(SkillMetadata {
        name: name.to_string(),
        description,
        version,
        authors,
        dependencies,
        compatible_agents,
        source,
    })
}

/// Loads skill metadata from a SKILL.md file.
///
/// This is a convenience function that combines reading and parsing a SKILL.md file.
/// If the `name` field is missing from the frontmatter, it falls back to using the
/// directory name as the skill name.
///
/// # Arguments
///
/// * `path` - Path to the SKILL.md file
///
/// # Returns
///
/// * `Ok(SkillMetadata)` - The parsed skill metadata
/// * `Err(SkillsError::IoError)` - If the file cannot be read
/// * `Err(SkillsError::MissingFrontmatter)` - If no frontmatter block is found
/// * `Err(SkillsError::MalformedSkillMetadata)` - If the YAML is invalid
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::skills::load_skill_metadata;
/// use std::path::Path;
///
/// let metadata = load_skill_metadata(Path::new("skills/example/SKILL.md"))?;
/// println!("Skill: {}", metadata.name);
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
pub fn load_skill_metadata(path: &Path) -> Result<SkillMetadata, SkillsError> {
    let content = read_skill_file(path)?;

    // Try to parse frontmatter
    match parse_skill_frontmatter(&content) {
        Ok(mut metadata) => {
            // If name is empty or missing, use directory name as fallback
            if metadata.name.is_empty() {
                if let Some(dir_name) = path.parent().and_then(|p| p.file_name()) {
                    let dir_name = dir_name.to_string_lossy().to_string();
                    metadata.name = dir_name;
                }
            }
            Ok(metadata)
        }
        Err(e) => {
            // Return the error so scan_skill_directory can collect a warning and use fallback
            Err(e)
        }
    }
}

/// Scans a skills directory and returns metadata for all installed skills.
///
/// This function recursively scans a directory for subdirectories that contain SKILL.md files.
/// Each subdirectory with a SKILL.md file is considered a valid skill.
///
/// # Arguments
///
/// * `dir` - Path to the skills directory to scan
///
/// # Returns
///
/// * `Ok((Vec<SkillMetadata>, Vec<String>))` - Tuple containing metadata for all found skills
///   and any warnings collected during scanning
/// * `Err(SkillsError::IoError)` - If the directory cannot be read
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::skills::scan_skill_directory;
/// use std::path::Path;
///
/// let (skills, warnings) = scan_skill_directory(Path::new("./skills"))?;
/// println!("Found {} skills", skills.len());
/// for warning in warnings {
///     eprintln!("{}", warning);
/// }
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
pub fn scan_skill_directory(dir: &Path) -> Result<(Vec<SkillMetadata>, Vec<String>), SkillsError> {
    let mut skills = Vec::new();
    let mut warnings = Vec::new();

    // Return empty vec if directory doesn't exist
    if !dir.exists() {
        return Ok((skills, warnings));
    }

    // Read directory entries
    let entries = fs::read_dir(dir).map_err(|e| SkillsError::IoError {
        operation: "read skills directory".to_string(),
        path: dir.display().to_string(),
        message: e.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| SkillsError::IoError {
            operation: "read directory entry".to_string(),
            path: dir.display().to_string(),
            message: e.to_string(),
        })?;

        let path = entry.path();

        // Skip if not a directory
        if !path.is_dir() {
            continue;
        }

        // Check if directory contains SKILL.md
        let skill_file_path = path.join("SKILL.md");
        if !skill_file_path.exists() {
            continue;
        }

        // Get skill name from directory for warning message
        let skill_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Load skill metadata
        match load_skill_metadata(&skill_file_path) {
            Ok(metadata) => skills.push(metadata),
            Err(_) => {
                // Collect warning and use directory name as fallback
                warnings.push(format!(
                    "Warning: Could not parse SKILL.md for '{}' — using directory name",
                    skill_name
                ));
                // Add skill with directory name
                skills.push(SkillMetadata {
                    name: skill_name.to_string(),
                    description: None,
                    version: None,
                    authors: Vec::new(),
                    dependencies: Vec::new(),
                    compatible_agents: Vec::new(),
                    source: None,
                });
            }
        }
    }

    Ok((skills, warnings))
}

/// Scans the project-level skills directory for installed skills.
///
/// This function scans the `./skills/` directory in the current workspace.
/// If the directory doesn't exist, it returns an empty vector.
///
/// # Returns
///
/// * `Ok((Vec<SkillMetadata>, Vec<String>))` - Tuple containing project-level skill metadata
///   and any warnings collected during scanning
/// * `Err(SkillsError::IoError)` - If the directory exists but cannot be read
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::skills::scan_project_skills;
///
/// let (skills, warnings) = scan_project_skills()?;
/// println!("Found {} project-level skills", skills.len());
/// for warning in warnings {
///     eprintln!("{}", warning);
/// }
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
#[allow(dead_code)]
pub fn scan_project_skills() -> Result<(Vec<SkillMetadata>, Vec<String>), SkillsError> {
    let project_skills_dir = PathBuf::from("./skills");

    // Create directory if it doesn't exist
    if !project_skills_dir.exists() {
        // Try to create it silently - if it fails, just return empty vec
        let _ = fs::create_dir_all(&project_skills_dir);
        return Ok((Vec::new(), Vec::new()));
    }

    scan_skill_directory(&project_skills_dir)
}

/// Scans the global skills directory for installed skills.
///
/// This function scans the `./skills/` directory.
/// If the directory doesn't exist, it returns an empty vector.
///
/// # Returns
///
/// * `Ok((Vec<SkillMetadata>, Vec<String>))` - Tuple containing global skill metadata
///   and any warnings collected during scanning
/// * `Err(SkillsError::IoError)` - If the directory exists but cannot be read
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::skills::scan_global_skills;
///
/// let (skills, warnings) = scan_global_skills()?;
/// println!("Found {} global skills", skills.len());
/// for warning in warnings {
///     eprintln!("{}", warning);
/// }
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
#[allow(dead_code)]
pub fn scan_global_skills() -> Result<(Vec<SkillMetadata>, Vec<String>), SkillsError> {
    let global_skills_dir = PathBuf::from("./skills");

    // Create directory if it doesn't exist
    if !global_skills_dir.exists() {
        // Try to create it silently - if it fails, just return empty vec
        let _ = fs::create_dir_all(&global_skills_dir);
        return Ok((Vec::new(), Vec::new()));
    }

    scan_skill_directory(&global_skills_dir)
}

/// Finds a skill directory by name in project or global scope.
///
/// This function searches for a skill directory in either the project-level
/// (./skills/) or global (./skills/) skills directory,
/// depending on the `global` flag.
///
/// # Arguments
///
/// * `skill_name` - The name of the skill to find
/// * `global` - If true, search global directory; if false, search project directory
///
/// # Returns
///
/// * `Ok(PathBuf)` - Path to the skill directory if found
/// * `Err(SkillsError::SkillDirectoryNotFound)` - If the skill is not found
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::skills::find_skill_directory;
///
/// // Find a project-level skill
/// let path = find_skill_directory("my-skill", false)?;
/// println!("Found skill at: {}", path.display());
///
/// // Find a global skill
/// let path = find_skill_directory("my-skill", true)?;
/// println!("Found global skill at: {}", path.display());
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
#[allow(dead_code)]
pub fn find_skill_directory(skill_name: &str, global: bool) -> Result<PathBuf, SkillsError> {
    use std::env;

    let base_dir = if global {
        // Global scope: ./skills/
        PathBuf::from("./skills")
    } else {
        // Project scope: ./skills/ from current directory
        let current = env::current_dir().map_err(|e| SkillsError::IoError {
            operation: "get current directory".to_string(),
            path: ".".to_string(),
            message: format!("Failed to get current directory: {}", e),
        })?;
        current.join("./skills")
    };

    let skill_path = base_dir.join(skill_name);

    // Check if the skill directory exists and is accessible
    if skill_path.exists() {
        if skill_path.is_dir() {
            // Found valid skill directory
            Ok(skill_path)
        } else {
            // Path exists but is a file, not a directory - config error
            Err(SkillsError::IoError {
                operation: "check skill directory".to_string(),
                path: skill_path.display().to_string(),
                message: "Path exists but is not a directory".to_string(),
            })
        }
    } else {
        // Directory doesn't exist - skill not installed or was removed
        Err(SkillsError::SkillDirectoryNotFound {
            skill_name: skill_name.to_string(),
        })
    }
}

/// Finds which agents have a specific skill assigned in the configuration.
///
/// This function scans the configuration for all agents that have the specified skill
/// in their `skills` list. It handles both `owner/repo` and `owner/repo@skill-name` formats.
///
/// # Arguments
///
/// * `skill_name` - The name of the skill to search for
/// * `config` - The switchboard configuration
///
/// # Returns
///
/// * `Vec<String>` - Names of agents that have the skill assigned (empty if none)
///
/// # Note
///
/// This function requires access to the configuration structure. The exact signature
/// may need adjustment based on the actual config module structure.
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::skills::get_agents_using_skill;
/// # let config = switchboard::config::Config::default();
/// // Assuming config has a way to access agents
/// let agents = get_agents_using_skill("discli", &config);
/// for agent in agents {
///     println!("Agent {} uses discli", agent);
/// }
/// ```
#[allow(dead_code)]
pub fn get_agents_using_skill(skill_name: &str, config: &Config) -> Vec<String> {
    let mut agents = Vec::new();

    // Scan all agents in the config
    for agent_config in config.agents.iter() {
        // Check if agent has a skills field
        if let Some(skills) = &agent_config.skills {
            for skill_entry in skills {
                // Check if this skill entry matches our skill_name
                // Handles formats: "owner/repo@skill-name" or "skill-name"
                if skill_entry.contains(skill_name) {
                    // More specific check: the skill name should be after @ or be the full entry
                    let matches = skill_entry.ends_with(&format!("@{}", skill_name))
                        || skill_entry == skill_name
                        || skill_entry.contains(&format!("/{}", skill_name));

                    if matches {
                        agents.push(agent_config.name.clone());
                        break; // One match per agent is enough
                    }
                }
            }
        }
    }

    agents
}

/// Removes a skill directory from the filesystem.
///
/// This function recursively removes a skill directory and all its contents.
/// It handles various IO errors and provides clear error messages.
///
/// # Arguments
///
/// * `path` - Path to the skill directory to remove
///
/// # Returns
///
/// * `Ok(())` - If the directory was successfully removed
/// * `Err(SkillsError::RemoveFailed)` - If removal fails for any reason
///
/// # Errors
///
/// This function will return an error if:
/// - The directory doesn't exist (NotFound)
/// - Permission is denied (PermissionDenied)
/// - The directory is in use by another process
/// - Any other IO error occurs
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::skills::remove_skill_directory;
/// use std::path::Path;
///
/// let skill_path = Path::new("./skills/my-skill");
/// match remove_skill_directory(skill_path) {
///     Ok(()) => println!("Skill removed successfully"),
///     Err(e) => eprintln!("Failed to remove skill: {}", e),
/// }
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
#[allow(dead_code)]
pub fn remove_skill_directory(path: &Path) -> Result<(), SkillsError> {
    // Check if path exists first
    if !path.exists() {
        return Err(SkillsError::RemoveFailed {
            skill_name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            reason: format!("Directory does not exist: {}", path.display()),
        });
    }

    // Attempt to remove the directory recursively
    fs::remove_dir_all(path).map_err(|e| {
        let skill_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let reason = match e.kind() {
            std::io::ErrorKind::PermissionDenied => {
                "Permission denied. Try running with appropriate permissions or check file ownership."
                    .to_string()
            }
            std::io::ErrorKind::NotFound => {
                format!("Directory not found: {}", path.display())
            }
            _ => format!("IO error: {}", e),
        };

        SkillsError::RemoveFailed { skill_name, reason }
    })?;

    Ok(())
}
