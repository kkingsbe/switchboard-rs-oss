//! Instance Registry Module.
//!
//! This module provides instance registry functionality for multi-instance support.
//! It tracks all running Switchboard instances and provides instance discovery
//! capabilities for orchestration tools.
//!
//! # Instance Registry
//!
//! The registry maintains a JSON file (`.switchboard/instances.json`) that tracks
//! all running instances with their metadata including:
//! - instance_id: Unique identifier for the instance
//! - port: API server port
//! - started_at: Timestamp when the instance started
//! - config_path: Path to the instance configuration file
//! - status: Current status (running, stopped)

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use tracing::{error, info, warn};

/// Path to the instances registry file
const INSTANCES_REGISTRY_FILE: &str = ".switchboard/instances.json";

/// Error types for instance registry operations.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// Failed to read the registry file.
    #[error("Failed to read registry: {0}")]
    ReadError(String),

    /// Failed to serialize/deserialize registry data.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Instance not found in registry.
    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    /// Failed to write registry file.
    #[error("Failed to write registry: {0}")]
    WriteError(String),
}

/// Status of a Switchboard instance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InstanceStatus {
    /// Instance is running.
    Running,
    /// Instance is stopped.
    Stopped,
    /// Instance is starting up.
    Starting,
    /// Instance is in an error state.
    Error,
}

impl Default for InstanceStatus {
    fn default() -> Self {
        InstanceStatus::Running
    }
}

/// Registration data for a single Switchboard instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceRegistration {
    /// Unique instance identifier.
    pub instance_id: String,
    /// API server port.
    pub port: u16,
    /// Host address for the API server.
    pub host: String,
    /// Timestamp when the instance started (ISO 8601 string).
    pub started_at: String,
    /// Path to the instance configuration file.
    pub config_path: String,
    /// Current status of the instance.
    pub status: InstanceStatus,
    /// Optional PID of the scheduler process.
    pub pid: Option<u32>,
}

impl InstanceRegistration {
    /// Create a new instance registration.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - Unique instance identifier.
    /// * `port` - API server port.
    /// * `host` - Host address.
    /// * `config_path` - Path to configuration file.
    ///
    /// # Returns
    ///
    /// * `Self` - New registration with running status.
    pub fn new(instance_id: String, port: u16, host: String, config_path: String) -> Self {
        Self {
            instance_id,
            port,
            host,
            started_at: chrono::Utc::now().to_rfc3339(),
            config_path,
            status: InstanceStatus::Running,
            pid: None,
        }
    }

    /// Create a new registration with PID.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - Unique instance identifier.
    /// * `port` - API server port.
    /// * `host` - Host address.
    /// * `config_path` - Path to configuration file.
    /// * `pid` - Process ID of the scheduler.
    ///
    /// # Returns
    ///
    /// * `Self` - New registration with running status.
    pub fn with_pid(
        instance_id: String,
        port: u16,
        host: String,
        config_path: String,
        pid: u32,
    ) -> Self {
        Self {
            instance_id,
            port,
            host,
            started_at: chrono::Utc::now().to_rfc3339(),
            config_path,
            status: InstanceStatus::Running,
            pid: Some(pid),
        }
    }
}

/// The instance registry containing all registered instances.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstanceRegistry {
    /// List of registered instances.
    #[serde(default)]
    pub instances: Vec<InstanceRegistration>,
}

impl InstanceRegistry {
    /// Create a new empty instance registry.
    ///
    /// # Returns
    ///
    /// * `Self` - New empty registry.
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
        }
    }

    /// Load the instance registry from disk.
    ///
    /// If the registry file doesn't exist, returns an empty registry.
    ///
    /// # Returns
    ///
    /// * `Result<Self, RegistryError>` - The loaded registry or an error.
    pub fn load() -> Result<Self, RegistryError> {
        let path = Self::registry_path();
        
        if !path.exists() {
            info!("No instance registry found, creating new one");
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            RegistryError::ReadError(e.to_string())
        })?;
        let registry: InstanceRegistry = serde_json::from_str(&content).map_err(|e| {
            RegistryError::SerializationError(e.to_string())
        })?;
        
        info!(
            "Loaded instance registry with {} instances",
            registry.instances.len()
        );
        
        Ok(registry)
    }

    /// Save the instance registry to disk.
    ///
    /// # Returns
    ///
    /// * `Result<(), RegistryError>` - Success or error.
    pub fn save(&self) -> Result<(), RegistryError> {
        let path = Self::registry_path();
        
        // Ensure the parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                RegistryError::WriteError(e.to_string())
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            RegistryError::SerializationError(e.to_string())
        })?;
        fs::write(&path, content).map_err(|e| {
            RegistryError::WriteError(e.to_string())
        })?;
        
        info!(
            "Saved instance registry with {} instances",
            self.instances.len()
        );
        
        Ok(())
    }

    /// Get the path to the instances registry file.
    ///
    /// # Returns
    ///
    /// * `PathBuf` - Path to the registry file.
    fn registry_path() -> PathBuf {
        PathBuf::from(INSTANCES_REGISTRY_FILE)
    }

    /// Register a new instance.
    ///
    /// If an instance with the same ID already exists, it will be updated.
    ///
    /// # Arguments
    ///
    /// * `registration` - The instance registration data.
    ///
    /// # Returns
    ///
    /// * `Result<(), RegistryError>` - Success or error.
    pub fn register(&mut self, registration: InstanceRegistration) -> Result<(), RegistryError> {
        let instance_id = registration.instance_id.clone();
        
        // Check if instance already exists
        if let Some(existing) = self.instances.iter_mut().find(|i| i.instance_id == instance_id) {
            // Update existing instance
            info!("Updating existing instance registration: {}", instance_id);
            *existing = registration;
        } else {
            // Add new instance
            info!("Registering new instance: {}", instance_id);
            self.instances.push(registration);
        }
        
        self.save()
    }

    /// Unregister an instance.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - The instance ID to unregister.
    ///
    /// # Returns
    ///
    /// * `Result<(), RegistryError>` - Success or error.
    pub fn unregister(&mut self, instance_id: &str) -> Result<(), RegistryError> {
        let initial_len = self.instances.len();
        self.instances.retain(|i| i.instance_id != instance_id);
        
        if self.instances.len() == initial_len {
            warn!("Instance not found in registry: {}", instance_id);
            return Err(RegistryError::InstanceNotFound(instance_id.to_string()));
        }
        
        info!("Unregistered instance: {}", instance_id);
        self.save()
    }

    /// Update the status of an instance.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - The instance ID to update.
    /// * `status` - The new status.
    ///
    /// # Returns
    ///
    /// * `Result<(), RegistryError>` - Success or error.
    pub fn update_status(
        &mut self,
        instance_id: &str,
        status: InstanceStatus,
    ) -> Result<(), RegistryError> {
        let instance = self
            .instances
            .iter_mut()
            .find(|i| i.instance_id == instance_id);
        
        match instance {
            Some(i) => {
                i.status = status;
                info!("Updated instance status: {} -> {:?}", instance_id, i.status);
                self.save()
            }
            None => Err(RegistryError::InstanceNotFound(instance_id.to_string())),
        }
    }

    /// Get an instance by ID.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - The instance ID to find.
    ///
    /// # Returns
    ///
    /// * `Option<&InstanceRegistration>` - The instance if found.
    pub fn get(&self, instance_id: &str) -> Option<&InstanceRegistration> {
        self.instances
            .iter()
            .find(|i| i.instance_id == instance_id)
    }

    /// Get all running instances.
    ///
    /// # Returns
    ///
    /// * `Vec<&InstanceRegistration>` - All running instances.
    pub fn running(&self) -> Vec<&InstanceRegistration> {
        self.instances
            .iter()
            .filter(|i| i.status == InstanceStatus::Running)
            .collect()
    }

    /// Get all registered instances.
    ///
    /// # Returns
    ///
    /// * `&Vec<InstanceRegistration>` - All registered instances.
    pub fn instances(&self) -> &Vec<InstanceRegistration> {
        &self.instances
    }

    /// Check if an instance is running.
    ///
    /// # Arguments
    ///
    /// * `instance_id` - The instance ID to check.
    ///
    /// # Returns
    ///
    /// * `bool` - True if the instance is running.
    pub fn is_running(&self, instance_id: &str) -> bool {
        self.instances
            .iter()
            .any(|i| i.instance_id == instance_id && i.status == InstanceStatus::Running)
    }

    /// Mark all instances as stopped (for cleanup).
    ///
    /// # Returns
    ///
    /// * `Result<(), RegistryError>` - Success or error.
    pub fn mark_all_stopped(&mut self) -> Result<(), RegistryError> {
        for instance in &mut self.instances {
            if instance.status == InstanceStatus::Running {
                instance.status = InstanceStatus::Stopped;
            }
        }
        self.save()
    }
}

/// Get the base directory for instance-specific data.
///
/// # Arguments
///
/// * `instance_id` - The instance identifier.
///
/// # Returns
///
/// * `PathBuf` - The instance data directory path.
pub fn get_instance_dir(instance_id: &str) -> PathBuf {
    PathBuf::from(".switchboard").join("instances").join(instance_id)
}

/// Get the log directory for an instance.
///
/// # Arguments
///
/// * `instance_id` - The instance identifier.
///
/// # Returns
///
/// * `PathBuf` - The instance log directory path.
pub fn get_instance_log_dir(instance_id: &str) -> PathBuf {
    get_instance_dir(instance_id).join("logs")
}

/// Get the PID file path for an instance.
///
/// # Arguments
///
/// * `instance_id` - The instance identifier.
///
/// # Returns
///
/// * `PathBuf` - The PID file path.
pub fn get_instance_pid_file(instance_id: &str) -> PathBuf {
    get_instance_dir(instance_id).join("scheduler.pid")
}

/// Get the metrics file path for an instance.
///
/// # Arguments
///
/// * `instance_id` - The instance identifier.
///
/// # Returns
///
/// * `PathBuf` - The metrics file path.
pub fn get_instance_metrics_file(instance_id: &str) -> PathBuf {
    get_instance_dir(instance_id).join("metrics.json")
}

/// Ensure instance directories exist.
///
/// # Arguments
///
/// * `instance_id` - The instance identifier.
///
/// # Returns
///
/// * `Result<(), RegistryError>` - Success or error.
pub fn ensure_instance_dirs(instance_id: &str) -> Result<(), RegistryError> {
    let instance_dir = get_instance_dir(instance_id);
    let log_dir = get_instance_log_dir(instance_id);
    
    fs::create_dir_all(instance_dir).map_err(|e| {
        RegistryError::WriteError(e.to_string())
    })?;
    fs::create_dir_all(log_dir).map_err(|e| {
        RegistryError::WriteError(e.to_string())
    })?;
    
    Ok(())
}

/// Derive instance ID from config file path.
///
/// If the config is at `dev/switchboard.toml`, the instance ID would be `switchboard-dev`
/// (or just `dev` if the config filename is not `switchboard.toml`).
///
/// # Arguments
///
/// * `config_path` - Path to the config file.
///
/// # Returns
///
/// * `String` - The derived instance ID.
pub fn derive_instance_id_from_config(config_path: &str) -> String {
    let path = PathBuf::from(config_path);
    
    // Get the parent directory and filename
    let parent = path.parent();
    let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("switchboard");
    
    // If config is just "switchboard.toml" in current dir, use "default"
    if parent.is_none() || parent == Some(PathBuf::from("").as_path()) {
        if filename == "switchboard" {
            return "default".to_string();
        }
        return filename.to_string();
    }
    
    // If in a subdirectory like "dev/switchboard.toml", combine them
    if let Some(parent) = parent {
        let parent_name = parent.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        if filename == "switchboard" {
            parent_name.to_string()
        } else {
            format!("{}-{}", parent_name, filename)
        }
    } else {
        filename.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_instance_id_from_config() {
        // Test basic case
        assert_eq!(
            derive_instance_id_from_config("switchboard.toml"),
            "default"
        );
        
        // Test subdirectory case
        assert_eq!(
            derive_instance_id_from_config("dev/switchboard.toml"),
            "dev"
        );
        
        // Test nested case
        assert_eq!(
            derive_instance_id_from_config("config/dev/switchboard.toml"),
            "dev"
        );
        
        // Test custom filename
        assert_eq!(
            derive_instance_id_from_config("prod/my-config.toml"),
            "prod-my-config"
        );
    }

    #[test]
    fn test_instance_registry_new() {
        let registry = InstanceRegistry::new();
        assert!(registry.instances.is_empty());
    }

    #[test]
    fn test_instance_registration_new() {
        let reg = InstanceRegistration::new(
            "test-instance".to_string(),
            18500,
            "127.0.0.1".to_string(),
            "switchboard.toml".to_string(),
        );
        
        assert_eq!(reg.instance_id, "test-instance");
        assert_eq!(reg.port, 18500);
        assert_eq!(reg.host, "127.0.0.1");
        assert_eq!(reg.status, InstanceStatus::Running);
    }
}
