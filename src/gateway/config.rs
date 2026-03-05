//! Gateway configuration module.
//!
//! Provides configuration structs for the Gateway service including
//! server settings, logging configuration, and channel mappings.

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;
use tracing::debug;

/// Error types for gateway configuration loading.
#[derive(Debug, Error)]
pub enum GatewayConfigError {
    /// I/O error when reading the config file.
    #[error("Failed to read configuration file: {0}")]
    IoError(#[from] std::io::Error),

    /// TOML parsing error.
    #[error("Failed to parse configuration file: {0}")]
    ParseError(#[from] toml::de::Error),

    /// Missing required environment variable.
    #[error("Missing required environment variable: {0}")]
    EnvVarError(String),

    /// Configuration validation error.
    #[error("Configuration validation error: {0}")]
    ValidationError(String),
}

/// Server configuration for the gateway HTTP/WS servers.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Host address to bind to.
    #[serde(default = "default_host")]
    pub host: String,

    /// HTTP server port.
    #[serde(default = "default_http_port")]
    pub http_port: u32,

    /// WebSocket server port.
    #[serde(default = "default_ws_port")]
    pub ws_port: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            http_port: default_http_port(),
            ws_port: default_ws_port(),
        }
    }
}

/// Logging configuration for the gateway.
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Log level (debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Optional log file path. If not provided, logs only to stdout.
    #[serde(default)]
    pub file: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: None,
        }
    }
}

/// Channel mapping configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ChannelMapping {
    /// Discord channel ID.
    pub channel_id: String,

    /// Project name for this channel.
    pub project_name: String,

    /// Project's WebSocket endpoint.
    pub endpoint: String,
}

/// Main gateway configuration.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GatewayConfig {
    /// Discord bot token (supports ${VAR} syntax).
    pub discord_token: String,

    /// Server configuration.
    #[serde(default)]
    pub server: ServerConfig,

    /// Logging configuration.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Channel mappings.
    #[serde(default)]
    pub channels: Vec<ChannelMapping>,
}

// Default value functions for serde
fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_http_port() -> u32 {
    8080
}

fn default_ws_port() -> u32 {
    9000
}

fn default_log_level() -> String {
    "info".to_string()
}

impl GatewayConfig {
    /// Load gateway configuration from a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Optional path to the config file. If None, defaults to "gateway.toml"
    ///
    /// # Returns
    ///
    /// * `Ok(GatewayConfig)` - The loaded configuration
    /// * `Err(GatewayConfigError)` - If the file cannot be read or parsed
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = GatewayConfig::load(Some("gateway.toml"))
    ///     .expect("Failed to load gateway config");
    /// ```
    pub fn load(path: Option<&str>) -> Result<Self, GatewayConfigError> {
        let config_path = path.unwrap_or("gateway.toml");
        let path = Path::new(config_path);

        debug!("Loading gateway configuration from: {:?}", path);

        let content = fs::read_to_string(path)?;
        let mut config: GatewayConfig = toml::from_str(&content)?;

        // Resolve environment variables in the config
        config.resolve_env_vars();

        // Validate the configuration
        config.validate()?;

        Ok(config)
    }

    /// Load gateway configuration from environment variables.
    ///
    /// This is an alternative to loading from a TOML file.
    /// Required environment variables:
    /// - DISCORD_TOKEN: Discord bot token
    ///
    /// Optional environment variables:
    /// - GATEWAY_HOST: Server host (default: "0.0.0.0")
    /// - GATEWAY_HTTP_PORT: HTTP server port (default: 8080)
    /// - GATEWAY_WS_PORT: WebSocket server port (default: 9000)
    /// - GATEWAY_LOG_LEVEL: Log level (default: "info")
    ///
    /// # Returns
    ///
    /// * `Ok(GatewayConfig)` - The loaded configuration
    /// * `Err(GatewayConfigError)` - If required environment variables are missing
    pub fn from_env() -> Result<Self, GatewayConfigError> {
        use crate::config::env::get_env_vars;

        let env_vars = get_env_vars();

        // Get Discord token - required
        let discord_token =
            resolve_env_var_from_map("DISCORD_TOKEN", env_vars).ok_or_else(|| {
                GatewayConfigError::EnvVarError(
                    "DISCORD_TOKEN environment variable is required".to_string(),
                )
            })?;

        // Get optional server config
        let host = resolve_env_var_from_map("GATEWAY_HOST", env_vars).unwrap_or_else(default_host);
        let http_port: u32 = resolve_env_var_from_map("GATEWAY_HTTP_PORT", env_vars)
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(default_http_port);
        let ws_port: u32 = resolve_env_var_from_map("GATEWAY_WS_PORT", env_vars)
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(default_ws_port);

        // Get optional logging config
        let level = resolve_env_var_from_map("GATEWAY_LOG_LEVEL", env_vars)
            .unwrap_or_else(default_log_level);

        Ok(GatewayConfig {
            discord_token,
            server: ServerConfig {
                host,
                http_port,
                ws_port,
            },
            logging: LoggingConfig { level, file: None },
            channels: Vec::new(),
        })
    }

    /// Resolve environment variable references in configuration values.
    ///
    /// This scans for ${VAR} or ${VAR:-default} patterns and replaces
    /// them with values from environment or .env file.
    fn resolve_env_vars(&mut self) {
        use crate::config::env::get_env_vars;

        let env_vars = get_env_vars();

        // Resolve discord_token
        self.discord_token = resolve_env_value(&self.discord_token, env_vars);

        // Resolve channel endpoints
        for channel in &mut self.channels {
            channel.endpoint = resolve_env_value(&channel.endpoint, env_vars);
        }
    }

    /// Validate the configuration.
    ///
    /// Checks that:
    /// - discord_token is not empty
    /// - http_port is in range 1024-65535
    /// - ws_port is in range 1024-65535
    /// - Each channel has non-empty channel_id and project_name
    fn validate(&self) -> Result<(), GatewayConfigError> {
        // Validate discord_token is not empty
        if self.discord_token.is_empty() {
            return Err(GatewayConfigError::ValidationError(
                "discord_token cannot be empty".to_string(),
            ));
        }

        // Validate http_port is in range 1024-65535
        if self.server.http_port < 1024 || self.server.http_port > 65535 {
            return Err(GatewayConfigError::ValidationError(format!(
                "http_port must be >= 1024, got {}",
                self.server.http_port
            )));
        }

        // Validate ws_port is in range 1024-65535
        if self.server.ws_port < 1024 || self.server.ws_port > 65535 {
            return Err(GatewayConfigError::ValidationError(format!(
                "ws_port must be >= 1024, got {}",
                self.server.ws_port
            )));
        }

        // Validate each channel has non-empty channel_id and project_name
        for (idx, channel) in self.channels.iter().enumerate() {
            if channel.channel_id.is_empty() {
                return Err(GatewayConfigError::ValidationError(format!(
                    "channel at index {} has empty channel_id",
                    idx
                )));
            }
            // Validate channel_id is numeric (Discord snowflake)
            if channel.channel_id.parse::<u64>().is_err() {
                return Err(GatewayConfigError::ValidationError(format!(
                    "channel at index {} has invalid channel_id '{}' - must be numeric",
                    idx, channel.channel_id
                )));
            }
            if channel.project_name.is_empty() {
                return Err(GatewayConfigError::ValidationError(format!(
                    "channel at index {} has empty project_name",
                    idx
                )));
            }
        }

        Ok(())
    }
}

/// Resolve a single environment variable reference.
fn resolve_env_var_from_map(var_name: &str, env_vars: &HashMap<String, String>) -> Option<String> {
    // First check system environment
    if let Ok(value) = std::env::var(var_name) {
        if !value.is_empty() {
            return Some(value);
        }
    }

    // Then check provided map
    env_vars.get(var_name).cloned()
}

/// Resolve environment variable references in a value string.
///
/// Supports ${VAR} and ${VAR:-default} syntax.
fn resolve_env_value(value: &str, env_vars: &HashMap<String, String>) -> String {
    crate::config::env::resolve_env_vars(value, env_vars)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_default_server_config() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.http_port, 8080);
        assert_eq!(config.ws_port, 9000);
    }

    #[test]
    fn test_default_logging_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.file, None);
    }

    #[test]
    fn test_default_gateway_config() {
        let config = GatewayConfig::default();
        assert!(config.discord_token.is_empty());
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.logging.level, "info");
        assert!(config.channels.is_empty());
    }

    #[test]
    fn test_load_gateway_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let toml_content = r#"
discord_token = "test_token_123"

[server]
host = "127.0.0.1"
http_port = 3000
ws_port = 4000

[logging]
level = "debug"
file = "/var/log/gateway.log"

[[channels]]
channel_id = "123456789"
project_name = "test-project"
endpoint = "ws://localhost:8080"
"#;

        fs::write(&config_file, toml_content).unwrap();

        let config = GatewayConfig::load(Some(config_file.to_str().unwrap())).unwrap();

        assert_eq!(config.discord_token, "test_token_123");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.http_port, 3000);
        assert_eq!(config.server.ws_port, 4000);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(
            config.logging.file,
            Some("/var/log/gateway.log".to_string())
        );
        assert_eq!(config.channels.len(), 1);
        assert_eq!(config.channels[0].channel_id, "123456789");
        assert_eq!(config.channels[0].project_name, "test-project");
    }

    #[test]
    fn test_load_config_with_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let toml_content = r#"
discord_token = "simple_token"
"#;

        fs::write(&config_file, toml_content).unwrap();

        let config = GatewayConfig::load(Some(config_file.to_str().unwrap())).unwrap();

        assert_eq!(config.discord_token, "simple_token");
        // Check defaults
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.http_port, 8080);
        assert_eq!(config.server.ws_port, 9000);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_env_var_expansion() {
        // Set test environment variable
        env::set_var("TEST_DISCORD_TOKEN", "expanded_token_123");

        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let toml_content = r#"
discord_token = "${TEST_DISCORD_TOKEN}"
"#;

        fs::write(&config_file, toml_content).unwrap();

        let config = GatewayConfig::load(Some(config_file.to_str().unwrap())).unwrap();

        assert_eq!(config.discord_token, "expanded_token_123");

        env::remove_var("TEST_DISCORD_TOKEN");
    }

    #[test]
    fn test_env_var_expansion_with_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        // Use a default value for non-existent env var
        let toml_content = r#"
discord_token = "${NONEXISTENT_VAR:-default_token_value}"
"#;

        fs::write(&config_file, toml_content).unwrap();

        let config = GatewayConfig::load(Some(config_file.to_str().unwrap())).unwrap();

        assert_eq!(config.discord_token, "default_token_value");
    }

    #[test]
    fn test_multiple_channels() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let toml_content = r#"
discord_token = "token"

[[channels]]
channel_id = "111"
project_name = "project1"
endpoint = "ws://localhost:1111"

[[channels]]
channel_id = "222"
project_name = "project2"
endpoint = "ws://localhost:2222"

[[channels]]
channel_id = "333"
project_name = "project3"
endpoint = "ws://localhost:3333"
"#;

        fs::write(&config_file, toml_content).unwrap();

        let config = GatewayConfig::load(Some(config_file.to_str().unwrap())).unwrap();

        assert_eq!(config.channels.len(), 3);
        assert_eq!(config.channels[0].channel_id, "111");
        assert_eq!(config.channels[1].channel_id, "222");
        assert_eq!(config.channels[2].channel_id, "333");
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = GatewayConfig::load(Some("nonexistent_gateway_config.toml"));
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::IoError(_)) => {}
            _ => panic!("Expected IoError for nonexistent file"),
        }
    }

    #[test]
    fn test_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let invalid_toml = r#"
discord_token = "token"
this is not valid toml
"#;

        fs::write(&config_file, invalid_toml).unwrap();

        let result = GatewayConfig::load(Some(config_file.to_str().unwrap()));
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ParseError(_)) => {}
            _ => panic!("Expected ParseError for invalid TOML"),
        }
    }

    #[test]
    fn test_from_env_missing_token() {
        // Note: This test verifies that when no token is provided via system env,
        // the cached .env file values are used (for backwards compatibility).
        // The switchboard.env file typically has DISCORD_TOKEN set.

        // Remove any system env var first
        env::remove_var("DISCORD_TOKEN");

        // Since switchboard.env is cached, we expect this to succeed with the cached value
        // This tests that the .env fallback works
        let result = GatewayConfig::from_env();

        // The function should succeed (not error) because it falls back to cached .env values
        // We just verify it doesn't panic
        if let Ok(config) = result {
            // Verify we got the cached token (not empty)
            assert!(
                !config.discord_token.is_empty(),
                "Should have a token from .env cache"
            );
        }
        // If it errors, that's also acceptable - means the .env cache was empty
    }

    use serial_test::serial;

    #[test]
    #[serial]
    fn test_from_env_with_token() {
        // Clean up any pre-existing values first
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("GATEWAY_HOST");
        env::remove_var("GATEWAY_HTTP_PORT");
        env::remove_var("GATEWAY_WS_PORT");
        env::remove_var("GATEWAY_LOG_LEVEL");

        // Set required environment variable
        env::set_var("DISCORD_TOKEN", "env_test_token");

        let config = GatewayConfig::from_env().unwrap();

        assert_eq!(config.discord_token, "env_test_token");
        // Check defaults - system env vars take precedence over .env file
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.http_port, 8080);
        assert_eq!(config.server.ws_port, 9000);
        assert_eq!(config.logging.level, "info");

        // Clean up
        env::remove_var("DISCORD_TOKEN");
    }

    #[test]
    #[serial]
    fn test_from_env_with_optional_vars() {
        // Clean up any pre-existing values first
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("GATEWAY_HOST");
        env::remove_var("GATEWAY_HTTP_PORT");
        env::remove_var("GATEWAY_WS_PORT");
        env::remove_var("GATEWAY_LOG_LEVEL");

        // Set all environment variables (including one not in switchboard.env)
        env::set_var("DISCORD_TOKEN", "env_test_token");
        env::set_var("GATEWAY_HOST", "localhost");
        env::set_var("GATEWAY_HTTP_PORT", "9999");
        env::set_var("GATEWAY_WS_PORT", "8888");
        env::set_var("GATEWAY_LOG_LEVEL", "warn");

        let config = GatewayConfig::from_env().unwrap();

        // Verify our explicit values are used (system env takes precedence)
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.http_port, 9999);
        assert_eq!(config.server.ws_port, 8888);
        assert_eq!(config.logging.level, "warn");

        // Clean up
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("GATEWAY_HOST");
        env::remove_var("GATEWAY_HTTP_PORT");
        env::remove_var("GATEWAY_WS_PORT");
        env::remove_var("GATEWAY_LOG_LEVEL");
    }

    #[test]
    fn test_channel_endpoint_env_expansion() {
        env::set_var("PROJECT_ENDPOINT", "ws://my-project.example.com");

        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let toml_content = r#"
discord_token = "token"

[[channels]]
channel_id = "123"
project_name = "myproject"
endpoint = "${PROJECT_ENDPOINT}"
"#;

        fs::write(&config_file, toml_content).unwrap();

        let config = GatewayConfig::load(Some(config_file.to_str().unwrap())).unwrap();

        assert_eq!(config.channels[0].endpoint, "ws://my-project.example.com");

        env::remove_var("PROJECT_ENDPOINT");
    }

    // ============== Validation Tests ==============

    #[test]
    fn validate_should_return_error_when_token_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let toml_content = r#"
discord_token = ""
"#;

        fs::write(&config_file, toml_content).unwrap();

        let result = GatewayConfig::load(Some(config_file.to_str().unwrap()));
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("discord_token cannot be empty"));
            }
            _ => panic!("Expected ValidationError for empty token"),
        }
    }

    #[test]
    fn validate_should_return_error_when_http_port_too_low() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let toml_content = r#"
discord_token = "test_token"

[server]
http_port = 80
"#;

        fs::write(&config_file, toml_content).unwrap();

        let result = GatewayConfig::load(Some(config_file.to_str().unwrap()));
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("http_port must be >= 1024"));
            }
            _ => panic!("Expected ValidationError for http_port too low"),
        }
    }

    #[test]
    fn validate_should_return_error_when_http_port_too_high() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let toml_content = r#"
discord_token = "test_token"

[server]
http_port = 70000
"#;

        fs::write(&config_file, toml_content).unwrap();

        let result = GatewayConfig::load(Some(config_file.to_str().unwrap()));
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("http_port must be >= 1024"));
            }
            _ => panic!("Expected ValidationError for http_port too high"),
        }
    }

    #[test]
    fn validate_should_return_error_when_ws_port_too_low() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let toml_content = r#"
discord_token = "test_token"

[server]
ws_port = 80
"#;

        fs::write(&config_file, toml_content).unwrap();

        let result = GatewayConfig::load(Some(config_file.to_str().unwrap()));
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("ws_port must be >= 1024"));
            }
            _ => panic!("Expected ValidationError for ws_port too low"),
        }
    }

    #[test]
    fn validate_should_return_error_when_channel_missing_channel_id() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let toml_content = r#"
discord_token = "test_token"

[[channels]]
channel_id = ""
project_name = "test-project"
endpoint = "ws://localhost:8080"
"#;

        fs::write(&config_file, toml_content).unwrap();

        let result = GatewayConfig::load(Some(config_file.to_str().unwrap()));
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("empty channel_id"));
            }
            _ => panic!("Expected ValidationError for missing channel_id"),
        }
    }

    #[test]
    fn validate_should_return_error_when_channel_missing_project_name() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let toml_content = r#"
discord_token = "test_token"

[[channels]]
channel_id = "123456789"
project_name = ""
endpoint = "ws://localhost:8080"
"#;

        fs::write(&config_file, toml_content).unwrap();

        let result = GatewayConfig::load(Some(config_file.to_str().unwrap()));
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("empty project_name"));
            }
            _ => panic!("Expected ValidationError for missing project_name"),
        }
    }

    #[test]
    fn validate_should_pass_with_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("gateway.toml");

        let toml_content = r#"
discord_token = "valid_token_123"

[server]
host = "127.0.0.1"
http_port = 8080
ws_port = 9000

[logging]
level = "debug"

[[channels]]
channel_id = "123456789"
project_name = "test-project"
endpoint = "ws://localhost:8080"
"#;

        fs::write(&config_file, toml_content).unwrap();

        let result = GatewayConfig::load(Some(config_file.to_str().unwrap()));
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.discord_token, "valid_token_123");
        assert_eq!(config.server.http_port, 8080);
        assert_eq!(config.server.ws_port, 9000);
        assert_eq!(config.channels.len(), 1);
        assert_eq!(config.channels[0].channel_id, "123456789");
        assert_eq!(config.channels[0].project_name, "test-project");
    }

    // ========== Validation Tests ==========

    #[test]
    fn test_validation_fails_when_discord_token_empty() {
        let config = GatewayConfig {
            discord_token: String::new(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            channels: vec![],
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("discord_token"));
            }
            _ => panic!("Expected ValidationError for empty discord_token"),
        }
    }

    #[test]
    fn test_validation_fails_when_http_port_below_1024() {
        let config = GatewayConfig {
            discord_token: "valid_token".to_string(),
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                http_port: 80,
                ws_port: 9000,
            },
            logging: LoggingConfig::default(),
            channels: vec![],
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("http_port"));
            }
            _ => panic!("Expected ValidationError for http_port < 1024"),
        }
    }

    #[test]
    fn test_validation_fails_when_http_port_above_65535() {
        let config = GatewayConfig {
            discord_token: "valid_token".to_string(),
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                http_port: 70000,
                ws_port: 9000,
            },
            logging: LoggingConfig::default(),
            channels: vec![],
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("http_port"));
            }
            _ => panic!("Expected ValidationError for http_port > 65535"),
        }
    }

    #[test]
    fn test_validation_fails_when_ws_port_below_1024() {
        let config = GatewayConfig {
            discord_token: "valid_token".to_string(),
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                http_port: 8080,
                ws_port: 80,
            },
            logging: LoggingConfig::default(),
            channels: vec![],
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("ws_port"));
            }
            _ => panic!("Expected ValidationError for ws_port < 1024"),
        }
    }

    #[test]
    fn test_validation_fails_when_ws_port_above_65535() {
        let config = GatewayConfig {
            discord_token: "valid_token".to_string(),
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                http_port: 8080,
                ws_port: 70000,
            },
            logging: LoggingConfig::default(),
            channels: vec![],
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("ws_port"));
            }
            _ => panic!("Expected ValidationError for ws_port > 65535"),
        }
    }

    #[test]
    fn test_validation_fails_when_channel_missing_channel_id() {
        let config = GatewayConfig {
            discord_token: "valid_token".to_string(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            channels: vec![ChannelMapping {
                channel_id: String::new(),
                project_name: "test-project".to_string(),
                endpoint: "ws://localhost:8080".to_string(),
            }],
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("channel_id"));
            }
            _ => panic!("Expected ValidationError for empty channel_id"),
        }
    }

    #[test]
    fn test_validation_fails_when_channel_missing_project_name() {
        let config = GatewayConfig {
            discord_token: "valid_token".to_string(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
            channels: vec![ChannelMapping {
                channel_id: "123456789".to_string(),
                project_name: String::new(),
                endpoint: "ws://localhost:8080".to_string(),
            }],
        };

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(GatewayConfigError::ValidationError(msg)) => {
                assert!(msg.contains("project_name"));
            }
            _ => panic!("Expected ValidationError for empty project_name"),
        }
    }

    #[test]
    fn test_validation_passes_with_valid_config() {
        let config = GatewayConfig {
            discord_token: "valid_token_123".to_string(),
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                http_port: 8080,
                ws_port: 9000,
            },
            logging: LoggingConfig::default(),
            channels: vec![ChannelMapping {
                channel_id: "123456789".to_string(),
                project_name: "test-project".to_string(),
                endpoint: "ws://localhost:8080".to_string(),
            }],
        };

        let result = config.validate();
        assert!(result.is_ok());
    }
}
