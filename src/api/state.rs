//! API Application State.
//!
//! This module provides the application state for the REST API server,
//! including configuration and instance identification.

use crate::config::{ApiConfig, Config};
use crate::api::registry::{
    derive_instance_id_from_config, ensure_instance_dirs, get_instance_dir,
    get_instance_log_dir, get_instance_metrics_file, get_instance_pid_file,
};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

/// Application state for the Axum router.
///
/// This state is shared across all request handlers and contains
/// the API configuration, instance identification, and access to
/// the main configuration for agent management.
#[derive(Clone)]
pub struct ApiState {
    /// API server configuration.
    pub config: ApiConfig,
    /// Unique instance identifier.
    pub instance_id: String,
    /// Main switchboard configuration (contains agents, settings, etc.)
    pub switchboard_config: Option<Config>,
    /// Path to the switchboard config file
    pub config_path: Option<PathBuf>,
    /// Instance-specific data directory
    pub instance_dir: PathBuf,
    /// Instance-specific log directory
    pub instance_log_dir: PathBuf,
    /// Instance-specific PID file path
    pub instance_pid_file: PathBuf,
    /// Instance-specific metrics file path
    pub instance_metrics_file: PathBuf,
}

impl ApiState {
    /// Create a new API state.
    ///
    /// # Arguments
    ///
    /// * `config` - The API configuration.
    ///
    /// # Returns
    ///
    /// * `Self` - The new application state.
    pub fn new(config: ApiConfig) -> Self {
        // Derive instance_id from config or generate a default based on port
        let instance_id = config.instance_id.clone().unwrap_or_else(|| {
            format!("switchboard-{}", config.port)
        });

        // Set up instance-specific paths
        let instance_dir = get_instance_dir(&instance_id);
        let instance_log_dir = get_instance_log_dir(&instance_id);
        let instance_pid_file = get_instance_pid_file(&instance_id);
        let instance_metrics_file = get_instance_metrics_file(&instance_id);

        // Ensure instance directories exist
        if let Err(e) = ensure_instance_dirs(&instance_id) {
            warn!("Failed to create instance directories: {}", e);
        }

        Self {
            config,
            instance_id,
            switchboard_config: None,
            config_path: None,
            instance_dir,
            instance_log_dir,
            instance_pid_file,
            instance_metrics_file,
        }
    }

    /// Create a new API state with switchboard config.
    ///
    /// # Arguments
    ///
    /// * `api_config` - The API configuration.
    /// * `switchboard_config` - The main switchboard configuration.
    /// * `config_path` - Path to the config file.
    ///
    /// # Returns
    ///
    /// * `Self` - The new application state.
    pub fn new_with_config(
        api_config: ApiConfig,
        switchboard_config: Config,
        config_path: PathBuf,
    ) -> Self {
        // Derive instance_id from config: priority is explicit config > env var > derived from config path
        let config_path_str = config_path.to_string_lossy().to_string();
        let instance_id = api_config.instance_id.clone().unwrap_or_else(|| {
            derive_instance_id_from_config(&config_path_str)
        });

        info!("Creating API state for instance: {}", instance_id);

        // Set up instance-specific paths
        let instance_dir = get_instance_dir(&instance_id);
        let instance_log_dir = get_instance_log_dir(&instance_id);
        let instance_pid_file = get_instance_pid_file(&instance_id);
        let instance_metrics_file = get_instance_metrics_file(&instance_id);

        // Ensure instance directories exist
        if let Err(e) = ensure_instance_dirs(&instance_id) {
            warn!("Failed to create instance directories: {}", e);
        }

        Self {
            config: api_config,
            instance_id,
            switchboard_config: Some(switchboard_config),
            config_path: Some(config_path),
            instance_dir,
            instance_log_dir,
            instance_pid_file,
            instance_metrics_file,
        }
    }

    /// Create a new Arc-wrapped API state.
    ///
    /// # Arguments
    ///
    /// * `config` - The API configuration.
    ///
    /// # Returns
    ///
    /// * `Arc<Self>` - The new wrapped application state.
    pub fn new_arc(config: ApiConfig) -> Arc<Self> {
        Arc::new(Self::new(config))
    }

    /// Create a new Arc-wrapped API state with switchboard config.
    ///
    /// # Arguments
    ///
    /// * `api_config` - The API configuration.
    /// * `switchboard_config` - The main switchboard configuration.
    /// * `config_path` - Path to the config file.
    ///
    /// # Returns
    ///
    /// * `Arc<Self>` - The new wrapped application state.
    pub fn new_arc_with_config(
        api_config: ApiConfig,
        switchboard_config: Config,
        config_path: PathBuf,
    ) -> Arc<Self> {
        Arc::new(Self::new_with_config(api_config, switchboard_config, config_path))
    }

    /// Get the log directory path.
    ///
    /// Returns the instance-specific log directory if configured,
    /// otherwise falls back to the switchboard config setting or default.
    ///
    /// # Returns
    ///
    /// * `PathBuf` - The log directory path.
    pub fn log_dir(&self) -> PathBuf {
        // Prefer instance-specific log directory
        if self.instance_log_dir.exists() 
            || !self.instance_log_dir.to_string_lossy().starts_with(".switchboard/instances") {
            return self.instance_log_dir.clone();
        }

        // Fall back to switchboard config setting
        let log_dir = self
            .switchboard_config
            .as_ref()
            .and_then(|c| c.settings.as_ref())
            .map(|s| s.log_dir.as_str())
            // Default to instance-specific logs if config not available
            .unwrap_or_else(|| {
                let log_str = self.instance_log_dir.to_string_lossy();
                if log_str.starts_with(".switchboard/instances") {
                    ""
                } else {
                    ".switchboard/logs"
                }
            });
        
        if log_dir.is_empty() {
            self.instance_log_dir.clone()
        } else {
            PathBuf::from(log_dir)
        }
    }
}
