// Observability Module - Core Event Infrastructure
// 
// This module provides the core event infrastructure for observability.
// It includes:
// - Event struct: represents a single event with id, timestamp, event_type, and payload
// - EventData enum: contains different event types with their payloads
// - EventEmitter: writes events to a file for persistence
// - Log rotation: automatic rotation with size threshold and retention
// - ObservabilityConfig: configuration for observability settings

pub mod error;
pub mod event;
pub mod emitter;
pub mod consumer;

pub use consumer::{ConsumerError, DerivedMetrics, EventConsumer, ReliabilityMetrics, ThroughputMetrics};
pub use error::EventError;
pub use event::{CommitInfo, Event, EventData, EventType};
pub use emitter::{EmitterConfig, EventEmitter, DEFAULT_RETENTION_DAYS, DEFAULT_ROTATION_SIZE_THRESHOLD};

use crate::config::ObservabilityConfig;

impl EmitterConfig {
    /// Create an EmitterConfig from ObservabilityConfig
    /// 
    /// This converts the TOML configuration into the emitter configuration.
    /// 
    /// # Arguments
    /// 
    /// * `obs_config` - The observability configuration from TOML
    /// 
    /// # Returns
    /// 
    /// * `Result<EmitterConfig, EventError>` - The emitter configuration, or error if parsing fails
    pub fn from_observability_config(
        obs_config: &ObservabilityConfig,
        config_path: &std::path::Path,
    ) -> Result<Self, EventError> {
        use crate::config::parse_log_size;
        
        let max_log_size = parse_log_size(&obs_config.max_log_size)
            .map_err(|e| EventError::ConfigError(e.to_string()))?;
        
        let file_path = config_path.join(&obs_config.event_log_dir).join("events.jsonl");
        
        Ok(EmitterConfig::new(file_path)
            .with_rotation_size_threshold(max_log_size)
            .with_retention_days(obs_config.retention_days)
            .with_rotation_enabled(obs_config.enabled))
    }
}
