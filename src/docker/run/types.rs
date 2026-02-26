//! Container configuration types
//!
//! This module provides types for configuring and creating Docker containers
//! for agent execution, including:
//! - ContainerConfig: Configuration for creating agent containers
//! - ContainerError: Error types for container operations
//!
//! These types are used by the container execution logic to create
//! properly configured containers with workspace mounts, environment
//! variables, timeouts, and read-only filesystem support.

/// Configuration for creating and running an agent container
///
/// `ContainerConfig` encapsulates all the parameters needed to configure and launch
/// a Docker container for AI agent execution. It includes agent identification,
/// environment configuration, execution parameters, and optional skill-based
/// entrypoint script generation.
///
/// # Skills Field Behavior
///
/// The `skills` field controls how the container entrypoint is configured:
///
/// - **`None`**: No skills specified, use the default container entrypoint
/// - **`Some([])`**: Empty skills list, use the default container entrypoint
/// - **`Some([...])`**: One or more skills, generate and inject a custom entrypoint script
///
/// When skills are specified (a non-empty list), the system will generate a shell
/// script that invokes each skill in order before executing the agent's primary task.
/// This generated script is then set as the container's entrypoint.
///
/// # Fields
///
/// * **`agent_name`** - Unique identifier for the agent, used in container naming and logging
/// * **`env_vars`** - Environment variables passed to the container in `KEY=value` format
/// * **`timeout`** - Optional execution timeout (e.g., `"30s"`, `"5m"`). If `None`, no timeout is enforced
/// * **`readonly`** - When `true`, mounts the workspace read-only to prevent agent modifications
/// * **`prompt`** - The instruction or task description provided to the AI agent
/// * **`skills`** - Optional list of skill identifiers. Controls entrypoint script generation
///
/// # Examples
///
/// Creating a basic container configuration without skills:
///
/// ```rust,ignore
/// use switchboard::docker::run::types::ContainerConfig;
///
/// let config = ContainerConfig::new("agent-1".to_string())
///     .with_prompt("Analyze this data");
/// ```
///
/// Creating a configuration with skills for custom entrypoint generation:
///
/// ```rust,ignore
/// use switchboard::docker::run::types::ContainerConfig;
///
/// let config = ContainerConfig::new("agent-2".to_string())
///     .with_skills(vec!["code-analyzer".to_string(), "report-generator".to_string()])
///     .with_prompt("Generate a report");
/// ```
///
/// Using default entrypoint with explicitly empty skills list:
///
/// ```rust,ignore
/// use switchboard::docker::run::types::ContainerConfig;
///
/// let config = ContainerConfig::new("agent-3".to_string())
///     .with_skills(vec![]);  // Empty list = default entrypoint
/// ```
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Name of the agent
    pub agent_name: String,
    /// Environment variables in KEY=value format
    pub env_vars: Vec<String>,
    /// Optional timeout duration (e.g., "30s", "5m")
    pub timeout: Option<String>,
    /// Whether to mount the workspace as read-only
    pub readonly: bool,
    /// Prompt for the AI agent
    pub prompt: String,
    /// Optional list of skill identifiers for the agent
    ///
    /// Three possible values:
    /// - `None`: No skills specified, use default container entrypoint
    /// - `Some([])`: Empty skills list, use default container entrypoint
    /// - `Some([...])`: One or more skills, generate and inject entrypoint script
    pub skills: Option<Vec<String>>,
}

impl ContainerConfig {
    /// Creates a new `ContainerConfig` with default values
    ///
    /// This constructor initializes a container configuration with sensible defaults
    /// that can be customized through method chaining. All optional fields are set
    /// to their default state, which generally means "no special behavior."
    ///
    /// # Default Values
    ///
    /// - `env_vars`: Empty vector (`Vec::new()`) - No environment variables set
    /// - `timeout`: `None` - No timeout enforced
    /// - `readonly`: `false` - Workspace mounted as read-write
    /// - `prompt`: Empty string (`String::new()`) - No agent prompt specified
    /// - `skills`: `None` - No skills specified, container will use default entrypoint
    ///
    /// # Parameters
    ///
    /// * **`agent_name`** - The unique identifier for this agent. This value is required
    ///   and is used for container naming and logging purposes
    ///
    /// # Examples
    ///
    /// Creating a basic configuration:
    ///
    /// ```rust,ignore
    /// use switchboard::docker::run::types::ContainerConfig;
    ///
    /// let config = ContainerConfig::new("my-agent".to_string());
    /// ```
    ///
    /// Creating and customizing a configuration:
    ///
    /// ```rust,ignore
    /// use switchboard::docker::run::types::ContainerConfig;
    ///
    /// let config = ContainerConfig::new("data-analyzer".to_string())
    ///     .with_timeout("5m")
    ///     .with_readonly(true)
    ///     .with_prompt("Analyze the data file");
    /// ```
    ///
    /// Creating with skills for custom entrypoint:
    ///
    /// ```rust,ignore
    /// use switchboard::docker::run::types::ContainerConfig;
    ///
    /// let config = ContainerConfig::new("code-reviewer".to_string())
    ///     .with_skills(vec!["linter".to_string(), "security-scan".to_string()]);
    /// ```
    pub fn new(agent_name: String) -> Self {
        ContainerConfig {
            agent_name,
            env_vars: Vec::new(),
            timeout: None,
            readonly: false,
            prompt: String::new(),
            skills: None,
        }
    }
}

/// Errors that can occur during container execution
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    /// Failed to create container
    #[error("Container creation failed: {0}")]
    ContainerCreationFailed(String),

    /// Failed to start container
    #[error("Container start failed: {0}")]
    ContainerStartFailed(String),

    /// Invalid timeout format
    #[error("Invalid timeout: {0}")]
    InvalidTimeout(String),
}

/// Converts a `ContainerError` into a `DockerError`.
///
/// This implementation enables automatic error conversion using the `?` operator
/// when working with container-specific errors in contexts that expect the more
/// general `DockerError` type. The conversion maps each `ContainerError` variant
/// to an appropriate `DockerError` variant:
///
/// - `ContainerCreationFailed` maps to `DockerError::ConnectionError`
/// - `ContainerStartFailed` maps to `DockerError::ConnectionError`
/// - `InvalidTimeout` maps to `DockerError::IoError`
///
/// This conversion preserves the error message and context while allowing
/// container operations to seamlessly integrate with the broader Docker error
/// handling system.
///
/// # Examples
///
/// ```rust,ignore
/// use switchboard::docker::run::types::ContainerError;
/// use switchboard::docker::DockerError;
///
/// // Automatic conversion with ? operator
/// fn example_function() -> Result<(), DockerError> {
///     let result = do_container_operation()?;
///     Ok(())
/// }
///
/// fn do_container_operation() -> Result<(), ContainerError> {
///     Err(ContainerError::ContainerCreationFailed("Failed".to_string()))
/// }
/// ```
impl From<ContainerError> for super::DockerError {
    fn from(err: ContainerError) -> Self {
        match err {
            ContainerError::ContainerCreationFailed(msg) => {
                super::DockerError::ConnectionError(format!("Container creation failed: {}", msg))
            }
            ContainerError::ContainerStartFailed(msg) => {
                super::DockerError::ConnectionError(format!("Container start failed: {}", msg))
            }
            ContainerError::InvalidTimeout(msg) => super::DockerError::IoError {
                operation: "parse timeout".to_string(),
                error_details: msg.to_string(),
            },
        }
    }
}
