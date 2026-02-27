//! Discord bot configuration module.
//!
//! Provides configuration structs for the Discord concierge integration,
//! including Discord connection settings, LLM provider configuration,
//! and conversation management options.

use serde::{Deserialize, Serialize};
use std::{env, fs};

/// Default system prompt for the Discord concierge bot.
pub const DEFAULT_SYSTEM_PROMPT: &str = r#"You are a helpful AI coding assistant in a Discord channel. 
You help users with their coding questions, debugging, and general programming assistance.
Be concise, helpful, and friendly in your responses."#;

/// Loads the system prompt from a file or returns the default.
///
/// If `system_prompt_file` is Some, attempts to read the file contents.
/// If the file doesn't exist or can't be read, returns an error.
/// If `system_prompt_file` is None, returns the default system prompt.
///
/// # Arguments
///
/// * `system_prompt_file` - Optional path to a custom system prompt file
///
/// # Returns
///
/// * `Ok(String)` - The system prompt content
/// * `Err(String)` - If the file cannot be read
pub fn load_system_prompt(system_prompt_file: Option<&str>) -> Result<String, String> {
    match system_prompt_file {
        Some(path) => fs::read_to_string(path)
            .map_err(|e| format!("Failed to read system prompt file '{}': {}", path, e)),
        None => Ok(DEFAULT_SYSTEM_PROMPT.to_string()),
    }
}

/// Environment variable name for Discord bot token.
pub const DISCORD_TOKEN_ENV: &str = "DISCORD_TOKEN";

/// Environment variable name for Discord channel ID.
pub const DISCORD_CHANNEL_ID_ENV: &str = "DISCORD_CHANNEL_ID";

/// Environment variable name for OpenRouter API key.
pub const OPENROUTER_API_KEY_ENV: &str = "OPENROUTER_API_KEY";

/// Configuration loaded from environment variables.
///
/// This struct holds the required credentials for the Discord bot
/// and LLM integration, loaded from environment variables at runtime.
///
/// # Example
///
/// ```ignore
/// let config = load_discord_config().expect("Failed to load config");
/// println!("Discord token: {}", config.discord_token);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscordEnvConfig {
    /// Discord bot token for authentication.
    pub discord_token: String,
    /// OpenRouter API key for LLM requests.
    pub openrouter_api_key: String,
}

/// Loads Discord configuration from environment variables.
///
/// Reads both `DISCORD_TOKEN` and `OPENROUTER_API_KEY` from the environment.
///
/// # Returns
///
/// * `Ok(DiscordEnvConfig)` - If both environment variables are set
/// * `Err(String)` - If either environment variable is missing or empty
///
/// # Example
///
/// ```ignore
/// match load_discord_config() {
///     Ok(config) => {
///         println!("Loaded config with token length: {}", config.discord_token.len());
///     }
///     Err(e) => {
///         eprintln!("Configuration error: {}", e);
///     }
/// }
/// ```
pub fn load_discord_config() -> Result<DiscordEnvConfig, String> {
    let discord_token = get_discord_token()?;
    let openrouter_api_key = get_openrouter_api_key()?;

    Ok(DiscordEnvConfig {
        discord_token,
        openrouter_api_key,
    })
}

/// Gets the Discord bot token from the environment.
///
/// # Returns
///
/// * `Ok(String)` - The Discord token value
/// * `Err(String)` - If the environment variable is not set or empty
pub fn get_discord_token() -> Result<String, String> {
    let token = env::var(DISCORD_TOKEN_ENV).map_err(|_| {
        format!(
            "Missing required environment variable: {}\n\n\
            Please set the DISCORD_TOKEN environment variable with your Discord bot token.\
            \n\
            You can get a token from: https://discord.com/developers/applications",
            DISCORD_TOKEN_ENV
        )
    })?;

    // Validate the token format
    validate_discord_token(&token)?;

    Ok(token)
}

/// Validates the Discord bot token format.
///
/// Discord bot tokens are typically 80+ characters and contain dots.
/// This function provides helpful warnings if the token format looks incorrect.
///
/// # Arguments
///
/// * `token` - The Discord bot token to validate
///
/// # Returns
///
/// * `Ok(())` - If the token appears valid
/// * `Err(String)` - If the token format is definitely invalid
pub fn validate_discord_token(token: &str) -> Result<(), String> {
    // Discord bot tokens are typically long (80+ characters)
    if token.len() < 50 {
        return Err(format!(
            "Discord token seems too short ({} characters). Bot tokens are typically 80+ characters. \
             Please check your token is correct.",
            token.len()
        ));
    }

    // Discord tokens contain dots (separator between parts)
    if !token.contains('.') {
        return Err(
            "Discord token doesn't contain expected dots. Bot tokens typically have format: \
             XXXXX.YYYYY.ZZZZZ. Please verify your token is correct."
                .to_string(),
        );
    }

    // Token should only contain alphanumeric characters and dots
    let valid_chars = token.chars().all(|c| c.is_alphanumeric() || c == '.');
    if !valid_chars {
        return Err(
            "Discord token contains invalid characters. Tokens should only contain \
             alphanumeric characters and dots."
                .to_string(),
        );
    }

    Ok(())
}

/// Gets the OpenRouter API key from the environment.
///
/// # Returns
///
/// * `Ok(String)` - The API key value
/// * `Err(String)` - If the environment variable is not set or empty
pub fn get_openrouter_api_key() -> Result<String, String> {
    env::var(OPENROUTER_API_KEY_ENV).map_err(|_| {
        format!(
            "Missing required environment variable: {}\n\n\
            Please set the OPENROUTER_API_KEY environment variable with your OpenRouter API key.\
            \n\
            You can get a key from: https://openrouter.ai/settings",
            OPENROUTER_API_KEY_ENV
        )
    })
}

/// Gets the Discord channel ID from the environment.
///
/// # Returns
///
/// * `Ok(String)` - The channel ID value
/// * `Err(String)` - If the environment variable is not set or empty
pub fn get_discord_channel_id() -> Result<String, String> {
    env::var(DISCORD_CHANNEL_ID_ENV).map_err(|_| {
        format!(
            "Missing required environment variable: {}\n\n\
            Please set the DISCORD_CHANNEL_ID environment variable with your Discord channel ID.\
            \n\
            You can get a channel ID by enabling Developer Mode in Discord and right-clicking a channel.",
            DISCORD_CHANNEL_ID_ENV
        )
    })
}

/// Main Discord configuration section.
///
/// Controls the Discord bot's core behavior including
/// enabling/disabling the integration and connection settings.
///
/// # Example
///
/// ```toml
/// [discord]
/// enabled = true
/// token_env = "DISCORD_TOKEN"
/// channel_id = "1474550134388949272"
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct DiscordConfig {
    /// Enable or disable the Discord integration.
    ///
    /// When set to `false`, the Discord bot will not start even if
    /// other configuration options are present.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Environment variable name containing the Discord bot token.
    ///
    /// The bot token is read from this environment variable at startup.
    /// Must have the `Bot` prefix for Discord API requests.
    #[serde(default = "default_token_env")]
    pub token_env: String,

    /// Discord channel ID to listen on and respond to.
    ///
    /// This is the snowflake ID of the Discord channel where the
    /// concierge will listen for messages and respond.
    /// This field has no default and must be explicitly provided.
    pub channel_id: String,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            token_env: default_token_env(),
            channel_id: String::new(),
        }
    }
}

/// LLM provider configuration for the Discord concierge.
///
/// Controls which LLM provider to use, authentication, model selection,
/// and response generation parameters.
///
/// # Example
///
/// ```toml
/// [discord.llm]
/// provider = "openrouter"
/// api_key_env = "OPENROUTER_API_KEY"
/// model = "anthropic/claude-sonnet-4"
/// max_tokens = 1024
/// system_prompt_file = "prompts/concierge.md"
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmConfig {
    /// LLM provider to use for generating responses.
    ///
    /// Currently only `openrouter` is supported, which provides
    /// access to multiple LLM models through a unified API.
    #[serde(default = "default_provider")]
    pub provider: String,

    /// Environment variable name containing the LLM API key.
    ///
    /// The API key is read from this environment variable at startup.
    #[serde(default = "default_api_key_env")]
    pub api_key_env: String,

    /// Model identifier to use for completions.
    ///
    /// Must be a valid model supported by the configured provider.
    /// Format varies by provider (e.g., "anthropic/claude-sonnet-4").
    #[serde(default = "default_model")]
    pub model: String,

    /// Maximum number of tokens to generate in each response.
    ///
    /// Controls the length of LLM responses. Higher values allow
    /// longer responses but may increase latency and costs.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Optional path to a custom system prompt file.
    ///
    /// If provided, the concierge will read the system prompt from
    /// this file instead of using the built-in default.
    /// The file should contain markdown-formatted instructions.
    #[serde(default)]
    pub system_prompt_file: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            api_key_env: default_api_key_env(),
            model: default_model(),
            max_tokens: default_max_tokens(),
            system_prompt_file: None,
        }
    }
}

/// Conversation management configuration.
///
/// Controls how conversations are stored, sized, and expired
/// for each user interacting with the Discord concierge.
///
/// # Example
///
/// ```toml
/// [discord.conversation]
/// max_history = 30
/// ttl_minutes = 120
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConversationConfig {
    /// Maximum number of messages to keep in conversation history.
    ///
    /// Controls how many recent messages are retained for context
    /// when generating responses. Older messages are discarded
    /// to prevent unbounded memory growth.
    #[serde(default = "default_max_history")]
    pub max_history: usize,

    /// Time-to-live for conversations in minutes.
    ///
    /// Conversations that have been inactive for longer than this
    /// duration will be automatically removed to free memory.
    /// Set to 0 to disable expiration.
    #[serde(default = "default_ttl_minutes")]
    pub ttl_minutes: u64,
}

impl Default for ConversationConfig {
    fn default() -> Self {
        Self {
            max_history: default_max_history(),
            ttl_minutes: default_ttl_minutes(),
        }
    }
}

/// Wrapper struct for the complete Discord configuration section.
///
/// Combines Discord bot settings, LLM provider configuration, and
/// conversation management into a single struct that can be parsed
/// from a TOML configuration file with nested sections.
///
/// # Example
///
/// ```toml
/// [discord]
/// enabled = true
/// token_env = "DISCORD_TOKEN"
/// channel_id = "1474550134388949272"
///
/// [discord.llm]
/// provider = "openrouter"
/// api_key_env = "OPENROUTER_API_KEY"
/// model = "anthropic/claude-sonnet-4"
/// max_tokens = 1024
/// system_prompt_file = "prompts/concierge.md"
///
/// [discord.conversation]
/// max_history = 30
/// ttl_minutes = 120
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscordSection {
    /// Enable or disable the Discord integration.
    ///
    /// When set to `false`, the Discord bot will not start even if
    /// other configuration options are present.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Environment variable name containing the Discord bot token.
    ///
    /// The bot token is read from this environment variable at startup.
    /// Must have the `Bot` prefix for Discord API requests.
    #[serde(default = "default_token_env")]
    pub token_env: String,

    /// Discord channel ID to listen on and respond to.
    ///
    /// This is the snowflake ID of the Discord channel where the
    /// concierge will listen for messages and respond.
    /// This field has no default and must be explicitly provided.
    pub channel_id: String,

    /// Optional gateway intents to use for the Discord connection.
    ///
    /// Intents determine which events the bot receives from Discord.
    ///
    /// Common values:
    /// - `512` = GUILD_MESSAGES (messages in servers)
    /// - `4096` = DIRECT_MESSAGES (direct messages)
    /// - `16384` = MESSAGE_CONTENT (message content - requires verification!)
    /// - `21504` = 512 + 4096 + 16384 (all message intents)
    ///
    /// NOTE: The MESSAGE_CONTENT intent (16384) requires verification in the
    /// Discord Developer Portal. If you get 400 errors, you may need to either:
    /// 1. Enable MESSAGE_CONTENT intent in your bot's settings (if verified)
    /// 2. Use a lower intent value like `512` (if not verified)
    ///
    /// If not specified, uses the default value of 21504 (all intents).
    #[serde(default)]
    pub intents: Option<u32>,

    /// Optional LLM provider configuration.
    ///
    /// If present, configures the LLM provider for generating responses.
    /// If not present, uses default values for all LLM settings.
    #[serde(default)]
    pub llm: Option<LlmConfig>,

    /// Optional conversation management configuration.
    ///
    /// If present, configures how conversations are stored and managed.
    /// If not present, uses default values for conversation settings.
    #[serde(default)]
    pub conversation: Option<ConversationConfig>,
}

impl Default for DiscordSection {
    fn default() -> Self {
        Self {
            enabled: false,
            token_env: default_token_env(),
            channel_id: String::new(),
            intents: None,
            llm: None,
            conversation: None,
        }
    }
}

// Default value functions for serde

fn default_enabled() -> bool {
    false
}

fn default_token_env() -> String {
    "DISCORD_TOKEN".to_string()
}

fn default_provider() -> String {
    "openrouter".to_string()
}

fn default_api_key_env() -> String {
    "OPENROUTER_API_KEY".to_string()
}

fn default_model() -> String {
    "anthropic/claude-sonnet-4".to_string()
}

fn default_max_tokens() -> u32 {
    1024
}

fn default_max_history() -> usize {
    20
}

fn default_ttl_minutes() -> u64 {
    60
}

/// Loads a DiscordSection from a TOML configuration file.
///
/// Reads the TOML file from the given path and attempts to parse the `[discord]`
/// section. Returns `Ok(Some(DiscordSection))` if the section is found, `Ok(None)`
/// if not found, or `Err(String)` if there's a file read or parse error.
///
/// # Arguments
///
/// * `path` - Path to the TOML configuration file
///
/// # Returns
///
/// * `Ok(Some(DiscordSection))` - If the [discord] section was found and parsed
/// * `Ok(None)` - If the [discord] section was not found in the file
/// * `Err(String)` - If the file could not be read or parsed
///
/// # Example
///
/// ```ignore
/// match load_discord_section_from_toml("switchboard.toml") {
///     Ok(Some(config)) => {
///         println!("Discord enabled: {}", config.enabled);
///     }
///     Ok(None) => {
///         println!("No Discord section found in config");
///     }
///     Err(e) => {
///         eprintln!("Error loading config: {}", e);
///     }
/// }
/// ```
pub fn load_discord_section_from_toml(path: &str) -> Result<Option<DiscordSection>, String> {
    // Read the TOML file
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file '{}': {}", path, e))?;

    // Try to parse the TOML, looking for a [discord] section
    // We use a wrapper struct to handle the nested table structure
    #[derive(Deserialize)]
    struct Config {
        #[serde(default)]
        discord: Option<DiscordSection>,
    }

    match toml::from_str::<Config>(&content) {
        Ok(config) => Ok(config.discord),
        Err(e) => Err(format!("Failed to parse config file '{}': {}", path, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_discord_config_defaults() {
        let config = DiscordConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.token_env, "DISCORD_TOKEN");
        assert!(config.channel_id.is_empty());
    }

    #[test]
    #[serial]
    fn test_env_config_missing_discord_token() {
        // Save original env var if it exists
        let original_token = env::var(DISCORD_TOKEN_ENV).ok();
        let original_key = env::var(OPENROUTER_API_KEY_ENV).ok();

        // Remove DISCORD_TOKEN and set OPENROUTER_API_KEY
        env::remove_var(DISCORD_TOKEN_ENV);
        if let Some(ref key) = original_key {
            env::set_var(OPENROUTER_API_KEY_ENV, key);
        } else {
            env::set_var(OPENROUTER_API_KEY_ENV, "test_key");
        }

        let result = load_discord_config();

        // Restore original values
        if let Some(token) = original_token {
            env::set_var(DISCORD_TOKEN_ENV, token);
        } else {
            env::remove_var(DISCORD_TOKEN_ENV);
        }
        if original_key.is_none() {
            env::remove_var(OPENROUTER_API_KEY_ENV);
        }

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains(DISCORD_TOKEN_ENV));
    }

    #[test]
    #[serial]
    fn test_env_config_missing_openrouter_api_key() {
        // Save original env var if it exists
        let original_token = env::var(DISCORD_TOKEN_ENV).ok();
        let original_key = env::var(OPENROUTER_API_KEY_ENV).ok();

        // Set DISCORD_TOKEN and remove OPENROUTER_API_KEY
        if let Some(ref token) = original_token {
            env::set_var(DISCORD_TOKEN_ENV, token);
        } else {
            env::set_var(DISCORD_TOKEN_ENV, "test_token");
        }
        env::remove_var(OPENROUTER_API_KEY_ENV);

        let result = load_discord_config();

        // Restore original values
        if original_token.is_none() {
            env::remove_var(DISCORD_TOKEN_ENV);
        }
        if let Some(key) = original_key {
            env::set_var(OPENROUTER_API_KEY_ENV, key);
        } else {
            env::remove_var(OPENROUTER_API_KEY_ENV);
        }

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains(OPENROUTER_API_KEY_ENV));
    }

    #[test]
    #[serial]
    fn test_env_config_success() {
        // Save original env vars if they exist
        let original_token = env::var(DISCORD_TOKEN_ENV).ok();
        let original_key = env::var(OPENROUTER_API_KEY_ENV).ok();

        // Set both environment variables
        env::set_var(DISCORD_TOKEN_ENV, "test_discord_token");
        env::set_var(OPENROUTER_API_KEY_ENV, "test_openrouter_key");

        let result = load_discord_config();

        // Restore original values
        match original_token {
            Some(token) => env::set_var(DISCORD_TOKEN_ENV, token),
            None => env::remove_var(DISCORD_TOKEN_ENV),
        }
        match original_key {
            Some(key) => env::set_var(OPENROUTER_API_KEY_ENV, key),
            None => env::remove_var(OPENROUTER_API_KEY_ENV),
        }

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.discord_token, "test_discord_token");
        assert_eq!(config.openrouter_api_key, "test_openrouter_key");
    }

    #[test]
    #[serial]
    fn test_get_discord_token_missing() {
        // Save original env var if it exists
        let original = env::var(DISCORD_TOKEN_ENV).ok();

        env::remove_var(DISCORD_TOKEN_ENV);
        let result = get_discord_token();

        // Restore original value
        if let Some(val) = original {
            env::set_var(DISCORD_TOKEN_ENV, val);
        }

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains(DISCORD_TOKEN_ENV));
    }

    #[test]
    #[serial]
    fn test_get_openrouter_api_key_missing() {
        // Save original env var if it exists
        let original = env::var(OPENROUTER_API_KEY_ENV).ok();

        env::remove_var(OPENROUTER_API_KEY_ENV);
        let result = get_openrouter_api_key();

        // Restore original value
        if let Some(val) = original {
            env::set_var(OPENROUTER_API_KEY_ENV, val);
        }

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains(OPENROUTER_API_KEY_ENV));
    }

    #[test]
    fn test_env_consts() {
        assert_eq!(DISCORD_TOKEN_ENV, "DISCORD_TOKEN");
        assert_eq!(OPENROUTER_API_KEY_ENV, "OPENROUTER_API_KEY");
    }

    #[test]
    fn test_llm_config_defaults() {
        let config = LlmConfig::default();
        assert_eq!(config.provider, "openrouter");
        assert_eq!(config.api_key_env, "OPENROUTER_API_KEY");
        assert_eq!(config.model, "anthropic/claude-sonnet-4");
        assert_eq!(config.max_tokens, 1024);
        assert!(config.system_prompt_file.is_none());
    }

    #[test]
    fn test_conversation_config_defaults() {
        let config = ConversationConfig::default();
        assert_eq!(config.max_history, 20);
        assert_eq!(config.ttl_minutes, 60);
    }

    #[test]
    fn test_discord_config_toml_parsing() {
        let toml = r#"
            enabled = true
            token_env = "MY_DISCORD_TOKEN"
            channel_id = "123456789"
        "#;
        let config: DiscordConfig = toml::from_str(toml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.token_env, "MY_DISCORD_TOKEN");
        assert_eq!(config.channel_id, "123456789");
    }

    #[test]
    fn test_llm_config_toml_parsing() {
        let toml = r#"
            provider = "openrouter"
            api_key_env = "MY_API_KEY"
            model = "anthropic/claude-3-opus"
            max_tokens = 2048
            system_prompt_file = "custom_prompt.md"
        "#;
        let config: LlmConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.provider, "openrouter");
        assert_eq!(config.api_key_env, "MY_API_KEY");
        assert_eq!(config.model, "anthropic/claude-3-opus");
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(
            config.system_prompt_file,
            Some("custom_prompt.md".to_string())
        );
    }

    #[test]
    fn test_conversation_config_toml_parsing() {
        let toml = r#"
            max_history = 50
            ttl_minutes = 60
        "#;
        let config: ConversationConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.max_history, 50);
        assert_eq!(config.ttl_minutes, 60);
    }

    #[test]
    fn test_discord_section_toml_parsing() {
        // Test parsing the full discord section matching switchboard.sample.toml format
        // DiscordSection is used inside a wrapper struct that has the "discord" prefix
        // for the nested table names like [discord.llm] and [discord.conversation]
        let toml = r#"
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

        // Define a wrapper struct to test the full TOML structure
        #[derive(Debug, Deserialize)]
        struct Config {
            discord: DiscordSection,
        }

        let config: Config = toml::from_str(toml).unwrap();

        // Check main discord config
        assert!(config.discord.enabled);
        assert_eq!(config.discord.token_env, "DISCORD_TOKEN");
        assert_eq!(config.discord.channel_id, "1474550134388949272");

        // Check LLM config
        let llm = config.discord.llm.expect("LLM config should be present");
        assert_eq!(llm.provider, "openrouter");
        assert_eq!(llm.api_key_env, "OPENROUTER_API_KEY");
        assert_eq!(llm.model, "anthropic/claude-sonnet-4");
        assert_eq!(llm.max_tokens, 1024);
        assert!(llm.system_prompt_file.is_none());

        // Check conversation config
        let conv = config
            .discord
            .conversation
            .expect("Conversation config should be present");
        assert_eq!(conv.max_history, 30);
        assert_eq!(conv.ttl_minutes, 120);
    }

    #[test]
    fn test_discord_section_defaults() {
        let config = DiscordSection::default();
        assert!(!config.enabled);
        assert_eq!(config.token_env, "DISCORD_TOKEN");
        assert!(config.channel_id.is_empty());
        assert!(config.llm.is_none());
        assert!(config.conversation.is_none());
    }

    #[test]
    fn test_discord_section_partial_config() {
        // Test with only discord section, no nested sections
        let toml = r#"
            enabled = true
            token_env = "MY_TOKEN"
            channel_id = "123456789"
        "#;
        let config: DiscordSection = toml::from_str(toml).unwrap();

        assert!(config.enabled);
        assert_eq!(config.token_env, "MY_TOKEN");
        assert_eq!(config.channel_id, "123456789");
        assert!(config.llm.is_none());
        assert!(config.conversation.is_none());
    }

    #[test]
    fn test_discord_section_only_llm() {
        // Test with only LLM section present
        let toml = r#"
            [discord]
            enabled = false
            token_env = "DISCORD_TOKEN"
            channel_id = "1474550134388949272"

            [discord.llm]
            provider = "openrouter"
            api_key_env = "CUSTOM_API_KEY"
            model = "anthropic/claude-3-opus"
            max_tokens = 2048
        "#;

        #[derive(Debug, Deserialize)]
        struct Config {
            discord: DiscordSection,
        }

        let config: Config = toml::from_str(toml).unwrap();

        assert!(!config.discord.enabled);
        assert!(config.discord.conversation.is_none());

        let llm = config.discord.llm.expect("LLM config should be present");
        assert_eq!(llm.api_key_env, "CUSTOM_API_KEY");
        assert_eq!(llm.model, "anthropic/claude-3-opus");
        assert_eq!(llm.max_tokens, 2048);
    }

    #[test]
    fn test_discord_section_partial_toml() {
        // Test parsing just the [discord] section without nested sections
        let toml = r#"
            [discord]
            enabled = true
            token_env = "MY_TOKEN"
            channel_id = "123456789"
        "#;

        #[derive(Debug, Deserialize)]
        struct Config {
            discord: DiscordSection,
        }

        let config: Config = toml::from_str(toml).unwrap();

        assert!(config.discord.enabled);
        assert_eq!(config.discord.token_env, "MY_TOKEN");
        assert_eq!(config.discord.channel_id, "123456789");
        assert!(config.discord.llm.is_none());
        assert!(config.discord.conversation.is_none());
    }

    // Tests for load_discord_section_from_toml

    #[test]
    fn test_load_discord_section_from_toml_file_not_found() {
        let result = load_discord_section_from_toml("/nonexistent/path/config.toml");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Failed to read config file"));
        assert!(err.contains("/nonexistent/path/config.toml"));
    }

    #[test]
    fn test_load_discord_section_from_toml_no_discord_section() {
        // Create a temp TOML file without [discord] section
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("switchboard_test_no_discord.toml");

        let toml_content = r#"
            [settings]
            log_dir = ".switchboard/logs"

            [[agent]]
            name = "test-agent"
            schedule = "0 * * * *"
            prompt = "Test prompt"
        "#;

        std::fs::write(&temp_path, toml_content).unwrap();

        let result = load_discord_section_from_toml(temp_path.to_str().unwrap());

        // Clean up
        std::fs::remove_file(&temp_path).ok();

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_load_discord_section_from_toml_with_discord_section() {
        // Create a temp TOML file with [discord] section
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("switchboard_test_with_discord.toml");

        let toml_content = r#"
            [settings]
            log_dir = ".switchboard/logs"

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

        std::fs::write(&temp_path, toml_content).unwrap();

        let result = load_discord_section_from_toml(temp_path.to_str().unwrap());

        // Clean up
        std::fs::remove_file(&temp_path).unwrap();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.is_some());

        let discord = config.unwrap();
        assert!(discord.enabled);
        assert_eq!(discord.token_env, "DISCORD_TOKEN");
        assert_eq!(discord.channel_id, "1474550134388949272");

        let llm = discord.llm.expect("LLM config should be present");
        assert_eq!(llm.provider, "openrouter");
        assert_eq!(llm.api_key_env, "OPENROUTER_API_KEY");
        assert_eq!(llm.model, "anthropic/claude-sonnet-4");
        assert_eq!(llm.max_tokens, 1024);

        let conv = discord
            .conversation
            .expect("Conversation config should be present");
        assert_eq!(conv.max_history, 30);
        assert_eq!(conv.ttl_minutes, 120);
    }

    #[test]
    fn test_load_discord_section_from_toml_invalid_toml() {
        // Create a temp TOML file with invalid content
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("switchboard_test_invalid.toml");

        let invalid_content = "this is not valid toml [[]] ";

        std::fs::write(&temp_path, invalid_content).unwrap();

        let result = load_discord_section_from_toml(temp_path.to_str().unwrap());

        // Clean up
        std::fs::remove_file(&temp_path).unwrap();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Failed to parse config file"));
    }

    #[test]
    fn test_load_discord_section_from_toml_disabled_discord() {
        // Create a temp TOML file with [discord] but enabled = false
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("switchboard_test_disabled.toml");

        let toml_content = r#"
            [discord]
            enabled = false
            token_env = "DISCORD_TOKEN"
            channel_id = "1474550134388949272"
        "#;

        std::fs::write(&temp_path, toml_content).unwrap();

        let result = load_discord_section_from_toml(temp_path.to_str().unwrap());

        // Clean up
        std::fs::remove_file(&temp_path).unwrap();

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.is_some());

        let discord = config.unwrap();
        assert!(!discord.enabled); // Should be false as specified
    }

    #[test]
    fn test_load_switchboard_toml_discord_section() {
        // Test loading the actual switchboard.toml file
        let result = load_discord_section_from_toml("switchboard.toml");

        assert!(
            result.is_ok(),
            "Failed to load switchboard.toml: {:?}",
            result.err()
        );
        let config = result.unwrap();
        assert!(
            config.is_some(),
            "Expected [discord] section in switchboard.toml"
        );

        let discord = config.unwrap();

        // Verify enabled = true from switchboard.toml
        assert!(
            discord.enabled,
            "Expected enabled = true from switchboard.toml"
        );

        // Verify token_env is read from switchboard.toml (it's set to a token value in the file)
        // Note: switchboard.toml has token_env set to an actual token, not the default "DISCORD_TOKEN"
        assert_eq!(
            discord.token_env, "${DISCORD_TOKEN}",
            "Expected token_env from switchboard.toml"
        );

        // Verify channel_id = "1472443428569874533" from switchboard.toml
        assert_eq!(
            discord.channel_id, "1472443428569874533",
            "Expected channel_id = 1472443428569874533 from switchboard.toml"
        );

        // Verify LLM configuration
        let llm = discord
            .llm
            .expect("LLM config should be present in switchboard.toml");
        assert_eq!(
            llm.provider, "openrouter",
            "Expected provider = openrouter from switchboard.toml"
        );
        assert_eq!(
            llm.api_key_env,
            "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",
            "Expected api_key_env from switchboard.toml"
        );
        assert_eq!(
            llm.model, "anthropic/claude-sonnet-4",
            "Expected model from switchboard.toml"
        );
        assert_eq!(
            llm.max_tokens, 1024,
            "Expected max_tokens = 1024 from switchboard.toml"
        );
        assert!(
            llm.system_prompt_file.is_none(),
            "Expected system_prompt_file = None (not set in switchboard.toml)"
        );

        // Verify conversation configuration
        let conversation = discord
            .conversation
            .expect("Conversation config should be present in switchboard.toml");
        assert_eq!(
            conversation.max_history, 30,
            "Expected max_history = 30 from switchboard.toml"
        );
        assert_eq!(
            conversation.ttl_minutes, 120,
            "Expected ttl_minutes = 120 from switchboard.toml"
        );
    }
}
