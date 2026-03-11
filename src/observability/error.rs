//! Error types for the observability module

use thiserror::Error;

/// Wrapper type for JSON serialization errors
#[derive(Debug, Error)]
#[error("JSON serialization error: {0}")]
pub struct JsonSerializationError(pub serde_json::Error);

impl From<serde_json::Error> for JsonSerializationError {
    fn from(err: serde_json::Error) -> Self {
        JsonSerializationError(err)
    }
}

/// Errors that can occur when working with events
#[derive(Debug, Error)]
pub enum EventError {
    /// Failed to serialize event to JSON
    #[error("Failed to serialize event: {0}")]
    SerializationError(#[from] JsonSerializationError),

    /// Failed to deserialize event from JSON
    #[error("Failed to deserialize event: {0}")]
    DeserializationError(#[from] serde_json::Error),

    /// Failed to write event to file
    #[error("Failed to write event to file: {0}")]
    WriteError(#[from] std::io::Error),

    /// Failed to create or open file
    #[error("Failed to create or open file: {0}")]
    FileOpenError(String),

    /// Invalid event data
    #[error("Invalid event data: {0}")]
    InvalidData(String),

    /// Event validation failed
    #[error("Event validation failed: {0}")]
    ValidationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
