//! Lockfile handling for skills management

use crate::skills::SkillsError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Filename for the skills lockfile
pub const LOCKFILE_FILENAME: &str = "skills.lock.json";

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

#[allow(dead_code)]
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
#[allow(dead_code)]
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
    let contents =
        fs::read_to_string(&lockfile_path).map_err(|e| SkillsError::LockfileReadError {
            path: lockfile_path.to_string_lossy().to_string(),
            message: e.to_string(),
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
#[allow(dead_code)]
pub fn save_lockfile(lockfile: &SkillsLockfile) -> Result<(), SkillsError> {
    let lockfile_path = PathBuf::from("./skills/").join(LOCKFILE_FILENAME);

    // Ensure the parent directory exists
    if let Some(parent) = lockfile_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| SkillsError::LockfileWriteError {
                path: lockfile_path.to_string_lossy().to_string(),
                message: format!("Failed to create directory: {}", e),
            })?;
        }
    }

    // Serialize to pretty-printed JSON
    let json =
        serde_json::to_string_pretty(lockfile).map_err(|e| SkillsError::LockfileWriteError {
            path: lockfile_path.to_string_lossy().to_string(),
            message: format!("Failed to serialize lockfile: {}", e),
        })?;

    // Write to file
    fs::write(&lockfile_path, json).map_err(|e| SkillsError::LockfileWriteError {
        path: lockfile_path.to_string_lossy().to_string(),
        message: e.to_string(),
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
#[allow(dead_code)]
pub fn sync_skills_to_lockfile(directory: &Path) -> Result<Vec<String>, SkillsError> {
    use crate::skills::metadata::scan_skill_directory;
    use chrono::Utc;

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
                installed_at: Utc::now().to_rfc3339(),
                version: skill_metadata.version.clone(),
            };

            lockfile.skills.insert(skill_name.clone(), entry);

            warnings.push(format!("Info: Added skill '{}' to lockfile", skill_name));
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
    use chrono::Utc;

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
        installed_at: Utc::now().to_rfc3339(),
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
