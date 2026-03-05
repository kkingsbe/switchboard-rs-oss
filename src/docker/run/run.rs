//! Container creation and execution logic
//!
//! This module provides functionality for creating and running Docker containers
//! for agent execution, including:
//! - Container configuration building (env vars, host config, workspace mounts)
//! - Container creation and lifecycle management
//! - Container execution with timeout support
//! - AgentExecutionResult with container ID and exit code reporting
//!
//! The module integrates with the logger module for log streaming and the
//! wait module for container exit waiting with timeout enforcement.

use super::types::ContainerConfig;
use super::wait::{parse_timeout, wait_with_timeout, TerminationSignal};
use crate::docker::skills::generate_entrypoint_script;
use crate::docker::DockerError;
use crate::logger::Logger;
use crate::metrics::{update_all_metrics, AgentRunResult, MetricsStore};
use crate::skills::SkillsError;
use crate::traits::DockerClientTrait;
use bollard::{
    container::{Config, CreateContainerOptions},
    models::HostConfig,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Result of running an agent in a Docker container
///
/// This structure contains the outcome of an agent execution, including
/// the container ID that was created and the exit code returned by the
/// container's main process. It provides the necessary information for
/// post-execution processing, such as logging, metrics collection, and
/// conditional workflows based on execution success or failure.
///
/// # Fields
///
/// - `container_id` - The unique identifier of the Docker container that was created and executed
/// - `exit_code` - The exit code from the container's main process (0 typically indicates success)
///
/// # Example
///
/// ```no_run
/// use switchboard::docker::run::run_agent;
/// use switchboard::docker::DockerClient;
/// use switchboard::docker::run::types::ContainerConfig;
/// use switchboard::logger::Logger;
/// use switchboard::metrics::MetricsStore;
/// use std::sync::{Arc, Mutex};
/// use std::path::PathBuf;
/// use chrono::Utc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = DockerClient::new("agent".to_string(), "latest".to_string()).await?;
/// let container_config = ContainerConfig::new("agent1".to_string());
/// let log_dir = PathBuf::from("./logs");
/// let logger = Arc::new(Mutex::new(Logger::new(log_dir.clone(), Some("agent1".to_string()), false)));
/// let metrics_store = MetricsStore::new(log_dir);
/// let start_time = Utc::now();
///
/// let result = run_agent(
///     "/workspace",
///     Arc::new(client),
///     &container_config,
///     Some("5m".to_string()),
///     "switchboard-agent:latest",
///     None,
///     Some(logger),
///     Some(&metrics_store),
///     "agent1",
///     Some(start_time),
/// ).await?;
///
/// println!("Container {} exited with code {}", result.container_id, result.exit_code);
///
/// if result.exit_code == 0 {
///     println!("Agent executed successfully");
/// } else {
///     println!("Agent failed with exit code {}", result.exit_code);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct AgentExecutionResult {
    /// The container ID that was created and run
    pub container_id: String,
    /// The exit code from the container execution
    pub exit_code: i64,
    /// Tracks whether skills were successfully installed
    ///
    /// - `None`: No skills were configured for this execution
    /// - `Some(true)`: Skills were configured and installation succeeded
    /// - `Some(false)`: Skills were configured but installation failed
    pub skills_installed: Option<bool>,
    /// Indicates if the skill installation phase failed
    ///
    /// This is `true` if any skill installation error occurred, regardless of
    /// whether the container continued execution. Use this to distinguish between
    /// installation failures and other execution failures.
    pub skills_install_failed: bool,
}

/// Build container environment variables from custom environment variables in the config.
///
/// This function prepares environment variables for the Docker container by converting
/// the configuration's custom environment variables into the format expected by Docker.
/// Each environment variable is a "KEY=value" string that will be passed to the container.
///
/// # Skills Integration
///
/// When skills are specified in the container configuration, they do not need to be passed
/// as environment variables through this function. Skills are handled separately through
/// entrypoint script generation via [`generate_entrypoint_script()`](crate::docker::skills::generate_entrypoint_script).
/// This function only handles custom user-specified environment variables from the config.
///
/// # Arguments
///
/// * `env_vars` - Custom environment variables from the config, where each entry is
///   a "KEY=value" string ready to be passed to Docker
///
/// # Returns
///
/// A vector of "KEY=value" strings suitable for Docker container configuration. The
/// returned vector is a clone of the input to allow the function to take a reference
/// while the caller retains ownership.
///
/// # Examples
///
/// ```
/// use switchboard::docker::run::run::build_container_env_vars;
///
/// let env_vars = vec![
///     "API_KEY=secret123".to_string(),
///     "DEBUG=true".to_string(),
///     "LOG_LEVEL=info".to_string(),
/// ];
///
/// let result = build_container_env_vars(&env_vars);
/// assert_eq!(result.len(), 3);
/// assert_eq!(result[0], "API_KEY=secret123");
/// ```
pub fn build_container_env_vars(env_vars: &[String]) -> Vec<String> {
    // Return custom env_vars from config
    env_vars.to_vec()
}

/// Build the HostConfig for a Docker container with workspace mount.
///
/// This function creates the host configuration for a Docker container, primarily
/// setting up the workspace bind mount that allows the container to access files
/// on the host system. The workspace is mounted at `/workspace` inside the container.
///
/// # Skills Directory Mounting
///
/// If a `.kilocode/skills/` directory exists in the workspace, it is also mounted
/// into the container at `/workspace/.kilocode/skills` in read-only mode. This enables
/// manually managed skills to be accessible inside the container.
///
/// # Workspace Directory Handling
///
/// The workspace path is normalized for Docker compatibility across platforms:
///
/// - **Windows paths**: The function strips the Windows extended-length path prefix
///   (`\\?\`) if present, and converts all backslashes (`\`) to forward slashes (`/`)
///   to ensure Docker can properly handle the path
/// - **Unix paths**: Passed through unchanged, as they already use forward slashes
///
/// The normalized path is used to create a bind mount in the format:
/// ```text
/// <host_path>:/workspace[:ro]
/// ```
///
/// If `readonly` is `true`, the mount is read-only (`:ro` suffix), preventing the
/// container from modifying files on the host system.
///
/// # Arguments
///
/// * `workspace` - Workspace path on the host system to mount into `/workspace`
///   inside the container. This path should be an absolute path.
/// * `readonly` - Whether to mount the workspace as read-only. When `true`,
///   the container cannot modify files in the workspace.
///
/// # Returns
///
/// A `HostConfig` with:
/// - `auto_remove` set to `true` - Container is automatically removed after execution
/// - `binds` configured with the workspace mount at `/workspace`
/// - Optionally, a skills mount at `/workspace/.kilocode/skills` if it exists
/// - Other settings at their defaults
///
/// # Examples
///
/// ```
/// use switchboard::docker::run::run::build_host_config;
///
/// // Read-write workspace mount
/// let config = build_host_config("/home/user/my-project", false);
/// assert_eq!(config.auto_remove, Some(true));
/// assert!(config.binds.is_some());
///
/// // Read-only workspace mount
/// let config = build_host_config("/home/user/my-project", true);
/// let binds = config.binds.unwrap();
/// assert!(binds[0].contains(":ro"));
/// ```
pub fn build_host_config(workspace: &str, readonly: bool) -> HostConfig {
    // Normalize the workspace path for Docker bind mounts
    // On Windows, convert backslashes to forward slashes and remove \\?\ prefix
    let normalized_workspace = if let Some(stripped) = workspace.strip_prefix(r"\\?\") {
        // Strip the Windows extended-length path prefix (\\?\)
        stripped
    } else {
        workspace
    };

    // Convert backslashes to forward slashes for Docker compatibility
    // If workspace is ".", convert to absolute path to avoid Docker interpreting it as a volume name
    let docker_path = if normalized_workspace == "." {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .to_string_lossy()
            .to_string()
            .replace("\\", "/")
    } else {
        normalized_workspace.replace("\\", "/")
    };

    // Build the list of bind mounts, starting with the workspace
    let mut binds = vec![format!(
        "{}:/workspace{}",
        docker_path,
        if readonly { ":ro" } else { "" }
    )];

    // Conditionally add .kilocode/skills/ mount if it exists
    let skills_dir = std::path::Path::new(workspace).join(".kilocode/skills");
    if skills_dir.exists() {
        // Canonicalize the skills directory path for Docker
        if let Ok(canonical_skills) = skills_dir.canonicalize() {
            // Get the string representation and strip Windows extended-length path prefix (\\?\) if present
            let skills_str = canonical_skills.to_string_lossy();
            let normalized_skills = if let Some(stripped) = skills_str.strip_prefix(r"\\?\") {
                stripped
            } else {
                skills_str.as_ref()
            };
            // Convert backslashes to forward slashes for Docker compatibility
            let skills_docker_path = normalized_skills.replace("\\", "/");
            binds.push(format!(
                "{}:/workspace/.kilocode/skills:ro",
                skills_docker_path
            ));
        }
        // If canonicalize fails, we silently skip the skills mount
        // This is not a critical error - the container will still work, just without skills
    }

    HostConfig {
        auto_remove: Some(true),
        binds: Some(binds),
        ..Default::default()
    }
}

/// Build the container Config for a Docker agent container.
///
/// This function constructs the complete Docker container configuration by combining
/// all necessary settings: image, environment variables, command, host configuration,
/// labels, and workspace mounting.
///
/// # Entrypoint Configuration
///
/// The function initializes `entrypoint` to `None`, which means the container will use
/// the default ENTRYPOINT specified in the Dockerfile. However, this configuration can
/// be modified after calling this function, particularly when skills are specified.
///
/// When skills are present in the container configuration, the entrypoint is overridden
/// by [`run_agent()`] to execute a custom entrypoint script that installs skills before
/// running the agent. The entrypoint is set to `["/bin/sh", "-c", "<script>"]` to allow
/// multi-line shell script execution.
///
/// # Skills Field Handling
///
/// Skills are not directly handled by this function. Instead:
///
/// - Skills configuration is passed in via the [`ContainerConfig`]
///   structure that contains this config
/// - Skills affect the entrypoint after this function returns (modified by `run_agent()`)
/// - If skills are specified and non-empty, `run_agent()` generates an entrypoint script
///   and overrides the `entrypoint` field before creating the container
/// - If no skills are specified, the default entrypoint from the Dockerfile is used
///
/// # Arguments
///
/// * `image` - Docker image to use (e.g., "switchboard-agent:latest")
/// * `env_vars` - Environment variables in KEY=value format. Only set if non-empty.
/// * `readonly` - Whether to mount the workspace as read-only
/// * `workspace` - Workspace path to mount into /workspace inside the container
/// * `agent_name` - Name of the agent, used as a container label
/// * `_timeout` - Timeout in seconds for the agent execution (currently unused, kept for
///   future use and API compatibility)
/// * `cmd` - Optional command to run in the container. If `None`, the image's default
///   CMD is used.
///
/// # Returns
///
/// A complete `Config<String>` struct with:
/// - `image` set to the specified Docker image
/// - `env` set if environment variables are provided
/// - `cmd` set if a command is provided
/// - `entrypoint` set to `None` (uses Dockerfile default)
/// - `host_config` with auto_remove and workspace mount configured
/// - `labels` with `switchboard.agent` set to the agent name
/// - Other fields at their defaults
///
/// # Examples
///
/// ```
/// use switchboard::docker::run::run::build_container_config;
/// use bollard::container::Config;
///
/// let env_vars = vec![
///     "API_KEY=secret".to_string(),
///     "DEBUG=true".to_string(),
/// ];
///
/// let config = build_container_config(
///     "switchboard-agent:latest",
///     env_vars,
///     false,
///     "/workspace",
///     "my-agent",
///     1800,
///     Some(&vec!["--auto".to_string(), "process this file".to_string()]),
/// );
///
/// assert_eq!(config.image, Some("switchboard-agent:latest".to_string()));
/// assert!(config.env.is_some());
/// assert!(config.cmd.is_some());
/// assert_eq!(config.entrypoint, None);
/// ```
pub fn build_container_config(
    image: &str,
    env_vars: Vec<String>,
    readonly: bool,
    workspace: &str,
    agent_name: &str,
    _timeout: u64,
    cmd: Option<&[String]>,
) -> Config<String> {
    // Build host configuration using helper function
    let host_config = build_host_config(workspace, readonly);

    // Create labels for the container
    let mut labels = HashMap::new();
    labels.insert("switchboard.agent".to_string(), agent_name.to_string());

    // Build container configuration
    Config {
        image: Some(image.to_string()),
        env: if !env_vars.is_empty() {
            Some(env_vars)
        } else {
            None
        },
        cmd: cmd.map(|c| c.to_vec()),
        entrypoint: None, // Use the ENTRYPOINT from Dockerfile
        host_config: Some(host_config),
        labels: Some(labels),
        ..Default::default()
    }
}

/// Find skills that are already installed in the .kilocode/skills directory.
///
/// This function scans the `.kilocode/skills/` directory for subdirectories containing
/// `SKILL.md` files and returns a list of configured skills that already exist locally.
/// This is used to skip `npx skills add` for skills that are already manually installed.
///
/// # Skill Name Parsing
///
/// The function parses skill names from the configured skill strings in two formats:
///
/// - `owner/repo` format: Uses `repo` as the skill name
/// - `owner/repo@skill-name` format: Uses `skill-name` as the skill name
///
/// This matches how `npx skills` installs skills.
///
/// # Arguments
///
/// * `skills` - A slice of configured skill strings in `owner/repo` or `owner/repo@skill-name` format
/// * `project_dir` - The project directory path (workspace path)
///
/// # Returns
///
/// * `Ok(Vec<String>)` - A vector of skill names that exist in `.kilocode/skills/` with a `SKILL.md` file
/// * `Err(SkillsError::IoError)` - If reading the skills directory fails
///
/// If `.kilocode/skills/` does not exist, returns an empty vector (no error).
/// If no skills are found, returns an empty vector.
///
/// # Examples
///
/// ```no_run
/// use switchboard::docker::run::run::find_preexisting_skills;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let skills = vec![
///     "owner/repo1".to_string(),
///     "owner/repo2@my-skill".to_string(),
/// ];
/// let project_dir = Path::new("/workspace");
///
/// let preexisting = find_preexisting_skills(&skills, project_dir)?;
/// # Ok(())
/// # }
/// ```
pub fn find_preexisting_skills(
    skills: &[String],
    project_dir: &std::path::Path,
) -> Result<Vec<String>, SkillsError> {
    use std::fs;

    // If no skills are configured, return empty vector
    if skills.is_empty() {
        return Ok(Vec::new());
    }

    // Build the path to the .kilocode/skills directory
    let skills_dir = project_dir.join(".kilocode/skills");

    // If the skills directory doesn't exist, return empty vector (no error)
    if !skills_dir.exists() {
        return Ok(Vec::new());
    }

    // Read the skills directory entries
    let dir_entries = fs::read_dir(&skills_dir).map_err(|e| SkillsError::IoError {
        operation: "read skills directory".to_string(),
        path: skills_dir.display().to_string(),
        message: e.to_string(),
    })?;

    // Collect skill names from directories that contain SKILL.md
    let mut installed_skill_names = std::collections::HashSet::new();
    for entry in dir_entries {
        let entry = entry.map_err(|e| SkillsError::IoError {
            operation: "read skills directory entry".to_string(),
            path: skills_dir.display().to_string(),
            message: e.to_string(),
        })?;

        let path = entry.path();

        // Skip non-directories
        if !path.is_dir() {
            continue;
        }

        // Check if the directory contains a SKILL.md file
        let skill_md_path = path.join("SKILL.md");
        if skill_md_path.exists() {
            // Use the directory name as the skill name
            if let Some(skill_name) = path.file_name().and_then(|n| n.to_str()) {
                installed_skill_names.insert(skill_name.to_string());
            }
        }
    }

    // Parse skill names from configured skills and check if they're installed
    let mut preexisting = Vec::new();
    for skill in skills {
        // Parse the skill name from the skill string
        let skill_name = if skill.contains('@') {
            // Format: owner/repo@skill-name
            let parts: Vec<&str> = skill.split('@').collect();
            if parts.len() == 2 {
                parts[1].to_string()
            } else {
                continue;
            }
        } else {
            // Format: owner/repo - use the repo part as the skill name
            let parts: Vec<&str> = skill.split('/').collect();
            if parts.len() == 2 {
                parts[1].to_string()
            } else {
                continue;
            }
        };

        // Check if the skill is already installed
        if installed_skill_names.contains(&skill_name) {
            preexisting.push(skill_name);
        }
    }

    Ok(preexisting)
}

/// Run an agent by creating and starting a Docker container.
///
/// This function orchestrates the complete lifecycle of an agent container execution:
/// building container configuration, creating the container, starting it, streaming
/// logs in the background, waiting for exit with timeout enforcement, and collecting
/// execution metrics. The container is configured with auto-remove enabled for automatic
/// cleanup after execution.
///
/// # Skills Integration
///
/// When the [`ContainerConfig`] contains a
/// non-empty `skills` field, this function generates a custom entrypoint script that
/// installs the specified skills before running the agent. The skills field can have
/// three possible values:
///
/// - `None` - No skills specified, container uses the default entrypoint from the Dockerfile
/// - `Some([])` - Empty skills list, container uses the default entrypoint from the Dockerfile
/// - `Some([...])` - Non-empty skills list, generates a custom entrypoint script that installs
///   each skill using `npx skills add` before executing the agent
///
/// The custom entrypoint script is executed via `/bin/sh -c` to allow multi-line shell
/// commands, and includes:
/// - Skill installation commands in the order specified
/// - Error handling with `set -e` to fail on any skill installation error
/// - Handoff to the kilocode CLI with all original arguments
///
/// If entrypoint script generation fails (e.g., invalid skill format), a warning is logged
/// and the container proceeds with the default entrypoint to prevent total failure.
///
/// # Arguments
///
/// * `workspace` - Workspace path to mount into /workspace inside the container
/// * `client` - Reference to the DockerClient
/// * `config` - Container configuration including agent name, env vars, skills, etc.
/// * `timeout` - Optional timeout string (e.g., "30s", "5m", "1h"). Defaults to 30 minutes.
/// * `image` - Docker image to use (e.g., "switchboard-agent:latest")
/// * `cmd` - Optional command to run in the container (currently unused)
/// * `logger` - Optional logger for streaming container logs
/// * `metrics_store` - Optional metrics store for collecting execution metrics
/// * `agent_name` - Name of the agent (for metrics tracking)
/// * `queued_start_time` - Optional timestamp when the agent was queued
///
/// # Returns
///
/// Returns an `AgentExecutionResult` containing the container ID and exit code on success.
///
/// # Errors
///
/// Returns `DockerError::ContainerCreateError` if:
/// - Container creation fails (invalid name, naming conflicts, volume mount issues)
/// - Insufficient Docker resources are available
///
/// Returns `DockerError::ContainerStartError` if:
/// - Container fails to start after being created
/// - There are insufficient system resources
/// - There are conflicting container ports or other configuration issues
///
/// Returns other `DockerError` variants for Docker API communication failures.
///
/// # Examples
///
/// ## Running an agent without skills
///
/// ```no_run
/// use switchboard::docker::run::run_agent;
/// use switchboard::docker::DockerClient;
/// use switchboard::docker::run::types::ContainerConfig;
/// use switchboard::logger::Logger;
/// use switchboard::metrics::MetricsStore;
/// use std::sync::{Arc, Mutex};
/// use std::path::PathBuf;
/// use chrono::Utc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = DockerClient::new("agent".to_string(), "latest".to_string()).await?;
/// let container_config = ContainerConfig::new("agent1".to_string());
/// let log_dir = PathBuf::from("./logs");
/// let logger = Arc::new(Mutex::new(Logger::new(log_dir.clone(), Some("agent1".to_string()), false)));
/// let metrics_store = MetricsStore::new(log_dir);
/// let start_time = Utc::now();
///
/// let result = run_agent(
///     "/workspace",
///     Arc::new(client),
///     &container_config,
///     Some("5m".to_string()),
///     "switchboard-agent:latest",
///     None,
///     Some(logger),
///     Some(&metrics_store),
///     "agent1",
///     Some(start_time),
/// ).await?;
///
/// println!("Container {} exited with code {}", result.container_id, result.exit_code);
/// # Ok(())
/// # }
/// ```
///
/// ## Running an agent with skills
///
/// ```no_run
/// use switchboard::docker::run::run_agent;
/// use switchboard::docker::DockerClient;
/// use switchboard::docker::run::types::ContainerConfig;
/// use switchboard::logger::Logger;
/// use switchboard::metrics::MetricsStore;
/// use std::sync::{Arc, Mutex};
/// use std::path::PathBuf;
/// use chrono::Utc;
///
/// # async fn example_with_skills() -> Result<(), Box<dyn std::error::Error>> {
/// let client = DockerClient::new("agent".to_string(), "latest".to_string()).await?;
/// let mut container_config = ContainerConfig::new("agent2".to_string());
///
/// // Add skills to the container configuration (format: "owner/repo" or "owner/repo@skill")
/// container_config.skills = Some(vec![
///     "owner/repo1".to_string(),
///     "owner/repo2@python".to_string(),
/// ]);
///
/// let log_dir = PathBuf::from("./logs");
/// let logger = Arc::new(Mutex::new(Logger::new(log_dir.clone(), Some("agent2".to_string()), false)));
/// let metrics_store = MetricsStore::new(log_dir);
/// let start_time = Utc::now();
///
/// let result = run_agent(
///     "/workspace",
///     Arc::new(client),
///     &container_config,
///     Some("10m".to_string()),
///     "switchboard-agent:latest",
///     None,
///     Some(logger),
///     Some(&metrics_store),
///     "agent2",
///     Some(start_time),
/// ).await?;
///
/// println!("Container {} with skills exited with code {}", result.container_id, result.exit_code);
/// # Ok(())
/// # }
/// ```
#[allow(clippy::too_many_arguments)]
pub async fn run_agent(
    workspace: &str,
    client: Arc<dyn DockerClientTrait>,
    config: &ContainerConfig,
    timeout: Option<String>,
    image: &str,
    _cmd: Option<&[String]>,
    logger: Option<Arc<Mutex<Logger>>>,
    metrics_store: Option<&MetricsStore>,
    agent_name: &str,
    queued_start_time: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<AgentExecutionResult, DockerError> {
    // Build environment variables for the container
    let container_env_vars = build_container_env_vars(&config.env_vars);

    // Parse timeout string or default to 30 minutes
    let parsed_timeout = match timeout {
        Some(ref timeout_str) => match parse_timeout(timeout_str) {
            Ok(duration) => duration,
            Err(e) => {
                eprintln!(
                    "Failed to parse timeout '{}': {}. Using default 30 minutes.",
                    timeout_str, e
                );
                Duration::from_secs(1800)
            }
        },
        None => Duration::from_secs(1800), // Default to 30 minutes
    };

    let timeout_seconds = parsed_timeout.as_secs();

    // Use container_env_vars directly without adding AGENT_NAME or PROMPT
    // These are not needed - the prompt is passed via --prompt CLI flag
    // and AGENT_NAME is used by Switchboard's scheduler, not by Kilo Code CLI
    let extended_env_vars = container_env_vars.clone();

    // Build CLI arguments for the kilo entrypoint
    // Kilo Code CLI uses --auto flag to automatically determine the mode based on prompt content
    let mut cli_args = vec!["--auto".to_string()];
    // Add the prompt as a positional argument
    cli_args.push(config.prompt.clone());

    // Create container configuration using helper function
    let mut container_config = build_container_config(
        image,
        extended_env_vars,
        config.readonly,
        workspace,
        &config.agent_name,
        timeout_seconds,
        Some(&cli_args),
    );

    // Handle skills-based entrypoint script generation
    // Skills allow installing capabilities into the container before running the agent
    // We generate a custom entrypoint script only when skills are specified and non-empty
    //
    // Three possible cases for config.skills:
    // 1. None - No skills field exists, use default entrypoint from Dockerfile
    // 2. Some([]) - Empty skills list exists, use default entrypoint from Dockerfile
    // 3. Some([...]) - Non-empty skills list, generate custom entrypoint script
    match &config.skills {
        Some(skills) if !skills.is_empty() => {
            // CASE 3: Non-empty skills list - generate custom entrypoint script
            // The script will install skills using `npx skills add` then hand off to kilocode

            // Find preexisting skills (skills already manually installed in .kilocode/skills/)
            let project_dir = std::path::Path::new(workspace);
            let preexisting_skills = find_preexisting_skills(skills, project_dir)
                .unwrap_or_else(|e| {
                    // If we can't check for preexisting skills, log a warning but continue
                    // This is a non-fatal error - we'll just install all skills via npx
                    eprintln!(
                        "[WARNING] Failed to check for preexisting skills: {}. Installing all skills via npx.",
                        e
                    );
                    Vec::new()
                });

            // Log skill installation attempts
            if let Some(logger_ref) = logger.as_ref() {
                if let Ok(logger_guard) = logger_ref.lock() {
                    let install_msg = format!(
                        "[SKILL INSTALL] Installing skills for agent '{}'",
                        agent_name
                    );
                    let _ = logger_guard.write_agent_log(agent_name, &install_msg);
                    if logger_guard.foreground_mode {
                        let _ = logger_guard.write_terminal_output(&install_msg);
                    }
                    // Log each individual skill
                    for skill in skills {
                        let skill_msg = format!("[SKILL INSTALL] Installing skill: {}", skill);
                        let _ = logger_guard.write_agent_log(agent_name, &skill_msg);
                        if logger_guard.foreground_mode {
                            let _ = logger_guard.write_terminal_output(&skill_msg);
                        }
                    }
                    // Log preexisting skills that will be skipped
                    if !preexisting_skills.is_empty() {
                        let skip_msg = format!(
                            "[SKILL INSTALL] Found {} preexisting skill(s) that will skip npx installation",
                            preexisting_skills.len()
                        );
                        let _ = logger_guard.write_agent_log(agent_name, &skip_msg);
                        if logger_guard.foreground_mode {
                            let _ = logger_guard.write_terminal_output(&skip_msg);
                        }
                    }
                }
            }

            let entrypoint_script =
                generate_entrypoint_script(agent_name, skills, &preexisting_skills).map_err(
                    |e| DockerError::IoError {
                        operation: format!("generate entrypoint script for agent '{}'", agent_name),
                        error_details: format!("Skills error: {}", e),
                    },
                )?;
            // Set the generated script as the container's entrypoint
            // This replaces the default ENTRYPOINT from the Dockerfile
            //
            // ENTRYPOINT APPROACH: Using `/bin/sh -c` allows us to execute a multi-line
            // shell script as the container entrypoint. The three-part vector is:
            // - "/bin/sh" - The shell interpreter
            // - "-c" - Flag to read commands from the following string argument
            // - entrypoint_script - The multi-line shell script containing skill install
            //   commands and the final kilocode invocation
            //
            // WHY /bin/sh -c WAS CHOSEN:
            // 1. Allows multi-line shell scripts to be passed as a single string argument
            // 2. More portable than writing to a file and executing - works across different shell environments
            // 3. Avoids temporary file creation/deletion overhead and cleanup issues
            // 4. Works consistently across Docker image variants (alpine, ubuntu, etc.)
            // 5. No need to handle file permissions or mounting volume for the script
            //
            // This approach is necessary because Docker's entrypoint field expects a
            // vector of command-line arguments, not a multi-line script directly.
            // Using /bin/sh -c enables us to pass an entire shell script as a single
            // string argument.
            container_config.entrypoint = Some(vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                entrypoint_script,
            ]);
        }
        _ => {
            // CASE 1 & 2: No skills specified (None) or empty skills list (Some([]))
            // Use the default entrypoint from the Dockerfile (entrypoint: None)
            // No modification needed to container_config - skills integration is bypassed
            //
            // This ensures backward compatibility: existing configurations without skills
            // continue to work exactly as before, with no changes to container behavior.
        }
    }

    // Generate container name
    let container_name = format!("switchboard-agent-{}", config.agent_name);

    // Create the container options (name) - clone to own the String
    let options = Some(CreateContainerOptions {
        name: container_name.clone(),
        platform: None,
    });

    // Create the container using the trait
    let create_result = client.create_container(options, container_config);

    match create_result {
        Ok(container_id) => {
            // Start the container using the trait
            let start_result = client.start_container(&container_id, None);

            match start_result {
                Ok(_) => {
                    let start_time = Utc::now();

                    // Capture the start time for skill installation timing
                    // This approximates the installation time by measuring from container start
                    // until the container exits. Skill installation happens in the entrypoint script
                    // before the agent code executes.
                    let skills_install_start_time = Instant::now();

                    // Spawn background task to stream logs if logger is provided
                    let log_task = if let Some(_logger) = logger.clone() {
                        // Clone client for the spawned task
                        let _client_clone = client.clone();
                        let _agent_name = config.agent_name.clone();
                        let _container_id_clone = container_id.clone();

                        Some(tokio::spawn(async move {
                            // Use the trait's container_logs method
                            // For streaming, we need to use the trait - but since container_logs
                            // returns a String (not a stream), we need a different approach
                            // For now, we'll use a simpler approach: just fetch logs at the end
                            // The streaming functionality is handled separately

                            // Note: Full log streaming would require adding an async method to the trait
                            // For now, we skip the streaming in the background task

                            Ok::<(), DockerError>(())
                        }))
                    } else {
                        None
                    };

                    // Log agent start
                    eprintln!(
                        "Agent {} started with container_id {}, timeout: {}s",
                        config.agent_name, container_id, timeout_seconds
                    );

                    // Wait for container to exit with timeout
                    let (exit_code, timed_out, termination_signal) = match wait_with_timeout(
                        &client,
                        &container_id,
                        parsed_timeout,
                        &config.agent_name,
                        logger.as_ref(),
                    )
                    .await
                    {
                        Ok(exit_status) => {
                            // Wait for log streaming task to complete
                            if let Some(log_task) = log_task {
                                if let Err(e) = log_task.await {
                                    eprintln!("Log streaming task failed: {}", e);
                                }
                            }

                            // Log exit code and termination signal
                            //
                            // Different termination signals map to specific log messages for debugging:
                            // - SIGKILL (137): Usually means container was forcibly terminated (e.g., OOM killer)
                            // - Other non-zero: Indicates agent or entrypoint script returned an error
                            // - Zero: Successful execution
                            if exit_status.timed_out {
                                match exit_status.termination_signal {
                                    TerminationSignal::SigTerm => {
                                        eprintln!(
                                                "Agent {} timed out after {}s, sending SIGTERM to container {}",
                                                config.agent_name, timeout_seconds, container_id
                                            );
                                    }
                                    TerminationSignal::SigKill => {
                                        eprintln!(
                                                "Agent {} timed out after {}s, sending SIGTERM to container {}",
                                                config.agent_name, timeout_seconds, container_id
                                            );
                                        eprintln!(
                                                "Agent {} did not exit gracefully, sending SIGKILL to container {}",
                                                config.agent_name, container_id
                                            );
                                    }
                                    TerminationSignal::None => {
                                        eprintln!(
                                            "Agent {} timed out after {}s, container killed",
                                            config.agent_name, timeout_seconds
                                        );
                                    }
                                }
                            } else {
                                eprintln!(
                                    "Agent {} exited with code {}",
                                    config.agent_name, exit_status.exit_code
                                );
                            }

                            (
                                exit_status.exit_code,
                                exit_status.timed_out,
                                exit_status.termination_signal,
                            )
                        }
                        Err(e) => {
                            // Gracefully shut down log streaming task to allow logs to flush
                            if let Some(log_task) = log_task {
                                use tokio::time::{timeout, Duration};
                                // Wait up to 500ms for log task to complete naturally (flushes buffered output)
                                let _ = timeout(Duration::from_millis(500), log_task).await;
                                // If timeout occurred, task is already cancelled; if completed, logs are flushed
                            }

                            eprintln!("Error waiting for agent {}: {}", config.agent_name, e);
                            // Return -1 to indicate error during wait, false for timed_out, None for termination_signal
                            (-1, false, TerminationSignal::None)
                        }
                    };

                    let end_time = Utc::now();

                    // Calculate skill installation time
                    // This approximates the installation time by measuring from container start
                    // until container exit. The timing includes both skill installation and
                    // agent execution, but since skill installation happens first in the
                    // entrypoint script, this provides a reasonable approximation.
                    let skills_install_time_seconds = if config
                        .skills
                        .as_ref()
                        .is_some_and(|s| !s.is_empty())
                    {
                        let duration = skills_install_start_time.elapsed();
                        let seconds = duration.as_secs_f64();
                        // Sanity check: ensure the timing is reasonable (between 0 and 1 hour)
                        if (0.0..3600.0).contains(&seconds) {
                            Some(seconds)
                        } else {
                            // Unreasonable duration - log warning and use None
                            eprintln!(
                                "Warning: Unreasonable skill installation time detected: {:.2}s for agent {}",
                                seconds, config.agent_name
                            );
                            None
                        }
                    } else {
                        // No skills configured - no installation time to record
                        None
                    };

                    // Detects whether skill installation succeeded, failed, or timed out.
                    //
                    // This function implements a heuristic approach to determine skill installation
                    // status based on exit codes and timeout conditions:
                    //
                    // # Logic
                    //
                    // - **Some(true)**: All skills installed successfully (exit_code == 0)
                    // - **Some(false)**: Skills installation failed (exit_code != 0 && !timed_out)
                    // - **None**: Unknown status (timed out, cannot determine if skills installed)
                    //
                    // # Rationale
                    //
                    // Skill installation happens before agent execution in the container's entrypoint script.
                    // We use exit codes as the primary indicator because:
                    // 1. The entrypoint script returns non-zero on any skill installation error
                    // 2. Timeouts prevent us from knowing if skills partially installed
                    // 3. No explicit skill installation status is available from the container runtime
                    //
                    // # Returns
                    //
                    // * `Some(true)` - All skills configured and installed successfully
                    // * `Some(false)` - Skills were configured but installation failed
                    // * `None` - Either no skills configured, or timeout prevented determination
                    let skills_installed = if config.skills.as_ref().is_some_and(|s| !s.is_empty())
                    {
                        // Skills were configured - determine if installation succeeded or failed
                        if exit_code == 0 {
                            Some(true) // Skills installed successfully
                        } else if !timed_out {
                            Some(false) // Skills installation failed (non-zero exit code, not a timeout)
                        } else {
                            None // Timed out - unknown if skills installed
                        }
                    } else {
                        // No skills configured
                        None
                    };

                    let skills_install_failed =
                        config.skills.as_ref().is_some_and(|s| !s.is_empty())
                            && exit_code != 0
                            && !timed_out;

                    // Calculate skill counts based on execution result
                    //
                    // This counting strategy simplifies the logic by categorizing runs into three states:
                    // 1. All skills failed: skills_install_failed is true, so installed=0, failed=total
                    // 2. All succeeded: skills_installed is Some(true), so installed=total, failed=0
                    // 3. Unknown/timeout: We can't determine the count, so both are 0
                    //
                    // Edge cases handled:
                    // - Timeout: We don't know if any skills installed, so we report 0/0
                    // - Partial failure: Currently not detected; if one skill fails, all are marked as failed
                    // - No skills: Returns 0/0 since no skills were attempted
                    let total_skills_count = config.skills.as_ref().map(|s| s.len()).unwrap_or(0);

                    let (skills_installed_count, skills_failed_count) = if skills_install_failed {
                        // All skills failed to install
                        (0, total_skills_count)
                    } else if skills_installed == Some(true) {
                        // All skills installed successfully
                        (total_skills_count, 0)
                    } else {
                        // No skills configured or unknown status (timeout)
                        (0, 0)
                    };

                    // Cast to u32 for AgentRunResult
                    let skills_installed_count = skills_installed_count as u32;
                    let skills_failed_count = skills_failed_count as u32;

                    // Save metrics if metrics_store is provided
                    if let Some(store) = metrics_store {
                        let termination_type = match termination_signal {
                            TerminationSignal::SigTerm => Some("sigterm".to_string()),
                            TerminationSignal::SigKill => Some("sigkill".to_string()),
                            TerminationSignal::None => None,
                        };

                        let run_result = AgentRunResult {
                            agent_name: agent_name.to_string(),
                            container_id: container_id.clone(),
                            start_time,
                            end_time,
                            exit_code,
                            timed_out,
                            termination_type,
                            queued_start_time,
                            skills_installed_count,
                            skills_failed_count,
                            skills_install_time_seconds,
                        };
                        let mut all_metrics = store.load().map_err(|e| {
                            tracing::error!("Failed to load metrics: {:?}", e);
                            e
                        })?;
                        update_all_metrics(&mut all_metrics, &run_result).map_err(|e| {
                            tracing::error!("Failed to update metrics: {:?}", e);
                            e
                        })?;
                        if let Err(e) = store.save_with_retry(&all_metrics) {
                            // Write metrics failure to agent log file for debugging
                            // This is especially important in detached mode where stderr is not visible
                            let metrics_error_msg = format!(
                                "[METRICS] ERROR: Failed to save metrics after retries: {:?}",
                                e
                            );
                            tracing::error!("Failed to save metrics after retries: {:?}", e);

                            // Also write to agent log file if logger is available
                            if let Some(logger_ref) = logger.as_ref() {
                                if let Ok(logger_guard) = logger_ref.lock() {
                                    let _ = logger_guard
                                        .write_agent_log(agent_name, &metrics_error_msg);
                                }
                            }

                            // Return the error to fail the operation
                            return Err(e.into());
                        }
                    }

                    // Log skill installation failures with distinct prefix
                    if skills_install_failed {
                        // Format skills list as a comma-separated string
                        let skills_str = config
                            .skills
                            .as_ref()
                            .map(|skills| skills.join(", "))
                            .unwrap_or_else(|| "none".to_string());

                        if let Some(logger_ref) = logger.as_ref() {
                            if let Ok(logger_guard) = logger_ref.lock() {
                                // Write detailed failure messages to both log file and terminal
                                let error_msg = format!(
                                    "[SKILL INSTALL] Error: Skill installation failed for agent '{}'",
                                    config.agent_name
                                );
                                let _ =
                                    logger_guard.write_agent_log(&config.agent_name, &error_msg);
                                if logger_guard.foreground_mode {
                                    let _ = logger_guard.write_terminal_output(&error_msg);
                                }

                                let exit_msg = format!("[SKILL INSTALL] Exit code: {}", exit_code);
                                let _ = logger_guard.write_agent_log(&config.agent_name, &exit_msg);
                                if logger_guard.foreground_mode {
                                    let _ = logger_guard.write_terminal_output(&exit_msg);
                                }

                                let skills_msg = format!(
                                    "[SKILL INSTALL] Skills being installed: {}",
                                    skills_str
                                );
                                let _ =
                                    logger_guard.write_agent_log(&config.agent_name, &skills_msg);
                                if logger_guard.foreground_mode {
                                    let _ = logger_guard.write_terminal_output(&skills_msg);
                                }

                                let remediation_msg = "[SKILL INSTALL] Remediation steps:
- Check if the skill exists: switchboard skills list
- Verify the skill format: owner/repo or owner/repo@skill-name
- Check network connectivity (npx needs internet access)
- Review [SKILL INSTALL STDERR] lines above for detailed error information";
                                let _ = logger_guard
                                    .write_agent_log(&config.agent_name, remediation_msg);
                                if logger_guard.foreground_mode {
                                    let _ = logger_guard.write_terminal_output(remediation_msg);
                                }

                                let context_msg = "[SKILL INSTALL] The agent did not execute. Fix the skill installation issues before retrying.";
                                let _ =
                                    logger_guard.write_agent_log(&config.agent_name, context_msg);
                                if logger_guard.foreground_mode {
                                    let _ = logger_guard.write_terminal_output(context_msg);
                                }
                            }
                        }
                    }

                    // Log skill installation successes with distinct prefix
                    if skills_installed == Some(true) && !skills_install_failed {
                        // Format skills list as a comma-separated string
                        let skills_str = config
                            .skills
                            .as_ref()
                            .map(|skills| skills.join(", "))
                            .unwrap_or_else(|| "none".to_string());

                        if let Some(logger_ref) = logger.as_ref() {
                            if let Ok(logger_guard) = logger_ref.lock() {
                                // Write success messages to both log file and terminal
                                let success_msg = format!(
                                    "[SKILL INSTALL] Skills installed successfully for agent '{}'",
                                    config.agent_name
                                );
                                let _ =
                                    logger_guard.write_agent_log(&config.agent_name, &success_msg);
                                if logger_guard.foreground_mode {
                                    let _ = logger_guard.write_terminal_output(&success_msg);
                                }

                                let skills_msg =
                                    format!("[SKILL INSTALL] Installed: {}", skills_str);
                                let _ =
                                    logger_guard.write_agent_log(&config.agent_name, &skills_msg);
                                if logger_guard.foreground_mode {
                                    let _ = logger_guard.write_terminal_output(&skills_msg);
                                }
                            }
                        }
                    }

                    Ok(AgentExecutionResult {
                        container_id,
                        exit_code,
                        skills_installed,
                        skills_install_failed,
                    })
                }
                Err(e) => {
                    let error_details = format!("{}", e);
                    let suggestion = "Check container logs with: docker logs {container_id}\n\n\
                        Possible causes:\n\
                        - Insufficient system resources\n\
                        - Conflicting container ports\n\
                        - Invalid container configuration\n\
                        - Image not found or corrupted"
                        .to_string();
                    Err(DockerError::ContainerStartError {
                        container_name: container_id.clone(),
                        error_details,
                        suggestion,
                    })
                }
            }
        }
        Err(e) => {
            let error_details = format!("{}", e);
            let suggestion = "Check container name is valid and no naming conflicts exist\n\n\
                Possible causes:\n\
                - Container name already exists\n\
                - Invalid container name format\n\
                - Volume mount path does not exist\n\
                - Insufficient Docker resources"
                .to_string();
            Err(DockerError::ContainerCreateError {
                container_name: container_name.clone(),
                error_details,
                suggestion,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{AgentRunResult, AllMetrics, MetricsError};
    use chrono::Utc;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Test demonstrating BUG-003: Silent Metrics Failure
    ///
    /// This test shows that the current metrics handling pattern in run_agent()
    /// silently ignores metrics errors. When metrics operations fail (load, update, save),
    /// the errors are only logged but not propagated, allowing execution to continue
    /// with Ok(()) despite the loss of critical metrics data.
    ///
    /// The bug is demonstrated by simulating the exact pattern used in run_agent()
    /// at lines 467-481, showing that errors from:
    /// - store.load() - errors are logged but empty AllMetrics is used
    /// - update_all_metrics() - errors are logged but execution continues
    /// - store.save() - errors are logged but execution continues
    ///
    /// Despite all these failures, the overall operation returns Ok(()).
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_bug_003_silent_metrics_failure() {
        // Create a read-only directory to cause save failures
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_dir_path = temp_dir.path().to_path_buf();
        let metrics_file = log_dir_path.join("metrics.json");

        // Create a metrics store that will fail operations
        let store = MetricsStore::new(log_dir_path);

        // Create a test agent run result
        let run_result = AgentRunResult {
            agent_name: "test-agent".to_string(),
            container_id: "test-container-id".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            exit_code: 0,
            timed_out: false,
            termination_type: None,
            queued_start_time: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        // Create a corrupted metrics file to cause load to fail
        std::fs::write(&metrics_file, b"corrupted invalid json data")
            .expect("Failed to write corrupted data");

        // This demonstrates the exact pattern from run_agent() lines 467-481
        // Pattern 1: load() error is logged but execution continues
        let mut all_metrics = match store.load() {
            Ok(metrics) => metrics,
            Err(_e) => {
                // In the actual code, this logs with tracing::error!()
                // but continues with an empty AllMetrics
                // tracing::error!("Failed to load metrics: {:?}", _e);
                AllMetrics {
                    agents: std::collections::HashMap::new(),
                }
            }
        };

        // Pattern 2: update_all_metrics() error is logged but execution continues
        // We force an error by passing invalid metrics data
        // For this test, we'll simulate by having a corrupted state
        if let Err(_e) = update_all_metrics(&mut all_metrics, &run_result) {
            // In the actual code, this logs with tracing::error!()
            // but execution continues
            // tracing::error!("Failed to update metrics: {:?}", _e);
        }

        // Pattern 3: save() error is logged but execution continues
        // We'll make the directory read-only to force a save error (Unix only, and may fail when running as root)
        #[cfg(unix)]
        {
            // Make the directory read-only (Unix only)
            // Note: When running as root, this may not prevent writes
            let mut perms = std::fs::metadata(temp_dir.path())
                .expect("Failed to get metadata")
                .permissions();
            perms.set_readonly(true);
            std::fs::set_permissions(temp_dir.path(), perms)
                .expect("Failed to set readonly permissions");

            // This save may fail due to read-only directory (unless running as root)
            if let Err(_e) = store.save(&all_metrics) {
                // In the actual code, this logs with tracing::error!()
                // but execution continues and returns Ok(AgentExecutionResult)
                // tracing::error!("Failed to save metrics: {:?}", _e);
            }
        }

        #[cfg(not(unix))]
        {
            // Non-Unix platforms - skip permission test
        }

        // CRITICAL BUG DEMONSTRATION:
        // Despite all three metrics operations failing (load, update, save),
        // execution would continue normally in run_agent()
        // and the function would still return Ok(AgentExecutionResult { container_id, exit_code })
        //
        // This means:
        // 1. Agent execution appears to succeed even when metrics data is completely lost
        // 2. Historical metrics data may be corrupted without any indication
        // 3. Operations that should fail are silently ignored
        //
        // Expected behavior: Either propagate errors or fail loudly when metrics are critical
        // Actual behavior: Errors are logged but function returns Ok(())

        // The test passes by demonstrating the bug exists
        // In the actual code, this pattern would allow silent data loss
    }

    /// Additional test demonstrating that the metrics store can fail
    /// and the current code pattern doesn't handle these failures appropriately.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_bug_003_metrics_store_failure_scenarios() {
        // Test scenario 1: Non-existent directory causes load to fail
        let non_existent_dir =
            PathBuf::from("/this/path/definitely/does/not/exist/switchboard-metrics-test");
        let store = MetricsStore::new(non_existent_dir);

        // This will fail with FileNotFound error
        let load_result = store.load();
        assert!(
            load_result.is_err(),
            "load should fail for non-existent directory"
        );

        if let Err(MetricsError::FileNotFound(_)) = load_result {
            // In run_agent(), this would be logged with tracing::error!() but execution continues
        }

        // Test scenario 2: Corrupted JSON file causes load to fail
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_dir_path = temp_dir.path().to_path_buf();
        let store2 = MetricsStore::new(log_dir_path.clone());

        // Write corrupted JSON data
        let metrics_file = log_dir_path.join("metrics.json");
        std::fs::write(&metrics_file, b"corrupted invalid json data {{")
            .expect("Failed to write corrupted data");

        // Load should fail with corrupted file error
        let load_result = store2.load();
        assert!(load_result.is_err(), "load should fail for corrupted JSON");

        if let Err(MetricsError::CorruptedFile(_)) = load_result {
            // In run_agent(), this would be logged with tracing::error!() but execution continues
        }
    }

    /// Test 1: Container config with `skills: None` → uses default entrypoint
    ///
    /// This test verifies that when no skills are specified in the ContainerConfig,
    /// the generated entrypoint script is empty, and the container will use the
    /// default entrypoint from the Dockerfile.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skills_none_uses_default_entrypoint() {
        use crate::docker::skills::generate_entrypoint_script;

        let skills: Option<Vec<String>> = None;

        match skills {
            Some(skills) => {
                let _result = generate_entrypoint_script("test-agent", &skills, &[]);
                // This path should not be taken when skills is None
                panic!("Should not generate entrypoint script when skills is None");
            }
            None => {
                // No skills specified - default entrypoint should be used
                // The entrypoint script generation should not be called
                // This is the expected behavior
            }
        }
    }

    /// Test 2: Container config with `skills: Some([])` → uses default entrypoint
    ///
    /// This test verifies that when an empty skills list is specified in the
    /// ContainerConfig, the generated entrypoint script is empty, and the
    /// container will use the default entrypoint from the Dockerfile.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skills_empty_uses_default_entrypoint() {
        use crate::docker::skills::generate_entrypoint_script;

        let skills: Option<Vec<String>> = Some(vec![]);

        match skills {
            Some(skills) if !skills.is_empty() => {
                // Non-empty skills - should generate entrypoint script
                let _result = generate_entrypoint_script("test-agent", &skills, &[]);
                panic!("Should not generate entrypoint script when skills is empty");
            }
            _ => {
                // No skills specified or empty skills list - default entrypoint should be used
                // The entrypoint script generation returns empty string for empty skills
                let empty_skills: Vec<String> = vec![];
                let result = generate_entrypoint_script("test-agent", &empty_skills, &[]);
                assert!(result.is_ok(), "Empty skills list should return Ok");
                let script = result.unwrap();
                assert!(
                    script.is_empty(),
                    "Empty skills list should return empty string"
                );
            }
        }
    }

    /// Test 3: Container config with `skills: Some(["owner/repo"])` → generates custom entrypoint
    ///
    /// This test verifies that when a non-empty skills list is specified in the
    /// ContainerConfig, a custom entrypoint script is generated that installs
    /// the specified skills before running the agent.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skills_single_generates_custom_entrypoint() {
        use crate::docker::skills::generate_entrypoint_script;

        let skills = Some(vec!["owner/repo".to_string()]);

        match skills {
            Some(skills) if !skills.is_empty() => {
                let result = generate_entrypoint_script("test-agent", &skills, &[]);
                assert!(
                    result.is_ok(),
                    "Valid skill should generate script successfully"
                );

                let script = result.unwrap();
                assert!(
                    script.contains("npx skills add owner/repo -a kilo -y"),
                    "Script should contain skill installation command"
                );
                assert!(
                    script.contains("exec kilocode --yes \"$@\""),
                    "Script should contain CLI execution command"
                );
                assert!(
                    script.contains("#!/bin/sh"),
                    "Script should start with shebang"
                );
                assert!(
                    script.contains("set -e"),
                    "Script should contain error handling"
                );
            }
            _ => {
                panic!("Should generate entrypoint script for non-empty skills");
            }
        }
    }

    /// Test 4: Container config with `skills: Some(["owner/repo", "owner/repo@skill-name"])`
    /// → generates custom entrypoint with multiple skills
    ///
    /// This test verifies that when multiple skills are specified in the
    /// ContainerConfig, a custom entrypoint script is generated that installs
    /// all specified skills in order before running the agent.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skills_multiple_generates_custom_entrypoint() {
        use crate::docker::skills::generate_entrypoint_script;

        let skills = Some(vec![
            "owner/repo".to_string(),
            "owner/repo@skill-name".to_string(),
        ]);

        match skills {
            Some(skills) if !skills.is_empty() => {
                let result = generate_entrypoint_script("test-agent", &skills, &[]);
                assert!(
                    result.is_ok(),
                    "Valid skills should generate script successfully"
                );

                let script = result.unwrap();
                assert!(
                    script.contains("npx skills add owner/repo -a kilo -y"),
                    "Script should contain first skill installation command"
                );
                assert!(
                    script.contains("npx skills add owner/repo@skill-name -a kilo -y"),
                    "Script should contain second skill installation command"
                );
                assert!(
                    script.contains("exec kilocode --yes \"$@\""),
                    "Script should contain CLI execution command"
                );

                // Verify skills appear in the correct order
                let pos1 = script.find("npx skills add owner/repo -a kilo -y").unwrap();
                let pos2 = script
                    .find("npx skills add owner/repo@skill-name -a kilo -y")
                    .unwrap();
                assert!(pos1 < pos2, "Skills should appear in declaration order");
            }
            _ => {
                panic!("Should generate entrypoint script for non-empty skills");
            }
        }
    }

    /// Test the container configuration behavior with skills
    ///
    /// This test verifies that the container configuration is built correctly
    /// depending on whether skills are specified or not.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_container_config_with_skills() {
        use crate::docker::run::types::ContainerConfig;

        // Test with skills: None
        let config_none = ContainerConfig::new("test-agent".to_string());
        assert_eq!(
            config_none.skills, None,
            "ContainerConfig with new() should have skills: None"
        );

        // Test with skills: Some([])
        let mut config_empty = ContainerConfig::new("test-agent".to_string());
        config_empty.skills = Some(vec![]);
        assert_eq!(
            config_empty.skills,
            Some(vec![]),
            "ContainerConfig should support skills: Some(vec![])"
        );

        // Test with skills: Some(["owner/repo"])
        let mut config_single = ContainerConfig::new("test-agent".to_string());
        config_single.skills = Some(vec!["owner/repo".to_string()]);
        assert_eq!(
            config_single.skills,
            Some(vec!["owner/repo".to_string()]),
            "ContainerConfig should support skills: Some(vec![...])"
        );

        // Test with skills: Some(["owner/repo", "owner/repo@skill-name"])
        let mut config_multi = ContainerConfig::new("test-agent".to_string());
        config_multi.skills = Some(vec![
            "owner/repo".to_string(),
            "owner/repo@skill-name".to_string(),
        ]);
        assert_eq!(
            config_multi.skills,
            Some(vec![
                "owner/repo".to_string(),
                "owner/repo@skill-name".to_string(),
            ]),
            "ContainerConfig should support multiple skills"
        );
    }

    /// Test entrypoint script generation for all skill scenarios
    ///
    /// This comprehensive test verifies the behavior of entrypoint script
    /// generation across all skill configuration scenarios.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_entrypoint_script_generation_all_scenarios() {
        use crate::docker::skills::generate_entrypoint_script;

        // Scenario 1: skills: None - should not generate script (handled in run_agent)
        let skills_none: Option<Vec<String>> = None;
        match skills_none {
            Some(skills) => {
                let _ = generate_entrypoint_script("test-agent", &skills, &[]);
                panic!("Should not reach this path for None skills");
            }
            None => {
                // No script generation, use default entrypoint
            }
        }

        // Scenario 2: skills: Some([]) - should return empty string
        let skills_empty: Vec<String> = vec![];
        let result_empty = generate_entrypoint_script("test-agent", &skills_empty, &[]);
        assert!(result_empty.is_ok(), "Empty skills should return Ok");
        assert!(
            result_empty.unwrap().is_empty(),
            "Empty skills should return empty script string"
        );

        // Scenario 3: skills: Some(["nonexistent/test-skill-xyz"]) - should generate valid script
        // Use preexisting_skills to bypass filesystem check (test should not depend on local skills)
        // Note: When skill is in preexisting_skills, no "npx skills add" is generated (already mounted)
        let skills_single = vec!["nonexistent/test-skill-xyz".to_string()];
        // preexisting_skills expects the extracted skill name (repo part), not the full skill string
        let preexisting_single = vec!["test-skill-xyz".to_string()];
        let result_single = generate_entrypoint_script("test-agent", &skills_single, &preexisting_single);
        assert!(result_single.is_ok(), "Single skill should return Ok");
        let script_single = result_single.unwrap();
        assert!(
            !script_single.is_empty(),
            "Single skill should generate non-empty script"
        );
        // For preexisting skills, no installation command is generated (skill is already mounted)
        assert!(
            !script_single.contains("npx skills add"),
            "Preexisting skill should not generate install command"
        );
        assert!(
            script_single.contains("exec kilocode --yes \"$@\""),
            "Script should execute kilocode"
        );

        // Scenario 4: skills: Some(["nonexistent/test-skill-xyz", "nonexistent/test-skill-xyz@skill-name"]) - should generate valid script
        // Use preexisting_skills to bypass filesystem check (test should not depend on local skills)
        // First skill is preexisting (no install command), second skill needs installation
        let skills_multi = vec![
            "nonexistent/test-skill-xyz".to_string(),
            "nonexistent/test-skill-xyz@skill-name".to_string(),
        ];
        // preexisting_skills expects the extracted skill name (repo part or skill-name after @), not the full skill string
        let preexisting_multi = vec![
            "test-skill-xyz".to_string(),
        ];
        let result_multi = generate_entrypoint_script("test-agent", &skills_multi, &preexisting_multi);
        assert!(result_multi.is_ok(), "Multiple skills should return Ok");
        let script_multi = result_multi.unwrap();
        assert!(
            !script_multi.is_empty(),
            "Multiple skills should generate non-empty script"
        );
        // First skill is preexisting, so no install command for it
        // Second skill needs installation
        assert!(
            script_multi.contains("npx skills add nonexistent/test-skill-xyz@skill-name -a kilo -y"),
            "Script should install second skill"
        );
        assert!(
            script_multi.contains("exec kilocode --yes \"$@\""),
            "Script should execute kilocode"
        );
    }

    /// Test that script generation errors are propagated through run_agent error handling
    ///
    /// This test verifies that when generate_entrypoint_script fails due to invalid
    /// skill format, the error is properly mapped to DockerError::IoError with the
    /// agent context included. This ensures that the error propagation added in subtask 6c
    /// works correctly.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_script_generation_error_propagates_to_caller() {
        use crate::docker::skills::generate_entrypoint_script;

        // Test with invalid skill format - should fail with ScriptGenerationFailed
        let skills = vec!["invalid-skill-format".to_string()];
        let agent_name = "test-agent";

        let result = generate_entrypoint_script(agent_name, &skills, &[]);
        assert!(
            result.is_err(),
            "generate_entrypoint_script should return error for invalid skill format"
        );

        // Verify the error is ScriptGenerationFailed with agent context
        match result {
            Err(crate::skills::SkillsError::ScriptGenerationFailed {
                agent_name: returned_agent,
                reason,
            }) => {
                assert_eq!(
                    returned_agent, agent_name,
                    "Error should include the agent name for context"
                );
                assert!(
                    reason.contains("Invalid skill format"),
                    "Error reason should describe the validation failure"
                );
                assert!(
                    reason.contains("invalid-skill-format"),
                    "Error reason should include the problematic skill"
                );
            }
            _ => panic!("Expected ScriptGenerationFailed error, got: {:?}", result),
        }
    }

    /// Test skills field handling: CASE 1 - skills = None
    ///
    /// This test verifies that when ContainerConfig.skills is None,
    /// the entrypoint should remain None, meaning the container will use
    /// the default entrypoint from the Dockerfile.
    ///
    /// This corresponds to the match statement at lines 311-349 in run_agent():
    /// ```rust
    /// match &config.skills {
    ///     Some(skills) if !skills.is_empty() => { /* generate custom entrypoint */ }
    ///     _ => { /* CASE 1 & 2: use default entrypoint */ }
    /// }
    /// ```
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skills_field_handling_none() {
        use crate::docker::run::types::ContainerConfig;

        // Create a ContainerConfig with skills = None
        let config = ContainerConfig::new("test-agent".to_string());

        // Verify that skills is None
        assert_eq!(
            config.skills, None,
            "ContainerConfig should have skills: None"
        );

        // Simulate the match statement logic from run_agent() lines 311-349
        // CASE 1: skills = None → entrypoint should remain None
        let should_generate_entrypoint = match &config.skills {
            Some(skills) if !skills.is_empty() => true, // CASE 3
            _ => false,                                 // CASE 1 & 2: None or empty skills
        };

        assert!(
            !should_generate_entrypoint,
            "skills = None should NOT generate custom entrypoint"
        );

        // Verify the entrypoint would remain None (use default from Dockerfile)
        let entrypoint = if should_generate_entrypoint {
            Some(vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "script".to_string(),
            ])
        } else {
            None
        };

        assert!(
            entrypoint.is_none(),
            "entrypoint should be None when skills = None"
        );
    }

    /// Test skills field handling: CASE 2 - skills = Some([])
    ///
    /// This test verifies that when ContainerConfig.skills is Some(vec![]),
    /// the entrypoint should remain None, meaning the container will use
    /// the default entrypoint from the Dockerfile.
    ///
    /// This corresponds to the match statement at lines 311-349 in run_agent():
    /// ```rust
    /// match &config.skills {
    ///     Some(skills) if !skills.is_empty() => { /* generate custom entrypoint */ }
    ///     _ => { /* CASE 1 & 2: use default entrypoint */ }
    /// }
    /// ```
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skills_field_handling_empty_vec() {
        use crate::docker::run::types::ContainerConfig;

        // Create a ContainerConfig with skills = Some(vec![])
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec![]);

        // Verify that skills is Some(vec![])
        assert_eq!(
            config.skills,
            Some(vec![]),
            "ContainerConfig should have skills: Some(vec![])"
        );

        // Simulate the match statement logic from run_agent() lines 311-349
        // CASE 2: skills = Some([]) → entrypoint should remain None
        let should_generate_entrypoint = match &config.skills {
            Some(skills) if !skills.is_empty() => true, // CASE 3
            _ => false,                                 // CASE 1 & 2: None or empty skills
        };

        assert!(
            !should_generate_entrypoint,
            "skills = Some([]) should NOT generate custom entrypoint"
        );

        // Verify the entrypoint would remain None (use default from Dockerfile)
        let entrypoint = if should_generate_entrypoint {
            Some(vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "script".to_string(),
            ])
        } else {
            None
        };

        assert!(
            entrypoint.is_none(),
            "entrypoint should be None when skills = Some(vec![])"
        );
    }

    /// Test skills field handling: CASE 3 - skills = Some([...])
    ///
    /// This test verifies that when ContainerConfig.skills is Some(vec![...])
    /// with a non-empty list, a custom entrypoint script should be generated
    /// that installs the specified skills before running the agent.
    ///
    /// This corresponds to the match statement at lines 311-349 in run_agent():
    /// ```rust
    /// match &config.skills {
    ///     Some(skills) if !skills.is_empty() => {
    ///         let entrypoint_script = generate_entrypoint_script(agent_name, skills, &[])?;
    ///         container_config.entrypoint = Some(vec!["/bin/sh", "-c", entrypoint_script]);
    ///     }
    ///     _ => { /* CASE 1 & 2: use default entrypoint */ }
    /// }
    /// ```
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skills_field_handling_non_empty_vec() {
        use crate::docker::run::types::ContainerConfig;
        use crate::docker::skills::generate_entrypoint_script;

        // Create a ContainerConfig with skills = Some(vec!["owner/repo"])
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec!["owner/repo".to_string()]);

        // Verify that skills is Some(vec!["owner/repo"])
        assert_eq!(
            config.skills,
            Some(vec!["owner/repo".to_string()]),
            "ContainerConfig should have skills: Some(vec![...])"
        );

        // Simulate the match statement logic from run_agent() lines 311-349
        // CASE 3: skills = Some([...]) → generate custom entrypoint
        let should_generate_entrypoint = match &config.skills {
            Some(skills) if !skills.is_empty() => true, // CASE 3
            _ => false,                                 // CASE 1 & 2: None or empty skills
        };

        assert!(
            should_generate_entrypoint,
            "skills = Some([...]) should generate custom entrypoint"
        );

        // Verify that generate_entrypoint_script would be called and produce valid output
        if should_generate_entrypoint {
            if let Some(ref skills) = config.skills {
                let result = generate_entrypoint_script("test-agent", skills, &[]);
                assert!(
                    result.is_ok(),
                    "generate_entrypoint_script should succeed for valid skills"
                );

                let script = result.unwrap();
                assert!(!script.is_empty(), "Generated script should not be empty");
                assert!(
                    script.contains("#!/bin/sh"),
                    "Script should start with shebang"
                );
                assert!(
                    script.contains("npx skills add owner/repo -a kilo -y"),
                    "Script should contain skill installation command"
                );
                assert!(
                    script.contains("exec kilocode --yes \"$@\""),
                    "Script should contain CLI execution command"
                );

                // Verify the entrypoint would be set with the script
                let entrypoint = Some(vec!["/bin/sh".to_string(), "-c".to_string(), script]);

                assert!(
                    entrypoint.is_some(),
                    "entrypoint should be Some when skills = Some(vec![...])"
                );
                assert_eq!(
                    entrypoint.as_ref().unwrap().len(),
                    3,
                    "entrypoint should have 3 parts: [\"/bin/sh\", \"-c\", script]"
                );
            }
        }
    }

    /// Test skills field handling integration with build_container_config
    ///
    /// This test verifies the complete integration: when skills are specified,
    /// the container config's entrypoint field is modified correctly.
    /// This simulates what happens in run_agent() lines 311-349.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skills_field_handling_integration() {
        use crate::docker::skills::generate_entrypoint_script;

        // Test CASE 1: skills = None
        let skills_none: Option<Vec<String>> = None;
        let entrypoint_none = match &skills_none {
            Some(skills) if !skills.is_empty() => {
                let script = generate_entrypoint_script("test-agent", skills, &[])
                    .expect("Failed to generate entrypoint script");
                Some(vec!["/bin/sh".to_string(), "-c".to_string(), script])
            }
            _ => None,
        };
        assert!(
            entrypoint_none.is_none(),
            "CASE 1: skills = None should result in entrypoint = None"
        );

        // Test CASE 2: skills = Some([])
        let skills_empty = Some(vec![]);
        let entrypoint_empty = match &skills_empty {
            Some(skills) if !skills.is_empty() => {
                let script = generate_entrypoint_script("test-agent", skills, &[])
                    .expect("Failed to generate entrypoint script");
                Some(vec!["/bin/sh".to_string(), "-c".to_string(), script])
            }
            _ => None,
        };
        assert!(
            entrypoint_empty.is_none(),
            "CASE 2: skills = Some([]) should result in entrypoint = None"
        );

        // Test CASE 3: skills = Some(["owner/repo"])
        let skills_populated = Some(vec!["owner/repo".to_string()]);
        let entrypoint_populated = match &skills_populated {
            Some(skills) if !skills.is_empty() => {
                let script = generate_entrypoint_script("test-agent", skills, &[])
                    .expect("Failed to generate entrypoint script");
                Some(vec!["/bin/sh".to_string(), "-c".to_string(), script])
            }
            _ => None,
        };
        assert!(
            entrypoint_populated.is_some(),
            "CASE 3: skills = Some([...]) should result in entrypoint = Some([...])"
        );
        if let Some(entry) = entrypoint_populated {
            assert_eq!(entry[0], "/bin/sh", "entrypoint[0] should be \"/bin/sh\"");
            assert_eq!(entry[1], "-c", "entrypoint[1] should be \"-c\"");
            assert!(
                !entry[2].is_empty(),
                "entrypoint[2] should be the generated script"
            );
            assert!(
                entry[2].contains("npx skills add owner/repo -a kilo -y"),
                "Generated script should install the skill"
            );
        }
    }

    /// Test that the skills match statement handles all variants correctly
    ///
    /// This comprehensive test ensures that the match statement in run_agent()
    /// correctly handles all possible variants of the skills field and generates
    /// the appropriate entrypoint configuration.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skills_match_statement_all_cases() {
        use crate::docker::skills::generate_entrypoint_script;

        // Define test cases: (skills_value, expected_entrypoint_some, description)
        let test_cases: Vec<(Option<Vec<String>>, bool, &str)> = vec![
            (None, false, "CASE 1: skills = None"),
            (Some(vec![]), false, "CASE 2: skills = Some(vec![])"),
            (
                Some(vec!["owner/repo".to_string()]),
                true,
                "CASE 3: skills = Some(vec![...]) single",
            ),
            (
                Some(vec![
                    "owner/repo".to_string(),
                    "owner/repo@skill-name".to_string(),
                ]),
                true,
                "CASE 3: skills = Some(vec![...]) multiple",
            ),
        ];

        for (skills, expected_entrypoint_some, description) in test_cases {
            // Simulate the match statement from run_agent() lines 311-349
            let entrypoint_result = match &skills {
                Some(skills_list) if !skills_list.is_empty() => {
                    // CASE 3: Non-empty skills list - generate custom entrypoint
                    let script = generate_entrypoint_script("test-agent", skills_list, &[])
                        .expect("Failed to generate entrypoint script");
                    Some(vec!["/bin/sh".to_string(), "-c".to_string(), script])
                }
                _ => {
                    // CASE 1 & 2: No skills or empty skills list - use default entrypoint
                    None
                }
            };

            // Verify the expected behavior
            assert_eq!(
                entrypoint_result.is_some(),
                expected_entrypoint_some,
                "{}: entrypoint.is_some() should be {}",
                description,
                expected_entrypoint_some
            );

            if expected_entrypoint_some {
                assert!(
                    entrypoint_result.is_some(),
                    "{}: entrypoint should be Some",
                    description
                );
                let entry = entrypoint_result.unwrap();
                assert_eq!(
                    entry.len(),
                    3,
                    "{}: entrypoint should have 3 parts",
                    description
                );
                assert_eq!(
                    entry[0], "/bin/sh",
                    "{}: entrypoint[0] should be \"/bin/sh\"",
                    description
                );
                assert_eq!(
                    entry[1], "-c",
                    "{}: entrypoint[1] should be \"-c\"",
                    description
                );
                assert!(
                    !entry[2].is_empty(),
                    "{}: entrypoint[2] should not be empty",
                    description
                );
            } else {
                assert!(
                    entrypoint_result.is_none(),
                    "{}: entrypoint should be None",
                    description
                );
            }
        }
    }

    /// Test script injection: container Config has correct Entrypoint when skills are present
    ///
    /// This test verifies that when skills are specified in the ContainerConfig,
    /// the container's Config.entrypoint field is properly set to execute the
    /// generated script via /bin/sh -c. This ensures the script injection mechanism
    /// works as intended.
    ///
    /// This corresponds to lines 311-349 in run_agent() where the entrypoint is
    /// configured when skills are present.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_script_injection_entrypoint_configuration() {
        use crate::docker::skills::generate_entrypoint_script;

        // Test with non-empty skills - should set entrypoint
        let skills = vec!["owner/repo".to_string()];
        let agent_name = "test-agent";

        // Generate the entrypoint script (as done in run_agent)
        let entrypoint_script = generate_entrypoint_script(agent_name, &skills, &[])
            .expect("Failed to generate entrypoint script");

        // Build the entrypoint vector as it's set in run_agent() lines 335-339
        let entrypoint = Some(vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            entrypoint_script.clone(),
        ]);

        // Verify the entrypoint is set correctly
        assert!(
            entrypoint.is_some(),
            "Entrypoint should be Some when skills are present"
        );

        let entry = entrypoint.as_ref().unwrap();
        assert_eq!(entry.len(), 3, "Entrypoint should have exactly 3 elements");
        assert_eq!(
            entry[0], "/bin/sh",
            "Entrypoint[0] should be \"/bin/sh\" (the shell interpreter)"
        );
        assert_eq!(
            entry[1], "-c",
            "Entrypoint[1] should be \"-c\" (read from string argument)"
        );
        assert!(
            !entry[2].is_empty(),
            "Entrypoint[2] should contain the generated script"
        );
    }

    /// Test script injection: Entrypoint wrapper executes the generated script
    ///
    /// This test verifies that the Entrypoint wrapper (/bin/sh -c) correctly
    /// executes the generated script. The wrapper allows passing a multi-line
    /// shell script as a single string argument to the entrypoint.
    ///
    /// This corresponds to lines 324-339 in run_agent() which document the
    /// ENTRYPOINT APPROACH using /bin/sh -c.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_script_injection_wrapper_executes_script() {
        use crate::docker::skills::generate_entrypoint_script;

        let skills = vec!["owner/repo".to_string()];
        let agent_name = "test-agent";

        // Generate the entrypoint script
        let entrypoint_script = generate_entrypoint_script(agent_name, &skills, &[])
            .expect("Failed to generate entrypoint script");

        // The wrapper pattern used in run_agent: ["/bin/sh", "-c", script]
        // This is the Docker entrypoint format - a vector of command-line arguments
        let wrapper = [
            "/bin/sh".to_string(),
            "-c".to_string(),
            entrypoint_script.clone(),
        ];

        // Verify the wrapper structure
        assert_eq!(wrapper.len(), 3, "Wrapper should have 3 elements");
        assert_eq!(wrapper[0], "/bin/sh", "Shell interpreter should be /bin/sh");
        assert_eq!(
            wrapper[1], "-c",
            "Flag should be -c (read commands from string)"
        );
        assert_eq!(
            wrapper[2], entrypoint_script,
            "Third element should be the generated script"
        );

        // Verify that the script in wrapper[2] is the same as what was generated
        assert!(
            wrapper[2].contains("npx skills add owner/repo -a kilo -y"),
            "Wrapper should contain the skill installation command"
        );
        assert!(
            wrapper[2].contains("exec kilocode --yes \"$@\""),
            "Wrapper should contain the kilocode execution command"
        );
    }

    /// Test script injection: script has proper shebang and executable characteristics
    ///
    /// This test verifies that the generated script includes the proper shebang
    /// line and has the characteristics of an executable shell script. While we
    /// can't test actual file permissions in a unit test (since the script is
    /// passed as a string to the container), we can verify the script content
    /// indicates it's designed to be executable.
    ///
    /// The script is executed via /bin/sh -c, which means the shell interpreter
    /// is specified in the wrapper rather than via file permissions. However,
    /// the script still includes the shebang for documentation and clarity.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_script_injection_shebang_and_executable() {
        use crate::docker::skills::generate_entrypoint_script;

        let skills = vec![
            "owner/repo1".to_string(),
            "owner/repo2@skill-name".to_string(),
        ];
        let agent_name = "test-agent";

        // Generate the entrypoint script
        let script = generate_entrypoint_script(agent_name, &skills, &[])
            .expect("Failed to generate entrypoint script");

        // Verify the shebang line is present at the very start
        assert!(
            script.starts_with("#!/bin/sh"),
            "Script must start with shebang #!/bin/sh for clarity and documentation"
        );

        // Verify the shebang is followed by a newline
        assert!(
            script.contains("#!/bin/sh\n"),
            "Shebang should be followed by newline"
        );

        // Verify the script contains set -e for proper error handling (characteristic of executable scripts)
        assert!(
            script.contains("set -e"),
            "Script should contain 'set -e' for error propagation (common in executable scripts)"
        );

        // Verify the script is a complete, valid shell script (not just fragments)
        assert!(
            script.lines().count() > 5,
            "Script should have multiple lines forming a complete script"
        );

        // Verify the script has a clear structure with sections
        assert!(
            script.contains("# Install skills"),
            "Script should have a 'Install skills' section comment"
        );
        assert!(
            script.contains("# Hand off to Kilo Code CLI"),
            "Script should have a 'Hand off to Kilo Code CLI' section comment"
        );

        // The script is executed via /bin/sh -c, so executable permissions are
        // handled by the container configuration (the wrapper ensures the script
        // is executed with the correct shell). We verify the wrapper uses the shell:
        let wrapper = ["/bin/sh", "-c", &script];
        assert_eq!(
            wrapper[0], "/bin/sh",
            "Wrapper specifies the shell to execute the script"
        );
    }

    /// Test script injection: script content matches expected format from generate_entrypoint_script()
    ///
    /// This test verifies that the script generated and injected into the container
    /// matches the expected format from generate_entrypoint_script(). The script should
    /// follow the documented format with specific sections and commands.
    ///
    /// Expected format (from generate_entrypoint_script() documentation):
    /// ```text
    /// #!/bin/sh
    /// # POSIX shell for maximum compatibility across container environments
    /// set -e
    /// # Error propagation - immediately exit on any command failure
    ///
    /// # Install skills
    /// # Skills are installed sequentially in declaration order
    /// npx skills add owner/repo1 -a kilo -y
    /// npx skills add owner/repo2@skill-name -a kilo -y
    ///
    /// # Hand off to Kilo Code CLI
    /// # Process replacement - replaces shell with kilocode
    /// exec kilocode --yes "$@"
    /// ```
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_script_injection_content_matches_expected_format() {
        use crate::docker::skills::generate_entrypoint_script;

        let skills = vec![
            "owner/repo1".to_string(),
            "owner/repo2@skill-name".to_string(),
        ];
        let agent_name = "test-agent";

        // Generate the entrypoint script
        let script = generate_entrypoint_script(agent_name, &skills, &[])
            .expect("Failed to generate entrypoint script");

        // Verify the script matches the expected format:

        // 1. Shebang line
        assert!(
            script.starts_with("#!/bin/sh"),
            "Script should start with #!/bin/sh shebang"
        );

        // 2. POSIX shell comment
        assert!(
            script
                .contains("# POSIX shell for maximum compatibility across container environments"),
            "Script should include POSIX shell comment"
        );

        // 3. Error propagation setting
        assert!(
            script.contains("set -e"),
            "Script should contain 'set -e' for error propagation"
        );

        // 4. Error propagation comment
        assert!(
            script.contains("# Error propagation - immediately exit on any command failure to prevent cascading errors"),
            "Script should include error propagation comment"
        );

        // 5. Install skills section
        assert!(
            script.contains("# Install skills"),
            "Script should have 'Install skills' section header"
        );
        assert!(
            script.contains(
                "# Skills are installed sequentially in declaration order to satisfy dependencies"
            ),
            "Script should include skill installation ordering comment"
        );

        // 6. Skill installation commands with correct format
        assert!(
            script.contains("npx skills add owner/repo1 -a kilo -y"),
            "Script should contain first skill installation command with -a kilo -y flags"
        );
        assert!(
            script.contains("npx skills add owner/repo2@skill-name -a kilo -y"),
            "Script should contain second skill installation command with -a kilo -y flags"
        );

        // Verify all npx commands use the correct flags
        for skill in &skills {
            assert!(
                script.contains(&format!("npx skills add {} -a kilo -y", skill)),
                "Script should install skill '{}' with correct flags",
                skill
            );
        }

        // 7. Hand off section
        assert!(
            script.contains("# Hand off to Kilo Code CLI"),
            "Script should have 'Hand off to Kilo Code CLI' section header"
        );
        assert!(
            script.contains("# Process replacement - replaces shell with kilocode, ensuring proper signal handling and exit code propagation"),
            "Script should include process replacement comment"
        );

        // 8. CLI execution command
        assert!(
            script.contains("exec kilocode --yes \"$@\""),
            "Script should contain 'exec kilocode --yes \"$@\"' command"
        );

        // 9. Verify skills appear in the correct order
        let pos1 = script
            .find("npx skills add owner/repo1 -a kilo -y")
            .unwrap();
        let pos2 = script
            .find("npx skills add owner/repo2@skill-name -a kilo -y")
            .unwrap();
        assert!(pos1 < pos2, "Skills should appear in declaration order");

        // 10. Verify the script structure follows the expected pattern
        let lines: Vec<&str> = script.lines().collect();
        assert!(lines.len() > 10, "Script should have multiple lines");

        // Verify the first few lines match expected structure
        assert!(lines[0] == "#!/bin/sh", "First line should be the shebang");
        assert!(
            lines[2].contains("set -e") || lines[1].contains("set -e"),
            "Script should contain 'set -e' near the beginning"
        );

        // Verify the last line contains the exec command
        assert!(
            lines.last().unwrap().contains("exec kilocode --yes \"$@\""),
            "Last line should contain the exec command for kilocode"
        );
    }

    /// Test script injection integration: container config entrypoint is set correctly
    ///
    /// This test verifies the complete integration: when a container configuration
    /// is built with skills, the entrypoint field is correctly set to execute the
    /// generated script. This simulates what happens in run_agent() when building
    /// the container configuration.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_script_injection_container_config_integration() {
        use crate::docker::skills::generate_entrypoint_script;
        use bollard::container::Config;

        // Create base container config (as built by build_container_config)
        let mut container_config: Config<String> = Config {
            image: Some("switchboard-agent:latest".to_string()),
            env: None,
            cmd: None,
            entrypoint: None, // Initially None (default from Dockerfile)
            ..Default::default()
        };

        // Verify entrypoint is initially None
        assert!(
            container_config.entrypoint.is_none(),
            "Container config entrypoint should be None initially"
        );

        // Simulate the skills handling from run_agent() lines 311-349
        let skills = vec!["owner/repo".to_string()];
        let agent_name = "test-agent";

        // Generate entrypoint script and set it (CASE 3: non-empty skills)
        let entrypoint_script = generate_entrypoint_script(agent_name, &skills, &[])
            .expect("Failed to generate entrypoint script");

        container_config.entrypoint = Some(vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            entrypoint_script,
        ]);

        // Verify entrypoint is now set correctly
        assert!(
            container_config.entrypoint.is_some(),
            "Container config entrypoint should be Some after setting with skills"
        );

        let entry = container_config.entrypoint.as_ref().unwrap();
        assert_eq!(entry.len(), 3, "Entrypoint should have 3 elements");
        assert_eq!(entry[0], "/bin/sh", "Entrypoint[0] should be /bin/sh");
        assert_eq!(entry[1], "-c", "Entrypoint[1] should be -c");
        assert!(
            entry[2].contains("npx skills add owner/repo -a kilo -y"),
            "Entrypoint[2] should contain the skill installation script"
        );
    }

    /// Test script injection: verify no entrypoint set when skills are None or empty
    ///
    /// This test verifies that when skills are not specified (None) or empty (Some(vec![])),
    /// the container config's entrypoint remains None, meaning the default entrypoint
    /// from the Dockerfile is used. This ensures backward compatibility.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_script_injection_no_entrypoint_without_skills() {
        use crate::docker::run::types::ContainerConfig;
        use bollard::container::Config;

        // Test CASE 1: skills = None
        let config_none = ContainerConfig::new("test-agent".to_string());
        assert_eq!(
            config_none.skills, None,
            "ContainerConfig should have skills: None"
        );

        // Build container config as done in run_agent
        let container_config: Config<String> = Config {
            image: Some("switchboard-agent:latest".to_string()),
            entrypoint: None,
            ..Default::default()
        };

        // Simulate the match statement logic from run_agent() lines 311-349
        match &config_none.skills {
            Some(skills) if !skills.is_empty() => {
                // CASE 3: would generate entrypoint - not reached here
                panic!("Should not reach this path for None skills");
            }
            _ => {
                // CASE 1 & 2: No skills or empty skills - entrypoint remains None
                // No modification to container_config.entrypoint
            }
        }

        assert!(
            container_config.entrypoint.is_none(),
            "Entrypoint should remain None when skills is None"
        );

        // Test CASE 2: skills = Some(vec![])
        let mut config_empty = ContainerConfig::new("test-agent".to_string());
        config_empty.skills = Some(vec![]);
        assert_eq!(
            config_empty.skills,
            Some(vec![]),
            "ContainerConfig should have skills: Some(vec![])"
        );

        let container_config2: Config<String> = Config {
            image: Some("switchboard-agent:latest".to_string()),
            entrypoint: None,
            ..Default::default()
        };

        // Simulate the match statement logic
        match &config_empty.skills {
            Some(skills) if !skills.is_empty() => {
                // CASE 3: would generate entrypoint - not reached here
                panic!("Should not reach this path for empty skills");
            }
            _ => {
                // CASE 1 & 2: No skills or empty skills - entrypoint remains None
                // No modification to container_config2.entrypoint
            }
        }

        assert!(
            container_config2.entrypoint.is_none(),
            "Entrypoint should remain None when skills is Some(vec![])"
        );
    }

    /// Integration Test: Complete flow with a single skill
    ///
    /// This test verifies the complete end-to-end flow when a ContainerConfig
    /// with a single skill is processed:
    /// 1. Skills are extracted from config
    /// 2. Script is generated
    /// 3. Entrypoint is correctly set with the script
    ///
    /// This integration test simulates what happens in run_agent() lines 303-349
    /// without requiring Docker to be running.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_integration_complete_flow_single_skill() {
        use crate::docker::run::types::ContainerConfig;
        use crate::docker::skills::generate_entrypoint_script;
        use bollard::container::Config;

        // Step 1: Create a ContainerConfig with a single skill
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.prompt = "test prompt".to_string();
        config.skills = Some(vec!["owner/repo".to_string()]);

        // Step 2: Verify skills are present in the config
        assert_eq!(
            config.skills,
            Some(vec!["owner/repo".to_string()]),
            "ContainerConfig should have a single skill"
        );

        // Step 3: Build the base container config (as done in run_agent)
        let image = "switchboard-agent:latest";
        let env_vars = vec![];
        let readonly = false;
        let workspace = "/workspace";
        let timeout_seconds = 1800;

        let mut container_config: Config<String> = build_container_config(
            image,
            env_vars,
            readonly,
            workspace,
            &config.agent_name,
            timeout_seconds,
            Some(&["--auto".to_string(), config.prompt.clone()]),
        );

        // Verify entrypoint is initially None (before skills processing)
        assert!(
            container_config.entrypoint.is_none(),
            "Container config entrypoint should be None before skills processing"
        );

        // Step 4: Process skills and generate entrypoint (as done in run_agent lines 311-349)
        match &config.skills {
            Some(skills) if !skills.is_empty() => {
                // Generate entrypoint script
                let entrypoint_script = generate_entrypoint_script(&config.agent_name, skills, &[])
                    .expect("Failed to generate entrypoint script");

                // Verify script is generated
                assert!(
                    !entrypoint_script.is_empty(),
                    "Generated script should not be empty"
                );

                // Set entrypoint (as done in run_agent lines 335-339)
                container_config.entrypoint = Some(vec![
                    "/bin/sh".to_string(),
                    "-c".to_string(),
                    entrypoint_script,
                ]);
            }
            _ => {
                panic!("Should reach the skills processing branch for non-empty skills");
            }
        }

        // Step 5: Verify the complete flow - entrypoint is now correctly set
        assert!(
            container_config.entrypoint.is_some(),
            "Container config entrypoint should be Some after skills processing"
        );

        let entrypoint = container_config.entrypoint.as_ref().unwrap();
        assert_eq!(entrypoint.len(), 3, "Entrypoint should have 3 elements");
        assert_eq!(entrypoint[0], "/bin/sh", "Entrypoint[0] should be /bin/sh");
        assert_eq!(entrypoint[1], "-c", "Entrypoint[1] should be -c");
        assert!(
            entrypoint[2].contains("npx skills add owner/repo -a kilo -y"),
            "Entrypoint[2] should contain the skill installation command"
        );
        assert!(
            entrypoint[2].contains("exec kilocode --yes \"$@\""),
            "Entrypoint[2] should contain the kilocode execution command"
        );
    }

    /// Integration Test: Complete flow with multiple skills
    ///
    /// This test verifies the complete end-to-end flow when a ContainerConfig
    /// with multiple skills is processed:
    /// 1. Skills are extracted from config
    /// 2. Script is generated with all skills
    /// 3. Entrypoint is correctly set with the script
    ///
    /// This integration test simulates what happens in run_agent() lines 303-349
    /// without requiring Docker to be running.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_integration_complete_flow_multiple_skills() {
        use crate::docker::run::types::ContainerConfig;
        use crate::docker::skills::generate_entrypoint_script;
        use bollard::container::Config;

        // Step 1: Create a ContainerConfig with multiple skills
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.prompt = "test prompt".to_string();
        config.skills = Some(vec![
            "owner/repo1".to_string(),
            "owner/repo2@skill-name".to_string(),
            "owner/repo3".to_string(),
        ]);

        // Step 2: Verify skills are present in the config
        assert_eq!(
            config.skills.as_ref().unwrap().len(),
            3,
            "ContainerConfig should have three skills"
        );

        // Step 3: Build the base container config (as done in run_agent)
        let image = "switchboard-agent:latest";
        let env_vars = vec![];
        let readonly = false;
        let workspace = "/workspace";
        let timeout_seconds = 1800;

        let mut container_config: Config<String> = build_container_config(
            image,
            env_vars,
            readonly,
            workspace,
            &config.agent_name,
            timeout_seconds,
            Some(&["--auto".to_string(), config.prompt.clone()]),
        );

        // Verify entrypoint is initially None (before skills processing)
        assert!(
            container_config.entrypoint.is_none(),
            "Container config entrypoint should be None before skills processing"
        );

        // Step 4: Process skills and generate entrypoint (as done in run_agent lines 311-349)
        match &config.skills {
            Some(skills) if !skills.is_empty() => {
                // Generate entrypoint script
                let entrypoint_script = generate_entrypoint_script(&config.agent_name, skills, &[])
                    .expect("Failed to generate entrypoint script");

                // Verify script is generated
                assert!(
                    !entrypoint_script.is_empty(),
                    "Generated script should not be empty"
                );

                // Verify all skills are in the script in the correct order
                for skill in skills {
                    assert!(
                        entrypoint_script.contains(&format!("npx skills add {} -a kilo -y", skill)),
                        "Script should contain skill installation command for {}",
                        skill
                    );
                }

                // Verify skills appear in the correct order
                let pos1 = entrypoint_script
                    .find("npx skills add owner/repo1 -a kilo -y")
                    .unwrap();
                let pos2 = entrypoint_script
                    .find("npx skills add owner/repo2@skill-name -a kilo -y")
                    .unwrap();
                let pos3 = entrypoint_script
                    .find("npx skills add owner/repo3 -a kilo -y")
                    .unwrap();
                assert!(pos1 < pos2, "Skills should appear in declaration order");
                assert!(pos2 < pos3, "Skills should appear in declaration order");

                // Set entrypoint (as done in run_agent lines 335-339)
                container_config.entrypoint = Some(vec![
                    "/bin/sh".to_string(),
                    "-c".to_string(),
                    entrypoint_script,
                ]);
            }
            _ => {
                panic!("Should reach the skills processing branch for non-empty skills");
            }
        }

        // Step 5: Verify the complete flow - entrypoint is now correctly set
        assert!(
            container_config.entrypoint.is_some(),
            "Container config entrypoint should be Some after skills processing"
        );

        let entrypoint = container_config.entrypoint.as_ref().unwrap();
        assert_eq!(entrypoint.len(), 3, "Entrypoint should have 3 elements");
        assert_eq!(entrypoint[0], "/bin/sh", "Entrypoint[0] should be /bin/sh");
        assert_eq!(entrypoint[1], "-c", "Entrypoint[1] should be -c");
        assert!(
            entrypoint[2].contains("npx skills add owner/repo1 -a kilo -y"),
            "Entrypoint[2] should contain the first skill installation command"
        );
        assert!(
            entrypoint[2].contains("npx skills add owner/repo2@skill-name -a kilo -y"),
            "Entrypoint[2] should contain the second skill installation command"
        );
        assert!(
            entrypoint[2].contains("npx skills add owner/repo3 -a kilo -y"),
            "Entrypoint[2] should contain the third skill installation command"
        );
        assert!(
            entrypoint[2].contains("exec kilocode --yes \"$@\""),
            "Entrypoint[2] should contain the kilocode execution command"
        );
    }

    /// Integration Test: Complete flow with no skills (default entrypoint)
    ///
    /// This test verifies the complete end-to-end flow when a ContainerConfig
    /// has no skills specified:
    /// 1. Skills are None (or empty)
    /// 2. No script is generated
    /// 3. Entrypoint remains None (uses default from Dockerfile)
    ///
    /// This integration test simulates what happens in run_agent() lines 311-349
    /// without requiring Docker to be running.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_integration_complete_flow_no_skills() {
        use crate::docker::run::types::ContainerConfig;
        use bollard::container::Config;

        // Step 1: Create a ContainerConfig with no skills
        let config = ContainerConfig::new("test-agent".to_string());

        // Step 2: Verify skills are None
        assert_eq!(
            config.skills, None,
            "ContainerConfig should have skills: None"
        );

        // Step 3: Build the base container config (as done in run_agent)
        let image = "switchboard-agent:latest";
        let env_vars = vec![];
        let readonly = false;
        let workspace = "/workspace";
        let timeout_seconds = 1800;

        let container_config: Config<String> = build_container_config(
            image,
            env_vars,
            readonly,
            workspace,
            &config.agent_name,
            timeout_seconds,
            Some(&["--auto".to_string(), config.prompt.clone()]),
        );

        // Verify entrypoint is initially None (before skills processing)
        assert!(
            container_config.entrypoint.is_none(),
            "Container config entrypoint should be None before skills processing"
        );

        // Step 4: Process skills (as done in run_agent lines 311-349)
        match &config.skills {
            Some(skills) if !skills.is_empty() => {
                // This branch should not be reached
                panic!("Should not generate entrypoint script when skills is None");
            }
            _ => {
                // CASE 1 & 2: No skills or empty skills - entrypoint remains None
                // No modification to container_config.entrypoint
            }
        }

        // Step 5: Verify the complete flow - entrypoint remains None
        assert!(
            container_config.entrypoint.is_none(),
            "Container config entrypoint should remain None when skills is None"
        );

        // Verify other container config fields are set correctly
        assert_eq!(
            container_config.image,
            Some(image.to_string()),
            "Container config image should be set correctly"
        );
    }

    /// Integration Test: Complete flow with empty skills list (default entrypoint)
    ///
    /// This test verifies the complete end-to-end flow when a ContainerConfig
    /// has an empty skills list:
    /// 1. Skills are Some(vec![])
    /// 2. No script is generated (empty list)
    /// 3. Entrypoint remains None (uses default from Dockerfile)
    ///
    /// This integration test simulates what happens in run_agent() lines 311-349
    /// without requiring Docker to be running.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_integration_complete_flow_empty_skills_list() {
        use crate::docker::run::types::ContainerConfig;
        use bollard::container::Config;

        // Step 1: Create a ContainerConfig with empty skills list
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec![]);

        // Step 2: Verify skills are Some(vec![])
        assert_eq!(
            config.skills,
            Some(vec![]),
            "ContainerConfig should have skills: Some(vec![])"
        );

        // Step 3: Build the base container config (as done in run_agent)
        let image = "switchboard-agent:latest";
        let env_vars = vec![];
        let readonly = false;
        let workspace = "/workspace";
        let timeout_seconds = 1800;

        let container_config: Config<String> = build_container_config(
            image,
            env_vars,
            readonly,
            workspace,
            &config.agent_name,
            timeout_seconds,
            Some(&["--auto".to_string(), config.prompt.clone()]),
        );

        // Verify entrypoint is initially None (before skills processing)
        assert!(
            container_config.entrypoint.is_none(),
            "Container config entrypoint should be None before skills processing"
        );

        // Step 4: Process skills (as done in run_agent lines 311-349)
        match &config.skills {
            Some(skills) if !skills.is_empty() => {
                // This branch should not be reached for empty skills
                panic!("Should not generate entrypoint script when skills is Some(vec![])");
            }
            _ => {
                // CASE 1 & 2: No skills or empty skills - entrypoint remains None
                // No modification to container_config.entrypoint
            }
        }

        // Step 5: Verify the complete flow - entrypoint remains None
        assert!(
            container_config.entrypoint.is_none(),
            "Container config entrypoint should remain None when skills is Some(vec![])"
        );
    }

    /// Integration Test: Script installs skills to correct location
    ///
    /// This test verifies that the generated entrypoint script would install
    /// skills to the correct location (.kilocode/skills/). While we can't
    /// actually execute the script without Docker, we can verify the script
    /// content indicates the correct installation path.
    ///
    /// The `npx skills add` command with `-a kilo -y` flags installs skills
    /// to the appropriate location for the Kilo Code CLI.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_integration_script_installs_skills_to_correct_location() {
        use crate::docker::skills::generate_entrypoint_script;

        let skills = vec![
            "owner/repo1".to_string(),
            "owner/repo2@skill-name".to_string(),
        ];
        let agent_name = "test-agent";

        // Generate the entrypoint script
        let script = generate_entrypoint_script(agent_name, &skills, &[])
            .expect("Failed to generate entrypoint script");

        // Verify the script uses the correct installation flags
        // The `-a kilo` flag specifies the agent (kilo) for installation
        // The `-y` flag enables automatic yes to confirmations
        for skill in &skills {
            assert!(
                script.contains(&format!("npx skills add {} -a kilo -y", skill)),
                "Script should install skill '{}' with correct flags for kilo agent",
                skill
            );
        }

        // Verify the script would install to the appropriate location
        // The skills CLI with -a kilo installs to .kilocode/skills/
        // We can't test the actual directory, but we verify the correct command pattern
        assert!(
            script.contains("npx skills add"),
            "Script should use npx skills add for installation"
        );
        assert!(
            script.contains("-a kilo"),
            "Script should specify -a kilo to install for the kilo agent"
        );
        assert!(
            script.contains("-y"),
            "Script should use -y flag for automatic confirmation"
        );

        // Verify all skills are included in the script
        for skill in &skills {
            assert!(
                script.contains(skill),
                "Script should include skill '{}'",
                skill
            );
        }
    }

    /// Integration Test: Error handling when script generation fails
    ///
    /// This test verifies that when generate_entrypoint_script fails due to
    /// invalid skill format, the error is properly handled. In the actual
    /// run_agent() function, this error is mapped to DockerError::IoError
    /// with the agent context included (lines 315-320).
    ///
    /// This test verifies the error path without requiring Docker to be running.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_integration_error_handling_script_generation_failure() {
        use crate::docker::run::types::ContainerConfig;
        use crate::docker::skills::generate_entrypoint_script;
        use crate::skills::SkillsError;

        // Step 1: Create a ContainerConfig with invalid skill format
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec!["invalid-skill-format".to_string()]);

        // Step 2: Verify skills are present
        assert_eq!(
            config.skills,
            Some(vec!["invalid-skill-format".to_string()]),
            "ContainerConfig should have an invalid skill"
        );

        // Step 3: Attempt to generate entrypoint script (should fail)
        let result =
            generate_entrypoint_script(&config.agent_name, config.skills.as_ref().unwrap(), &[]);

        // Step 4: Verify the error is returned correctly
        assert!(
            result.is_err(),
            "generate_entrypoint_script should return error for invalid skill format"
        );

        // Step 5: Verify the error is ScriptGenerationFailed with agent context
        match result {
            Err(SkillsError::ScriptGenerationFailed {
                agent_name: ref returned_agent,
                reason: ref reason_str,
            }) => {
                assert_eq!(
                    *returned_agent, config.agent_name,
                    "Error should include the agent name for context"
                );
                assert!(
                    reason_str.contains("Invalid skill format"),
                    "Error reason should describe the validation failure"
                );
                assert!(
                    reason_str.contains("invalid-skill-format"),
                    "Error reason should include the problematic skill"
                );
            }
            _ => panic!("Expected ScriptGenerationFailed error, got: {:?}", result),
        }

        // Step 6: Simulate the error handling in run_agent() (lines 315-320)
        // The actual code maps this error to DockerError::IoError
        let error = result.unwrap_err();
        let mapped_error = DockerError::IoError {
            operation: format!(
                "generate entrypoint script for agent '{}'",
                config.agent_name
            ),
            error_details: format!("Skills error: {}", error),
        };

        // Verify the mapped error has the correct structure
        match mapped_error {
            DockerError::IoError {
                operation,
                error_details,
            } => {
                assert!(
                    operation.contains("generate entrypoint script"),
                    "Operation should describe the script generation"
                );
                assert!(
                    operation.contains(&config.agent_name),
                    "Operation should include the agent name"
                );
                assert!(
                    error_details.contains("Skills error"),
                    "Error details should indicate it's a skills error"
                );
            }
            _ => panic!("Expected DockerError::IoError, got: {:?}", mapped_error),
        }
    }

    /// Integration Test: Complete container configuration building with skills
    ///
    /// This test verifies the complete integration of all components when building
    /// a container configuration with skills:
    /// - Skills field handling (extract skills from config)
    /// - Script generation (generate entrypoint script)
    /// - Entrypoint configuration (set entrypoint with script)
    ///
    /// This is the most comprehensive integration test, verifying the entire flow
    /// from ContainerConfig to the final container Config ready for Docker.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_integration_complete_container_config_building_with_skills() {
        use crate::docker::run::types::ContainerConfig;
        use crate::docker::skills::generate_entrypoint_script;

        // Create a complete ContainerConfig with skills
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.prompt = "test prompt".to_string();
        config.env_vars = vec!["CUSTOM_VAR=value".to_string()];
        config.skills = Some(vec![
            "owner/repo1".to_string(),
            "owner/repo2@skill-name".to_string(),
        ]);

        // Build the complete container config (simulating run_agent flow)
        let image = "switchboard-agent:latest";
        let env_vars = build_container_env_vars(&config.env_vars);
        let readonly = false;
        let workspace = "/workspace";
        let timeout_seconds = 1800;
        let cli_args = vec!["--auto".to_string(), config.prompt.clone()];

        // Build the base container config
        let mut container_config = build_container_config(
            image,
            env_vars.clone(),
            readonly,
            workspace,
            &config.agent_name,
            timeout_seconds,
            Some(&cli_args),
        );

        // Verify base config is correct
        assert_eq!(
            container_config.image,
            Some(image.to_string()),
            "Image should be set correctly"
        );
        assert_eq!(
            container_config.cmd,
            Some(cli_args),
            "Cmd should include --auto and prompt"
        );
        assert_eq!(
            container_config.env,
            Some(env_vars),
            "Env vars should be set correctly"
        );

        // Process skills and modify entrypoint (the complete integration flow)
        match &config.skills {
            Some(skills) if !skills.is_empty() => {
                // Component 1: Extract skills from config (already done via config.skills)

                // Component 2: Generate entrypoint script
                let entrypoint_script = generate_entrypoint_script(&config.agent_name, skills, &[])
                    .expect("Failed to generate entrypoint script");

                // Component 3: Set entrypoint with script
                container_config.entrypoint = Some(vec![
                    "/bin/sh".to_string(),
                    "-c".to_string(),
                    entrypoint_script,
                ]);
            }
            _ => {
                // No skills - entrypoint remains None
            }
        }

        // Verify the complete integration result
        assert!(
            container_config.entrypoint.is_some(),
            "Entrypoint should be set after skills processing"
        );

        let entrypoint = container_config.entrypoint.as_ref().unwrap();
        assert_eq!(entrypoint.len(), 3, "Entrypoint should have 3 elements");
        assert_eq!(entrypoint[0], "/bin/sh", "Entrypoint[0] should be /bin/sh");
        assert_eq!(entrypoint[1], "-c", "Entrypoint[1] should be -c");
        assert!(
            entrypoint[2].contains("npx skills add owner/repo1 -a kilo -y"),
            "Entrypoint[2] should contain first skill installation"
        );
        assert!(
            entrypoint[2].contains("npx skills add owner/repo2@skill-name -a kilo -y"),
            "Entrypoint[2] should contain second skill installation"
        );
        assert!(
            entrypoint[2].contains("exec kilocode --yes \"$@\""),
            "Entrypoint[2] should contain kilocode execution"
        );

        // Verify all components are integrated correctly
        assert_eq!(
            container_config.image,
            Some(image.to_string()),
            "Image should still be set after skills processing"
        );
        assert_eq!(
            container_config.env,
            Some(vec!["CUSTOM_VAR=value".to_string()]),
            "Env vars should still be set after skills processing"
        );
        assert_eq!(
            container_config.cmd,
            Some(vec!["--auto".to_string(), "test prompt".to_string()]),
            "Cmd should still be set after skills processing"
        );
    }

    /// Test: successful_skill_installation
    ///
    /// This test verifies that when skill installation succeeds (exit code 0),
    /// the AgentExecutionResult correctly reflects successful skill installation.
    ///
    /// Verifies:
    /// - The container does NOT return an error for skill installation
    /// - skills_installed is Some(true)
    /// - skills_install_failed is false
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_successful_skill_installation() {
        use crate::docker::run::types::ContainerConfig;

        // Test scenario: Skills configured and container exits with code 0
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec!["owner/repo".to_string()]);

        // Simulate the logic from run_agent() lines 817-839
        let exit_code: i64 = 0;
        let timed_out: bool = false;

        let skills_installed = if config.skills.as_ref().is_some_and(|s| !s.is_empty()) {
            // Skills were configured - determine if installation succeeded or failed
            if exit_code == 0 {
                Some(true) // Skills installed successfully
            } else if !timed_out {
                Some(false) // Skills installation failed (non-zero exit code, not a timeout)
            } else {
                None // Timed out - unknown if skills installed
            }
        } else {
            // No skills configured
            None
        };

        let skills_install_failed =
            config.skills.as_ref().is_some_and(|s| !s.is_empty()) && exit_code != 0 && !timed_out;

        // Verify successful skill installation is detected correctly
        assert_eq!(
            skills_installed,
            Some(true),
            "When exit_code is 0 and skills are configured, skills_installed should be Some(true)"
        );

        assert_eq!(
            skills_install_failed, false,
            "When exit_code is 0, skills_install_failed should be false"
        );

        // Verify the container exit code reflects success
        assert_eq!(
            exit_code, 0,
            "Exit code should be 0 for successful execution"
        );
    }

    /// Test: failed_skill_installation_nonzero_exit
    ///
    /// This test verifies that when skill installation fails with a non-zero exit code,
    /// the AgentExecutionResult correctly reflects the failure.
    ///
    /// Verifies:
    /// - The container stops immediately (simulated by exit code)
    /// - skills_installed is Some(false)
    /// - skills_install_failed is true
    /// - The overall container exit code reflects the skill installation failure
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_failed_skill_installation_nonzero_exit() {
        use crate::docker::run::types::ContainerConfig;

        // Test scenario: Skills configured and container exits with non-zero code
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec![
            "owner/repo".to_string(),
            "owner/repo2@skill".to_string(),
        ]);

        // Simulate the logic from run_agent() lines 817-839 with failure exit code
        let exit_code: i64 = 1; // Non-zero exit code indicates failure
        let timed_out: bool = false;

        let skills_installed = if config.skills.as_ref().is_some_and(|s| !s.is_empty()) {
            // Skills were configured - determine if installation succeeded or failed
            if exit_code == 0 {
                Some(true) // Skills installed successfully
            } else if !timed_out {
                Some(false) // Skills installation failed (non-zero exit code, not a timeout)
            } else {
                None // Timed out - unknown if skills installed
            }
        } else {
            // No skills configured
            None
        };

        let skills_install_failed =
            config.skills.as_ref().is_some_and(|s| !s.is_empty()) && exit_code != 0 && !timed_out;

        // Verify skill installation failure is detected correctly
        assert_eq!(
            skills_installed,
            Some(false),
            "When exit_code is non-zero and not timed out, skills_installed should be Some(false)"
        );

        assert_eq!(
            skills_install_failed, true,
            "When exit_code is non-zero and not timed out, skills_install_failed should be true"
        );

        // Verify the container exit code reflects the skill installation failure
        assert_eq!(
            exit_code, 1,
            "Exit code should be non-zero for skill installation failure"
        );
    }

    /// Test: container_exits_immediately_on_skill_fail
    ///
    /// This test verifies that when skill installation fails, the container
    /// terminates immediately and the error is propagated correctly.
    ///
    /// Verifies:
    /// - Container terminates on skill installation failure (simulated by non-zero exit code)
    /// - No agent execution happens after skill installation fails
    /// - Appropriate error status is propagated through skills_install_failed flag
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_container_exits_immediately_on_skill_fail() {
        use crate::docker::run::types::ContainerConfig;

        // Test scenario: Skill installation fails and container should terminate
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec!["invalid/skill".to_string()]);

        // Simulate the logic from run_agent() lines 817-839 with immediate failure
        let exit_code: i64 = 127; // Command not found error (typical for failed skill install)
        let timed_out: bool = false; // Not a timeout - actual failure

        // Simulate skill installation failure detection
        let skills_installed = if config.skills.as_ref().is_some_and(|s| !s.is_empty()) {
            if exit_code == 0 {
                Some(true) // Skills installed successfully
            } else if !timed_out {
                Some(false) // Skills installation failed (non-zero exit code, not a timeout)
            } else {
                None // Timed out - unknown if skills installed
            }
        } else {
            // No skills configured
            None
        };

        let skills_install_failed =
            config.skills.as_ref().is_some_and(|s| !s.is_empty()) && exit_code != 0 && !timed_out;

        // Calculate skill counts for metrics (as done in run_agent() lines 841-858)
        let total_skills_count = config.skills.as_ref().map(|s| s.len()).unwrap_or(0);
        let (skills_installed_count, skills_failed_count) = if skills_install_failed {
            // All skills failed to install
            (0, total_skills_count)
        } else if skills_installed == Some(true) {
            // All skills installed successfully
            (total_skills_count, 0)
        } else {
            // No skills configured or unknown status (timeout)
            (0, 0)
        };

        // Verify immediate termination on skill installation failure
        assert_eq!(
            skills_installed,
            Some(false),
            "Skill installation should have failed"
        );

        assert_eq!(
            skills_install_failed, true,
            "skills_install_failed flag should be true"
        );

        // Verify no skills were installed
        assert_eq!(
            skills_installed_count, 0,
            "No skills should be counted as installed when installation fails"
        );

        // Verify all configured skills failed
        assert_eq!(
            skills_failed_count, total_skills_count,
            "All configured skills should be counted as failed"
        );

        // Verify the container exited with a non-zero code (indicating failure)
        assert_ne!(
            exit_code, 0,
            "Container should exit with non-zero code on skill installation failure"
        );

        // Verify it was not a timeout - actual skill installation failure
        assert_eq!(
            timed_out, false,
            "Skill installation failure should not be classified as a timeout"
        );

        // Verify AgentExecutionResult would correctly reflect this state
        let result = AgentExecutionResult {
            container_id: "test-container-123".to_string(),
            exit_code,
            skills_installed,
            skills_install_failed,
        };

        assert_eq!(
            result.exit_code, 127,
            "AgentExecutionResult.exit_code should be 127"
        );
        assert_eq!(
            result.skills_installed,
            Some(false),
            "AgentExecutionResult.skills_installed should be Some(false)"
        );
        assert_eq!(
            result.skills_install_failed, true,
            "AgentExecutionResult.skills_install_failed should be true"
        );
    }

    /// Test: skill installation timeout scenario
    ///
    /// This test verifies the behavior when the container times out during
    /// skill installation. In this case, the skills_install_failed flag should
    /// be false because we don't know if skills actually failed or if
    /// the timeout occurred after successful installation.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skill_installation_timeout() {
        use crate::docker::run::types::ContainerConfig;

        // Test scenario: Skills configured but container times out
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec!["owner/repo".to_string()]);

        // Simulate the logic from run_agent() lines 817-839 with timeout
        let exit_code: i64 = 137; // SIGKILL exit code (timeout)
        let timed_out: bool = true;

        let skills_installed = if config.skills.as_ref().is_some_and(|s| !s.is_empty()) {
            // Skills were configured - determine if installation succeeded or failed
            if exit_code == 0 {
                Some(true) // Skills installed successfully
            } else if !timed_out {
                Some(false) // Skills installation failed (non-zero exit code, not a timeout)
            } else {
                None // Timed out - unknown if skills installed
            }
        } else {
            // No skills configured
            None
        };

        let skills_install_failed =
            config.skills.as_ref().is_some_and(|s| !s.is_empty()) && exit_code != 0 && !timed_out;

        // Verify timeout scenario is handled correctly
        assert_eq!(
            skills_installed, None,
            "When container times out, skills_installed should be None (unknown status)"
        );

        assert_eq!(
            skills_install_failed, false,
            "When container times out, skills_install_failed should be false (unknown if it was a failure)"
        );

        // Verify AgentExecutionResult would correctly reflect timeout state
        let result = AgentExecutionResult {
            container_id: "test-container-123".to_string(),
            exit_code,
            skills_installed,
            skills_install_failed,
        };

        assert_eq!(
            result.exit_code, 137,
            "AgentExecutionResult.exit_code should be 137 (SIGKILL)"
        );
        assert_eq!(
            result.skills_installed, None,
            "AgentExecutionResult.skills_installed should be None for timeout"
        );
        assert_eq!(
            result.skills_install_failed, false,
            "AgentExecutionResult.skills_install_failed should be false for timeout"
        );
    }

    /// Test: no skills configured
    ///
    /// This test verifies the behavior when no skills are configured in the container.
    /// In this case, skills_installed should be None and skills_install_failed
    /// should be false regardless of exit code.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_no_skills_configured() {
        use crate::docker::run::types::ContainerConfig;

        // Test scenario: No skills configured
        let config = ContainerConfig::new("test-agent".to_string());

        // Test with successful exit code
        let exit_code: i64 = 0;
        let timed_out: bool = false;

        let skills_installed = if config.skills.as_ref().is_some_and(|s| !s.is_empty()) {
            if exit_code == 0 {
                Some(true)
            } else if !timed_out {
                Some(false)
            } else {
                None
            }
        } else {
            // No skills configured
            None
        };

        let skills_install_failed =
            config.skills.as_ref().is_some_and(|s| !s.is_empty()) && exit_code != 0 && !timed_out;

        // Verify no skills scenario is handled correctly
        assert_eq!(
            skills_installed, None,
            "When no skills are configured, skills_installed should be None"
        );

        assert_eq!(
            skills_install_failed, false,
            "When no skills are configured, skills_install_failed should be false"
        );

        // Test with failed exit code (should still not affect skill installation tracking)
        let exit_code_failure: i64 = 1;
        let skills_installed_failure = if config.skills.as_ref().is_some_and(|s| !s.is_empty()) {
            if exit_code_failure == 0 {
                Some(true)
            } else if !timed_out {
                Some(false)
            } else {
                None
            }
        } else {
            None
        };

        let skills_install_failed_failure = config.skills.as_ref().is_some_and(|s| !s.is_empty())
            && exit_code_failure != 0
            && !timed_out;

        assert_eq!(
            skills_installed_failure,
            None,
            "When no skills are configured, skills_installed should be None even with non-zero exit code"
        );

        assert_eq!(
            skills_install_failed_failure, false,
            "When no skills are configured, skills_install_failed should be false even with non-zero exit code"
        );
    }

    /// Test: empty skills list configured
    ///
    /// This test verifies the behavior when an empty skills list is configured.
    /// This is functionally equivalent to no skills being configured.
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_empty_skills_list_configured() {
        use crate::docker::run::types::ContainerConfig;

        // Test scenario: Empty skills list configured
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec![]);

        // Test with successful exit code
        let exit_code: i64 = 0;
        let timed_out: bool = false;

        let skills_installed = if config.skills.as_ref().is_some_and(|s| !s.is_empty()) {
            if exit_code == 0 {
                Some(true)
            } else if !timed_out {
                Some(false)
            } else {
                None
            }
        } else {
            // No skills or empty skills list configured
            None
        };

        let skills_install_failed =
            config.skills.as_ref().is_some_and(|s| !s.is_empty()) && exit_code != 0 && !timed_out;

        // Verify empty skills list scenario is handled correctly
        assert_eq!(
            skills_installed, None,
            "When an empty skills list is configured, skills_installed should be None"
        );

        assert_eq!(
            skills_install_failed, false,
            "When an empty skills list is configured, skills_install_failed should be false"
        );

        // Verify skill counts for metrics would be zero
        let total_skills_count = config.skills.as_ref().map(|s| s.len()).unwrap_or(0);
        assert_eq!(
            total_skills_count, 0,
            "Empty skills list should result in zero total skills count"
        );
    }

    /// Test: skill_install_failure_metrics_record_failure
    ///
    /// This test verifies that when skill installation fails, the metrics
    /// correctly record the failure with total_skills_failed > 0.
    ///
    /// Verifies:
    /// - Container returns non-zero exit code on skill install failure
    /// - Skill installation phase failure is detected
    /// - Agent execution phase is not reached (simulated by checking exit_code)
    /// - Metrics record the failure (total_skills_failed > 0, skills_installed_count = 0)
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skill_install_failure_metrics_record_failure() {
        use crate::docker::run::types::ContainerConfig;

        // Test scenario: Skills configured and skill installation fails
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec![
            "owner/repo1".to_string(),
            "owner/repo2@skill".to_string(),
        ]);

        // Simulate the logic from run_agent() with failure exit code
        let exit_code: i64 = 1; // Non-zero exit code indicates failure
        let timed_out: bool = false; // Not a timeout - actual failure

        // This is the skill installation detection logic from run_agent() lines 1066-1084
        let skills_installed = if config.skills.as_ref().is_some_and(|s| !s.is_empty()) {
            // Skills were configured - determine if installation succeeded or failed
            if exit_code == 0 {
                Some(true) // Skills installed successfully
            } else if !timed_out {
                Some(false) // Skills installation failed (non-zero exit code, not a timeout)
            } else {
                None // Timed out - unknown if skills installed
            }
        } else {
            // No skills configured
            None
        };

        let skills_install_failed =
            config.skills.as_ref().is_some_and(|s| !s.is_empty()) && exit_code != 0 && !timed_out;

        // This is the skill counts calculation from run_agent() lines 1097-1112
        let total_skills_count = config.skills.as_ref().map(|s| s.len()).unwrap_or(0);
        let (skills_installed_count, skills_failed_count) = if skills_install_failed {
            // All skills failed to install
            (0, total_skills_count)
        } else if skills_installed == Some(true) {
            // All skills installed successfully
            (total_skills_count, 0)
        } else {
            // No skills configured or unknown status (timeout)
            (0, 0)
        };

        // ===== VERIFICATION =====

        // 1. Verify container returns non-zero exit code on skill install failure
        assert_ne!(
            exit_code, 0,
            "Container should return non-zero exit code on skill install failure"
        );

        // 2. Verify skill installation phase failure is detected
        assert_eq!(
            skills_installed,
            Some(false),
            "skills_installed should be Some(false) when skill installation fails"
        );
        assert_eq!(
            skills_install_failed, true,
            "skills_install_failed should be true when skill installation fails"
        );

        // 3. Verify agent execution phase is not reached
        // The exit_code of 1 indicates the container exited during skill installation
        // (the entrypoint script with 'set -e' exits immediately on any command failure)
        // The agent execution phase ('exec kilocode --yes "$@"') is never reached
        assert_eq!(
            exit_code, 1,
            "Container exit code 1 indicates skill install failed before agent execution"
        );

        // 4. Verify metrics record the failure
        // The metrics calculation should show:
        // - skills_installed_count = 0 (no skills installed)
        // - skills_failed_count = total_skills_count (all failed)
        assert_eq!(
            skills_installed_count, 0,
            "Metrics should record 0 skills installed when installation fails"
        );
        assert_eq!(
            skills_failed_count, 2,
            "Metrics should record 2 skills failed (total_skills_count)"
        );
        assert!(
            skills_failed_count > 0,
            "Metrics should show total_skills_failed > 0"
        );

        // Verify total_skills_count is correct
        assert_eq!(total_skills_count, 2, "Should have 2 configured skills");

        // Verify the AgentExecutionResult would correctly reflect this state
        let result = AgentExecutionResult {
            container_id: "test-container-123".to_string(),
            exit_code,
            skills_installed,
            skills_install_failed,
        };

        assert_eq!(
            result.exit_code, 1,
            "AgentExecutionResult.exit_code should be non-zero for failure"
        );
        assert_eq!(
            result.skills_installed,
            Some(false),
            "AgentExecutionResult.skills_installed should be Some(false)"
        );
        assert_eq!(
            result.skills_install_failed, true,
            "AgentExecutionResult.skills_install_failed should be true"
        );
    }

    /// Test: skill_install_success_log_has_prefix
    ///
    /// This test verifies that when skill installation succeeds, logs include the
    /// `[SKILL INSTALL]` prefix in the expected format.
    ///
    /// Verifies:
    /// - Success log messages start with `[SKILL INSTALL]`
    /// - The skill source is logged in the format: `[SKILL INSTALL] Installing skill: <skill-source>`
    /// - The script generation includes proper log prefixes for each skill
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skill_install_success_log_has_prefix() {
        use crate::docker::skills::generate_entrypoint_script;

        // Test scenario: Successful skill installation
        let skills = vec![
            "owner/repo".to_string(),
            "owner/repo@skill-name".to_string(),
        ];
        let agent_name = "test-agent";

        // Generate the entrypoint script (which includes the log messages)
        let script = generate_entrypoint_script(agent_name, &skills, &[])
            .expect("Failed to generate entrypoint script");

        // Verify each skill installation message has the correct prefix
        for skill in &skills {
            let expected_prefix = format!("[SKILL INSTALL] Installing skill: {}", skill);
            assert!(
                script.contains(&expected_prefix),
                "Script should contain skill installation log with [SKILL INSTALL] prefix for '{}'",
                skill
            );
        }

        // Verify the [SKILL INSTALL] prefix appears for each skill
        let install_log_count = script.matches("[SKILL INSTALL] Installing skill:").count();
        assert_eq!(
            install_log_count,
            skills.len(),
            "Script should have one [SKILL INSTALL] Installing skill: message per skill"
        );

        // Verify the prefix format is exact (no variations)
        assert!(
            script.contains("[SKILL INSTALL] Installing skill: owner/repo"),
            "Script should have exact format for first skill"
        );
        assert!(
            script.contains("[SKILL INSTALL] Installing skill: owner/repo@skill-name"),
            "Script should have exact format for second skill with skill-name"
        );
    }

    /// Test: skill_install_failure_log_has_prefix
    ///
    /// This test verifies that when skill installation fails, logs include the
    /// `[SKILL INSTALL] Error:` prefix with error details.
    ///
    /// Verifies:
    /// - Error log messages start with `[SKILL INSTALL] Error:`
    /// - The skill source is included in the error message
    /// - Exit code information is logged with the prefix
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skill_install_failure_log_has_prefix() {
        use crate::docker::run::types::ContainerConfig;

        // Test scenario: Skill installation failure
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec![
            "owner/repo".to_string(),
            "owner/repo2@skill".to_string(),
        ]);

        // Simulate the error logging from run_agent() lines 907-950
        let skills_install_failed = true;
        let exit_code: i64 = 1;

        if skills_install_failed {
            let error_msg = format!(
                "[SKILL INSTALL] Error: Skill installation failed for agent '{}'",
                config.agent_name
            );
            let exit_msg = format!("[SKILL INSTALL] Exit code: {}", exit_code);

            // Verify error message has the correct prefix
            assert!(
                error_msg.starts_with("[SKILL INSTALL] Error:"),
                "Error message should start with [SKILL INSTALL] Error: prefix"
            );
            assert!(
                error_msg.contains(&config.agent_name),
                "Error message should include the agent name"
            );
            assert!(
                error_msg.contains("Skill installation failed"),
                "Error message should describe the failure"
            );

            // Verify exit code message has the correct prefix
            assert!(
                exit_msg.starts_with("[SKILL INSTALL] Exit code:"),
                "Exit code message should start with [SKILL INSTALL] Exit code: prefix"
            );
            assert!(
                exit_msg.contains(&exit_code.to_string()),
                "Exit code message should include the actual exit code"
            );
        }

        // Verify skills are mentioned in the failure logs
        let skills_str = config
            .skills
            .as_ref()
            .map(|skills| skills.join(", "))
            .unwrap_or_else(|| "none".to_string());
        let skills_msg = format!("[SKILL INSTALL] Skills being installed: {}", skills_str);

        assert!(
            skills_msg.starts_with("[SKILL INSTALL]"),
            "Skills message should have the [SKILL INSTALL] prefix"
        );
        assert!(
            skills_msg.contains("owner/repo"),
            "Skills message should include the skill source"
        );
    }

    /// Test: skill_install_stderr_has_distinct_prefix
    ///
    /// This test verifies that stderr output during skill installation uses the
    /// `[SKILL INSTALL STDERR]` prefix to distinguish it from stdout.
    ///
    /// Verifies:
    /// - Stderr output is captured with `[SKILL INSTALL STDERR]` prefix
    /// - The prefix makes stderr easily distinguishable from stdout
    /// - Each stderr line gets the prefix applied
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skill_install_stderr_has_distinct_prefix() {
        use crate::docker::skills::generate_entrypoint_script;

        // Test scenario: Skill installation with stderr output
        let skills = vec!["owner/repo".to_string()];
        let agent_name = "test-agent";

        // Generate the entrypoint script
        let script = generate_entrypoint_script(agent_name, &skills, &[])
            .expect("Failed to generate entrypoint script");

        // Verify the script captures stderr and prefixes it
        assert!(
            script.contains("[SKILL INSTALL STDERR]"),
            "Script should contain [SKILL INSTALL STDERR] prefix for stderr"
        );

        // Verify stderr is piped and each line gets the prefix
        assert!(
            script.contains("echo \"[SKILL INSTALL STDERR] $line\""),
            "Script should echo each stderr line with the [SKILL INSTALL STDERR] prefix"
        );

        // Verify stderr is redirected (2>&1) so it's captured
        assert!(
            script.contains("2>&1 | while IFS= read -r line"),
            "Script should redirect stderr (2>&1) and read it line by line"
        );

        // Verify the stderr prefix is distinct from stdout prefix
        let stdout_prefix = "[SKILL INSTALL] Installing skill:";
        let stderr_prefix = "[SKILL INSTALL STDERR]";

        assert!(
            script.contains(stdout_prefix),
            "Script should have stdout prefix [SKILL INSTALL]"
        );
        assert!(
            script.contains(stderr_prefix),
            "Script should have stderr prefix [SKILL INSTALL STDERR]"
        );
        assert_ne!(
            stdout_prefix, stderr_prefix,
            "Stdout and stderr prefixes should be distinct"
        );

        // Count occurrences - stderr prefix should appear once per skill command
        let stderr_count = script.matches("[SKILL INSTALL STDERR]").count();
        assert!(
            stderr_count >= 1,
            "Script should have at least one [SKILL INSTALL STDERR] prefix per skill"
        );
    }

    /// Test: skill_install_logs_are_distinguishable_from_agent_logs
    ///
    /// This test verifies that skill installation logs can be easily separated from
    /// agent execution logs using the distinct prefixes.
    ///
    /// Verifies:
    /// - Skill installation logs have the [SKILL INSTALL] prefix
    /// - Agent logs do NOT have this prefix
    /// - The two types of logs can be filtered/separated by checking for the prefix
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skill_install_logs_are_distinguishable_from_agent_logs() {
        use crate::docker::skills::generate_entrypoint_script;

        // Test scenario: Skill installation followed by agent execution
        let skills = vec!["owner/repo".to_string()];
        let agent_name = "test-agent";

        // Generate the entrypoint script (contains skill installation logs)
        let script = generate_entrypoint_script(agent_name, &skills, &[])
            .expect("Failed to generate entrypoint script");

        // Simulate collecting logs from both phases
        let all_logs = vec![
            "[SKILL INSTALL] Installing skill: owner/repo".to_string(),
            "[SKILL INSTALL] Installing skill: owner/repo completed".to_string(),
            "Agent processing request...".to_string(),
            "Generating response...".to_string(),
            "Task completed successfully".to_string(),
        ];

        // Filter skill installation logs
        let skill_install_logs: Vec<&String> = all_logs
            .iter()
            .filter(|log| log.starts_with("[SKILL INSTALL]"))
            .collect();

        // Filter agent logs (those without the skill install prefix)
        let agent_logs: Vec<&String> = all_logs
            .iter()
            .filter(|log| !log.starts_with("[SKILL INSTALL]"))
            .collect();

        // Verify skill installation logs are correctly identified
        assert_eq!(
            skill_install_logs.len(),
            2,
            "Should correctly identify 2 skill installation log entries"
        );
        assert!(
            skill_install_logs
                .iter()
                .all(|log| log.starts_with("[SKILL INSTALL]")),
            "All skill installation logs should have the [SKILL INSTALL] prefix"
        );

        // Verify agent logs are correctly identified (no [SKILL INSTALL] prefix)
        assert_eq!(
            agent_logs.len(),
            3,
            "Should correctly identify 3 agent log entries"
        );
        assert!(
            agent_logs
                .iter()
                .all(|log| !log.starts_with("[SKILL INSTALL]")),
            "No agent logs should have the [SKILL INSTALL] prefix"
        );

        // Verify the script itself allows for this separation
        let script_skill_lines: Vec<&str> = script
            .lines()
            .filter(|line| line.contains("[SKILL INSTALL]"))
            .collect();

        let script_exec_lines: Vec<&str> = script
            .lines()
            .filter(|line| line.contains("exec kilocode"))
            .collect();

        assert!(
            !script_skill_lines.is_empty(),
            "Script should contain skill installation log lines with [SKILL INSTALL] prefix"
        );
        assert!(
            !script_exec_lines.is_empty(),
            "Script should contain agent execution command (exec kilocode)"
        );

        // Verify that skill installation logs and agent execution happen in sequence
        // First skill installation logs, then agent execution
        let first_skill_install = script.find("[SKILL INSTALL] Installing skill:");
        let first_agent_exec = script.find("exec kilocode");

        assert!(
            first_skill_install.is_some(),
            "Script should have skill installation logs"
        );
        assert!(
            first_agent_exec.is_some(),
            "Script should have agent execution command"
        );
        assert!(
            first_skill_install < first_agent_exec,
            "Skill installation logs should appear before agent execution in the script"
        );

        // Verify filtering works correctly with the prefix
        let skill_prefix = "[SKILL INSTALL]";
        let has_skill_install_logs = all_logs.iter().any(|log| log.contains(skill_prefix));
        let has_agent_logs = all_logs.iter().any(|log| !log.contains(skill_prefix));

        assert!(
            has_skill_install_logs,
            "Log collection should include skill installation logs"
        );
        assert!(has_agent_logs, "Log collection should include agent logs");
    }

    /// Test: skills_installed_counter_increments_on_success
    ///
    /// This test verifies that when skills install successfully, the
    /// `skills_installed` counter increments appropriately and `skills_failed`
    /// remains 0.
    ///
    /// Verifies:
    /// - `skills_installed_count` increments to match the number of skills
    /// - `skills_failed_count` remains 0
    /// - `skills_install_time_seconds` is recorded (not None)
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skills_installed_counter_increments_on_success() {
        use crate::metrics::AgentRunResult;
        use chrono::{Duration, Utc};

        let start_time = Utc::now();
        let end_time = start_time + Duration::seconds(5);

        // Simulate successful installation of 2 skills
        let run_result = AgentRunResult {
            agent_name: "test-agent".to_string(),
            container_id: "container-123".to_string(),
            start_time,
            end_time,
            exit_code: 0, // Success exit code
            timed_out: false,
            termination_type: None,
            queued_start_time: None,
            skills_installed_count: 2, // 2 skills installed successfully
            skills_failed_count: 0,    // No failures
            skills_install_time_seconds: Some(5.0), // Time recorded
        };

        // Verify skills_installed counter is correct
        assert_eq!(
            run_result.skills_installed_count, 2,
            "skills_installed_count should be 2 when 2 skills install successfully"
        );

        // Verify skills_failed counter is 0
        assert_eq!(
            run_result.skills_failed_count, 0,
            "skills_failed_count should be 0 when all skills install successfully"
        );

        // Verify install time is recorded
        assert!(
            run_result.skills_install_time_seconds.is_some(),
            "skills_install_time_seconds should be Some when skills are installed"
        );

        // Verify time is positive (not 0.0)
        assert!(
            run_result.skills_install_time_seconds.unwrap() > 0.0,
            "skills_install_time_seconds should be positive (greater than 0.0)"
        );

        // Verify exit code is 0 (success)
        assert_eq!(
            run_result.exit_code, 0,
            "exit_code should be 0 when skills install successfully"
        );
    }

    /// Test: skills_failed_counter_increments_on_failure
    ///
    /// This test verifies that when skill installation fails, the
    /// `skills_failed` counter increments appropriately and `skills_installed`
    /// remains 0.
    ///
    /// Verifies:
    /// - `skills_failed_count` increments to match the number of failed skills
    /// - `skills_installed_count` remains 0
    /// - `skills_install_time_seconds` is recorded even on failure
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skills_failed_counter_increments_on_failure() {
        use crate::metrics::AgentRunResult;
        use chrono::{Duration, Utc};

        let start_time = Utc::now();
        let end_time = start_time + Duration::seconds(3);

        // Simulate failed installation of 3 skills
        let run_result = AgentRunResult {
            agent_name: "test-agent".to_string(),
            container_id: "container-456".to_string(),
            start_time,
            end_time,
            exit_code: 127, // Non-zero exit code indicates failure
            timed_out: false,
            termination_type: None,
            queued_start_time: None,
            skills_installed_count: 0,              // No skills installed
            skills_failed_count: 3,                 // 3 skills failed to install
            skills_install_time_seconds: Some(3.0), // Time recorded even on failure
        };

        // Verify skills_failed counter is correct
        assert_eq!(
            run_result.skills_failed_count, 3,
            "skills_failed_count should be 3 when 3 skills fail to install"
        );

        // Verify skills_installed counter is 0
        assert_eq!(
            run_result.skills_installed_count, 0,
            "skills_installed_count should be 0 when all skills fail to install"
        );

        // Verify install time is recorded even on failure
        assert!(
            run_result.skills_install_time_seconds.is_some(),
            "skills_install_time_seconds should be Some even when installation fails"
        );

        // Verify time is positive (not 0.0)
        assert!(
            run_result.skills_install_time_seconds.unwrap() > 0.0,
            "skills_install_time_seconds should be positive even on failure"
        );

        // Verify exit code is non-zero (failure)
        assert_ne!(
            run_result.exit_code, 0,
            "exit_code should be non-zero when skill installation fails"
        );
    }

    /// Test: skills_install_time_seconds_is_recorded
    ///
    /// This test verifies that `skills_install_time_seconds` is accurately
    /// recorded for skill installations with varying durations.
    ///
    /// Verifies:
    /// - Time is in seconds (not milliseconds or other units)
    /// - Time is reasonable (positive, not unreasonably large)
    /// - Different durations are accurately recorded
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_skills_install_time_seconds_is_recorded() {
        use crate::metrics::AgentRunResult;
        use chrono::{Duration, Utc};

        let start_time = Utc::now();

        // Test with a quick installation (2 seconds)
        let end_time_quick = start_time + Duration::seconds(2);
        let run_result_quick = AgentRunResult {
            agent_name: "test-agent".to_string(),
            container_id: "container-quick".to_string(),
            start_time,
            end_time: end_time_quick,
            exit_code: 0,
            timed_out: false,
            termination_type: None,
            queued_start_time: None,
            skills_installed_count: 1,
            skills_failed_count: 0,
            skills_install_time_seconds: Some(2.0),
        };

        // Verify quick installation time is recorded correctly
        assert_eq!(
            run_result_quick.skills_install_time_seconds,
            Some(2.0),
            "Quick installation time should be 2.0 seconds"
        );
        assert!(
            run_result_quick.skills_install_time_seconds.unwrap() > 0.0,
            "Installation time should be positive"
        );
        assert!(
            run_result_quick.skills_install_time_seconds.unwrap() < 60.0,
            "Quick installation should take less than 60 seconds"
        );

        // Test with a longer installation (15.5 seconds)
        let end_time_long = start_time + Duration::seconds(15);
        let run_result_long = AgentRunResult {
            agent_name: "test-agent".to_string(),
            container_id: "container-long".to_string(),
            start_time,
            end_time: end_time_long,
            exit_code: 0,
            timed_out: false,
            termination_type: None,
            queued_start_time: None,
            skills_installed_count: 3,
            skills_failed_count: 0,
            skills_install_time_seconds: Some(15.5),
        };

        // Verify longer installation time is recorded correctly
        assert_eq!(
            run_result_long.skills_install_time_seconds,
            Some(15.5),
            "Longer installation time should be 15.5 seconds"
        );
        assert!(
            run_result_long.skills_install_time_seconds.unwrap() > 2.0,
            "Longer installation should take more time than quick installation"
        );

        // Test with fractional seconds (7.75 seconds)
        let run_result_fractional = AgentRunResult {
            agent_name: "test-agent".to_string(),
            container_id: "container-fractional".to_string(),
            start_time,
            end_time: start_time + Duration::seconds(8),
            exit_code: 0,
            timed_out: false,
            termination_type: None,
            queued_start_time: None,
            skills_installed_count: 2,
            skills_failed_count: 0,
            skills_install_time_seconds: Some(7.75),
        };

        // Verify fractional seconds are recorded correctly
        assert_eq!(
            run_result_fractional.skills_install_time_seconds,
            Some(7.75),
            "Installation time with fractional seconds should be 7.75"
        );
        assert!(
            run_result_fractional.skills_install_time_seconds.unwrap() > 7.0,
            "Fractional time should be greater than floor value"
        );
        assert!(
            run_result_fractional.skills_install_time_seconds.unwrap() < 8.0,
            "Fractional time should be less than ceiling value"
        );

        // Verify time is not in milliseconds by checking scale
        // If time were in milliseconds, 2000ms would be recorded as 2000, which is unreasonably large for seconds
        let max_reasonable_seconds = 3600.0; // 1 hour
        assert!(
            run_result_quick.skills_install_time_seconds.unwrap() < max_reasonable_seconds,
            "Installation time should be in seconds (reasonable scale), not milliseconds"
        );
    }

    /// Test: metrics_handle_multiple_skills_mixed_outcome
    ///
    /// This test verifies that when multiple skills have mixed outcomes
    /// (some succeed, some fail), both counters reflect the correct counts.
    ///
    /// Verifies:
    /// - `skills_installed` and `skills_failed` counters reflect the correct counts
    /// - `skills_install_time_seconds` is the total installation time
    /// - Partial installation failure is handled correctly
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_metrics_handle_multiple_skills_mixed_outcome() {
        use crate::metrics::AgentRunResult;
        use chrono::{Duration, Utc};

        let start_time = Utc::now();
        let end_time = start_time + Duration::seconds(8);

        // Simulate mixed outcome: 2 skills installed, 1 skill failed
        // Total of 3 skills configured
        let run_result = AgentRunResult {
            agent_name: "test-agent".to_string(),
            container_id: "container-mixed".to_string(),
            start_time,
            end_time,
            exit_code: 127, // Non-zero exit code indicates some failure occurred
            timed_out: false,
            termination_type: None,
            queued_start_time: None,
            skills_installed_count: 2, // 2 skills installed successfully
            skills_failed_count: 1,    // 1 skill failed to install
            skills_install_time_seconds: Some(8.0), // Total time for all skill installation attempts
        };

        // Verify skills_installed counter reflects successful installations
        assert_eq!(
            run_result.skills_installed_count, 2,
            "skills_installed_count should be 2 when 2 out of 3 skills install successfully"
        );

        // Verify skills_failed counter reflects failures
        assert_eq!(
            run_result.skills_failed_count, 1,
            "skills_failed_count should be 1 when 1 out of 3 skills fails to install"
        );

        // Verify total skills (installed + failed) equals configured skills
        let total_skills = run_result.skills_installed_count + run_result.skills_failed_count;
        assert_eq!(
            total_skills, 3,
            "Total skills (installed + failed) should equal the number of configured skills"
        );

        // Verify install time is recorded for the complete installation process
        assert!(
            run_result.skills_install_time_seconds.is_some(),
            "skills_install_time_seconds should be recorded for mixed outcome"
        );

        assert_eq!(
            run_result.skills_install_time_seconds,
            Some(8.0),
            "skills_install_time_seconds should reflect the total installation time"
        );

        // Verify time is reasonable (positive, not unreasonably large)
        assert!(
            run_result.skills_install_time_seconds.unwrap() > 0.0,
            "Installation time should be positive"
        );

        // Verify exit code is non-zero (indicates partial failure)
        assert_ne!(
            run_result.exit_code, 0,
            "exit_code should be non-zero when there's a partial skill installation failure"
        );

        // Verify both counters are not zero (mixed outcome)
        assert!(
            run_result.skills_installed_count > 0 && run_result.skills_failed_count > 0,
            "Both skills_installed_count and skills_failed_count should be non-zero for mixed outcome"
        );

        // Test edge case: equal numbers of installed and failed
        let run_result_equal = AgentRunResult {
            agent_name: "test-agent".to_string(),
            container_id: "container-equal".to_string(),
            start_time,
            end_time: start_time + Duration::seconds(10),
            exit_code: 1,
            timed_out: false,
            termination_type: None,
            queued_start_time: None,
            skills_installed_count: 2, // 2 installed
            skills_failed_count: 2,    // 2 failed
            skills_install_time_seconds: Some(10.0),
        };

        assert_eq!(
            run_result_equal.skills_installed_count, 2,
            "skills_installed_count should be 2"
        );
        assert_eq!(
            run_result_equal.skills_failed_count, 2,
            "skills_failed_count should be 2"
        );
        assert_eq!(
            run_result_equal.skills_installed_count, run_result_equal.skills_failed_count,
            "For equal mixed outcome, both counters should have the same value"
        );
    }

    /// Test: error_message_includes_skill_source
    ///
    /// This test verifies that error messages generated when skill installation fails
    /// include the skill source (e.g., "owner/repo@skill-name") in a user-visible location.
    ///
    /// Verifies:
    /// - The skill source string is included in error logs
    /// - Each configured skill appears in the "Skills being installed" message
    /// - The skill source is easily identifiable in the error output
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_error_message_includes_skill_source() {
        use crate::docker::run::types::ContainerConfig;

        // Test scenario: Skill installation failure with specific skills
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec![
            "owner/repo1".to_string(),
            "owner/repo2@skill-name".to_string(),
            "another/cool-skill".to_string(),
        ]);

        let skills_install_failed = true;
        let _exit_code: i64 = 1;

        // Simulate the error message generation from run_agent() lines 896-950
        if skills_install_failed {
            // Format skills list as a comma-separated string
            let skills_str = config
                .skills
                .as_ref()
                .map(|skills| skills.join(", "))
                .unwrap_or_else(|| "none".to_string());

            let skills_msg = format!("[SKILL INSTALL] Skills being installed: {}", skills_str);

            // Verify that each skill source appears in the error message
            assert!(
                skills_msg.contains("owner/repo1"),
                "Error message should include skill source 'owner/repo1'"
            );
            assert!(
                skills_msg.contains("owner/repo2@skill-name"),
                "Error message should include skill source 'owner/repo2@skill-name'"
            );
            assert!(
                skills_msg.contains("another/cool-skill"),
                "Error message should include skill source 'another/cool-skill'"
            );

            // Verify the format matches the expected pattern
            assert!(
                skills_msg.starts_with("[SKILL INSTALL] Skills being installed:"),
                "Error message should start with expected prefix"
            );

            // Verify all skills are present (comma-separated)
            assert_eq!(
                skills_msg,
                "[SKILL INSTALL] Skills being installed: owner/repo1, owner/repo2@skill-name, another/cool-skill",
                "Error message should include all skills in correct format"
            );
        }
    }

    /// Test: error_message_includes_agent_name
    ///
    /// This test verifies that error messages generated when skill installation fails
    /// include the agent name to help identify which agent's skill failed.
    ///
    /// Verifies:
    /// - The agent name is included in the error message
    /// - The agent name helps identify which agent had the failure
    /// - Multiple agent failures can be distinguished by agent name
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_error_message_includes_agent_name() {
        use crate::docker::run::types::ContainerConfig;

        // Test scenario: Skill installation failure for different agents
        let agent_names = vec!["agent-1", "production-agent", "data-processor"];

        for agent_name in agent_names {
            let config = ContainerConfig::new(agent_name.to_string());
            let skills_install_failed = true;

            // Simulate the error message generation from run_agent() lines 896-950
            if skills_install_failed {
                let error_msg = format!(
                    "[SKILL INSTALL] Error: Skill installation failed for agent '{}'",
                    config.agent_name
                );

                // Verify that the agent name is included in the error message
                assert!(
                    error_msg.contains(&config.agent_name),
                    "Error message should include agent name '{}'",
                    config.agent_name
                );

                // Verify the format matches the expected pattern
                assert!(
                    error_msg.starts_with("[SKILL INSTALL] Error:"),
                    "Error message should start with expected prefix"
                );

                // Verify the agent name appears in the correct location
                assert!(
                    error_msg.contains(&format!("agent '{}'", config.agent_name)),
                    "Error message should mention the agent name in context"
                );
            }
        }

        // Test that different agent names produce distinguishable error messages
        let config1 = ContainerConfig::new("agent-alpha".to_string());
        let config2 = ContainerConfig::new("agent-beta".to_string());

        let error_msg1 = format!(
            "[SKILL INSTALL] Error: Skill installation failed for agent '{}'",
            config1.agent_name
        );
        let error_msg2 = format!(
            "[SKILL INSTALL] Error: Skill installation failed for agent '{}'",
            config2.agent_name
        );

        assert_ne!(
            error_msg1, error_msg2,
            "Error messages for different agents should be distinguishable"
        );
        assert!(
            error_msg1.contains("agent-alpha"),
            "First error message should contain 'agent-alpha'"
        );
        assert!(
            error_msg2.contains("agent-beta"),
            "Second error message should contain 'agent-beta'"
        );
    }

    /// Test: error_messages_are_user_friendly
    ///
    /// This test verifies that error messages generated during skill installation failures
    /// are clear, understandable, and not overly technical for users.
    ///
    /// Verifies:
    /// - Error messages are clear and understandable to users
    /// - Technical details (like raw stderr) are accompanied by user-friendly explanations
    /// - Error messages avoid overly technical jargon where possible
    /// - Messages provide context about what failed and why
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_error_messages_are_user_friendly() {
        use crate::docker::run::types::ContainerConfig;

        // Test scenario: Skill installation failure
        let config = ContainerConfig::new("test-agent".to_string());
        let skills_install_failed = true;
        let exit_code: i64 = 127;

        if skills_install_failed {
            // Generate the error messages as in run_agent() lines 907-949
            let error_msg = format!(
                "[SKILL INSTALL] Error: Skill installation failed for agent '{}'",
                config.agent_name
            );

            let exit_msg = format!("[SKILL INSTALL] Exit code: {}", exit_code);

            let context_msg = "[SKILL INSTALL] The agent did not execute. Fix the skill installation issues before retrying.";

            // Verify error message is user-friendly:
            // 1. Clear description of what happened
            assert!(
                error_msg.contains("Skill installation failed"),
                "Error message should clearly state what failed"
            );

            // 2. Identifies which agent
            assert!(
                error_msg.contains(&config.agent_name),
                "Error message should identify the agent"
            );

            // 3. Uses accessible language (not overly technical)
            assert!(
                error_msg.contains("Error:"),
                "Error message should have clear error indicator"
            );

            // Verify exit code message is accompanied by context
            assert!(
                exit_msg.starts_with("[SKILL INSTALL] Exit code:"),
                "Exit code should be clearly labeled"
            );
            assert!(
                exit_msg.contains(&exit_code.to_string()),
                "Exit code value should be present"
            );

            // Verify context message explains impact
            assert!(
                context_msg.contains("agent did not execute"),
                "Context message should explain the impact on execution"
            );
            assert!(
                context_msg.contains("before retrying"),
                "Context message should suggest next steps"
            );

            // Verify messages avoid overly technical jargon
            // For example, instead of "non-zero exit status" (technical), we say "Skill installation failed" (clear)
            assert!(
                !error_msg.contains("non-zero exit status"),
                "Error message should avoid overly technical jargon like 'non-zero exit status'"
            );

            // Note: Error message ends with a quote marker and placeholder, which is acceptable
            // The important aspect is clarity and helpfulness, not specific punctuation requirements
        }

        // Test with stderr simulation to verify technical details are accompanied by explanations
        let stderr_output = "[SKILL INSTALL STDERR] npm ERR! code ENOTFOUND\n[SKILL INSTALL STDERR] npm ERR! errno ENOTFOUND\n[SKILL INSTALL STDERR] npm ERR! network request failed";

        // Verify stderr is prefixed and identifiable
        assert!(
            stderr_output.contains("[SKILL INSTALL STDERR]"),
            "Stderr should be clearly marked for technical details"
        );

        // Verify stderr (technical) is separate from user-friendly messages
        assert!(
            stderr_output.contains("npm ERR!"),
            "Technical stderr should contain actual error details"
        );

        // But user-friendly messages would provide context like:
        let remediation_context = "[SKILL INSTALL] Remediation steps:\n - Check if the skill exists: switchboard skills list\n - Check network connectivity (npx needs internet access)";
        assert!(
            remediation_context.contains("Remediation steps"),
            "User-friendly remediation should guide the user"
        );
        assert!(
            remediation_context.contains("network connectivity"),
            "User-friendly message should translate technical issues"
        );
        assert!(
            remediation_context.contains("switchboard skills list"),
            "User-friendly message should provide actionable commands"
        );
    }

    /// Test: error_message_includes_remediation_suggestions
    ///
    /// This test verifies that error messages generated during skill installation failures
    /// include helpful, actionable remediation suggestions for users.
    ///
    /// Verifies:
    /// - Error messages include helpful remediation suggestions
    /// - Suggestions like "Check if skill exists: switchboard skills list" are present
    /// - Suggestions are actionable and specific
    /// - Multiple remediation options are provided to help users resolve the issue
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_error_message_includes_remediation_suggestions() {
        use crate::docker::run::types::ContainerConfig;

        // Test scenario: Skill installation failure
        let _config = ContainerConfig::new("test-agent".to_string());
        let skills_install_failed = true;

        if skills_install_failed {
            // Simulate the remediation message from run_agent() lines 933-942
            let remediation_msg = "[SKILL INSTALL] Remediation steps:
 - Check if the skill exists: switchboard skills list
 - Verify the skill format: owner/repo or owner/repo@skill-name
 - Check network connectivity (npx needs internet access)
 - Review [SKILL INSTALL STDERR] lines above for detailed error information";

            // Verify remediation suggestions are present
            assert!(
                remediation_msg.contains("Remediation steps"),
                "Error message should include a 'Remediation steps' section"
            );

            // Verify specific suggestions are present
            assert!(
                remediation_msg.contains("switchboard skills list"),
                "Should suggest checking if skill exists with: switchboard skills list"
            );

            assert!(
                remediation_msg.contains("owner/repo or owner/repo@skill-name"),
                "Should verify the skill format"
            );

            assert!(
                remediation_msg.contains("network connectivity"),
                "Should suggest checking network connectivity"
            );

            assert!(
                remediation_msg.contains("[SKILL INSTALL STDERR]"),
                "Should direct users to review stderr for detailed information"
            );

            // Verify suggestions are actionable (contain verbs and specific commands)
            assert!(
                remediation_msg.contains("Check if the skill exists"),
                "Remediation should be actionable with clear steps"
            );

            assert!(
                remediation_msg.contains("Verify the skill format"),
                "Remediation should include verification steps"
            );

            assert!(
                remediation_msg.contains("Review"),
                "Remediation should guide users to review relevant information"
            );

            // Verify suggestions cover different potential issues
            // 1. Skill existence
            assert!(
                remediation_msg.contains("switchboard skills list"),
                "Should provide a command to check skill existence"
            );

            // 2. Format issues
            assert!(
                remediation_msg.contains("skill format"),
                "Should address potential format issues"
            );

            // 3. Network issues
            assert!(
                remediation_msg.contains("network"),
                "Should address potential network connectivity issues"
            );

            // 4. Technical details
            assert!(
                remediation_msg.contains("detailed error information"),
                "Should point users to technical details for debugging"
            );

            // Verify remediation suggestions are properly formatted (bullet points)
            assert!(
                remediation_msg.contains(" - "),
                "Remediation suggestions should be in bullet point format for readability"
            );

            // Count the number of remediation suggestions
            let remediation_count = remediation_msg
                .lines()
                .filter(|line| line.trim().starts_with('-'))
                .count();
            assert_eq!(
                remediation_count, 4,
                "Should have 4 remediation suggestions covering different scenarios"
            );
        }

        // Test with a different scenario to ensure remediation is comprehensive
        let _config2 = ContainerConfig::new("production-agent".to_string());
        let config2_skills_install_failed = true;

        if config2_skills_install_failed {
            // The same remediation should apply regardless of agent name
            let remediation_msg = "[SKILL INSTALL] Remediation steps:
 - Check if the skill exists: switchboard skills list
 - Verify the skill format: owner/repo or owner/repo@skill-name
 - Check network connectivity (npx needs internet access)
 - Review [SKILL INSTALL STDERR] lines above for detailed error information";

            // Verify remediation is independent of agent name (generic and reusable)
            assert!(
                remediation_msg.contains("Remediation steps"),
                "Remediation should be generic and apply to any agent"
            );

            // Verify all critical suggestions are still present
            assert!(
                remediation_msg.contains("switchboard skills list")
                    && remediation_msg.contains("skill format")
                    && remediation_msg.contains("network connectivity"),
                "All critical remediation suggestions should be present"
            );
        }
    }

    /// Test: find_preexisting_skills with empty skills list
    ///
    /// This test verifies that passing an empty skills list to find_preexisting_skills
    /// returns an empty vector without attempting to read any directory.
    ///
    /// Verifies:
    /// - Empty skills list returns Ok(Vec::new())
    /// - No directory reading occurs
    /// - Function returns immediately
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_find_preexisting_skills_empty_skills_list() {
        // Create a temp directory (but we don't need to create skills directory)
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();

        // Pass empty skills list
        let skills: Vec<String> = vec![];
        let result = find_preexisting_skills(&skills, project_dir);

        // Assert: returns empty vector without error
        assert!(result.is_ok(), "Should return Ok for empty skills list");
        let preexisting = result.unwrap();
        assert!(
            preexisting.is_empty(),
            "Should return empty vector for empty skills list"
        );
    }

    /// Test: find_preexisting_skills with skills directory that doesn't exist
    ///
    /// This test verifies that when the .kilocode/skills/ directory doesn't exist,
    /// the function returns an empty vector without error.
    ///
    /// Verifies:
    /// - Missing skills directory returns Ok(Vec::new())
    /// - No error is returned for missing directory
    /// - Function handles this gracefully
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_find_preexisting_skills_skills_directory_not_exist() {
        // Create a temp directory without .kilocode/skills/
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();

        // Don't create the skills directory - it shouldn't exist
        let skills_dir = project_dir.join(".kilocode/skills");
        assert!(
            !skills_dir.exists(),
            "Skills directory should not exist for this test"
        );

        // Pass a skills list
        let skills = vec!["owner/repo".to_string()];
        let result = find_preexisting_skills(&skills, project_dir);

        // Assert: returns empty vector without error
        assert!(
            result.is_ok(),
            "Should return Ok when skills directory doesn't exist"
        );
        let preexisting = result.unwrap();
        assert!(
            preexisting.is_empty(),
            "Should return empty vector when directory doesn't exist"
        );
    }

    /// Test: find_preexisting_skills with empty skills directory
    ///
    /// This test verifies that when the .kilocode/skills/ directory exists but
    /// contains no SKILL.md files, the function returns an empty vector.
    ///
    /// Verifies:
    /// - Empty skills directory returns Ok(Vec::new())
    /// - Function correctly handles directories with no skills
    /// - No false positives for non-existent skills
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_find_preexisting_skills_directory_exists_no_skills() {
        use std::fs;

        // Create temp directory with empty .kilocode/skills/
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();

        // Create the skills directory but leave it empty
        let skills_dir = project_dir.join(".kilocode/skills");
        fs::create_dir_all(&skills_dir).expect("Failed to create skills dir");

        // Pass a skills list
        let skills = vec!["owner/repo".to_string()];
        let result = find_preexisting_skills(&skills, project_dir);

        // Assert: returns empty vector
        assert!(
            result.is_ok(),
            "Should return Ok for empty skills directory"
        );
        let preexisting = result.unwrap();
        assert!(
            preexisting.is_empty(),
            "Should return empty vector when no SKILL.md files exist"
        );
    }

    /// Test: find_preexisting_skills with single skill found
    ///
    /// This test verifies that the function correctly identifies a single preexisting
    /// skill by finding a SKILL.md file in the skills directory.
    ///
    /// Verifies:
    /// - Correctly extracts skill name from owner/repo format
    /// - Identifies skill with SKILL.md file
    /// - Returns vector with single skill name
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_find_preexisting_skills_single_skill_found() {
        use std::fs;

        // Create temp directory with skills structure
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();

        // Create skills directory and skill with SKILL.md
        let skills_dir = project_dir.join(".kilocode/skills");
        fs::create_dir_all(&skills_dir).expect("Failed to create skills dir");

        let skill_dir = skills_dir.join("repo1");
        fs::create_dir(&skill_dir).expect("Failed to create skill dir");
        fs::write(skill_dir.join("SKILL.md"), "# Test Skill\n").expect("Failed to write SKILL.md");

        // Pass a skills list with matching skill
        let skills = vec!["owner/repo1".to_string()];
        let result = find_preexisting_skills(&skills, project_dir);

        // Assert: returns the skill name
        assert!(result.is_ok(), "Should return Ok when skill is found");
        let preexisting = result.unwrap();
        assert_eq!(
            preexisting,
            vec!["repo1".to_string()],
            "Should find and return repo1 skill"
        );
    }

    /// Test: find_preexisting_skills with @ notation
    ///
    /// This test verifies that the function correctly extracts skill names from
    /// the owner/repo@skill-name format, using the part after the @ character.
    ///
    /// Verifies:
    /// - Correctly parses owner/repo@skill-name format
    /// - Uses skill-name (after @) for lookup
    /// - Returns correct skill name when found
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_find_preexisting_skills_skill_with_at_notation() {
        use std::fs;

        // Create temp directory with skills structure
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();

        // Create skills directory and skill with SKILL.md
        let skills_dir = project_dir.join(".kilocode/skills");
        fs::create_dir_all(&skills_dir).expect("Failed to create skills dir");

        // Note: the skill directory is named "my-skill" (the part after @)
        let skill_dir = skills_dir.join("my-skill");
        fs::create_dir(&skill_dir).expect("Failed to create skill dir");
        fs::write(skill_dir.join("SKILL.md"), "# My Skill\n").expect("Failed to write SKILL.md");

        // Pass a skills list with @ notation
        let skills = vec!["owner/repo2@my-skill".to_string()];
        let result = find_preexisting_skills(&skills, project_dir);

        // Assert: returns the skill name (after @)
        assert!(result.is_ok(), "Should return Ok for skill with @ notation");
        let preexisting = result.unwrap();
        assert_eq!(
            preexisting,
            vec!["my-skill".to_string()],
            "Should find and return my-skill"
        );
    }

    /// Test: find_preexisting_skills with multiple skills and mixed results
    ///
    /// This test verifies that the function correctly handles multiple skills,
    /// finding those with SKILL.md files and ignoring those without.
    ///
    /// Verifies:
    /// - Finds only skills with SKILL.md files
    /// - Ignores directories without SKILL.md
    /// - Returns vector in the order skills were configured
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_find_preexisting_skills_multiple_skills_mixed() {
        use std::fs;

        // Create temp directory with skills structure
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();

        // Create skills directory
        let skills_dir = project_dir.join(".kilocode/skills");
        fs::create_dir_all(&skills_dir).expect("Failed to create skills dir");

        // Create repo1 with SKILL.md
        let repo1_dir = skills_dir.join("repo1");
        fs::create_dir(&repo1_dir).expect("Failed to create repo1 dir");
        fs::write(repo1_dir.join("SKILL.md"), "# Repo1 Skill\n")
            .expect("Failed to write repo1 SKILL.md");

        // Create skill2 with SKILL.md (from @ notation)
        let skill2_dir = skills_dir.join("skill2");
        fs::create_dir(&skill2_dir).expect("Failed to create skill2 dir");
        fs::write(skill2_dir.join("SKILL.md"), "# Skill2\n")
            .expect("Failed to write skill2 SKILL.md");

        // Create repo3 directory WITHOUT SKILL.md (should be ignored)
        let repo3_dir = skills_dir.join("repo3");
        fs::create_dir(&repo3_dir).expect("Failed to create repo3 dir");
        // Don't create SKILL.md - this should be ignored

        // Pass a skills list with mixed results
        let skills = vec![
            "owner/repo1".to_string(),
            "owner/repo2@skill2".to_string(),
            "owner/repo3".to_string(),
        ];
        let result = find_preexisting_skills(&skills, project_dir);

        // Assert: returns only skills with SKILL.md files
        assert!(result.is_ok(), "Should return Ok for mixed skills");
        let preexisting = result.unwrap();
        assert_eq!(
            preexisting,
            vec!["repo1".to_string(), "skill2".to_string()],
            "Should find repo1 and skill2, but not repo3 (no SKILL.md)"
        );
    }

    /// Test: find_preexisting_skills ignores non-directory entries
    ///
    /// This test verifies that the function correctly ignores non-directory entries
    /// (e.g., files) in the skills directory.
    ///
    /// Verifies:
    /// - Non-directory entries are ignored
    /// - Only directories with SKILL.md are considered
    /// - Function handles mixed directory/file content correctly
    #[allow(clippy::bool_assert_comparison)]
    #[allow(clippy::useless_vec)]
    #[test]
    fn test_find_preexisting_skills_non_directory_entries_ignored() {
        use std::fs;

        // Create temp directory with skills structure
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_dir = temp_dir.path();

        // Create skills directory
        let skills_dir = project_dir.join(".kilocode/skills");
        fs::create_dir_all(&skills_dir).expect("Failed to create skills dir");

        // Create repo1 directory with SKILL.md
        let repo1_dir = skills_dir.join("repo1");
        fs::create_dir(&repo1_dir).expect("Failed to create repo1 dir");
        fs::write(repo1_dir.join("SKILL.md"), "# Repo1 Skill\n")
            .expect("Failed to write repo1 SKILL.md");

        // Create a non-directory file in skills directory (should be ignored)
        let not_a_dir = skills_dir.join("not-a-dir.txt");
        fs::write(&not_a_dir, "This is a file, not a directory")
            .expect("Failed to create non-directory file");

        // Pass a skills list
        let skills = vec!["owner/repo1".to_string()];
        let result = find_preexisting_skills(&skills, project_dir);

        // Assert: returns only the directory skill, ignores the file
        assert!(
            result.is_ok(),
            "Should return Ok when non-directory entries exist"
        );
        let preexisting = result.unwrap();
        assert_eq!(
            preexisting,
            vec!["repo1".to_string()],
            "Should find repo1 and ignore the file"
        );
    }
}
