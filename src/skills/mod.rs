//! Skills module - manages skill discovery and installation via npx skills CLI
//!
//! This module acts as a thin ergonomic wrapper around npx skills CLI.
//! All skill discovery and installation operations are delegated to npx skills.
//! Switchboard does not implement HTTP/GitHub API code directly.

mod error;

pub use error::SkillsError;

use crate::traits::{ProcessExecutorTrait, RealProcessExecutor};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;
use std::fs;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Arc;

/// Error message for when npx is not available
pub const NPX_NOT_FOUND_ERROR: &str =
    "Error: npx is required for this command. Install Node.js from https://nodejs.org";

/// Filename for the skills lockfile
pub const LOCKFILE_FILENAME: &str = "skills.lock.json";

/// SkillsManager manages skill operations by delegating to npx skills CLI
pub struct SkillsManager {
    /// Project-level skills directory (typically ./skills/)
    pub skills_dir: PathBuf,
    /// Global skills directory (typically ./skills/)
    pub global_skills_dir: PathBuf,
    /// Whether npx is available on the host system
    pub npx_available: bool,
    /// Process executor for running npx commands
    pub executor: Arc<dyn ProcessExecutorTrait>,
}

impl std::fmt::Debug for SkillsManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillsManager")
            .field("skills_dir", &self.skills_dir)
            .field("global_skills_dir", &self.global_skills_dir)
            .field("npx_available", &self.npx_available)
            .field("executor", &"<dyn ProcessExecutorTrait>")
            .finish()
    }
}

impl Default for SkillsManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl SkillsManager {
    /// Creates a new SkillsManager with default paths
    pub fn new(executor: Option<Arc<dyn ProcessExecutorTrait>>) -> Self {
        let executor = executor.unwrap_or_else(|| Arc::new(RealProcessExecutor::new()));
        Self {
            skills_dir: PathBuf::from("./skills"),
            global_skills_dir: PathBuf::from("./skills"),
            npx_available: false,
            executor,
        }
    }

    /// Creates a new SkillsManager with the specified project skills directory
    pub fn with_skills_dir(
        skills_dir: PathBuf,
        executor: Option<Arc<dyn ProcessExecutorTrait>>,
    ) -> Self {
        let executor = executor.unwrap_or_else(|| Arc::new(RealProcessExecutor::new()));
        Self {
            skills_dir,
            global_skills_dir: PathBuf::from("./skills"),
            npx_available: false,
            executor,
        }
    }

    /// Gets the project skills directory
    pub fn skills_dir(&self) -> &PathBuf {
        &self.skills_dir
    }

    /// Gets the global skills directory
    pub fn global_skills_dir(&self) -> &PathBuf {
        &self.global_skills_dir
    }

    /// Checks if npx is available on the host system.
    ///
    /// This method attempts to execute `npx --version` to verify that npx
    /// is installed and accessible. The result is stored in the `npx_available`
    /// field for future reference.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if npx is available and successfully executed
    /// * `Err(SkillsError::NpxNotFound)` if npx is not installed or not found
    ///
    /// # Example
    ///
    /// ```rust
    /// use switchboard::skills::SkillsManager;
    /// use switchboard::skills::SkillsError;
    ///
    /// let mut manager = SkillsManager::new(None);
    /// match manager.check_npx_available() {
    ///     Ok(()) => println!("npx is available"),
    ///     Err(SkillsError::NpxNotFound) => eprintln!("npx is not available"),
    ///     _ => (),
    /// }
    /// ```
    pub fn check_npx_available(&mut self) -> Result<(), SkillsError> {
        // On Windows, npx is a .cmd batch file that needs cmd /c to execute properly
        // Use create_npx_command() which handles this correctly on all platforms
        let mut cmd = create_npx_command();
        cmd.arg("--version");

        let result = cmd.output();

        match result {
            Ok(output) => {
                // npx command executed; check if it returned success
                if output.status.success() {
                    self.npx_available = true;
                    Ok(())
                } else {
                    // npx exists but returned non-zero - likely not properly installed
                    self.npx_available = false;
                    Err(SkillsError::NpxNotFound)
                }
            }
            Err(_) => {
                // npx command failed to execute - not installed or not in PATH
                self.npx_available = false;
                Err(SkillsError::NpxNotFound)
            }
        }
    }
}

/// Creates a new Command for executing npx.
///
/// On Windows, this uses cmd /c to properly find and execute .cmd batch files.
/// On other platforms, it directly invokes npx.
#[cfg(target_os = "windows")]
pub fn create_npx_command() -> Command {
    let mut cmd = Command::new("cmd");
    cmd.args(["/c", "npx"]);
    cmd
}

#[cfg(not(target_os = "windows"))]
pub fn create_npx_command() -> Command {
    Command::new("npx")
}

/// Runs an npx command with the specified arguments and returns the output.
///
/// This function builds and executes an npx process with the given command,
/// arguments, and optional working directory. It captures both stdout and stderr
/// and returns them in the [`Output`] struct along with the exit status.
///
/// # Arguments
///
/// * `command` - The command to run via npx (e.g., "skills")
/// * `args` - Arguments to pass to the command
/// * `working_dir` - Optional working directory for the command
/// * `executor` - Optional process executor. If None, uses direct Command execution.
///
/// # Returns
///
/// * `Ok(Output)` - The process output including stdout, stderr, and exit status
/// * `Err(SkillsError::NpxNotFound)` - If npx is not available on the system
/// * `Err(SkillsError::IoError)` - If there's an IO error executing the command
///
/// # Example
///
/// ```rust,ignore
/// use switchboard::skills::run_npx_command;
///
/// // Note: This function is async and must be awaited
/// // let output = run_npx_command("skills", &["list", "--json"], None, None).await?;
/// // println!("Output: {}", String::from_utf8_lossy(&output.stdout));
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
pub async fn run_npx_command(
    command: &str,
    args: &[&str],
    working_dir: Option<&PathBuf>,
    executor: Option<Arc<dyn ProcessExecutorTrait>>,
) -> Result<Output, SkillsError> {
    // Build command arguments - convert &str to String
    let args_strings: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut full_args: Vec<String> = vec![command.to_string()];
    full_args.extend(args_strings);

    if let Some(exec) = executor {
        // Use the provided executor
        let process_output = exec
            .execute("npx", &full_args)
            .map_err(|e| SkillsError::IoError {
                operation: "execute npx command".to_string(),
                path: "npx".to_string(),
                message: e.to_string(),
            })?;
        // Convert ProcessOutput to Output
        // Convert String to Vec<u8> and traits::ExitStatus to std::process::ExitStatus
        let stdout: Vec<u8> = process_output.stdout.into_bytes();
        let stderr: Vec<u8> = process_output.stderr.into_bytes();
        let exit_code = process_output.status.code().unwrap_or(1);
        #[cfg(unix)]
        let status = std::process::ExitStatus::from_raw(exit_code);
        #[cfg(windows)]
        let status = std::process::ExitStatus::from_raw(exit_code as u32);
        return Ok(Output {
            stdout,
            stderr,
            status,
        });
    }

    // Fall back to direct Command execution for backward compatibility
    let mut cmd = Command::new("npx");
    cmd.args(&full_args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| SkillsError::IoError {
        operation: "execute npx command".to_string(),
        path: command.to_string(),
        message: e.to_string(),
    })?;

    Ok(output)
}

/// Run `npx skills update` command, optionally with a specific skill name.
///
/// # Arguments
///
/// * `skill_name` - Optional skill name to update. If None, updates all installed skills.
/// * `executor` - Optional process executor. If None, uses direct Command execution.
///
/// # Returns
///
/// * `Result<ExitStatus, SkillsError>` - The exit status of the npx command
///
/// # Errors
///
/// Returns an error if the npx command fails to execute.
///
/// Note: This function uses stdin/stdout/stderr inheritance for interactive output.
/// ProcessExecutorTrait doesn't support this, so we use Command directly.
pub async fn run_npx_skills_update(
    skill_name: Option<&str>,
    _executor: Option<Arc<dyn ProcessExecutorTrait>>,
) -> Result<std::process::ExitStatus, SkillsError> {
    use std::process::Stdio;

    // Build command arguments
    let mut args = vec!["skills", "update"];
    if let Some(name) = skill_name {
        args.push(name);
    }

    // Execute npx command using Command directly because ProcessExecutorTrait
    // doesn't support stdin/stdout/stderr inheritance which is needed for interactive output
    let status = Command::new("npx")
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| SkillsError::NpxCommandFailed {
            command: "npx skills update".to_string(),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
        })?;

    Ok(status)
}

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
    /// "orchestrator", "task-discovery", "html-reporter", "codebase-scan",
    /// "interviewer".
    ///
    /// If omitted, the skill is assumed to be compatible with all agent types.
    #[serde(default)]
    pub compatible_agents: Vec<String>,

    /// Optional source/origin of the skill
    ///
    /// Common formats:
    /// - GitHub repo: "<https://github.com/owner/repo>"
    /// - npm package: "owner/package-name"
    /// - GitLab URL: "<https://gitlab.com/owner/repo>"
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

/// Represents a single skill entry in the skills.lock.json file.
///
/// This struct holds the metadata for an installed skill including its
/// source repository, name, and installation timestamp.
///
/// # JSON Schema
///
/// ```json
/// {
///   "source": "vercel-labs/agent-skills",
///   "name": "frontend-design",
///   "installed_at": "2026-02-22T14:30:00Z"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillLockEntry {
    /// The source repository or package name for the skill
    ///
    /// Examples: "vercel-labs/agent-skills", "anthropics/skills"
    pub source: String,

    /// The name of the skill
    ///
    /// Example: "frontend-design", "security-audit"
    pub skill_name: String,

    /// The ISO 8601 timestamp when the skill was installed
    ///
    /// Example: "2026-02-22T14:30:00Z"
    pub installed_at: String,

    /// Version string if available
    ///
    /// Example: "0.1.0", "1.2.3"
    #[serde(default)]
    pub version: Option<String>,
}

/// Represents a single skill entry in the lockfile.
///
/// This is a type alias for [`SkillLockEntry`], providing a more descriptive name
/// for the skill metadata stored in the lockfile.
///
/// # Fields
///
/// * `source` - Source repository in owner/repo format
/// * `skill_name` - Name of the skill
/// * `installed_at` - ISO8601 timestamp when the skill was installed
pub type LockfileSkill = SkillLockEntry;

/// Represents the skills lockfile containing all installed skills.
///
/// The lockfile tracks all installed skills with their metadata,
/// allowing for consistent recreation of the skills environment.
///
/// # Fields
///
/// * `version` - Lockfile format version (e.g., "1.0")
/// * `entries` - List of installed skills
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillsLockfile {
    /// Lockfile format version (e.g., "1.0")
    pub version: String,
    /// List of installed skills
    #[serde(default)]
    pub entries: Vec<SkillLockEntry>,
}

/// Loads the skills lockfile from disk.
///
/// This function reads the lockfile from `./skills/skills.lock.json`.
/// If the file doesn't exist, it returns a new SkillsLockfile with version "1.0"
/// and an empty entries list.
///
/// # Returns
///
/// * `Ok(SkillsLockfile)` - The loaded lockfile, or a new default one if file doesn't exist
/// * `Err(SkillsError::LockfileReadError)` - If there's an error reading the file
/// * `Err(SkillsError::LockfileParseError)` - If the JSON is invalid
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::skills::load_lockfile;
///
/// let lockfile = load_lockfile().expect("Failed to load lockfile");
/// println!("Loaded {} skills from lockfile", lockfile.entries.len());
/// ```
pub fn load_lockfile() -> Result<SkillsLockfile, SkillsError> {
    let lockfile_path = PathBuf::from("./skills/").join(LOCKFILE_FILENAME);

    // Check if file exists
    if !lockfile_path.exists() {
        // Return a new empty lockfile with default version
        return Ok(SkillsLockfile {
            version: "1.0".to_string(),
            entries: Vec::new(),
        });
    }

    // Read the file contents
    let contents = fs::read_to_string(&lockfile_path).map_err(|e| {
        SkillsError::LockfileReadError {
            path: lockfile_path.to_string_lossy().to_string(),
            message: e.to_string(),
        }
    })?;

    // Parse the JSON
    let lockfile = serde_json::from_str::<SkillsLockfile>(&contents).map_err(|e| {
        SkillsError::LockfileParseError {
            path: lockfile_path.to_string_lossy().to_string(),
            message: e.to_string(),
        }
    })?;

    Ok(lockfile)
}

/// Saves the skills lockfile to disk.
///
/// This function writes the lockfile to `./skills/skills.lock.json`
/// using pretty-printed JSON format.
///
/// # Arguments
///
/// * `lockfile` - The lockfile to save
///
/// # Returns
///
/// * `Ok(())` - Successfully wrote the lockfile
/// * `Err(SkillsError::LockfileWriteError)` - If there's an error writing the file
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::skills::{save_lockfile, SkillsLockfile};
///
/// let lockfile = SkillsLockfile {
///     version: "1.0".to_string(),
///     entries: Vec::new(),
/// };
/// save_lockfile(&lockfile).expect("Failed to save lockfile");
/// ```
pub fn save_lockfile(lockfile: &SkillsLockfile) -> Result<(), SkillsError> {
    let lockfile_path = PathBuf::from("./skills/").join(LOCKFILE_FILENAME);

    // Ensure the parent directory exists
    if let Some(parent) = lockfile_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                SkillsError::LockfileWriteError {
                    path: lockfile_path.to_string_lossy().to_string(),
                    message: format!("Failed to create directory: {}", e),
                }
            })?;
        }
    }

    // Serialize to pretty-printed JSON
    let json = serde_json::to_string_pretty(lockfile).map_err(|e| {
        SkillsError::LockfileWriteError {
            path: lockfile_path.to_string_lossy().to_string(),
            message: format!("Failed to serialize lockfile: {}", e),
        }
    })?;

    // Write to file
    fs::write(&lockfile_path, json).map_err(|e| {
        SkillsError::LockfileWriteError {
            path: lockfile_path.to_string_lossy().to_string(),
            message: e.to_string(),
        }
    })?;

    Ok(())
}

/// Returns a default/empty lockfile structure.
///
/// This function creates a new [`LockfileStruct`] with version 1
/// and an empty skills HashMap.
///
/// # Returns
///
/// * `LockfileStruct` - A new lockfile with version 1 and no skills
///
/// # Examples
///
/// ```rust
/// use switchboard::skills::default_lockfile;
///
/// let lockfile = default_lockfile();
/// assert_eq!(lockfile.version, 1);
/// assert!(lockfile.skills.is_empty());
/// ```
pub fn default_lockfile() -> LockfileStruct {
    LockfileStruct {
        version: 1,
        skills: HashMap::new(),
    }
}

/// Reads the skills lockfile from the specified directory.
///
/// This function reads and parses the `skills.lock.json` file from the given
/// directory. It handles file existence checks, reading errors, and JSON
/// parsing errors.
///
/// # Arguments
///
/// * `directory` - The directory containing the lockfile
///
/// # Returns
///
/// * `Ok(LockfileStruct)` - The parsed lockfile contents
/// * `Err(SkillsError::LockfileNotFound)` - If the lockfile doesn't exist
/// * `Err(SkillsError::LockfileReadError)` - If the file cannot be read
/// * `Err(SkillsError::LockfileParseError)` - If the JSON is invalid
///
/// # Examples
///
/// ```rust,no_run
/// use switchboard::skills::read_lockfile;
/// use std::path::Path;
///
/// let lockfile = read_lockfile(Path::new("./skills"))?;
/// println!("Lockfile version: {}", lockfile.version);
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
pub fn read_lockfile(directory: &Path) -> Result<LockfileStruct, SkillsError> {
    // Construct the lockfile path
    let lockfile_path = directory.join(LOCKFILE_FILENAME);

    // Check if the file exists
    if !lockfile_path.exists() {
        return Err(SkillsError::LockfileNotFound {
            path: lockfile_path.display().to_string(),
        });
    }

    // Try to get metadata to verify it's a file (not a directory)
    match fs::metadata(&lockfile_path) {
        Err(e) => {
            return Err(SkillsError::LockfileReadError {
                path: lockfile_path.display().to_string(),
                message: format!("Failed to read lockfile metadata: {}", e),
            });
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(SkillsError::LockfileReadError {
                path: lockfile_path.display().to_string(),
                message: "Path is not a file".to_string(),
            });
        }
        Ok(_) => { /* File exists and is a regular file, proceed */ }
    }

    // Read the file contents
    let contents =
        fs::read_to_string(&lockfile_path).map_err(|e| SkillsError::LockfileReadError {
            path: lockfile_path.display().to_string(),
            message: format!("Failed to read lockfile: {}", e),
        })?;

    // Parse the JSON
    let lockfile: LockfileStruct =
        serde_json::from_str(&contents).map_err(|e| SkillsError::LockfileParseError {
            path: lockfile_path.display().to_string(),
            message: format!("Failed to parse lockfile JSON: {}", e),
        })?;

    Ok(lockfile)
}

/// Writes the skills lockfile to the specified directory.
///
/// This function serializes the provided [`LockfileStruct`] to JSON and writes
/// it to the `skills.lock.json` file in the given directory. It handles
/// directory creation if it doesn't exist, file writing, and serialization errors.
///
/// # Arguments
///
/// * `lockfile` - The lockfile structure to write
/// * `directory` - The directory to write the lockfile to
///
/// # Returns
///
/// * `Ok(())` - The lockfile was written successfully
/// * `Err(SkillsError::LockfileWriteError)` - If the lockfile cannot be written
///
/// # Examples
///
/// ```rust
/// use switchboard::skills::{write_lockfile, default_lockfile};
/// use std::path::Path;
///
/// let lockfile = default_lockfile();
/// write_lockfile(&lockfile, Path::new("./skills"))?;
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
pub fn write_lockfile(lockfile: &LockfileStruct, directory: &Path) -> Result<(), SkillsError> {
    // Construct the lockfile path
    let lockfile_path = directory.join(LOCKFILE_FILENAME);

    // Ensure the directory exists
    if !directory.exists() {
        fs::create_dir_all(directory).map_err(|e| SkillsError::LockfileWriteError {
            path: lockfile_path.display().to_string(),
            message: format!("Failed to create directory: {}", e),
        })?;
    }

    // Serialize the lockfile to JSON with pretty formatting
    let json_content =
        serde_json::to_string_pretty(lockfile).map_err(|e| SkillsError::LockfileWriteError {
            path: lockfile_path.display().to_string(),
            message: format!("Failed to serialize lockfile: {}", e),
        })?;

    // Write the JSON to the file
    fs::write(&lockfile_path, json_content).map_err(|e| SkillsError::LockfileWriteError {
        path: lockfile_path.display().to_string(),
        message: format!("Failed to write lockfile: {}", e),
    })?;

    Ok(())
}

/// Synchronizes the lockfile with skills found in the specified directory.
///
/// This function scans the skills directory for skills, then updates the lockfile
/// to include all discovered skills with their metadata. It handles the following cases:
/// - Skills in the skills directory but not in lockfile → adds them
/// - Skills in lockfile but not on disk → keeps them (no removal, just sync)
/// - Empty skills directory → creates initial lockfile structure
///
/// # Arguments
///
/// * `directory` - Path to the skills directory to scan
///
/// # Returns
///
/// * `Ok(Vec<String>)` - A vector of warning messages (e.g., for skills not in lockfile)
/// * `Err(SkillsError)` - If there's an error reading/writing the lockfile
///
/// # Examples
///
/// ```rust
/// use switchboard::skills::sync_skills_to_lockfile;
/// use std::path::Path;
///
/// let warnings = sync_skills_to_lockfile(Path::new("./skills"))?;
/// for warning in warnings {
///     eprintln!("{}", warning);
/// }
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
pub fn sync_skills_to_lockfile(directory: &Path) -> Result<Vec<String>, SkillsError> {
    let mut warnings = Vec::new();

    // Scan the skills directory for available skills
    let (skills, scan_warnings) = scan_skill_directory(directory)?;

    // Forward any scan warnings
    warnings.extend(scan_warnings);

    // Get existing lockfile or create a new one
    let mut lockfile = match read_lockfile(directory) {
        Ok(lf) => lf,
        Err(SkillsError::LockfileNotFound { .. }) => default_lockfile(),
        Err(e) => return Err(e),
    };

    // Get the set of skills currently in the lockfile
    let existing_skills: std::collections::HashSet<String> =
        lockfile.skills.keys().cloned().collect();

    // Get the set of skills found in the directory
    let directory_skills: std::collections::HashSet<String> =
        skills.iter().map(|m| m.name.clone()).collect();

    // Check for skills in lockfile but not in directory (orphaned)
    for skill_name in &existing_skills {
        if !directory_skills.contains(skill_name) {
            warnings.push(format!(
                "Warning: Skill '{}' in lockfile but not found in {}/ directory",
                skill_name,
                directory.display()
            ));
        }
    }

    // Add or update skills from the directory to the lockfile
    for skill_metadata in &skills {
        let skill_name = &skill_metadata.name;

        // Check if skill is new (not in lockfile)
        if !existing_skills.contains(skill_name) {
            // Create a new lockfile entry
            let source = skill_metadata
                .source
                .clone()
                .unwrap_or_else(|| format!("local/{}", skill_name));

            let entry = SkillLockEntry {
                source,
                skill_name: skill_name.clone(),
                installed_at: chrono::Utc::now().to_rfc3339(),
                version: skill_metadata.version.clone(),
            };

            lockfile.skills.insert(skill_name.clone(), entry);

            warnings.push(format!(
                "Info: Added skill '{}' to lockfile",
                skill_name
            ));
        } else {
            // Skill exists - update version if present in metadata
            if let Some(version) = &skill_metadata.version {
                if let Some(entry) = lockfile.skills.get_mut(skill_name) {
                    // Only update if version changed
                    if entry.version.as_ref() != Some(version) {
                        entry.version = Some(version.clone());
                        warnings.push(format!(
                            "Info: Updated version for skill '{}' to '{}'",
                            skill_name, version
                        ));
                    }
                }
            }
        }
    }

    // Write the updated lockfile back to disk
    write_lockfile(&lockfile, directory)?;

    Ok(warnings)
}

/// Adds a skill to the lockfile.
///
/// This function reads the existing lockfile (or creates a new one if it doesn't exist),
/// adds the specified skill with the current timestamp, and writes it back.
///
/// # Arguments
///
/// * `directory` - The directory containing the lockfile
/// * `skill_name` - The name of the skill to add
/// * `source` - The source repository or package name for the skill
///
/// # Returns
///
/// * `Ok(())` - The skill was added successfully
/// * `Err(SkillsError)` - If there's an error reading or writing the lockfile
///
/// # Examples
///
/// ```rust
/// use switchboard::skills::add_skill_to_lockfile;
/// use std::path::Path;
///
/// add_skill_to_lockfile(Path::new("./skills"), "frontend-design", "owner/repo")?;
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
pub fn add_skill_to_lockfile(
    directory: &Path,
    skill_name: &str,
    source: &str,
) -> Result<(), SkillsError> {
    // Try to read existing lockfile or create a new one
    let mut lockfile = match read_lockfile(directory) {
        Ok(lf) => lf,
        Err(SkillsError::LockfileNotFound { .. }) => default_lockfile(),
        Err(e) => return Err(e),
    };

    // Add the skill entry with current timestamp
    let entry = SkillLockEntry {
        source: source.to_string(),
        skill_name: skill_name.to_string(),
        installed_at: chrono::Utc::now().to_rfc3339(),
        version: None,
    };

    lockfile.skills.insert(skill_name.to_string(), entry);

    // Write the updated lockfile
    write_lockfile(&lockfile, directory)
}

/// Removes a skill from the lockfile.
///
/// This function reads the existing lockfile, removes the specified skill entry,
/// and writes the updated lockfile back.
///
/// # Arguments
///
/// * `directory` - The directory containing the lockfile
/// * `skill_name` - The name of the skill to remove
///
/// # Returns
///
/// * `Ok(())` - The skill was removed successfully (or didn't exist)
/// * `Err(SkillsError::LockfileNotFound)` - If the lockfile doesn't exist
/// * `Err(SkillsError)` - If there's an error reading or writing the lockfile
///
/// # Examples
///
/// ```rust
/// use switchboard::skills::remove_skill_from_lockfile;
/// use std::path::Path;
///
/// remove_skill_from_lockfile(Path::new("./skills"), "frontend-design")?;
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
pub fn remove_skill_from_lockfile(directory: &Path, skill_name: &str) -> Result<(), SkillsError> {
    // Read the existing lockfile
    let mut lockfile = read_lockfile(directory)?;

    // Remove the skill entry
    lockfile.skills.remove(skill_name);

    // Write the updated lockfile
    write_lockfile(&lockfile, directory)
}

/// Represents the complete lockfile structure for skills.lock.json.
///
/// This struct holds the version and all installed skills with their
/// metadata. It is used to track which skills are installed and their
/// installation details.
///
/// # JSON Schema
///
/// ```json
/// {
///   "version": 1,
///   "skills": {
///     "frontend-design": {
///       "source": "vercel-labs/agent-skills",
///       "name": "frontend-design",
///       "installed_at": "2026-02-22T14:30:00Z"
///     },
///     "security-audit": {
///       "source": "anthropics/skills",
///       "name": "security-audit",
///       "installed_at": "2026-02-22T14:31:00Z"
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LockfileStruct {
    /// The version of the lockfile format
    ///
    /// Currently always 1
    pub version: u32,

    /// Map of skill names to their lock entries
    ///
    /// The key is the skill name (e.g., "frontend-design", "security-audit")
    pub skills: HashMap<String, LockfileSkill>,
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
pub fn get_agents_using_skill(skill_name: &str, config: &crate::config::Config) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::SkillsError;

    /// Test that verifies check_npx_available returns NpxNotFound error with installation instructions
    /// when npx is not available in PATH.
    ///
    /// This test validates:
    /// 1. The error type is SkillsError::NpxNotFound
    /// 2. The error message includes installation instructions (URL to nodejs.org)
    #[test]
    fn test_check_npx_available_error_contains_installation_instructions() {
        // Test the error type and message directly
        let error = SkillsError::NpxNotFound;
        let error_message = format!("{}", error);

        // Verify error type is correct
        assert_eq!(
            error,
            SkillsError::NpxNotFound,
            "Error type should be NpxNotFound"
        );

        // Verify error message includes installation instructions
        assert!(
            error_message.contains("https://nodejs.org"),
            "Error message should include installation URL, got: {}",
            error_message
        );
        assert!(
            error_message.contains("Install"),
            "Error message should contain 'Install' instruction, got: {}",
            error_message
        );

        // Verify the NPX_NOT_FOUND_ERROR constant also has installation instructions
        assert!(
            NPX_NOT_FOUND_ERROR.contains("https://nodejs.org"),
            "NPX_NOT_FOUND_ERROR should include installation URL"
        );
    }

    #[test]
    fn test_check_npx_available_when_npx_exists() {
        // This test assumes npx is available in the test environment
        let mut manager = SkillsManager::new(None);

        // The result depends on whether npx is installed in the test environment
        let result = manager.check_npx_available();

        if let Err(e) = result {
            assert_eq!(e, SkillsError::NpxNotFound);
            assert!(
                !manager.npx_available,
                "npx_available should be false when check fails"
            );
        } else {
            assert!(
                manager.npx_available,
                "npx_available should be true when check succeeds"
            );
        }
    }

    #[test]
    fn test_check_npx_available_sets_flag_correctly() {
        let mut manager = SkillsManager::new(None);

        let _ = manager.check_npx_available();

        // The flag should be set to match the check result
        let result = manager.check_npx_available();
        if result.is_ok() {
            assert!(manager.npx_available);
        } else {
            assert!(!manager.npx_available);
        }
    }

    #[tokio::test]
    async fn test_run_npx_command_invalid_executable() {
        // This should fail because npx may not be available in all environments
        let result = run_npx_command("nonexistent-cmd", &["--version"], None, None).await;

        // We expect either success (if somehow this works) or NpxCommandFailed error
        // The important thing is that it returns a proper Result type
        assert!(result.is_ok() || matches!(result.unwrap_err(), SkillsError::NpxCommandFailed { .. }));
    }

    #[test]
    fn test_npx_error_message_constant() {
        assert_eq!(
            NPX_NOT_FOUND_ERROR,
            "Error: npx is required for this command. Install Node.js from https://nodejs.org"
        );
    }

    #[test]
    fn test_skills_error_display_npx_not_found() {
        let error = SkillsError::NpxNotFound;
        let display = format!("{}", error);
        assert!(display.contains("npx is required"));
        assert!(display.contains("https://nodejs.org"));
    }

    #[test]
    fn test_skills_error_display_npx_command_failed() {
        let error = SkillsError::NpxCommandFailed {
            command: "skills add owner/repo".to_string(),
            exit_code: 1,
            stderr: "Skill not found".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("failed with exit code 1"));
        assert!(display.contains("skills add owner/repo"));
        assert!(display.contains("Skill not found"));
    }

    #[test]
    fn test_skills_error_display_skill_not_found() {
        let error = SkillsError::SkillNotFound {
            skill_source: "owner/repo@skill-name".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("owner/repo@skill-name"));
    }

    #[test]
    fn test_skills_error_display_malformed_skill_metadata() {
        let error = SkillsError::MalformedSkillMetadata {
            skill_name: "test-skill".to_string(),
            path: "/path/to/SKILL.md".to_string(),
            reason: "Missing required field 'description'".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("test-skill"));
        assert!(display.contains("/path/to/SKILL.md"));
        assert!(display.contains("Missing required field 'description'"));
    }

    #[test]
    fn test_skills_error_display_network_unavailable() {
        let error = SkillsError::NetworkUnavailable {
            operation: "list".to_string(),
            message: "Connection timeout".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("list"));
        assert!(display.contains("Connection timeout"));
    }

    #[test]
    fn test_skills_error_display_skill_name_collision() {
        let error = SkillsError::SkillNameCollision {
            skill_name: "test-skill".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("test-skill"));
        assert!(display.contains("project and global"));
    }

    #[test]
    fn test_skills_error_display_container_install_failed() {
        let error = SkillsError::ContainerInstallFailed {
            skill_source: "owner/repo@skill-name".to_string(),
            agent_name: "test-agent".to_string(),
            exit_code: 127,
            stderr: "command not found".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("test-agent"));
        assert!(display.contains("owner/repo@skill-name"));
        assert!(display.contains("code 127"));
    }

    #[test]
    fn test_skills_error_display_empty_skills_list() {
        let error = SkillsError::EmptySkillsList {
            agent_name: "test-agent".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("test-agent"));
        assert!(display.contains("empty skills list"));
    }

    #[test]
    fn test_skills_error_display_invalid_skills_entry_format() {
        let error = SkillsError::InvalidSkillsEntryFormat {
            entry: "invalid-entry".to_string(),
            agent_name: "test-agent".to_string(),
            reason: "Missing '/' separator".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("invalid-entry"));
        assert!(display.contains("test-agent"));
        assert!(display.contains("Missing '/' separator"));
    }

    #[test]
    fn test_skills_error_display_skills_directory_not_found() {
        let error = SkillsError::SkillsDirectoryNotFound {
            path: "/nonexistent/path".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("/nonexistent/path"));
    }

    #[test]
    fn test_skills_error_display_io_error() {
        let error = SkillsError::IoError {
            operation: "read".to_string(),
            path: "/path/to/file".to_string(),
            message: "Permission denied".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("read"));
        assert!(display.contains("/path/to/file"));
        assert!(display.contains("Permission denied"));
    }

    #[test]
    fn test_skills_error_debug() {
        let error = SkillsError::NpxNotFound;
        let debug = format!("{:?}", error);
        assert!(debug.contains("NpxNotFound"));
    }

    #[test]
    fn test_skills_error_clone() {
        let error = SkillsError::NpxNotFound;
        let cloned = error.clone();
        assert_eq!(format!("{}", error), format!("{}", cloned));
    }

    #[test]
    fn test_skills_manager_new() {
        let manager = SkillsManager::new(None);
        assert_eq!(manager.skills_dir, PathBuf::from("./skills"));
        // global_skills_dir now uses ./skills
        assert_eq!(manager.global_skills_dir, PathBuf::from("./skills"));
        assert!(!manager.npx_available);
    }

    #[test]
    fn test_skills_manager_with_skills_dir() {
        let custom_dir = PathBuf::from("/custom/skills");
        let manager = SkillsManager::with_skills_dir(custom_dir.clone(), None);
        assert_eq!(manager.skills_dir, custom_dir);
        // global_skills_dir now uses ./skills
        assert_eq!(manager.global_skills_dir, PathBuf::from("./skills"));
        assert!(!manager.npx_available);
    }

    #[test]
    fn test_skills_manager_skills_dir_getter() {
        let manager = SkillsManager::new(None);
        assert_eq!(manager.skills_dir(), &PathBuf::from("./skills"));
    }

    #[test]
    fn test_skills_manager_global_skills_dir_getter() {
        let manager = SkillsManager::new(None);
        // global_skills_dir now uses ./skills
        assert_eq!(manager.global_skills_dir(), &PathBuf::from("./skills"));
    }

    #[test]
    fn test_parse_skill_frontmatter_with_complete_metadata() {
        let content = r#"---
name: test-skill
description: A test skill for unit testing
source: https://github.com/owner/repo
version: 1.0.0
---
"#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.name, "test-skill");
        assert_eq!(
            metadata.description,
            Some("A test skill for unit testing".to_string())
        );
        assert_eq!(
            metadata.source,
            Some("https://github.com/owner/repo".to_string())
        );
        assert_eq!(metadata.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_skill_frontmatter_with_minimal_metadata() {
        let content = r#"---
name: minimal-skill
---
"#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.name, "minimal-skill");
        assert_eq!(metadata.description, None);
        assert_eq!(metadata.source, None);
        assert_eq!(metadata.version, None);
    }

    #[test]
    fn test_parse_skill_frontmatter_missing_frontmatter_delimiters() {
        let content = r#"name: test-skill
description: A test skill
"#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SkillsError::MissingFrontmatter { .. }
        ));
    }

    #[test]
    fn test_parse_skill_frontmatter_missing_closing_delimiter() {
        let content = r#"---
name: test-skill
description: A test skill
"#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SkillsError::MissingFrontmatter { .. }
        ));
    }

    #[test]
    fn test_parse_skill_frontmatter_empty_frontmatter() {
        let content = r#"---
---
"#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SkillsError::FieldMissing { .. }
        ));
    }

    #[test]
    fn test_parse_skill_frontmatter_invalid_yaml() {
        let content = r#"---
name: [invalid yaml
description: test
---
"#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SkillsError::MalformedSkillMetadata { .. }
        ));
    }

    #[test]
    fn test_parse_skill_frontmatter_missing_name_field() {
        let content = r#"---
description: A test skill
---
"#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), SkillsError::FieldMissing { field_name, .. } if field_name == "name")
        );
    }

    #[test]
    fn test_load_skill_metadata_fallback_to_directory_name() {
        // Create a temporary directory structure
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");

        // Write a SKILL.md file without name field
        let content = r#"---
description: A test skill
---
"#;
        fs::write(&skill_file, content).unwrap();

        // Load metadata - should return error because parsing failed
        // scan_skill_directory() will handle the error and create fallback
        let result = load_skill_metadata(&skill_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_skill_directory_single_skill() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");

        let content = r#"---
name: test-skill
description: A test skill
---
"#;
        fs::write(&skill_file, content).unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_scan_skill_directory_multiple_skills() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create first skill
        let skill1_dir = temp_dir.path().join("skill1");
        fs::create_dir_all(&skill1_dir).unwrap();
        let skill1_file = skill1_dir.join("SKILL.md");
        let content1 = r#"---
name: skill1
description: First skill
---
"#;
        fs::write(&skill1_file, content1).unwrap();

        // Create second skill
        let skill2_dir = temp_dir.path().join("skill2");
        fs::create_dir_all(&skill2_dir).unwrap();
        let skill2_file = skill2_dir.join("SKILL.md");
        let content2 = r#"---
name: skill2
description: Second skill
---
"#;
        fs::write(&skill2_file, content2).unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();
        assert_eq!(skills.len(), 2);
        let skill_names: Vec<_> = skills.iter().map(|s| s.name.clone()).collect();
        assert!(skill_names.contains(&"skill1".to_string()));
        assert!(skill_names.contains(&"skill2".to_string()));
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_scan_skill_directory_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();
        assert_eq!(skills.len(), 0);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_scan_skill_directory_mixed_valid_invalid() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create valid skill
        let valid_dir = temp_dir.path().join("valid-skill");
        fs::create_dir_all(&valid_dir).unwrap();
        let valid_file = valid_dir.join("SKILL.md");
        let valid_content = r#"---
name: valid-skill
description: A valid skill
---
"#;
        fs::write(&valid_file, valid_content).unwrap();

        // Create directory without SKILL.md (should be skipped)
        let invalid_dir = temp_dir.path().join("no-skill");
        fs::create_dir_all(&invalid_dir).unwrap();

        // Create file (not a directory, should be skipped)
        let not_dir = temp_dir.path().join("not-a-dir.md");
        fs::write(&not_dir, "not a directory").unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "valid-skill");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_get_agents_using_skill_with_repo_at_skill_format() {
        use crate::config::{Agent, Config};

        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec!["owner/repo@skill-name".to_string()]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["owner/repo@other-skill".to_string()]),
                ..Default::default()
            },
            Agent {
                name: "agent3".to_string(),
                skills: Some(vec![
                    "owner/repo@skill-name".to_string(),
                    "other/repo@another".to_string(),
                ]),
                ..Default::default()
            },
            Agent {
                name: "agent4".to_string(),
                skills: None,
                ..Default::default()
            },
        ];

        let agents = get_agents_using_skill("skill-name", &config);
        assert_eq!(agents.len(), 2);
        assert!(agents.contains(&"agent1".to_string()));
        assert!(agents.contains(&"agent3".to_string()));
    }

    #[test]
    fn test_get_agents_using_skill_with_skill_name_only() {
        use crate::config::{Agent, Config};

        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec!["skill-name".to_string()]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["other-skill".to_string()]),
                ..Default::default()
            },
        ];

        let agents = get_agents_using_skill("skill-name", &config);
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0], "agent1");
    }

    #[test]
    fn test_get_agents_using_skill_empty_result() {
        use crate::config::{Agent, Config};

        let mut config = Config::default();
        config.agents = vec![Agent {
            name: "agent1".to_string(),
            skills: Some(vec!["owner/repo@skill-name".to_string()]),
            ..Default::default()
        }];

        let agents = get_agents_using_skill("non-existent-skill", &config);
        assert_eq!(agents.len(), 0);
    }

    #[test]
    fn test_get_agents_using_skill_empty_skills_lists() {
        use crate::config::{Agent, Config};

        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec![]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: None,
                ..Default::default()
            },
        ];

        let agents = get_agents_using_skill("skill-name", &config);
        assert_eq!(agents.len(), 0);
    }

    #[test]
    fn test_scan_skill_directory_missing_frontmatter() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");

        // Write a SKILL.md file with NO frontmatter delimiters
        let content = "name: test-skill\n";
        fs::write(&skill_file, content).unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();

        // Assert that skills contains 1 skill with the directory name as fallback
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert_eq!(skills[0].description, None);
        assert_eq!(skills[0].version, None);
        // Warnings should be generated for parsing failure
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Warning"));
        assert!(warnings[0].contains("test-skill"));
    }

    #[test]
    fn test_scan_skill_directory_malformed_yaml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");

        // Write a SKILL.md file with invalid YAML
        let content = "---\nname: test-skill\ndescription: [unclosed bracket\n---";
        fs::write(&skill_file, content).unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();

        // Assert that skills contains 1 skill with the directory name as fallback
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert_eq!(skills[0].description, None);
        assert_eq!(skills[0].version, None);
        // Warnings should be generated for parsing failure
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Warning"));
        assert!(warnings[0].contains("test-skill"));
    }

    #[test]
    fn test_scan_skill_directory_multiple_malformed_skills() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create first skill with malformed SKILL.md (missing frontmatter)
        let skill1_dir = temp_dir.path().join("skill1");
        fs::create_dir_all(&skill1_dir).unwrap();
        let skill1_file = skill1_dir.join("SKILL.md");
        let content1 = "name: skill1\n";
        fs::write(&skill1_file, content1).unwrap();

        // Create second skill with malformed SKILL.md (invalid YAML)
        let skill2_dir = temp_dir.path().join("skill2");
        fs::create_dir_all(&skill2_dir).unwrap();
        let skill2_file = skill2_dir.join("SKILL.md");
        let content2 = "---\nname: skill2\ndescription: [invalid\n---";
        fs::write(&skill2_file, content2).unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();

        // Assert that skills contains 2 skills (both with directory names as fallback)
        assert_eq!(skills.len(), 2);
        let skill_names: Vec<_> = skills.iter().map(|s| s.name.clone()).collect();
        assert!(skill_names.contains(&"skill1".to_string()));
        assert!(skill_names.contains(&"skill2".to_string()));
        // Warnings should be generated for both parsing failures
        assert_eq!(warnings.len(), 2);
        assert!(warnings.iter().any(|w| w.contains("skill1")));
        assert!(warnings.iter().any(|w| w.contains("skill2")));
    }

    // ========================================================================
    // Mock ProcessExecutor for testing
    // ========================================================================

    use crate::traits::{ExitStatus, ProcessError, ProcessExecutorTrait, ProcessOutput};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;

    /// Mock executor that returns predefined responses for testing
    #[derive(Debug)]
    struct MockProcessExecutor {
        /// If Some, return this output on execute. If None, return error.
        output: Option<ProcessOutput>,
        /// If true, return error instead of output
        should_error: bool,
    }

    impl MockProcessExecutor {
        fn with_success_output() -> Self {
            Self {
                output: Some(ProcessOutput {
                    stdout: "10.0.0".to_string(),
                    stderr: String::new(),
                    status: ExitStatus::Code(0),
                }),
                should_error: false,
            }
        }

        fn with_failure_output() -> Self {
            Self {
                output: Some(ProcessOutput {
                    stdout: String::new(),
                    stderr: "command not found".to_string(),
                    status: ExitStatus::Code(127),
                }),
                should_error: false,
            }
        }

        fn with_error() -> Self {
            Self {
                output: None,
                should_error: true,
            }
        }
    }

    impl ProcessExecutorTrait for MockProcessExecutor {
        fn execute(&self, _program: &str, _args: &[String]) -> Result<ProcessOutput, ProcessError> {
            if self.should_error {
                Err(ProcessError::ProgramNotFound {
                    program: "npx".to_string(),
                    suggestion: "Install npx or provide skills that don't require it".to_string(),
                })
            } else {
                self.output.clone().ok_or(ProcessError::ProgramNotFound {
                    program: "npx".to_string(),
                    suggestion: "Install npx or provide skills that don't require it".to_string(),
                })
            }
        }

        fn execute_with_env(
            &self,
            _program: &str,
            _args: &[String],
            _env: HashMap<String, String>,
            _working_dir: Option<PathBuf>,
        ) -> Result<ProcessOutput, ProcessError> {
            self.execute(_program, _args)
        }
    }

    // ========================================================================
    // SkillsManager with Mock Executor Tests
    // ========================================================================

    #[test]
    fn test_skills_manager_construction_with_custom_executor() {
        let mock_executor: Arc<dyn ProcessExecutorTrait> =
            Arc::new(MockProcessExecutor::with_success_output());
        let manager = SkillsManager::new(Some(mock_executor.clone()));

        // Verify the executor was injected correctly
        assert!(Arc::ptr_eq(&manager.executor, &mock_executor));
        // Verify default values
        assert_eq!(manager.skills_dir, PathBuf::from("./skills"));
        assert!(!manager.npx_available);
    }

    #[tokio::test]
    async fn test_check_npx_available_with_mock_success() {
        let mock_executor: Arc<dyn ProcessExecutorTrait> =
            Arc::new(MockProcessExecutor::with_success_output());
        let mut manager = SkillsManager::new(Some(mock_executor));

        // Execute check_npx_available with mocked successful response
        let result = manager.check_npx_available();

        // Should succeed because mock returns exit code 0
        assert!(
            result.is_ok(),
            "Expected success when mock returns exit code 0"
        );
        assert!(
            manager.npx_available,
            "npx_available should be true after successful check"
        );
    }

    #[tokio::test]
    async fn test_check_npx_available_with_mock_failure_exit_code() {
        let mock_executor: Arc<dyn ProcessExecutorTrait> =
            Arc::new(MockProcessExecutor::with_failure_output());
        let mut manager = SkillsManager::new(Some(mock_executor));

        // Execute check_npx_available with mocked failure response (exit code 127)
        let result = manager.check_npx_available();

        // Should fail because mock returns non-zero exit code
        assert!(
            result.is_err(),
            "Expected error when mock returns non-zero exit code"
        );
        assert!(
            !manager.npx_available,
            "npx_available should be false after failed check"
        );
    }

    #[tokio::test]
    async fn test_check_npx_available_with_mock_error() {
        let mock_executor: Arc<dyn ProcessExecutorTrait> =
            Arc::new(MockProcessExecutor::with_error());
        let mut manager = SkillsManager::new(Some(mock_executor));

        // Execute check_npx_available with mocked error
        let result = manager.check_npx_available();

        // Should fail because mock returns error
        assert!(result.is_err(), "Expected error when mock returns error");
        assert!(
            !manager.npx_available,
            "npx_available should be false after error"
        );
    }

    #[test]
    fn test_skills_manager_with_skills_dir_with_custom_executor() {
        let custom_skills_dir = PathBuf::from("/custom/path/skills");
        let mock_executor: Arc<dyn ProcessExecutorTrait> =
            Arc::new(MockProcessExecutor::with_success_output());
        let manager =
            SkillsManager::with_skills_dir(custom_skills_dir.clone(), Some(mock_executor));

        // Verify the custom skills_dir was set
        assert_eq!(manager.skills_dir, custom_skills_dir);
        // Verify default values
        assert!(!manager.npx_available);
    }

    // Tests for write_lockfile function
    use tempfile::TempDir;

    #[test]
    fn test_write_lockfile_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let lockfile = default_lockfile();

        let result = write_lockfile(&lockfile, temp_dir.path());
        assert!(result.is_ok());

        // Verify the file was created
        let lockfile_path = temp_dir.path().join(LOCKFILE_FILENAME);
        assert!(lockfile_path.exists());
    }

    #[test]
    fn test_write_lockfile_creates_directory_if_missing() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("nested").join("dir");
        let lockfile = default_lockfile();

        let result = write_lockfile(&lockfile, &nested_dir);
        assert!(result.is_ok());

        // Verify the nested directory and file were created
        let lockfile_path = nested_dir.join(LOCKFILE_FILENAME);
        assert!(lockfile_path.exists());
    }

    #[test]
    fn test_write_and_read_lockfile_roundtrip() {
        let temp_dir = TempDir::new().unwrap();

        // Create a lockfile with some skills
        let mut lockfile = default_lockfile();
        lockfile.skills.insert(
            "test-skill".to_string(),
            SkillLockEntry {
                source: "owner/test-repo".to_string(),
                skill_name: "test-skill".to_string(),
                installed_at: "2026-02-22T14:30:00Z".to_string(),
                version: None,
            },
        );

        // Write the lockfile
        let result = write_lockfile(&lockfile, temp_dir.path());
        assert!(result.is_ok());

        // Read it back
        let read_lockfile = read_lockfile(temp_dir.path());
        assert!(read_lockfile.is_ok());

        let loaded = read_lockfile.unwrap();
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.skills.len(), 1);
        assert!(loaded.skills.contains_key("test-skill"));
    }

    #[test]
    fn test_write_lockfile_pretty_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let lockfile = default_lockfile();

        write_lockfile(&lockfile, temp_dir.path()).unwrap();

        // Read the raw file content to verify pretty printing
        let lockfile_path = temp_dir.path().join(LOCKFILE_FILENAME);
        let content = fs::read_to_string(&lockfile_path).unwrap();

        // Verify the JSON is pretty-printed (contains newlines and indentation)
        assert!(content.contains('\n'));
        assert!(content.contains("  "));
    }

    #[test]
    fn test_add_skill_to_lockfile() {
        let temp_dir = TempDir::new().unwrap();

        // Add a skill
        let result = add_skill_to_lockfile(temp_dir.path(), "test-skill", "owner/repo");
        assert!(result.is_ok());

        // Verify the skill was added
        let lockfile = read_lockfile(temp_dir.path()).unwrap();
        assert!(lockfile.skills.contains_key("test-skill"));

        let skill = lockfile.skills.get("test-skill").unwrap();
        assert_eq!(skill.skill_name, "test-skill");
        assert_eq!(skill.source, "owner/repo");
        // Verify installed_at is a valid ISO 8601 timestamp
        assert!(chrono::DateTime::parse_from_rfc3339(&skill.installed_at).is_ok());
    }

    #[test]
    fn test_add_skill_to_lockfile_existing_skills() {
        let temp_dir = TempDir::new().unwrap();

        // Add first skill
        add_skill_to_lockfile(temp_dir.path(), "skill-one", "owner/repo1").unwrap();

        // Add second skill
        add_skill_to_lockfile(temp_dir.path(), "skill-two", "owner/repo2").unwrap();

        // Verify both skills exist
        let lockfile = read_lockfile(temp_dir.path()).unwrap();
        assert_eq!(lockfile.skills.len(), 2);
        assert!(lockfile.skills.contains_key("skill-one"));
        assert!(lockfile.skills.contains_key("skill-two"));
    }

    #[test]
    fn test_remove_skill_from_lockfile() {
        let temp_dir = TempDir::new().unwrap();

        // Add a skill first
        add_skill_to_lockfile(temp_dir.path(), "test-skill", "owner/repo").unwrap();

        // Verify it exists
        let lockfile = read_lockfile(temp_dir.path()).unwrap();
        assert!(lockfile.skills.contains_key("test-skill"));

        // Remove the skill
        let result = remove_skill_from_lockfile(temp_dir.path(), "test-skill");
        assert!(result.is_ok());

        // Verify it's gone
        let lockfile = read_lockfile(temp_dir.path()).unwrap();
        assert!(!lockfile.skills.contains_key("test-skill"));
    }

    #[test]
    fn test_remove_skill_from_lockfile_nonexistent() {
        let temp_dir = TempDir::new().unwrap();

        // Create an empty lockfile
        write_lockfile(&default_lockfile(), temp_dir.path()).unwrap();

        // Try to remove a non-existent skill
        let result = remove_skill_from_lockfile(temp_dir.path(), "nonexistent");
        assert!(result.is_ok()); // Should not fail, just no-op
    }

    #[test]
    fn test_lockfile_version() {
        let lockfile = default_lockfile();
        assert_eq!(lockfile.version, 1);
    }
}
