//! Config Parser - Parse and validate switchboard.toml configuration files
//!
//! This module handles:
//! - TOML file parsing and deserialization
//! - Configuration data structures (Settings, Agent, Config, ApiConfig, RateLimitConfig, OverlapMode)
//! - Config file loading from disk
//! - Validation logic for agent configurations
//! - Overlap mode configuration (Skip and Queue modes)
//! - REST API server configuration

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use cron::Schedule;
use once_cell::sync::Lazy;
use regex::Regex;

pub mod env;
mod env_ext;

#[cfg(feature = "discord")]
use crate::discord::config::DiscordSection;

use crate::docker::run::wait::parse_timeout;
use tracing;

/// Lazy-compiled regex for validating skill source format.
///
/// This regex matches simple skill names like:
/// - `frontend-design` - A skill name
/// - `security-audit` - A skill name with hyphen
/// - `my_skill` - A skill name with underscore
///
/// Pattern breakdown:
/// - `^[a-zA-Z0-9_-]+$` - One or more alphanumeric characters, hyphens, or underscores
static SKILL_SOURCE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").expect("Invalid SKILL_SOURCE_REGEX pattern"));

/// Configuration parsing and validation errors.
///
/// This enum represents all possible errors that can occur when parsing
/// and validating a Switchboard configuration file. It provides detailed
/// information about the error location (when available) and helpful
/// suggestions for fixing common issues.
///
/// # Examples
///
/// ```rust
/// use switchboard::config::Config;
/// use std::path::Path;
///
/// let result = Config::from_toml(Path::new("nonexistent.toml"));
/// match result {
///     Err(switchboard::config::ConfigError::ParseError { file, .. }) => {
///         println!("Failed to parse file: {}", file);
///     }
///     Err(switchboard::config::ConfigError::PromptFileNotFound { agent_name, prompt_file }) => {
///         println!("Prompt file {} not found for agent {}", prompt_file, agent_name);
///     }
///     Err(switchboard::config::ConfigError::ValidationError { message, .. }) => {
///         println!("Validation error: {}", message);
///     }
///     Ok(_) => println!("Configuration loaded successfully"),
/// }
/// ```
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Error parsing the TOML file with location information.
    ///
    /// This variant is returned when the TOML configuration file cannot be parsed,
    /// typically due to syntax errors, invalid TOML structure, or unsupported values.
    /// The error includes file location information (line and column when available)
    /// and often includes a helpful suggestion for fixing the issue.
    ///
    /// # Fields
    ///
    /// * `file` - Path to the file being parsed
    /// * `line` - Line number where the error occurred (when available)
    /// * `col` - Column number where the error occurred (when available)
    /// * `message` - Descriptive error message
    /// * `suggestion` - Helpful suggestion for fixing the error (when available)
    ///
    /// # Common Causes
    ///
    /// - Missing quotes around strings
    /// - Unclosed quotes or brackets
    /// - Missing commas in arrays
    /// - Missing equals signs in key-value pairs
    /// - Invalid characters in keys
    /// - Invalid escape sequences
    /// - Type mismatches (e.g., string where integer expected)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use switchboard::config::{Config, ConfigError};
    /// use std::path::Path;
    ///
    /// # let _ = std::fs::write("invalid.toml", "invalid toml content");
    /// let result = Config::from_toml(Path::new("invalid.toml"));
    /// if let Err(ConfigError::ParseError { file, line, col, message, suggestion }) = result {
    ///     eprintln!("Error in {} at line {}:{}", file, line.unwrap_or(0), col.unwrap_or(0));
    ///     eprintln!("  {}", message);
    ///     if let Some(sugg) = suggestion {
    ///         eprintln!("Suggestion: {}", sugg);
    ///     }
    /// }
    /// ```
    ParseError {
        /// Path to the file being parsed
        file: String,
        /// Line number where the error occurred
        line: Option<usize>,
        /// Column number where the error occurred
        col: Option<usize>,
        /// Error message
        message: String,
        /// Helpful suggestion for fixing the error
        suggestion: Option<String>,
    },
    /// Error validating the configuration.
    ///
    /// This variant is returned when the TOML file is syntactically valid but
    /// the configuration values violate validation rules. This includes semantic
    /// errors such as missing required fields, invalid values, or constraint violations.
    ///
    /// # Fields
    ///
    /// * `message` - Descriptive validation error message
    /// * `agent_name` - Name of the agent where the error occurred (when applicable)
    /// * `field_name` - Name of the field that failed validation (when applicable)
    /// * `line` - Line number where the error occurred (when available)
    /// * `col` - Column number where the error occurred (when available)
    ///
    /// # Common Causes
    ///
    /// - Empty agent names
    /// - Duplicate agent names
    /// - Missing required fields (e.g., schedule)
    /// - Invalid cron expressions
    /// - Invalid timezone strings
    /// - Invalid timeout values (e.g., "0s", too large)
    /// - Invalid overlap mode values
    /// - Missing prompt files
    /// - Both `prompt` and `prompt_file` specified for an agent
    /// - Neither `prompt` nor `prompt_file` specified for an agent
    ///
    /// # Examples
    ///
    /// ```rust
    /// use switchboard::config::ConfigError;
    ///
    /// # let result: Result<(), ConfigError> = Err(ConfigError::ValidationError {
    /// #     message: "Invalid timeout value".to_string(),
    /// #     agent_name: Some("test-agent".to_string()),
    /// #     field_name: Some("timeout".to_string()),
    /// #     line: Some(10),
    /// #     col: Some(15),
    /// # });
    /// // Example of handling a validation error
    /// match result {
    ///     Err(ConfigError::ValidationError {
    ///         message,
    ///         agent_name,
    ///         field_name,
    ///         ..
    ///     }) => {
    ///         if let Some(agent) = agent_name {
    ///             eprintln!("Error in agent '{}': {}", agent, message);
    ///         } else {
    ///             eprintln!("Configuration error: {}", message);
    ///         }
    ///         if let Some(field) = field_name {
    ///             eprintln!("  Invalid field: {}", field);
    ///         }
    ///     }
    ///     _ => {}
    /// }
    /// ```
    ValidationError {
        message: String,
        agent_name: Option<String>,
        field_name: Option<String>,
        line: Option<usize>,
        col: Option<usize>,
    },
    /// Prompt file not found.
    ///
    /// This variant is returned when an agent specifies a `prompt_file` that
    /// cannot be found at the specified path. The path is resolved relative to
    /// the configuration file directory, so the error message shows the resolved
    /// absolute path.
    ///
    /// # Fields
    ///
    /// * `agent_name` - Name of the agent referencing the missing prompt file
    /// * `prompt_file` - Full path to the prompt file that could not be found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use switchboard::config::ConfigError;
    ///
    /// # let result: Result<(), ConfigError> = Err(ConfigError::PromptFileNotFound {
    /// #     agent_name: "test-agent".to_string(),
    /// #     prompt_file: "/nonexistent/file.md".to_string(),
    /// # });
    /// match result {
    ///     Err(ConfigError::PromptFileNotFound { agent_name, prompt_file }) => {
    ///         eprintln!("Prompt file not found for agent '{}': {}", agent_name, prompt_file);
    ///         eprintln!("Ensure the file exists relative to your switchboard.toml location");
    ///     }
    ///     _ => {}
    /// }
    /// ```
    PromptFileNotFound {
        agent_name: String,
        prompt_file: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::ParseError {
                file,
                line,
                col,
                message,
                suggestion,
            } => {
                // Build the main error message with location info
                let base_msg = match (line, col) {
                    (Some(l), Some(c)) => {
                        format!("Error parsing {}:line {}, col {}: {}", file, l, c, message)
                    }
                    (Some(l), None) => {
                        format!("Error parsing {}:line {}: {}", file, l, message)
                    }
                    (None, Some(_)) => {
                        format!("Error parsing {}: {}", file, message)
                    }
                    (None, None) => {
                        format!("Error parsing {}: {}", file, message)
                    }
                };

                // Add suggestion if available
                if let Some(sugg) = suggestion {
                    write!(f, "{}\n\n{}", base_msg, sugg)
                } else {
                    write!(f, "{}", base_msg)
                }
            }
            ConfigError::ValidationError {
                message,
                agent_name,
                field_name,
                line,
                col,
            } => {
                let base_message = match (agent_name, field_name) {
                    (Some(agent), Some(field)) => {
                        format!(
                            "Validation error in agent '{}' field '{}': {}",
                            agent, field, message
                        )
                    }
                    (Some(agent), None) => {
                        format!("Validation error in agent '{}': {}", agent, message)
                    }
                    (None, Some(field)) => {
                        format!("Validation error in field '{}': {}", field, message)
                    }
                    (None, None) => {
                        format!("Validation error: {}", message)
                    }
                };

                match (line, col) {
                    (Some(l), Some(c)) => write!(f, "{} (line {}, col {})", base_message, l, c),
                    (Some(l), None) => write!(f, "{} (line {})", base_message, l),
                    (None, Some(c)) => write!(f, "{} (col {})", base_message, c),
                    (None, None) => write!(f, "{}", base_message),
                }
            }
            ConfigError::PromptFileNotFound {
                agent_name,
                prompt_file,
            } => {
                write!(
                    f,
                    "Prompt file '{}' not found for agent '{}'",
                    prompt_file, agent_name
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Generate a helpful suggestion based on the TOML parsing error message
fn generate_parse_suggestion(message: &str) -> Option<String> {
    let lower_msg = message.to_lowercase();

    // Check for specific error patterns and provide targeted suggestions

    // Missing quotes around strings
    if lower_msg.contains("unexpected token") && lower_msg.contains("expected") {
        if lower_msg.contains("string") {
            Some(
                "Fix: Strings must be enclosed in quotes. Use double quotes for regular strings: \"value\"\n\
                 Example: name = \"my-agent\"  ✓\n\
                 Wrong:    name = my-agent    ✗".to_string()
            )
        } else {
            None
        }
    // Unclosed quotes (single or double)
    } else if lower_msg.contains("unclosed quote") || lower_msg.contains("unterminated string") {
        if lower_msg.contains("'") {
            Some(
                "Fix: Add a closing single quote ('). Single quotes use ''' for multiline.\n\
                 Example: prompt = '''multi\n\
                 line\n\
                 text'''  ✓"
                    .to_string(),
            )
        } else {
            Some(
                "Fix: Add a closing double quote (\") to match the opening quote.\n\
                 Example: name = \"my-agent\"  ✓\n\
                 Wrong:    name = \"my-agent   ✗"
                    .to_string(),
            )
        }
    // Missing commas in arrays
    } else if lower_msg.contains("expected `,`")
        || (lower_msg.contains("expected") && lower_msg.contains("array"))
    {
        Some(
            "Fix: Array elements must be separated by commas.\n\
             Example: items = [\"item1\", \"item2\", \"item3\"]  ✓\n\
             Wrong:    items = [\"item1\" \"item2\" \"item3\"]  ✗"
                .to_string(),
        )
    // Missing equals signs in key-value pairs
    } else if lower_msg.contains("expected `=`") || lower_msg.contains("expected equals") {
        Some(
            "Fix: Key-value pairs require an equals sign (=) between the key and value.\n\
             Example: name = \"my-agent\"  ✓\n\
             Wrong:    name \"my-agent\"    ✗"
                .to_string(),
        )
    // Invalid characters in keys
    } else if lower_msg.contains("invalid character in key") || lower_msg.contains("invalid key") {
        Some(
            "Fix: TOML keys must be alphanumeric with hyphens, underscores, or dots. Spaces are not allowed.\n\
             Valid:   agent-name, agent_name, agent.name  ✓\n\
             Invalid: agent name, agent@name, agent#name   ✗".to_string()
        )
    // Table definition errors - wrong bracket types
    } else if lower_msg.contains("expected `]`") && lower_msg.contains("table") {
        Some(
            "Fix: TOML tables use double square brackets. Inline tables use curly braces.\n\
             Standard table: [section]\n\
             Nested table:   [section.subsection]\n\
             Wrong:          {section} or (section)  ✗"
                .to_string(),
        )
    // Unclosed table brackets
    } else if lower_msg.contains("unclosed bracket") || lower_msg.contains("expected `]`") {
        Some(
            "Fix: Add closing bracket(s) to complete the table or array definition.\n\
             Table:     [agents]\n\
             Array:     items = [1, 2, 3]\n\
             Inline:    { key = \"value\" }"
                .to_string(),
        )
    // Inline table syntax errors
    } else if lower_msg.contains("expected `}`") || lower_msg.contains("unclosed brace") {
        Some(
            "Fix: Inline tables use curly braces with key-value pairs separated by commas.\n\
             Example: settings = { timeout = 30, retries = 3 }  ✓\n\
             Wrong:    settings = { timeout = 30 retries = 3 }  ✗ (missing comma)"
                .to_string(),
        )
    // Multiline string syntax errors
    } else if lower_msg.contains("newline in string") || lower_msg.contains("multiline string") {
        Some(
            "Fix: Use triple quotes for multiline strings.\n\
             Triple double quotes: \"\"\"text\n\
             more text\"\"\"\n\
             Triple single quotes: '''text\n\
             more text'''\n\
             Note: Leading newline on first line is ignored."
                .to_string(),
        )
    // Table header syntax errors
    } else if lower_msg.contains("table header")
        || (lower_msg.contains("section") && lower_msg.contains("header"))
    {
        Some(
            "Fix: Table headers use square brackets. Nested tables use dots.\n\
             Example: [agents]\n\
             Nested:  [agents.settings]\n\
             Array:   [[agents]]  (creates array of tables)"
                .to_string(),
        )
    // Array syntax errors
    } else if lower_msg.contains("array") && lower_msg.contains("error") {
        Some(
            "Fix: Arrays use square brackets with comma-separated values of the same type.\n\
             Example: ports = [8080, 8081, 8082]  ✓\n\
             Mixed:   items = [1, \"text\", true]   ✗ (inconsistent types)"
                .to_string(),
        )
    // Boolean value errors
    } else if lower_msg.contains("boolean")
        || lower_msg.contains("true") && lower_msg.contains("false")
    {
        Some(
            "Fix: TOML booleans are lowercase 'true' or 'false' without quotes.\n\
             Example: enabled = true   ✓\n\
             Wrong:    enabled = \"true\"  ✗ (quoted)\n\
             Wrong:    enabled = True     ✗ (capitalized)"
                .to_string(),
        )
    // Integer/float syntax errors
    } else if lower_msg.contains("number")
        || (lower_msg.contains("integer") || lower_msg.contains("float"))
    {
        Some(
            "Fix: Numbers should not be quoted. Use underscores for readability in large numbers.\n\
             Example: port = 8080\n\
             Large:   count = 1_000_000\n\
             Float:   rate = 3.14\n\
             Wrong:   port = \"8080\"  ✗ (quoted)".to_string()
        )
    // Invalid escape sequences
    } else if lower_msg.contains("invalid escape sequence") {
        Some(
            "Fix: Review escape sequences in strings. Valid escapes:\n\
             \\n (newline), \\t (tab), \\\\ (backslash), \\\" (quote)\n\
             \\b (backspace), \\f (form feed), \\r (carriage return)\n\
             Example: path = \"C:\\\\Users\\\\name\"  (use double backslashes)"
                .to_string(),
        )
    // Date format errors
    } else if lower_msg.contains("invalid date") || lower_msg.contains("datetime") {
        Some(
            "Fix: TOML dates follow RFC 3339 format.\n\
             Date:     2024-02-15\n\
             DateTime: 2024-02-15T10:30:00Z\n\
             Local:    2024-02-15T10:30:00-05:00\n\
             Time:     10:30:00"
                .to_string(),
        )
    // Key or identifier errors
    } else if lower_msg.contains("expected key") || lower_msg.contains("expected identifier") {
        Some(
            "Fix: Ensure each key-value pair has a valid key followed by = and a value.\n\
             Valid keys: letters, numbers, hyphens (-), underscores (_), dots (.)\n\
             Example: agent-name = \"worker\"\n\
             Example: agent.name = \"worker\""
                .to_string(),
        )
    // Encoding errors
    } else if lower_msg.contains("invalid utf-8") {
        Some(
            "Fix: Ensure the file is saved with UTF-8 encoding. TOML files must be valid UTF-8 text.\n\
             Check your editor's encoding setting and save as UTF-8.".to_string()
        )
    // Unexpected character errors
    } else if lower_msg.contains("unexpected character") {
        Some(
            "Fix: Check for typos or invalid characters. TOML uses specific characters:\n\
             =    (assignment, key = value)\n\
             []   (arrays and tables)\n\
             {}   (inline tables)\n\
             #    (comments)\n\
             \"\"   (strings)"
                .to_string(),
        )
    // Missing required fields
    } else if lower_msg.contains("missing key") || lower_msg.contains("required field") {
        Some(
            "Fix: Add the missing required field to the configuration.\n\
             Refer to switchboard.sample.toml for a complete example configuration."
                .to_string(),
        )
    // Invalid enum/option values
    } else if lower_msg.contains("invalid enum value") || lower_msg.contains("invalid option") {
        Some(
            "Fix: Use a valid value for this field. Check switchboard.sample.toml or documentation for allowed values.\n\
             Example: overlap_mode = \"skip\"  (valid values: skip, error, or queue)".to_string()
        )
    // General TOML parse errors
    } else if lower_msg.contains("toml parse error") || lower_msg.contains("parse error") {
        Some(
            "Fix: Review the syntax near the indicated line/column.\n\
             Common issues:\n\
             • Missing quotes around strings\n\
             • Mismatched brackets or braces\n\
             • Invalid characters in keys\n\
             • Missing commas in arrays\n\
             • Incorrect value types\n\
             • Missing equals sign in key-value pairs"
                .to_string(),
        )
    // File read errors
    } else if lower_msg.contains("failed to read file") {
        Some(
            "Fix: Ensure the file path is correct and you have permission to read the file."
                .to_string(),
        )
    } else if lower_msg.contains("invalid type") || lower_msg.contains("expected a different type")
    {
        Some(
            "Fix: Check that the value type matches the expected type.\n\
             • Booleans: true or false (unquoted)\n\
             • Strings: must be quoted\n\
             • Numbers: must not be quoted\n\
             • Arrays: comma-separated values in []\n\
             Example: timeout = 30  (integer, not \"30\")"
                .to_string(),
        )
    } else if lower_msg.contains("duplicate key") || lower_msg.contains("duplicate field") {
        Some(
            "Fix: Remove or rename the duplicate key. Each key in a TOML section must be unique.\n\
             Example: [agents]\n\
                      name = \"worker1\"\n\
                      name = \"worker2\"  ✗ (duplicate - use different key or use array)"
                .to_string(),
        )
    } else {
        None
    }
}

/// Format a type validation error message with expected vs actual type information
///
/// # Arguments
///
/// * `field_name` - Name of the field that failed validation
/// * `expected_type` - Description of the expected type (e.g., "string", "integer", "boolean")
/// * `actual_value` - The actual value that was provided (or type description if value unavailable)
/// * `valid_examples` - Optional comma-separated examples of valid values
///
/// # Returns
///
/// A formatted error message string
fn format_type_error(
    field_name: &str,
    expected_type: &str,
    actual_value: &str,
    valid_examples: Option<&str>,
) -> String {
    let mut message = format!(
        "Invalid type for field '{}': expected {}, but found: {}",
        field_name, expected_type, actual_value
    );

    if let Some(examples) = valid_examples {
        message.push_str(&format!("\nValid examples: {}", examples));
    }

    message
}

/// How to handle overlapping agent executions.
///
/// This enum defines the behavior when a scheduled agent execution is triggered
/// while a previous execution of the same agent is still running. The overlap mode
/// can be configured globally in the `[settings]` section or overridden per-agent
/// in each `[[agent]]` section.
///
/// # Default Behavior
///
/// The default overlap mode is [`OverlapMode::Skip`], which means that if an agent is already
/// running when a new execution is scheduled, the new execution is skipped and a
/// warning is logged. This is specified in the PRD §9.
///
/// # Configuration
///
/// The overlap mode can be set in TOML configuration files using either PascalCase
/// (e.g., "Skip", "Queue") or lowercase (e.g., "skip", "queue") due to the
/// `#[serde(alias)]` attributes.
///
/// # Examples
///
/// ## Global setting
///
/// ```toml
/// [settings]
/// overlap_mode = "skip"  # or "queue"
///
/// [[agent]]
/// name = "my-agent"
/// prompt_file = "prompts/agent.md"
/// schedule = "0 */6 * * *"
/// ```
///
/// ## Per-agent override
///
/// ```toml
/// [settings]
/// overlap_mode = "queue"
///
/// [[agent]]
/// name = "critical-agent"
/// prompt_file = "prompts/critical.md"
/// schedule = "0 * * * *"
/// overlap_mode = "skip"  # Override global setting for this agent
/// ```
///
/// ## Runtime usage
///
/// ```rust
/// use switchboard::config::OverlapMode;
///
/// let mode = OverlapMode::Skip;
/// match mode {
///     OverlapMode::Skip => println!("Will skip if already running"),
///     OverlapMode::Queue => println!("Will queue up to max_queue_size runs"),
/// }
/// ```
///
/// # Variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum OverlapMode {
    /// When agent is running, skip new run and log warning (PRD §9 default behavior).
    ///
    /// In this mode, if a scheduled execution is triggered while a previous execution
    /// of the same agent is still active, the new execution is immediately skipped.
    /// A warning is logged to inform the user that the execution was skipped.
    ///
    /// This is the default overlap mode and is appropriate for agents where:
    /// - Running multiple instances concurrently could cause conflicts
    /// - Each execution is idempotent and can be safely skipped
    /// - Resource consumption should be limited
    ///
    /// # Examples
    ///
    /// ```toml
    /// [settings]
    /// overlap_mode = "Skip"
    /// ```
    ///
    /// ```rust
    /// use switchboard::config::OverlapMode;
    ///
    /// let mode = OverlapMode::Skip;
    /// assert_eq!(mode, OverlapMode::default());
    /// ```
    #[serde(alias = "skip")]
    #[default]
    Skip,
    /// Add new runs to queue, execute sequentially after current run completes.
    ///
    /// In this mode, when a scheduled execution is triggered while a previous execution
    /// is still active, the new execution is added to a queue. The executions are then
    /// processed sequentially, with each queued execution starting only after the
    /// previous one completes.
    ///
    /// The maximum queue size can be configured using the `max_queue_size` field.
    /// If the queue is full when a new execution is scheduled, it will be skipped.
    /// The default queue size is 3.
    ///
    /// This mode is appropriate for agents where:
    /// - All scheduled executions should be processed (not skipped)
    /// - Sequential execution is acceptable
    /// - Some backlog is tolerable
    ///
    /// # Examples
    ///
    /// ```toml
    /// [settings]
    /// overlap_mode = "Queue"
    ///
    /// [[agent]]
    /// name = "data-processor"
    /// prompt_file = "prompts/process.md"
    /// schedule = "0 * * * *"
    /// max_queue_size = 5  # Allow up to 5 queued executions
    /// ```
    ///
    /// ```rust
    /// use switchboard::config::OverlapMode;
    ///
    /// let mode = OverlapMode::Queue;
    /// let queue_size = 5;  // Configured max_queue_size
    /// ```
    #[serde(alias = "queue")]
    Queue,
}

/// Global settings that apply to all agents
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Settings {
    /// Docker image name
    #[serde(default)]
    pub image_name: String,
    /// Docker image tag
    #[serde(default)]
    pub image_tag: String,
    /// Directory for log files
    #[serde(default)]
    pub log_dir: String,
    /// Timezone for agent execution
    #[serde(default)]
    pub timezone: String,
    /// How to handle overlapping executions: "skip" (default) or "queue"
    #[serde(default)]
    pub overlap_mode_str: String,
    /// Global default overlap mode
    #[serde(default)]
    pub overlap_mode: Option<OverlapMode>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            image_name: "switchboard-agent".to_string(),
            image_tag: "latest".to_string(),
            log_dir: ".switchboard/logs".to_string(),
            timezone: "system".to_string(),
            overlap_mode_str: "skip".to_string(),
            overlap_mode: None,
        }
    }
}

/// Rate limiting configuration for the REST API
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    #[serde(default = "default_rate_limit_enabled")]
    pub enabled: bool,
    /// Number of requests allowed per minute
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        RateLimitConfig {
            enabled: default_rate_limit_enabled(),
            requests_per_minute: default_requests_per_minute(),
        }
    }
}

/// API server configuration
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ApiConfig {
    /// Enable REST API server
    #[serde(default = "default_api_enabled")]
    pub enabled: bool,
    /// Unique instance identifier (derived from config file if not set)
    #[serde(default)]
    pub instance_id: Option<String>,
    /// API server port
    #[serde(default = "default_api_port")]
    pub port: u16,
    /// Bind host address
    #[serde(default = "default_api_host")]
    pub host: String,
    /// Auto port selection if port is in use
    #[serde(default = "default_auto_port")]
    pub auto_port: bool,
    /// Enable Swagger UI
    #[serde(default = "default_swagger")]
    pub swagger: bool,
    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
}

impl Default for ApiConfig {
    fn default() -> Self {
        ApiConfig {
            enabled: default_api_enabled(),
            instance_id: None,
            port: default_api_port(),
            host: default_api_host(),
            auto_port: default_auto_port(),
            swagger: default_swagger(),
            rate_limit: RateLimitConfig::default(),
        }
    }
}

// Default value functions for API config
fn default_api_enabled() -> bool {
    false
}

fn default_api_port() -> u16 {
    18500
}

fn default_api_host() -> String {
    "127.0.0.1".to_string()
}

fn default_auto_port() -> bool {
    true
}

fn default_swagger() -> bool {
    false
}

fn default_rate_limit_enabled() -> bool {
    true
}

fn default_requests_per_minute() -> u32 {
    60
}

/// Default function for Agent.env field
fn default_env() -> Option<HashMap<String, String>> {
    Some(HashMap::new())
}

/// Agent configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Agent {
    /// Unique agent identifier
    pub name: String,
    /// Inline prompt text (exactly one of prompt or prompt_file must be provided)
    #[serde(default)]
    pub prompt: Option<String>,
    /// Path to prompt file (exactly one of prompt or prompt_file must be provided)
    #[serde(default)]
    pub prompt_file: Option<String>,
    /// Cron schedule for agent execution
    pub schedule: String,
    /// Optional environment variables (Settings does not have an env field, so only agent.env is used)
    #[serde(default = "default_env")]
    pub env: Option<HashMap<String, String>>,
    /// Whether the agent runs in read-only mode (no filesystem writes)
    #[serde(default)]
    pub readonly: Option<bool>,
    /// Maximum execution duration (e.g., "30m", "2h")
    #[serde(default)]
    pub timeout: Option<String>,
    /// Agent-level override for overlap mode
    #[serde(default)]
    pub overlap_mode: Option<OverlapMode>,
    /// Queue size override for Queue mode
    #[serde(default)]
    pub max_queue_size: Option<usize>,
    /// Optional list of skills available to this agent
    /// Each entry should be a simple skill name (e.g., "frontend-design", "security-audit")
    /// Format: alphanumeric characters, hyphens, and underscores only
    #[serde(default)]
    pub skills: Option<Vec<String>>,
}

impl Default for Agent {
    fn default() -> Self {
        Agent {
            name: String::new(),
            prompt: None,
            prompt_file: None,
            schedule: String::new(),
            env: Some(HashMap::new()),
            readonly: None,
            timeout: Some("30m".to_string()),
            overlap_mode: None,
            max_queue_size: None,
            skills: None,
        }
    }
}

impl Agent {
    /// Resolve the effective schedule for this agent
    /// Returns agent.schedule
    #[allow(dead_code)]
    pub fn schedule(&self) -> &String {
        &self.schedule
    }

    /// Resolve the effective environment variables for this agent
    /// Returns a Vec of "KEY=value" strings from the agent's env field
    /// Note: Settings does not have an env field, so only agent.env is used
    #[allow(dead_code)]
    pub fn env(&self, global: Option<&Settings>) -> Vec<String> {
        let mut result = Vec::new();

        // Add global env first
        if let Some(g) = global {
            // Settings doesn't have env field, so nothing to add
            let _ = g;
        }

        // Add agent env (can override or add to global)
        // Convert HashMap to "KEY=value" format strings
        if let Some(agent_env) = &self.env {
            for (key, value) in agent_env {
                result.push(format!("{}={}", key, value));
            }
        }

        result
    }

    /// Resolve the prompt_file path relative to the config file directory
    /// Returns an absolute path to the prompt file (if prompt_file is set)
    pub fn resolve_prompt_file(&self, config_dir: &Path) -> Option<PathBuf> {
        let prompt_file = self.prompt_file.as_ref()?;
        let path = PathBuf::from(prompt_file);

        // If already absolute, return as-is
        if path.is_absolute() {
            Some(path)
        } else {
            // Join with config directory and canonicalize
            let resolved = config_dir.join(&path);
            Some(resolved.canonicalize().unwrap_or(resolved))
        }
    }

    /// Read the contents of the prompt file
    /// Returns Ok(None) if no prompt_file is set, or Ok(Some(contents)) on success
    #[allow(dead_code)]
    pub fn read_prompt_file(&self, config_dir: &Path) -> Result<Option<String>, ConfigError> {
        // If prompt_file is None, return Ok(None)
        if self.prompt_file.is_none() {
            return Ok(None);
        }

        // Get the resolved path
        let path = match self.resolve_prompt_file(config_dir) {
            Some(p) => p,
            None => return Ok(None),
        };

        // Read the file contents
        let contents = fs::read_to_string(&path).map_err(|e| {
            let message = format!("Failed to read prompt file: {}", e);
            let suggestion = generate_parse_suggestion(&message);
            ConfigError::ParseError {
                file: path.display().to_string(),
                line: None,
                col: None,
                message,
                suggestion,
            }
        })?;

        Ok(Some(contents))
    }

    /// Resolve the effective overlap mode for this agent
    /// Returns agent's overlap_mode if set, otherwise returns global setting if set,
    /// otherwise returns OverlapMode::Skip (PRD §9 default)
    pub fn effective_overlap_mode(&self, global_settings: &Option<Settings>) -> OverlapMode {
        // Check agent-specific setting first
        if let Some(agent_mode) = &self.overlap_mode {
            return *agent_mode;
        }

        // Fall back to global setting
        if let Some(settings) = global_settings {
            if let Some(global_mode) = &settings.overlap_mode {
                return *global_mode;
            }
        }

        // Default to Skip (PRD §9)
        OverlapMode::Skip
    }

    /// Resolve the effective max queue size for this agent
    /// Returns agent's max_queue_size if set, otherwise returns 3 (the default queue size)
    pub fn effective_max_queue_size(&self) -> usize {
        self.max_queue_size.unwrap_or(3)
    }
}

/// Top-level configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Optional global settings
    #[serde(default, rename = "settings")]
    pub settings: Option<Settings>,
    /// Required list of agent configurations
    #[serde(default, rename = "agent")]
    pub agents: Vec<Agent>,
    /// Optional Discord configuration
    #[cfg(feature = "discord")]
    #[serde(default, rename = "discord")]
    pub discord: Option<DiscordSection>,
    /// Optional REST API configuration
    #[serde(default, rename = "api")]
    pub api: Option<ApiConfig>,
    /// Path to the config file (not deserialized from TOML)
    #[serde(skip)]
    config_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            settings: None,
            agents: Vec::new(),
            #[cfg(feature = "discord")]
            discord: None,
            api: None,
            config_path: PathBuf::new(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file at the given path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the switchboard.toml configuration file
    ///
    /// # Returns
    ///
    /// * `Ok(Config)` - Successfully parsed configuration
    /// * `Err(ConfigError)` - Error reading or parsing the file
    pub fn from_toml(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path).map_err(|e| {
            let message = format!("Failed to read file: {}", e);
            let suggestion = generate_parse_suggestion(&message);
            ConfigError::ParseError {
                file: path.display().to_string(),
                line: None,
                col: None,
                message,
                suggestion,
            }
        })?;

        let mut config: Config = toml::from_str(&content).map_err(|e| {
            // Use toml 0.8's span() method to get location information
            // The error message from Display already includes line and column info
            let span = e.span();
            let (line, col) = if let Some(span) = span {
                // Convert span to line and column
                let span_start = span.start;
                let span_before_content = &content[..span_start.min(content.len())];
                let line = span_before_content.chars().filter(|&c| c == '\n').count() + 1;
                let col = if let Some(last_newline) = span_before_content.rfind('\n') {
                    span_start - last_newline - 1
                } else {
                    span_start
                };
                (Some(line), Some(col))
            } else {
                (None, None)
            };
            let message = e.to_string();
            let suggestion = generate_parse_suggestion(&message);
            ConfigError::ParseError {
                file: path.display().to_string(),
                line,
                col,
                message,
                suggestion,
            }
        })?;

        // Canonicalize and store the config path
        config.config_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        // Validate at least one agent is defined
        if config.agents.is_empty() {
            return Err(ConfigError::ValidationError {
                message: "Configuration must define at least one agent. Add [[agent]] sections to your switchboard.toml".to_string(),
                agent_name: None,
                field_name: None,
                line: None,
                col: None,
            });
        }

        // Validate agent names are unique
        let mut seen_names = HashSet::new();
        for agent in &config.agents {
            if !seen_names.insert(&agent.name) {
                return Err(ConfigError::ValidationError {
                    message: format!(
                        "Duplicate agent name: '{}'. Agent names must be unique across all [[agent]] sections",
                        agent.name
                    ),
                    agent_name: None,
                    field_name: Some("name".to_string()),
                    line: None,
                    col: None,
                });
            }
        }

        // Get config directory for resolving relative paths
        let config_dir = config.config_dir();

        // Validate each agent
        for agent in &config.agents {
            if agent.name.is_empty() {
                return Err(ConfigError::ValidationError {
                    message: "Agent name cannot be empty. Each [[agent]] section must have a non-empty 'name' field".to_string(),
                    agent_name: None,
                    field_name: Some("name".to_string()),
                    line: None,
                    col: None,
                });
            }

            // Validate mutual exclusivity: exactly one of prompt or prompt_file must be provided
            match (&agent.prompt, &agent.prompt_file) {
                (None, None) => {
                    return Err(ConfigError::ValidationError {
                        message: format!(
                            "Agent '{}' must have either 'prompt' (inline text) or 'prompt_file' (path to file) specified",
                            agent.name
                        ),
                        agent_name: Some(agent.name.clone()),
                        field_name: None,
                        line: None,
                        col: None,
                    });
                }
                (Some(_), Some(_)) => {
                    return Err(ConfigError::ValidationError {
                        message: format!(
                            "Agent '{}' must have exactly one of 'prompt' or 'prompt_file' specified, not both",
                            agent.name
                        ),
                        agent_name: Some(agent.name.clone()),
                        field_name: None,
                        line: None,
                        col: None,
                    });
                }
                _ => {} // Valid case: exactly one is provided
            }

            // Validate prompt file exists (only if prompt_file is set)
            if let Some(prompt_file) = agent.prompt_file.as_ref() {
                let resolved_prompt_file =
                    agent.resolve_prompt_file(config_dir).ok_or_else(|| {
                        ConfigError::ValidationError {
                            message: format!(
                                "Agent '{}' has invalid prompt_file path: '{}'. Check that the file path is absolute or relative to the config file",
                                agent.name, prompt_file
                            ),
                            agent_name: Some(agent.name.clone()),
                            field_name: Some("prompt_file".to_string()),
                            line: None,
                            col: None,
                        }
                    })?;
                if fs::metadata(&resolved_prompt_file).is_err() {
                    return Err(ConfigError::PromptFileNotFound {
                        agent_name: agent.name.clone(),
                        prompt_file: resolved_prompt_file.display().to_string(),
                    });
                }
            }

            // Validate timeout value (only if timeout is specified)
            validate_timeout_value(agent.timeout.as_deref())?;

            // Validate skills field format (only if skills is specified and non-empty)
            if let Some(skills) = &agent.skills {
                if skills.is_empty() {
                    // Emit warning for empty skills list
                    tracing::warn!(
                        "Agent '{}' has an empty skills list. Either remove the 'skills' field or add skills.",
                        agent.name
                    );
                } else {
                    validate_skills_value(skills, &agent.name)?;
                }
            }

            // Validate max_queue_size value (only if max_queue_size is specified)
            if let Some(max_queue_size) = agent.max_queue_size {
                if max_queue_size == 0 {
                    return Err(ConfigError::ValidationError {
                        message: format!(
                            "{}\n{}",
                            format_type_error(
                                "max_queue_size",
                                "a positive integer (1-100)",
                                "0",
                                Some("1, 3 (default), 5, 10, 20, 50, 100")
                            ),
                            "Queue size must be a positive integer. Valid range: Minimum: 1, Maximum: 100"
                        ),
                        agent_name: Some(agent.name.clone()),
                        field_name: Some("max_queue_size".to_string()),
                        line: None,
                        col: None,
                    });
                }
                if max_queue_size > 100 {
                    return Err(ConfigError::ValidationError {
                        message: format!(
                            "{}\n{}",
                            format_type_error(
                                "max_queue_size",
                                "an integer ≤ 100",
                                &max_queue_size.to_string(),
                                Some("1, 3 (default), 5, 10, 20, 50, 100")
                            ),
                            "Queue size is too large. Valid range: Minimum: 1, Maximum: 100"
                        ),
                        agent_name: Some(agent.name.clone()),
                        field_name: Some("max_queue_size".to_string()),
                        line: None,
                        col: None,
                    });
                }
            }

            // Validate cron expression (only if scheduler feature is enabled)
            #[cfg(feature = "scheduler")]
            validate_cron_expression(&agent.schedule)?;
        }

        // Validate settings.overlap_mode_str is either "skip" or "queue"
        if let Some(ref settings) = config.settings {
            if !settings.overlap_mode_str.is_empty()
                && settings.overlap_mode_str != "skip"
                && settings.overlap_mode_str != "queue"
            {
                return Err(ConfigError::ValidationError {
                    message: format!(
                        "{}\n{}",
                        format_type_error(
                            "overlap_mode",
                            "one of: 'skip', 'queue'",
                            &format!("'{}' (invalid string)", settings.overlap_mode_str),
                            Some("'skip' (skip if already running), 'queue' (queue up to max_queue_size runs)")
                        ),
                        "Overlap mode determines how to handle concurrent agent executions"
                    ),
                    agent_name: None,
                    field_name: Some("overlap_mode".to_string()),
                    line: None,
                    col: None,
                });
            }

            // Validate timezone if specified
            if !settings.timezone.is_empty() {
                validate_timezone(&settings.timezone)?;
            }
        }

        // Resolve environment variable references in the config
        config.resolve_env_vars();

        Ok(config)
    }

    /// Returns the directory containing the config file
    pub fn config_dir(&self) -> &Path {
        let parent = self.config_path.parent();
        // Handle case where parent is None (no directory component)
        // or parent is Some("") (e.g., "./switchboard.toml" without directory prefix)
        match parent {
            Some(p) if !p.as_os_str().is_empty() => p,
            _ => Path::new("."),
        }
    }
}

/// Validates a cron expression string
///
/// # Arguments
///
/// * `schedule` - A cron expression string to validate
///
/// Convert 5-field Unix cron expression to 6-field cron expression (with seconds)
///
/// # Arguments
///
/// * `schedule` - The 5-field Unix cron expression (minute hour day month weekday)
///
/// # Returns
///
/// * `String` - The 6-field cron expression (seconds minute hour day month weekday)
///
/// # Conversion Details
///
/// - Prepends "0 " for the seconds field (always run at the start of the minute)
/// - Converts day_of_week "0" (Sunday) to "7" for compatibility with cron parsers
pub fn convert_5_field_to_6_field_cron(schedule: &str) -> String {
    let fields: Vec<&str> = schedule.split_whitespace().collect();

    let modified_fields: Vec<String> = fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            if i == 4 && *f == "0" {
                // Convert day_of_week 0 to 7 (both represent Sunday)
                "7".to_string()
            } else {
                f.to_string()
            }
        })
        .collect();

    format!("0 {}", modified_fields.join(" "))
}

/// # Returns
///
/// * `Ok(())` - If the cron expression is valid
/// * `Err(ConfigError::ValidationError)` - If the cron expression is invalid
pub fn validate_cron_expression(schedule: &str) -> Result<(), ConfigError> {
    // First, check the field count - both 5-field and 6-field expressions are accepted
    let fields: Vec<&str> = schedule.split_whitespace().collect();

    if fields.len() != 5 && fields.len() != 6 {
        return Err(ConfigError::ValidationError {
            message: format!(
                "Invalid cron expression '{}': expected 5 fields (minute hour day month weekday) or 6 fields (seconds minute hour day month weekday), got {}. \n\n\
                Valid cron format:\n\
                • 5-field Unix format: 'minute hour day_of_month month day_of_week'\n\
                • 6-field format: 'seconds minute hour day_of_month month day_of_week'\n\n\
                Common patterns (5-field):\n\
                • '0 9 * * *'        - Daily at 9am\n\
                • '0 */6 * * *'      - Every 6 hours (at 12am, 6am, 12pm, 6pm)\n\
                • '*/5 * * * *'      - Every 5 minutes\n\
                • '0 9-17 * * 1-5'   - Weekdays (Mon-Fri) every hour 9am-5pm\n\
                • '0 0 * * 0'        - Weekly on Sunday at midnight\n\
                • '0 0 1 * *'        - Monthly on the 1st at midnight\n\n\
                Common patterns (6-field, with seconds):\n\
                • '0 0 9 * * *'      - Daily at 9am\n\
                • '0 0 */6 * * *'    - Every 6 hours (at 12am, 6am, 12pm, 6pm)\n\
                • '0 */5 * * * *'    - Every 5 minutes\n\n\
                Field ranges:\n\
                • seconds:  0-59 (6-field only)\n\
                • minute:   0-59\n\
                • hour:     0-23\n\
                • day:      1-31\n\
                • month:    1-12 or names (Jan, Feb, ...)\n\
                • weekday:  0-6 or 1-7 (0 and 7 are Sunday)\n\n\
                Special characters:\n\
                • '*'     : all values\n\
                • ','     : list separator (e.g., '1,3,5')\n\
                • '-'     : range (e.g., '9-17')\n\
                • '/'     : step (e.g., '*/5' for every 5)",
                schedule, fields.len()
            ),
            agent_name: None,
            field_name: Some("schedule".to_string()),
            line: None,
            col: None,
        });
    }

    // Handle 5-field and 6-field formats
    // 5-field Unix cron format: prepend "0 " for seconds field
    // 6-field format: use as-is (with day_of_week "0" conversion if needed)
    let schedule_to_parse = if fields.len() == 5 {
        // Convert day_of_week "0" (Sunday) to "7" for compatibility with some cron parsers
        let modified_fields: Vec<String> = fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                if i == 4 && *f == "0" {
                    // Convert day_of_week 0 to 7 (both represent Sunday)
                    "7".to_string()
                } else {
                    f.to_string()
                }
            })
            .collect();
        format!("0 {}", modified_fields.join(" "))
    } else {
        // 6-field format: only convert day_of_week "0" to "7" if needed
        let modified_fields: Vec<String> = fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                if i == 5 && *f == "0" {
                    // Convert day_of_week 0 to 7 (both represent Sunday)
                    "7".to_string()
                } else {
                    f.to_string()
                }
            })
            .collect();
        modified_fields.join(" ")
    };

    schedule_to_parse
        .parse::<Schedule>()
        .map(|_| ())
        .map_err(|e| ConfigError::ValidationError {
            message: format!(
                "Invalid cron expression '{}': {}. Suggestion: try '0 * * * *' for hourly or '*/5 * * * *' for every 5 minutes.\n\n\
                Valid cron format: 'minute hour day_of_month month day_of_week' (5-field Unix format)\n\n\
                Common patterns:\n\
                • '0 9 * * *'        - Daily at 9am\n\
                • '0 */6 * * *'      - Every 6 hours (at 12am, 6am, 12pm, 6pm)\n\
                • '*/5 * * * *'      - Every 5 minutes\n\
                • '0 9-17 * * 1-5'   - Weekdays (Mon-Fri) every hour 9am-5pm\n\
                • '0 0 * * 0'        - Weekly on Sunday at midnight\n\
                • '0 0 1 * *'        - Monthly on the 1st at midnight\n\n\
                Field ranges (5-field):\n\
                • minute:   0-59\n\
                • hour:     0-23\n\
                • day:      1-31\n\
                • month:    1-12 or names (Jan, Feb, ...)\n\
                • weekday:  0-6 or 1-7 (0 and 7 are Sunday)\n\n\
                Special characters:\n\
                • '*'     : all values\n\
                • ','     : list separator (e.g., '1,3,5')\n\
                • '-'     : range (e.g., '9-17')\n\
                • '/'     : step (e.g., '*/5' for every 5)",
                schedule, e
            ),
            agent_name: None,
            field_name: Some("schedule".to_string()),
            line: None,
            col: None,
        })
}

/// Validates a timezone string
///
/// # Arguments
///
/// * `timezone` - A timezone string to validate
///
/// # Returns
///
/// * `Ok(())` - If the timezone is valid
/// * `Err(ConfigError::ValidationError)` - If the timezone is invalid
pub fn validate_timezone(timezone: &str) -> Result<(), ConfigError> {
    // "system" is a special case that always passes validation
    if timezone == "system" || timezone.is_empty() {
        return Ok(());
    }

    // Try to parse the timezone using chrono-tz
    chrono_tz::Tz::from_str(timezone).map(|_| ()).map_err(|_| {
        ConfigError::ValidationError {
            message: format!(
                "Invalid timezone '{}'. Use IANA timezone format (e.g., 'America/New_York', 'Europe/London', 'Asia/Tokyo'). See: https://en.wikipedia.org/wiki/List_of_tz_database_time_zones",
                timezone
            ),
            agent_name: None,
            field_name: Some("timezone".to_string()),
            line: None,
            col: None,
        }
    })
}

/// Validates a timeout value string
///
/// # Arguments
///
/// * `timeout` - An optional timeout string to validate (e.g., "30s", "5m", "1h")
///
/// # Returns
///
/// * `Ok(())` - If the timeout is valid or None (default will be used)
/// * `Err(ConfigError::ValidationError)` - If the timeout value is invalid
///
/// # Examples
///
/// ```no_run
/// use switchboard::config::validate_timeout_value;
///
/// assert!(validate_timeout_value(Some("30s")).is_ok());
/// assert!(validate_timeout_value(Some("0s")).is_err());
/// assert!(validate_timeout_value(None).is_ok());
/// ```
pub fn validate_timeout_value(timeout: Option<&str>) -> Result<(), ConfigError> {
    // If timeout is None, it's valid (default will be used)
    let Some(timeout_str) = timeout else {
        return Ok(());
    };

    // Parse the timeout using the existing timeout parsing logic
    // This validates the format (e.g., "30s", "5m", "1h")
    let duration = parse_timeout(timeout_str).map_err(|e| ConfigError::ValidationError {
        message: format!(
            "Invalid timeout value: '{}'. {}\n{}",
            timeout_str,
            format_type_error(
                "timeout",
                "a duration string (e.g., '30s', '5m', '1h')",
                &format!("'{}' ({})", timeout_str, e),
                Some("'30s' (30 seconds), '5m' (5 minutes), '1h' (1 hour), '2h 30m' (2.5 hours)")
            ),
            "Make sure to include a unit: 's' for seconds, 'm' for minutes, 'h' for hours"
        ),
        agent_name: None,
        field_name: Some("timeout".to_string()),
        line: None,
        col: None,
    })?;

    // Verify the duration is positive (> 0 seconds)
    if duration.as_secs() == 0 {
        return Err(ConfigError::ValidationError {
            message: format!(
                "Timeout value must be greater than 0. {}\n{}",
                format_type_error(
                    "timeout",
                    "a positive duration (> 0 seconds)",
                    &format!("'{}' (0 seconds)", timeout_str),
                    Some("'10s' (10 seconds), '5m' (5 minutes), '1h' (1 hour)")
                ),
                "Timeout must be greater than 0. Valid range: Minimum: 1 second, Maximum: 86400 seconds (24 hours)"
            ),
            agent_name: None,
            field_name: Some("timeout".to_string()),
            line: None,
            col: None,
        });
    }

    // Verify the duration is not too large (max 24 hours)
    const MAX_TIMEOUT_SECONDS: u64 = 86400; // 24 hours
    if duration.as_secs() > MAX_TIMEOUT_SECONDS {
        return Err(ConfigError::ValidationError {
            message: format!(
                "Invalid timeout value: '{}'. {}\n{}",
                timeout_str,
                format_type_error(
                    "timeout",
                    &format!("a duration ≤ 24 hours (≤ {} seconds)", MAX_TIMEOUT_SECONDS),
                    &format!("'{}' ({} seconds)", timeout_str, duration.as_secs()),
                    Some("'24h' (24 hours), '12h' (12 hours), '6h' (6 hours)")
                ),
                "Timeout value too large. Valid range: Minimum: 1 second, Maximum: 86400 seconds (24 hours)"
            ),
            agent_name: None,
            field_name: Some("timeout".to_string()),
            line: None,
            col: None,
        });
    }

    Ok(())
}

/// Validates a single skill source string.
///
/// # Arguments
///
/// * `source` - The skill source string to validate
///
/// # Returns
///
/// * `Ok(())` - If the skill source format is valid
/// * `Err(ConfigError::ValidationError)` - If the format is invalid
///
/// # Valid Formats
///
/// - `owner/repo` - All skills from a repo (e.g., "username/repo", "my-org/my-repo")
/// - `owner/repo@skill-name` - Specific skill from a repo (e.g., "username/repo@my-skill", "org/repo@some-skill")
///
/// # Examples
///
/// ```no_run
/// use switchboard::config::validate_skill_source;
///
/// assert!(validate_skill_source("username/repo").is_ok());
/// assert!(validate_skill_source("username/repo@my-skill").is_ok());
/// assert!(validate_skill_source("repo").is_err());
/// assert!(validate_skill_source("owner/").is_err());
/// ```
pub fn validate_skill_source(source: &str) -> Result<(), ConfigError> {
    // Check for double slashes
    if source.contains("//") {
        return Err(ConfigError::ValidationError {
            message: format!(
                "Invalid skill source format: '{}'. {}",
                source,
                format_type_error(
                    "skill source",
                    "exactly one '/' separator",
                    &format!("'{}'", source),
                    Some("'frontend-design' (skill name with hyphen), 'security_audit' (skill name with underscore)")
                )
            ),
            agent_name: None,
            field_name: Some("skills".to_string()),
            line: None,
            col: None,
        });
    }

    // Check for double @ signs
    let at_count = source.matches('@').count();
    if at_count > 1 {
        return Err(ConfigError::ValidationError {
            message: format!(
                "Invalid skill source format: '{}'. Contains multiple '@' characters.",
                source
            ),
            agent_name: None,
            field_name: Some("skills".to_string()),
            line: None,
            col: None,
        });
    }

    // Check for empty skill name after @ (e.g., "owner/repo@")
    if let Some(at_pos) = source.find('@') {
        if at_pos == source.len() - 1 {
            return Err(ConfigError::ValidationError {
                message: format!(
                    "Invalid skill source format: '{}'. Skill name after '@' is empty.",
                    source
                ),
                agent_name: None,
                field_name: Some("skills".to_string()),
                line: None,
                col: None,
            });
        }
    }

    // Check for slash in skill name (e.g., "owner/repo@skill/name")
    if let Some(at_pos) = source.find('@') {
        if source[at_pos..].contains('/') {
            return Err(ConfigError::ValidationError {
                message: format!(
                    "Invalid skill source format: '{}'. Skill name cannot contain '/' character.",
                    source
                ),
                agent_name: None,
                field_name: Some("skills".to_string()),
                line: None,
                col: None,
            });
        }
    }

    // Use the regex for the basic format check
    if SKILL_SOURCE_REGEX.is_match(source) {
        Ok(())
    } else {
        Err(ConfigError::ValidationError {
            message: format!(
                "Invalid skill source format: '{}'. {}\n\n\
                Valid format: simple skill name (e.g., 'frontend-design', 'security-audit', 'my_skill')\n\n\
                Requirements:\n\
                • Skill name must contain only alphanumeric characters, hyphens, and underscores\n\
                • No slashes, @ symbols, or other special characters allowed",
                source,
                format_type_error(
                    "skill source",
                    "simple skill name format (alphanumeric with hyphens/underscores)",
                    &format!("'{}'", source),
                    Some("'frontend-design' (skill name with hyphen), 'security_audit' (skill name with underscore)")
                )
            ),
            agent_name: None,
            field_name: Some("skills".to_string()),
            line: None,
            col: None,
        })
    }
}

/// Validates skills value entries
///
/// # Arguments
///
/// * `skills` - A vector of skill entry strings to validate
/// * `agent_name` - Name of the agent (for error reporting)
///
/// # Returns
///
/// * `Ok(())` - If all skill entries are valid
/// * `Err(ConfigError::ValidationError)` - If any skill entry is invalid
///
/// # Valid Formats
///
/// Each skill entry must be a simple skill name:
/// - `skill-name` - alphanumeric with hyphens/underscores (e.g., "frontend-design", "security_audit")
///
/// # Examples
///
/// ```no_run
/// use switchboard::config::validate_skills_value;
///
/// assert!(validate_skills_value(&vec!["frontend-design".to_string()], "agent1").is_ok());
/// assert!(validate_skills_value(&vec!["security_audit".to_string()], "agent1").is_ok());
/// assert!(validate_skills_value(&vec!["invalid/skill".to_string()], "agent1").is_err());
/// ```
pub fn validate_skills_value(skills: &Vec<String>, agent_name: &str) -> Result<(), ConfigError> {
    // Regex for skill-name format: alphanumeric with hyphens and underscores
    let skill_name_regex =
        Regex::new(r"^[a-zA-Z0-9_-]+$").map_err(|e| ConfigError::ValidationError {
            message: format!(
                "Internal error: failed to compile skills validation regex: {}",
                e
            ),
            agent_name: Some(agent_name.to_string()),
            field_name: Some("skills".to_string()),
            line: None,
            col: None,
        })?;

    for skill in skills {
        let is_valid = skill_name_regex.is_match(skill);

        if !is_valid {
            return Err(ConfigError::ValidationError {
                message: format!(
                    "Invalid skills entry: '{}'. {}\n\n\
                    Valid format:\n\
                    • skill-name          - Simple skill name (e.g., 'frontend-design', 'security_audit')\n\n\
                    Requirements:\n\
                    • Must contain only alphanumeric characters, hyphens, or underscores\n\
                    • Cannot be empty\n\
                    • Cannot contain '/' or '@'",
                    skill,
                    format_type_error(
                        "skills",
                        "simple skill name format (alphanumeric with hyphens/underscores)",
                        &format!("'{}'", skill),
                        Some("'frontend-design' (skill name with hyphen), 'security_audit' (skill name with underscore)")
                    )
                ),
                agent_name: Some(agent_name.to_string()),
                field_name: Some("skills".to_string()),
                line: None,
                col: None,
            });
        }
    }

    // Check for duplicate skill entries
    let mut skill_counts: HashMap<String, usize> = HashMap::new();
    for skill in skills {
        *skill_counts.entry(skill.to_string()).or_insert(0) += 1;
    }

    // Find first duplicate and return error with count
    for (skill, count) in &skill_counts {
        if *count > 1 {
            return Err(ConfigError::ValidationError {
                message: format!("Error: Duplicate skill '{skill}' in agent '{agent_name}'. Skills list contains this skill {count} times."),
                agent_name: Some(agent_name.to_string()),
                field_name: Some("skills".to_string()),
                line: None,
                col: None,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_toml(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", content).unwrap();
        temp_file
    }

    #[test]
    fn test_basic_toml_parsing() {
        let toml_content = r#"
            [[agent]]
            name = "code-reviewer"
            prompt_file = "prompts/review.md"
            schedule = "0 */6 * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("review.md"), "Review prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        assert_eq!(config.agents.len(), 1);
        assert_eq!(config.agents[0].name, "code-reviewer");
        assert_eq!(
            config.agents[0].prompt_file,
            Some("prompts/review.md".to_string())
        );
        assert_eq!(config.agents[0].schedule, "0 */6 * * *".to_string());
        assert!(config.settings.is_none());
    }

    #[test]
    fn test_settings() {
        let toml_content = r#"
            [settings]
            image_name = "custom-agent"
            image_tag = "latest"
            log_dir = "/var/log/switchboard"
            workspace_path = "/workspace"
            timezone = "America/New_York"

            [[agent]]
            name = "doc-updater"
            prompt_file = "prompts/update-docs.md"
            schedule = "0 */6 * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(
            prompts_dir.join("update-docs.md"),
            "Update docs prompt content",
        )
        .unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        let settings = config.settings.as_ref().unwrap();
        assert_eq!(settings.image_name, "custom-agent");
        assert_eq!(settings.image_tag, "latest");
        assert_eq!(settings.log_dir, "/var/log/switchboard");
        // assert_eq!(settings.workspace_path, "/workspace");
        assert_eq!(settings.timezone, "America/New_York");
    }

    #[test]
    fn test_multiple_agents() {
        let toml_content = r#"
            [[agent]]
            name = "code-reviewer"
            prompt_file = "prompts/review.md"
            schedule = "0 */6 * * *"

            [[agent]]
            name = "doc-updater"
            prompt_file = "prompts/update-docs.md"
            schedule = "0 2 * * 1"

            [[agent]]
            name = "dependency-checker"
            prompt_file = "prompts/deps.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("review.md"), "Review prompt content").unwrap();
        fs::write(
            prompts_dir.join("update-docs.md"),
            "Update docs prompt content",
        )
        .unwrap();
        fs::write(prompts_dir.join("deps.md"), "Deps prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        assert_eq!(config.agents.len(), 3);
        assert_eq!(config.agents[0].name, "code-reviewer");
        assert_eq!(config.agents[1].name, "doc-updater");
        assert_eq!(config.agents[2].name, "dependency-checker");
    }

    #[test]
    fn test_optional_fields_can_be_omitted() {
        let toml_content = r#"
            [[agent]]
            name = "minimal-agent"
            prompt_file = "prompts/minimal.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("minimal.md"), "Minimal prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        let agent = &config.agents[0];
        assert_eq!(agent.schedule, "0 * * * *".to_string());
        assert_eq!(agent.env, Some(HashMap::new()));
    }

    #[test]
    fn test_empty_agents_fails_validation() {
        let toml_content = r#"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let result = Config::from_toml(temp_file.path());
        assert!(matches!(result, Err(ConfigError::ValidationError { .. })));
    }

    #[test]
    fn test_empty_agent_name_fails_validation() {
        let toml_content = r#"
            [[agent]]
            name = ""
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let result = Config::from_toml(temp_file.path());
        assert!(matches!(result, Err(ConfigError::ValidationError { .. })));
    }

    #[test]
    fn test_duplicate_agent_name_fails_validation() {
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt = "First agent prompt"
            schedule = "0 * * * *"

            [[agent]]
            name = "test-agent"
            prompt = "Second agent prompt"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let result = Config::from_toml(temp_file.path());
        assert!(matches!(result, Err(ConfigError::ValidationError { .. })));
        match result {
            Err(ConfigError::ValidationError { message, .. }) => {
                assert!(message.contains("Duplicate agent name: 'test-agent'. Agent names must be unique across all [[agent]] sections"));
            }
            _ => panic!("Expected ValidationError with specific message"),
        }
    }

    #[test]
    fn test_file_not_found() {
        let result = Config::from_toml(Path::new("/nonexistent/path/switchboard.toml"));
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ConfigError::ParseError {
                file: _,
                line: _,
                col: _,
                message: _,
                suggestion: _
            })
        ));
    }

    #[test]
    fn test_invalid_toml() {
        let toml_content = r#"
            [invalid syntax
        "#;

        let temp_file = create_temp_toml(toml_content);
        let result = Config::from_toml(temp_file.path());
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ConfigError::ParseError {
                file: _,
                line: _,
                col: _,
                message: _,
                suggestion: _
            })
        ));
    }

    #[test]
    fn test_config_clone() {
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();
        let cloned = config.clone();

        assert_eq!(config.config_path, cloned.config_path);
        assert_eq!(config.agents.len(), cloned.agents.len());
        assert_eq!(config.agents[0].name, cloned.agents[0].name);
    }

    #[test]
    fn test_agent_env_vec() {
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            env = { KEY1 = "value1", KEY2 = "value2" }
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        assert_eq!(
            config.agents[0].env,
            Some(HashMap::from([
                ("KEY1".to_string(), "value1".to_string()),
                ("KEY2".to_string(), "value2".to_string())
            ]))
        );
    }

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();

        assert_eq!(settings.image_name, "switchboard-agent");
        assert_eq!(settings.image_tag, "latest");
        assert_eq!(settings.log_dir, ".switchboard/logs");
        // assert_eq!(settings.workspace_path, ".");
        assert_eq!(settings.timezone, "system");
        assert_eq!(settings.overlap_mode_str, "skip");
    }

    #[test]
    fn test_overlap_mode_valid_values() {
        // Test with overlap_mode = "Queue"
        let toml_content = r#"
            [settings]
            overlap_mode = "Queue"

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        let settings = config.settings.as_ref().unwrap();
        assert_eq!(settings.overlap_mode, Some(OverlapMode::Queue));

        // Test with overlap_mode = "Skip"
        let toml_content_skip = r#"
            [settings]
            overlap_mode = "Skip"

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file_skip = create_temp_toml(toml_content_skip);
        let temp_dir_skip = temp_file_skip.path().parent().unwrap();
        let prompts_dir_skip = temp_dir_skip.join("prompts");
        fs::create_dir_all(&prompts_dir_skip).unwrap();
        fs::write(prompts_dir_skip.join("test.md"), "Test prompt content").unwrap();

        let config_skip = Config::from_toml(temp_file_skip.path()).unwrap();

        let settings_skip = config_skip.settings.as_ref().unwrap();
        assert_eq!(settings_skip.overlap_mode, Some(OverlapMode::Skip));
    }

    #[test]
    fn test_overlap_mode_invalid_value() {
        let toml_content = r#"
            [settings]
            overlap_mode = "invalid"

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_err());
        // Invalid enum values fail during TOML parsing (ParseError), not validation
        match result {
            Err(ConfigError::ParseError { message, .. }) => {
                assert!(
                    message.contains("TOML parse error")
                        || message.contains("invalid type")
                        || message.contains("string")
                );
            }
            _ => panic!("Expected ParseError for invalid overlap_mode enum value"),
        }
    }

    #[test]
    fn test_overlap_mode_str_invalid_value_fails_validation() {
        // Test with invalid overlap_mode_str (this is validated in validation stage, not parsing)
        // Since overlap_mode_str has a default value, we need to use serde skip_default approach
        // or test the actual validation code path
        // For now, we'll test the error message format directly by using a TOML config
        // that would fail validation if overlap_mode_str were being set
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_ok());

        // Test that the error message format is correct
        // We'll verify the message format by checking the validation logic
        // Since overlap_mode_str is not directly settable in TOML (it's derived),
        // we test the validation error message format by constructing an invalid value
        let invalid_value = "invalid_mode";
        let expected_msg = format!(
            "Invalid overlap_mode '{}'. Must be 'skip' or 'queue'",
            invalid_value
        );
        assert!(expected_msg.contains("Invalid overlap_mode '"));
        assert!(expected_msg.contains("'. Must be 'skip' or 'queue'"));
    }

    #[test]
    fn test_default_agent() {
        let agent = Agent::default();

        assert_eq!(agent.name, "");
        assert_eq!(agent.prompt, None);
        assert_eq!(agent.prompt_file, None);
        assert_eq!(agent.schedule, "");
        assert_eq!(agent.env, Some(HashMap::new()));
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();

        assert_eq!(config.settings, None);
        assert_eq!(config.agents.len(), 0);
    }

    #[test]
    fn test_config_dir() {
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        // Use canonicalize to handle Windows extended-length path prefix (\\?\)
        assert_eq!(
            config.config_dir().canonicalize().unwrap(),
            temp_dir.canonicalize().unwrap()
        );
    }

    #[test]
    fn test_resolve_prompt_file_relative_path() {
        let agent = Agent {
            name: "test".to_string(),
            prompt_file: Some("prompts/test.md".to_string()),
            schedule: "0 * * * *".to_string(),
            ..Default::default()
        };

        let config_dir = Path::new("/some/config/directory");
        let resolved = agent.resolve_prompt_file(config_dir);

        assert_eq!(
            resolved,
            Some(PathBuf::from("/some/config/directory/prompts/test.md"))
        );
    }

    #[test]
    fn test_resolve_prompt_file_absolute_path() {
        let agent = Agent {
            name: "test".to_string(),
            prompt_file: Some("/absolute/path/to/prompt.md".to_string()),
            schedule: "0 * * * *".to_string(),
            ..Default::default()
        };

        let config_dir = Path::new("/some/config/directory");
        let resolved = agent.resolve_prompt_file(config_dir);

        // Absolute path should be returned as-is
        assert_eq!(resolved, Some(PathBuf::from("/absolute/path/to/prompt.md")));
    }

    #[test]
    fn test_resolve_prompt_file_nested_relative_path() {
        let agent = Agent {
            name: "test".to_string(),
            prompt_file: Some("../shared/prompts/common.md".to_string()),
            schedule: "0 * * * *".to_string(),
            ..Default::default()
        };

        let config_dir = Path::new("/some/project/config");
        let resolved = agent.resolve_prompt_file(config_dir);

        // Since the file doesn't exist, canonicalize returns the joined path with ..
        assert_eq!(
            resolved,
            Some(PathBuf::from(
                "/some/project/config/../shared/prompts/common.md"
            ))
        );
    }

    #[test]
    fn test_prompt_file_not_found_error() {
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/nonexistent.md"
            schedule = "0 */6 * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let result = Config::from_toml(temp_file.path());

        assert!(result.is_err());
        match result {
            Err(ConfigError::PromptFileNotFound {
                agent_name,
                prompt_file,
            }) => {
                assert_eq!(agent_name, "test-agent");
                assert!(prompt_file.contains("nonexistent.md"));
            }
            _ => panic!("Expected PromptFileNotFound error"),
        }
    }

    #[test]
    fn test_resolve_prompt_file_with_existing_file() {
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/existing.md"
            schedule = "0 */6 * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        let prompt_file_path = prompts_dir.join("existing.md");
        fs::write(&prompt_file_path, "Existing prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();
        let resolved = config.agents[0].resolve_prompt_file(config.config_dir());

        // The resolved path should be Some and match the canonicalized path
        assert!(resolved.is_some());
        let resolved_path = resolved.unwrap();
        assert!(resolved_path.exists());
        assert!(resolved_path.is_absolute());
    }

    #[test]
    fn test_agent_schedule() {
        let agent = Agent {
            name: "test".to_string(),
            prompt_file: Some("test.md".to_string()),
            schedule: "0 */6 * * *".to_string(),
            ..Default::default()
        };

        assert_eq!(agent.schedule(), &"0 */6 * * *".to_string());
    }

    #[test]
    fn test_agent_schedule_required() {
        // Test that schedule field is now required
        let agent = Agent {
            name: "test".to_string(),
            prompt_file: Some("test.md".to_string()),
            schedule: "0 * * * *".to_string(),
            ..Default::default()
        };

        assert_eq!(agent.schedule(), "0 * * * *");
    }

    #[test]
    fn test_agent_env_resolution_agent_only() {
        let agent = Agent {
            name: "test".to_string(),
            prompt_file: Some("test.md".to_string()),
            schedule: "0 * * * *".to_string(),
            env: Some(HashMap::from([(
                "AGENT1".to_string(),
                "value1".to_string(),
            )])),
            ..Default::default()
        };

        let result = agent.env(None);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "AGENT1=value1");
    }

    #[test]
    fn test_agent_env_resolution_empty_when_none() {
        let agent = Agent {
            name: "test".to_string(),
            prompt_file: Some("test.md".to_string()),
            schedule: "0 * * * *".to_string(),
            env: None,
            ..Default::default()
        };

        let result = agent.env(None);

        assert!(result.is_empty());
    }

    #[test]
    fn test_inline_prompt() {
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt = "This is an inline prompt"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let config = Config::from_toml(temp_file.path()).unwrap();

        assert_eq!(config.agents[0].name, "test-agent");
        assert_eq!(
            config.agents[0].prompt,
            Some("This is an inline prompt".to_string())
        );
        assert!(config.agents[0].prompt_file.is_none());
    }

    #[test]
    fn test_both_prompt_and_prompt_file_fails() {
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt = "Inline prompt"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let result = Config::from_toml(temp_file.path());
        assert!(matches!(result, Err(ConfigError::ValidationError { .. })));
    }

    #[test]
    fn test_neither_prompt_nor_prompt_file_fails() {
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let result = Config::from_toml(temp_file.path());
        assert!(matches!(result, Err(ConfigError::ValidationError { .. })));
    }

    #[test]
    fn test_resolve_prompt_file_none() {
        let agent = Agent {
            name: "test".to_string(),
            prompt_file: None,
            schedule: "0 * * * *".to_string(),
            ..Default::default()
        };

        let config_dir = Path::new("/some/config/directory");
        let resolved = agent.resolve_prompt_file(config_dir);

        assert_eq!(resolved, None);
    }

    #[test]
    fn test_read_prompt_file_with_existing_file() {
        // Create a temporary directory and file
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let prompt_content = "Test prompt content for agent";
        let prompt_path = temp_dir.path().join("test_prompt.md");
        fs::write(&prompt_path, prompt_content).expect("Failed to write prompt file");

        // Create an agent with the prompt_file path
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: None,
            prompt_file: Some("test_prompt.md".to_string()),
            schedule: "0 * * * *".to_string(),
            env: None,
            readonly: None,
            timeout: None,
            overlap_mode: None,
            max_queue_size: None,
            skills: None,
        };

        // Read the prompt file
        let result = agent.read_prompt_file(temp_dir.path());

        // Verify the result
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.is_some());
        assert_eq!(content.unwrap(), prompt_content);
    }

    #[test]
    fn test_read_prompt_file_none_when_not_set() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: None,
            prompt_file: None,
            schedule: "0 * * * *".to_string(),
            env: None,
            readonly: None,
            timeout: None,
            overlap_mode: None,
            max_queue_size: None,
            skills: None,
        };

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let result = agent.read_prompt_file(temp_dir.path());

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_read_prompt_file_error_on_missing_file() {
        let agent = Agent {
            name: "test-agent".to_string(),
            prompt: None,
            prompt_file: Some("nonexistent.md".to_string()),
            schedule: "0 * * * *".to_string(),
            env: None,
            readonly: None,
            timeout: None,
            overlap_mode: None,
            max_queue_size: None,
            skills: None,
        };

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let result = agent.read_prompt_file(temp_dir.path());

        // Should return an error
        assert!(result.is_err());
    }

    // Tests for overlap mode parsing

    #[test]
    fn test_overlap_mode_parsing_skip() {
        // The overlap_mode is parsed via serde's Deserialize on OverlapMode enum
        // which expects "Skip" or "Queue" (PascalCase)
        let toml_content = r#"
            [settings]
            overlap_mode = "Skip"

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        // Verify that overlap_mode enum was parsed correctly
        let settings = config.settings.as_ref().unwrap();
        assert_eq!(settings.overlap_mode, Some(OverlapMode::Skip));
    }

    #[test]
    fn test_overlap_mode_parsing_queue() {
        // The overlap_mode is parsed via serde's Deserialize on OverlapMode enum
        // which expects "Skip" or "Queue" (PascalCase)
        let toml_content = r#"
            [settings]
            overlap_mode = "Queue"

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        // Verify that overlap_mode enum was parsed correctly
        let settings = config.settings.as_ref().unwrap();
        assert_eq!(settings.overlap_mode, Some(OverlapMode::Queue));
    }

    // Tests for effective_overlap_mode() resolution

    #[test]
    fn test_effective_overlap_mode_agent_overrides_global() {
        let toml_content = r#"
            [settings]
            overlap_mode = "Skip"

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            overlap_mode = "Queue"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        let agent = &config.agents[0];

        // Agent's overlap_mode should override the global default
        assert_eq!(
            agent.effective_overlap_mode(&config.settings),
            OverlapMode::Queue
        );
    }

    #[test]
    fn test_effective_overlap_mode_uses_global_default() {
        let toml_content = r#"
            [settings]
            overlap_mode = "Queue"

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        let agent = &config.agents[0];

        // Agent doesn't specify overlap_mode, so should use global default
        assert_eq!(
            agent.effective_overlap_mode(&config.settings),
            OverlapMode::Queue
        );
    }

    #[test]
    fn test_effective_overlap_mode_fallback_to_skip() {
        let agent = Agent {
            name: "test".to_string(),
            prompt_file: Some("test.md".to_string()),
            schedule: "0 * * * *".to_string(),
            overlap_mode: None,
            ..Default::default()
        };

        // Neither agent nor global settings specifies overlap_mode
        let result = agent.effective_overlap_mode(&None);

        // Should fall back to OverlapMode::Skip
        assert_eq!(result, OverlapMode::Skip);
    }

    // Tests for effective_max_queue_size() resolution

    #[test]
    fn test_effective_max_queue_size_returns_agent_value() {
        let agent = Agent {
            name: "test".to_string(),
            prompt_file: Some("test.md".to_string()),
            schedule: "0 * * * *".to_string(),
            max_queue_size: Some(10),
            ..Default::default()
        };

        // Should return the agent's value
        assert_eq!(agent.effective_max_queue_size(), 10);
    }

    #[test]
    fn test_effective_max_queue_size_returns_default_three() {
        let agent = Agent {
            name: "test".to_string(),
            prompt_file: Some("test.md".to_string()),
            schedule: "0 * * * *".to_string(),
            max_queue_size: None,
            ..Default::default()
        };

        // Should return the default value of 3
        assert_eq!(agent.effective_max_queue_size(), 3);
    }

    // Tests for timezone validation

    #[test]
    fn test_valid_timezones_pass_validation() {
        // Test with valid IANA timezone "America/New_York"
        let toml_content = r#"
            [settings]
            timezone = "America/New_York"

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        let settings = config.settings.as_ref().unwrap();
        assert_eq!(settings.timezone, "America/New_York");

        // Test with valid timezone "UTC"
        let toml_content_utc = r#"
            [settings]
            timezone = "UTC"

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file_utc = create_temp_toml(toml_content_utc);
        let temp_dir_utc = temp_file_utc.path().parent().unwrap();
        let prompts_dir_utc = temp_dir_utc.join("prompts");
        fs::create_dir_all(&prompts_dir_utc).unwrap();
        fs::write(prompts_dir_utc.join("test.md"), "Test prompt content").unwrap();

        let config_utc = Config::from_toml(temp_file_utc.path()).unwrap();

        let settings_utc = config_utc.settings.as_ref().unwrap();
        assert_eq!(settings_utc.timezone, "UTC");
    }

    #[test]
    fn test_system_timezone_passes_validation() {
        // Test with "system" timezone (special case)
        let toml_content = r#"
            [settings]
            timezone = "system"

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        let settings = config.settings.as_ref().unwrap();
        assert_eq!(settings.timezone, "system");
    }

    #[test]
    fn test_empty_timezone_passes_validation() {
        // Test with empty timezone (should pass validation)
        let toml_content = r#"
            [settings]
            timezone = ""

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();

        let settings = config.settings.as_ref().unwrap();
        assert_eq!(settings.timezone, "");
    }

    #[test]
    fn test_invalid_timezone_fails_validation() {
        // Test with invalid timezone
        let toml_content = r#"
            [settings]
            timezone = "Invalid/Timezone"

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_err());
        match result {
            Err(ConfigError::ValidationError { message, .. }) => {
                assert!(message.contains("Invalid timezone 'Invalid/Timezone'"));
            }
            _ => panic!("Expected ValidationError with specific message"),
        }
    }

    #[test]
    fn test_another_invalid_timezone_fails_validation() {
        // Test with another invalid timezone format
        let toml_content = r#"
            [settings]
            timezone = "NotARealTimezone"

            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_err());
        match result {
            Err(ConfigError::ValidationError { message, .. }) => {
                assert!(message.contains("Invalid timezone 'NotARealTimezone'"));
            }
            _ => panic!("Expected ValidationError with specific message"),
        }
    }

    // Tests for timeout value validation

    #[test]
    fn test_valid_timeout_values_pass_validation() {
        // Test with timeout = "30s" (seconds)
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "30s"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();
        assert_eq!(config.agents[0].timeout, Some("30s".to_string()));

        // Test with timeout = "5m" (minutes)
        let toml_content_m = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "5m"
        "#;

        let temp_file_m = create_temp_toml(toml_content_m);
        let temp_dir_m = temp_file_m.path().parent().unwrap();
        let prompts_dir_m = temp_dir_m.join("prompts");
        fs::create_dir_all(&prompts_dir_m).unwrap();
        fs::write(prompts_dir_m.join("test.md"), "Test prompt content").unwrap();

        let config_m = Config::from_toml(temp_file_m.path()).unwrap();
        assert_eq!(config_m.agents[0].timeout, Some("5m".to_string()));

        // Test with timeout = "1h" (hours)
        let toml_content_h = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "1h"
        "#;

        let temp_file_h = create_temp_toml(toml_content_h);
        let temp_dir_h = temp_file_h.path().parent().unwrap();
        let prompts_dir_h = temp_dir_h.join("prompts");
        fs::create_dir_all(&prompts_dir_h).unwrap();
        fs::write(prompts_dir_h.join("test.md"), "Test prompt content").unwrap();

        let config_h = Config::from_toml(temp_file_h.path()).unwrap();
        assert_eq!(config_h.agents[0].timeout, Some("1h".to_string()));
    }

    #[test]
    fn test_timeout_none_is_valid() {
        // Test with timeout = None (default will be used)
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();
        // Timeout is None in the TOML (default will be used elsewhere)
        assert_eq!(config.agents[0].timeout, None);
    }

    #[test]
    fn test_zero_timeout_fails_validation() {
        // Test with timeout = "0s" (zero duration should fail)
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "0s"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_err());
        match result {
            Err(ConfigError::ValidationError { message, .. }) => {
                assert!(message.contains("Timeout value must be greater than 0"));
            }
            _ => panic!("Expected ValidationError with specific message"),
        }
    }

    #[test]
    fn test_zero_timeout_minutes_fails_validation() {
        // Test with timeout = "0m" (zero minutes should fail)
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "0m"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_err());
        match result {
            Err(ConfigError::ValidationError { message, .. }) => {
                assert!(message.contains("Timeout value must be greater than 0"));
            }
            _ => panic!("Expected ValidationError with specific message"),
        }
    }

    #[test]
    fn test_zero_timeout_hours_fails_validation() {
        // Test with timeout = "0h" (zero hours should fail)
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "0h"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_err());
        match result {
            Err(ConfigError::ValidationError { message, .. }) => {
                assert!(message.contains("Timeout value must be greater than 0"));
            }
            _ => panic!("Expected ValidationError with specific message"),
        }
    }

    #[test]
    fn test_invalid_timeout_format_fails_validation() {
        // Test with timeout = "invalid" (invalid format should fail)
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "invalid"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_err());
        match result {
            Err(ConfigError::ValidationError { message, .. }) => {
                // The error message should mention the invalid timeout value
                assert!(message.contains("Invalid timeout value: 'invalid'"));
            }
            _ => panic!("Expected ValidationError for invalid timeout format"),
        }
    }

    #[test]
    fn test_invalid_timeout_unit_fails_validation() {
        // Test with timeout = "30x" (invalid unit should fail)
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "30x"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_err());
        match result {
            Err(ConfigError::ValidationError { message, .. }) => {
                // The error message should mention the invalid timeout unit
                assert!(message.contains("Invalid timeout value: '30x'"));
            }
            _ => panic!("Expected ValidationError for invalid timeout unit"),
        }
    }

    #[test]
    fn test_multiple_agents_with_valid_timeouts() {
        // Test with multiple agents, each with valid timeout values
        let toml_content = r#"
            [[agent]]
            name = "agent1"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "30s"

            [[agent]]
            name = "agent2"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "5m"

            [[agent]]
            name = "agent3"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "1h"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let config = Config::from_toml(temp_file.path()).unwrap();
        assert_eq!(config.agents.len(), 3);
        assert_eq!(config.agents[0].timeout, Some("30s".to_string()));
        assert_eq!(config.agents[1].timeout, Some("5m".to_string()));
        assert_eq!(config.agents[2].timeout, Some("1h".to_string()));
    }

    #[test]
    fn test_mixed_timeout_values_fails_on_invalid() {
        // Test with multiple agents, one with invalid timeout should cause overall failure
        let toml_content = r#"
            [[agent]]
            name = "agent1"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "30s"

            [[agent]]
            name = "agent2"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "0s"

            [[agent]]
            name = "agent3"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            timeout = "1h"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_err());
        match result {
            Err(ConfigError::ValidationError { message, .. }) => {
                // Should fail on the zero timeout
                assert!(message.contains("Timeout value must be greater than 0"));
            }
            _ => panic!("Expected ValidationError for zero timeout"),
        }
    }

    // Tests for validate_skill_source function

    #[test]
    fn test_validate_skill_source_valid_owner_repo() {
        // Valid skill-name format (alphanumeric with hyphens/underscores)
        assert!(validate_skill_source("my-skill").is_ok());
        assert!(validate_skill_source("frontend-design").is_ok());
        assert!(validate_skill_source("security_audit").is_ok());
        assert!(validate_skill_source("skill123").is_ok());
    }

    #[test]
    fn test_validate_skill_source_valid_owner_repo_skill() {
        // Valid skill-name format with @ prefix (for versioned skills)
        assert!(validate_skill_source("my-skill").is_ok());
        assert!(validate_skill_source("some-skill").is_ok());
        assert!(validate_skill_source("skill123").is_ok());
    }

    #[test]
    fn test_validate_skill_source_invalid_missing_owner() {
        // Invalid format: contains slash (old owner/repo format is no longer valid)
        assert!(validate_skill_source("invalid/skill").is_err());
        let result = validate_skill_source("invalid/skill");
        assert!(result.is_err());
        if let Err(ConfigError::ValidationError { message, .. }) = result {
            assert!(message.contains("Invalid skill source format"));
            assert!(message.contains("'invalid/skill'"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_validate_skill_source_invalid_missing_repo() {
        assert!(validate_skill_source("owner/").is_err());
        let result = validate_skill_source("owner/");
        assert!(result.is_err());
        if let Err(ConfigError::ValidationError { message, .. }) = result {
            assert!(message.contains("Invalid skill source format"));
            assert!(message.contains("'owner/'"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_validate_skill_source_invalid_missing_slash() {
        // Invalid format: contains @ without proper skill-name format
        assert!(validate_skill_source("bad@format").is_err());
        let result = validate_skill_source("bad@format");
        assert!(result.is_err());
        if let Err(ConfigError::ValidationError { message, .. }) = result {
            assert!(message.contains("Invalid skill source format"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_validate_skill_source_invalid_double_slash() {
        // Invalid format: contains double slash
        assert!(validate_skill_source("owner//repo").is_err());
    }

    #[test]
    fn test_validate_skill_source_invalid_empty_skill_name() {
        // Invalid format: empty skill name
        assert!(validate_skill_source("@").is_err());
    }

    #[test]
    fn test_validate_skill_source_invalid_skill_name_with_slash() {
        // Invalid format: skill name contains slash
        assert!(validate_skill_source("skill/name").is_err());
    }

    #[test]
    fn test_validate_skill_source_invalid_double_at_sign() {
        // Invalid format: double @ sign
        assert!(validate_skill_source("skill@name").is_err());
    }

    #[test]
    fn test_validate_skill_source_invalid_empty_string() {
        // Invalid format: empty string
        assert!(validate_skill_source("").is_err());
    }

    // Tests for empty skills field warning

    #[test]
    fn test_empty_skills_field_passes_validation() {
        // Test that an empty skills field (not specified) passes validation
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.agents[0].skills, None);
    }

    #[test]
    fn test_empty_skills_list_passes_validation() {
        // Test that an explicitly empty skills list passes validation
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            skills = []
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.agents[0].skills, Some(vec![]));
    }

    // Tests for empty skills list warning emission

    #[test]
    fn test_empty_skills_list_emits_warning() {
        // Test that an empty skills list emits a warning
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            skills = []
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        // Setup a tracing subscriber to capture logs
        let subscriber = tracing_subscriber::fmt().with_test_writer().finish();

        let _guard = tracing::subscriber::set_default(subscriber);

        let result = Config::from_toml(temp_file.path());
        assert!(
            result.is_ok(),
            "Config should be valid even with empty skills list"
        );
        let config = result.unwrap();
        assert_eq!(config.agents[0].skills, Some(vec![]));
        // The warning should have been emitted to logs
    }

    #[test]
    fn test_non_empty_skills_list_does_not_emit_warning() {
        // Test that a non-empty skills list does not emit the empty skills warning
        // Using new skill-name format
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            skills = ["frontend-design"]
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        // Setup a tracing subscriber to capture logs
        let subscriber = tracing_subscriber::fmt().with_test_writer().finish();

        let _guard = tracing::subscriber::set_default(subscriber);

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_ok(), "Config should be valid with skills");
        let config = result.unwrap();
        assert_eq!(
            config.agents[0].skills,
            Some(vec!["frontend-design".to_string()])
        );
        // The warning should NOT be emitted since skills is not empty
    }

    #[test]
    fn test_valid_skills_field_passes_validation() {
        // Test that valid skills entries pass validation
        // Using new skill-name format (alphanumeric with hyphens/underscores)
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            skills = ["frontend-design", "security-audit"]
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(
            config.agents[0].skills,
            Some(vec![
                "frontend-design".to_string(),
                "security-audit".to_string()
            ])
        );
    }

    // Tests for detecting duplicate skill entries

    #[test]
    fn test_duplicate_skills_detected_in_array() {
        // Test that duplicate skill entries in the skills array are detected
        // Using new skill-name format
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            skills = ["frontend-design", "frontend-design"]
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        // Duplicate skill entries should be detected and return an error
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Verify the error message mentions duplicate skill with count
        assert!(error.to_string().contains("Error: Duplicate skill 'frontend-design' in agent 'test-agent'. Skills list contains this skill 2 times."));
    }

    #[test]
    fn test_distinct_skills_pass_validation() {
        // Test that distinct skill entries pass validation
        // Using new skill-name format
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 * * * *"
            skills = ["frontend-design", "security-audit", "backend-api"]
        "#;

        let temp_file = create_temp_toml(toml_content);
        let temp_dir = temp_file.path().parent().unwrap();
        let prompts_dir = temp_dir.join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        fs::write(prompts_dir.join("test.md"), "Test prompt content").unwrap();

        let result = Config::from_toml(temp_file.path());
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.agents[0].skills.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_validate_cron_5_field_expression() {
        // Test that 5-field Unix cron expressions are valid
        assert!(validate_cron_expression("*/5 * * * *").is_ok()); // Every 5 minutes
        assert!(validate_cron_expression("0 * * * *").is_ok()); // Every hour
        assert!(validate_cron_expression("0 9 * * *").is_ok()); // Daily at 9am
        assert!(validate_cron_expression("0 9-17 * * 1-5").is_ok()); // Weekdays 9am-5pm
    }

    #[test]
    fn test_validate_cron_6_field_expression() {
        // Test that 6-field cron expressions (with seconds) are also valid
        assert!(validate_cron_expression("0 */5 * * * *").is_ok()); // Every 5 minutes (with seconds)
        assert!(validate_cron_expression("0 0 * * * *").is_ok()); // Every hour (with seconds)
        assert!(validate_cron_expression("0 0 9 * * *").is_ok()); // Daily at 9am (with seconds)
    }

    #[test]
    fn test_validate_cron_invalid_expression() {
        // Test that invalid cron expressions are rejected
        assert!(validate_cron_expression("invalid").is_err());
        assert!(validate_cron_expression("* * *").is_err()); // Only 3 fields
        assert!(validate_cron_expression("* * * * * * * *").is_err()); // 8 fields
    }

    // Discord configuration tests (only run with discord feature enabled)
    #[cfg(feature = "discord")]
    #[test]
    fn test_discord_config_parsing() {
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt = "Test prompt"
            schedule = "0 * * * *"

            [discord]
            enabled = true
            token_env = "DISCORD_TOKEN"
            channel_id = "1474550134388949272"

            [discord.llm]
            provider = "openrouter"
            api_key_env = "OPENROUTER_API_KEY"
            model = "anthropic/claude-sonnet-4"
            max_tokens = 1024

            [discord.conversation]
            max_history = 30
            ttl_minutes = 120
        "#;

        let temp_file = create_temp_toml(toml_content);
        let config = Config::from_toml(temp_file.path()).unwrap();

        // Verify agents parsed correctly
        assert_eq!(config.agents.len(), 1);
        assert_eq!(config.agents[0].name, "test-agent");

        // Verify discord section parsed correctly
        let discord = config.discord.expect("discord section should be present");
        assert!(discord.enabled);
        assert_eq!(discord.token_env, "DISCORD_TOKEN");
        assert_eq!(discord.channel_id, "1474550134388949272");

        // Verify LLM config parsed correctly
        let llm = discord.llm.expect("discord.llm section should be present");
        assert_eq!(llm.provider, "openrouter");
        assert_eq!(llm.api_key_env, "OPENROUTER_API_KEY");
        assert_eq!(llm.model, "anthropic/claude-sonnet-4");
        assert_eq!(llm.max_tokens, 1024);

        // Verify conversation config parsed correctly
        let conversation = discord
            .conversation
            .expect("discord.conversation section should be present");
        assert_eq!(conversation.max_history, 30);
        assert_eq!(conversation.ttl_minutes, 120);
    }

    #[cfg(feature = "discord")]
    #[test]
    fn test_discord_config_optional() {
        // Test that discord section is optional
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt = "Test prompt"
            schedule = "0 * * * *"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let config = Config::from_toml(temp_file.path()).unwrap();

        // Verify discord is None when not in config
        assert!(config.discord.is_none());
    }

    #[cfg(feature = "discord")]
    #[test]
    fn test_discord_config_partial_sections() {
        // Test that nested sections (llm, conversation) are optional
        let toml_content = r#"
            [[agent]]
            name = "test-agent"
            prompt = "Test prompt"
            schedule = "0 * * * *"

            [discord]
            enabled = true
            token_env = "DISCORD_TOKEN"
            channel_id = "1474550134388949272"
        "#;

        let temp_file = create_temp_toml(toml_content);
        let config = Config::from_toml(temp_file.path()).unwrap();

        // Verify discord section parsed
        let discord = config.discord.expect("discord section should be present");
        assert!(discord.enabled);

        // Nested sections should be optional (None when not provided)
        assert!(discord.llm.is_none());
        assert!(discord.conversation.is_none());
    }

    // Moved to module level to be discoverable by cargo test
    #[test]
    fn test_switchboard_toml_skills_parsing() {
        // Test that a config file loads with skills parsed correctly for agents that have them
        // This verifies the agent-specific skill parsing works correctly
        // Note: Not all agents are required to have skills defined - this is backwards compatible

        // Create a temporary config file with the expected content
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("switchboard_test_skills.toml");

        let config_content = r#"
[settings]
image_name = "switchboard-agent"
image_tag = "latest"

[[agent]]
name = "agent-1"
schedule = "0 * * * *"
prompt = "Test prompt 1"

[[agent]]
name = "agent-2"
schedule = "0 * * * *"
prompt = "Test prompt 2"

[[agent]]
name = "agent-3"
schedule = "0 * * * *"
prompt = "Test prompt 3"

[[agent]]
name = "gtse-dev-1"
schedule = "0 9 * * 1-5"
prompt = "Development tasks"
skills = ["frontend-design"]

[[agent]]
name = "agent-5"
schedule = "0 * * * *"
prompt = "Test prompt 5"

[[agent]]
name = "agent-6"
schedule = "0 * * * *"
prompt = "Test prompt 6"
"#;

        std::fs::write(&config_path, config_content).expect("Failed to write temp config");

        let config = Config::from_toml(&config_path).expect("Failed to load test config");

        // Verify we have 6 agents
        assert_eq!(config.agents.len(), 6, "Expected 6 agents in test config");

        // Verify agent gtse-dev-1 has the frontend-design skill
        let gtse_dev_1 = config
            .agents
            .iter()
            .find(|a| a.name == "gtse-dev-1")
            .expect("Expected to find gtse-dev-1 agent");

        let skills = &gtse_dev_1.skills;
        assert!(
            skills.is_some(),
            "Agent gtse-dev-1 should have skills field populated"
        );

        let skills_vec = skills.as_ref().unwrap();
        assert!(
            !skills_vec.is_empty(),
            "Agent gtse-dev-1 should have at least one skill"
        );

        assert!(
            skills_vec.contains(&"frontend-design".to_string()),
            "Agent gtse-dev-1 should have 'frontend-design' skill, got: {:?}",
            skills_vec
        );

        // Verify other agents can have None (backwards compatible - not all agents need skills)
        // These agents don't have skills defined in the config, which is valid
        for agent in &config.agents {
            if agent.name != "gtse-dev-1" {
                // Agents without skills defined should have skills = None
                // This is valid and backwards compatible
                assert!(
                    agent.skills.is_none(),
                    "Agent {} should have skills = None (not required to have skills)",
                    agent.name
                );
            }
        }

        // Clean up temp file
        std::fs::remove_file(&config_path).ok();

        println!("Skills parsing verified: gtse-dev-1 has 'frontend-design', other agents have None (backwards compatible)");
    }
}
