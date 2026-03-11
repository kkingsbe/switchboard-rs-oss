// Observability Module - Core Event Infrastructure
// 
// This module provides the core event infrastructure for observability.
// It includes:
// - Event struct: represents a single event with id, timestamp, event_type, and payload
// - EventData enum: contains different event types with their payloads
// - EventEmitter: writes events to a file for persistence
// - Log rotation: automatic rotation with size threshold and retention

pub mod error;
pub mod event;
pub mod emitter;

pub use error::EventError;
pub use event::{CommitInfo, Event, EventData, EventType};
pub use emitter::{EmitterConfig, EventEmitter, DEFAULT_RETENTION_DAYS, DEFAULT_ROTATION_SIZE_THRESHOLD};
