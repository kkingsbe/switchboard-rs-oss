//! Test utilities for creating mock ApiState instances.
//!
//! This module provides builders for creating ApiState instances
//! with test-specific configurations for use in unit and integration tests.

use crate::api::state::ApiState;
use crate::config::{ApiConfig, Config, Settings};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

/// Builder for creating test ApiState instances.
///
/// # Example
///
/// ```ignore
/// use switchboard::api::tests::TestApiStateBuilder;
///
/// // Create a basic test state
/// let state = TestApiStateBuilder::new()
///     .with_instance_id("test-instance")
///     .build();
///
/// // Create a state with full config
/// let state_with_config = TestApiStateBuilder::new()
///     .with_test_instance()
///     .with_config(switchboard_config)
///     .build();
/// ```
#[derive(Debug)]
pub struct TestApiStateBuilder {
    /// API configuration
    config: ApiConfig,
    /// Optional switchboard config
    switchboard_config: Option<Config>,
    /// Optional config file path
    config_path: Option<PathBuf>,
    /// Optional temp directory for instance
    temp_dir: Option<TempDir>,
    /// Instance ID (overrides config)
    instance_id: Option<String>,
}

impl Default for TestApiStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TestApiStateBuilder {
    /// Create a new builder with default test configuration.
    pub fn new() -> Self {
        Self {
            config: ApiConfig {
                enabled: true,
                instance_id: None,
                port: 18500,
                host: "127.0.0.1".to_string(),
                auto_port: false,
                swagger: false,
                rate_limit: crate::config::RateLimitConfig::default(),
            },
            switchboard_config: None,
            config_path: None,
            temp_dir: None,
            instance_id: None,
        }
    }

    /// Create a builder pre-configured with a test instance.
    ///
    /// This sets up:
    /// - Instance ID: "test-instance"
    /// - Test port: 18501
    /// - Temp directory for instance files
    pub fn with_test_instance(mut self) -> Self {
        self.instance_id = Some("test-instance".to_string());
        self.config.port = 18501;
        self.config.instance_id = Some("test-instance".to_string());

        // Create a temp directory for the instance
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        self.temp_dir = Some(temp_dir);

        self
    }

    /// Set the API configuration.
    pub fn with_config(mut self, config: ApiConfig) -> Self {
        self.config = config;
        self
    }

    /// Set a custom port.
    pub fn with_port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// Set a custom host.
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }

    /// Set the instance ID.
    pub fn with_instance_id(mut self, instance_id: impl Into<String>) -> Self {
        let id = instance_id.into();
        self.instance_id = Some(id.clone());
        self.config.instance_id = Some(id);
        self
    }

    /// Set the switchboard configuration.
    pub fn with_switchboard_config(mut self, config: Config) -> Self {
        self.switchboard_config = Some(config);
        self
    }

    /// Set a minimal switchboard config for testing.
    ///
    /// Creates a config with minimal required fields.
    pub fn with_minimal_switchboard_config(mut self) -> Self {
        let config = Config {
            agents: vec![],
            settings: Some(Settings::default()),
            #[cfg(feature = "discord")]
            discord: None,
            api: None,
            config_path: PathBuf::new(),
        };
        self.switchboard_config = Some(config);
        self
    }

    /// Set the config file path.
    pub fn with_config_path(mut self, path: PathBuf) -> Self {
        self.config_path = Some(path);
        self
    }

    /// Enable swagger UI.
    pub fn with_swagger(mut self) -> Self {
        self.config.swagger = true;
        self
    }

    /// Build the ApiState instance.
    ///
    /// temp directory was set up, it will be kept alive for the
    /// lifetime of the returned state (via Arc).
    pub fn build(self) -> ApiState {
        let instance_id = self.instance_id.unwrap_or_else(|| {
            format!("test-instance-{}", self.config.port)
        });

        // Create instance directories in temp dir if available
        let (instance_dir, instance_log_dir, instance_pid_file, instance_metrics_file) =
            if let Some(ref temp_dir) = self.temp_dir {
                let base = temp_dir.path().to_path_buf();
                (
                    base.join("instance"),
                    base.join("logs"),
                    base.join("pid"),
                    base.join("metrics"),
                )
            } else {
                // Use the registry functions to get standard paths
                let dir = crate::api::registry::get_instance_dir(&instance_id);
                let log_dir = crate::api::registry::get_instance_log_dir(&instance_id);
                let pid_file = crate::api::registry::get_instance_pid_file(&instance_id);
                let metrics_file = crate::api::registry::get_instance_metrics_file(&instance_id);
                (dir, log_dir, pid_file, metrics_file)
            };

        ApiState {
            config: self.config,
            instance_id,
            switchboard_config: self.switchboard_config,
            config_path: self.config_path,
            instance_dir,
            instance_log_dir,
            instance_pid_file,
            instance_metrics_file,
        }
    }

    /// Build the ApiState wrapped in Arc.
    ///
    /// This is useful for sharing state across handlers.
    pub fn build_arc(self) -> Arc<ApiState> {
        Arc::new(self.build())
    }
}

/// Create a test ApiState with minimal configuration.
///
/// This is a convenience function for quick test setup.
pub fn create_test_state() -> ApiState {
    TestApiStateBuilder::new()
        .with_test_instance()
        .build()
}

/// Create a test ApiState with a specific instance ID.
pub fn create_test_state_with_id(instance_id: &str) -> ApiState {
    TestApiStateBuilder::new()
        .with_instance_id(instance_id)
        .build()
}

/// Create a test ApiState with switchboard config.
pub fn create_test_state_with_config(config: Config) -> ApiState {
    TestApiStateBuilder::new()
        .with_test_instance()
        .with_switchboard_config(config)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let state = TestApiStateBuilder::new().build();
        assert_eq!(state.config.port, 18500);
        assert!(state.switchboard_config.is_none());
    }

    #[test]
    fn test_builder_with_test_instance() {
        let state = TestApiStateBuilder::new()
            .with_test_instance()
            .build();
        assert_eq!(state.instance_id, "test-instance");
        assert_eq!(state.config.port, 18501);
    }

    #[test]
    fn test_builder_with_custom_instance_id() {
        let state = TestApiStateBuilder::new()
            .with_instance_id("custom-id")
            .build();
        assert_eq!(state.instance_id, "custom-id");
        assert_eq!(state.config.instance_id, Some("custom-id".to_string()));
    }

    #[test]
    fn test_builder_with_port() {
        let state = TestApiStateBuilder::new()
            .with_port(19000)
            .build();
        assert_eq!(state.config.port, 19000);
    }

    #[test]
    fn test_create_test_state() {
        let state = create_test_state();
        assert_eq!(state.instance_id, "test-instance");
    }

    #[test]
    fn test_build_arc() {
        let state = TestApiStateBuilder::new()
            .with_test_instance()
            .build_arc();
        assert!(Arc::strong_count(&state) > 0);
    }
}
