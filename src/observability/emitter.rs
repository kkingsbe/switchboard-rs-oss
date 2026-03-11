//! Event Emitter for persisting events to files
//!
//! This module provides the EventEmitter which handles writing events
//! to a file for persistence and later analysis.

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use crate::observability::error::EventError;
use crate::observability::event::Event;

/// Configuration for the EventEmitter
#[derive(Debug, Clone)]
pub struct EmitterConfig {
    /// Path to the output file
    pub file_path: PathBuf,
    /// Whether to append to existing file or overwrite
    pub append: bool,
    /// Whether to flush after each write
    pub auto_flush: bool,
}

impl EmitterConfig {
    /// Create a new configuration with the given file path
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
            append: true,
            auto_flush: true,
        }
    }

    /// Set whether to append to existing file
    pub fn with_append(mut self, append: bool) -> Self {
        self.append = append;
        self
    }

    /// Set whether to flush after each write
    pub fn with_auto_flush(mut self, auto_flush: bool) -> Self {
        self.auto_flush = auto_flush;
        self
    }
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self::new("events.jsonl")
    }
}

/// Event emitter for writing events to a file
///
/// # Features
/// - JSON Lines format (one JSON object per line)
/// - Configurable append/overwrite behavior
/// - Automatic or manual flushing
/// - Thread-safe write operations
///
/// # Example
/// ```rust
/// use switchboard::observability::{Event, EventType, EventData, EventEmitter, EmitterConfig};
///
/// let config = EmitterConfig::new("events.jsonl").with_auto_flush(true);
/// let emitter = EventEmitter::new(config).unwrap();
/// let event = Event::new(EventType::AgentStarted, EventData::agent("agent-1"));
/// emitter.emit(event).unwrap();
/// ```
#[derive(Debug)]
pub struct EventEmitter {
    file: File,
    writer: BufWriter<File>,
    config: EmitterConfig,
}

impl EventEmitter {
    /// Create a new EventEmitter with the given configuration
    pub fn new(config: EmitterConfig) -> Result<Self, EventError> {
        let file = Self::open_file(&config)?;
        let writer = BufWriter::new(file.try_clone()?);

        Ok(Self { file, writer, config })
    }

    /// Open the file based on configuration
    fn open_file(config: &EmitterConfig) -> Result<File, EventError> {
        // Ensure parent directory exists
        if let Some(parent) = config.file_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| EventError::FileOpenError(e.to_string()))?;
            }
        }

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(config.append)
            .truncate(!config.append)
            .open(&config.file_path)
            .map_err(|e| EventError::FileOpenError(e.to_string()))?;

        Ok(file)
    }

    /// Emit an event to the file
    pub fn emit(&mut self, event: Event) -> Result<(), EventError> {
        // Validate the event before emitting
        event.validate()?;

        // Serialize to JSON (JSON Lines format - one JSON per line)
        let json = event.to_json()?;
        writeln!(self.writer, "{}", json).map_err(EventError::WriteError)?;

        // Flush if auto_flush is enabled
        if self.config.auto_flush {
            self.flush()?;
        }

        Ok(())
    }

    /// Emit multiple events at once
    pub fn emit_batch(&mut self, events: impl IntoIterator<Item = Event>) -> Result<(), EventError> {
        for event in events {
            self.emit(event)?;
        }
        Ok(())
    }

    /// Flush the writer buffer to the file
    pub fn flush(&mut self) -> Result<(), EventError> {
        self.writer.flush().map_err(EventError::WriteError)?;
        self.file.sync_all().map_err(EventError::WriteError)?;
        Ok(())
    }

    /// Get the file path being written to
    pub fn file_path(&self) -> &PathBuf {
        &self.config.file_path
    }

    /// Get the number of events written (approximate, based on file position)
    pub fn events_written(&self) -> Result<u64, EventError> {
        // This is a rough estimate - we could track this more precisely
        let metadata = self.file.metadata().map_err(EventError::WriteError)?;
        // Each line is roughly the size of an event, average ~200 bytes
        Ok(metadata.len() / 200)
    }
}

impl Write for EventEmitter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::{Event, EventData, EventType};
    use tempfile::TempDir;

    fn create_temp_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[test]
    fn emitter_new_should_create_file_if_not_exists() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        let config = EmitterConfig::new(&file_path).with_append(false);
        let emitter = EventEmitter::new(config);
        
        assert!(emitter.is_ok());
        assert!(file_path.exists());
    }

    #[test]
    fn emitter_new_should_create_parent_directories() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("nested").join("dir").join("events.jsonl");
        
        let config = EmitterConfig::new(&file_path).with_append(false);
        let emitter = EventEmitter::new(config);
        
        assert!(emitter.is_ok());
        assert!(file_path.exists());
    }

    #[test]
    fn emitter_emit_should_write_valid_jsonl() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        let mut config = EmitterConfig::new(&file_path).with_append(false);
        let mut emitter = EventEmitter::new(config.clone()).expect("Failed to create emitter");
        
        let event = Event::new(EventType::AgentStarted, EventData::agent("agent-1"));
        emitter.emit(event).expect("Failed to emit event");
        emitter.flush().expect("Failed to flush");
        
        // Read the file and verify content
        let content = std::fs::read_to_string(&file_path).expect("Failed to read file");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);
        
        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(lines[0]).expect("Invalid JSON");
        assert!(parsed.get("id").is_some());
        assert!(parsed.get("timestamp").is_some());
        assert!(parsed.get("event_type").is_some());
    }

    #[test]
    fn emitter_emit_multiple_should_write_multiple_lines() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        let mut config = EmitterConfig::new(&file_path).with_append(false);
        let mut emitter = EventEmitter::new(config.clone()).expect("Failed to create emitter");
        
        let events = vec![
            Event::new(EventType::AgentStarted, EventData::agent("agent-1")),
            Event::new(EventType::TaskStarted, EventData::task("task-1")),
            Event::new(EventType::WorkflowStarted, EventData::workflow("wf-1")),
        ];
        
        emitter.emit_batch(events).expect("Failed to emit batch");
        emitter.flush().expect("Failed to flush");
        
        let content = std::fs::read_to_string(&file_path).expect("Failed to read file");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn emitter_append_should_append_to_existing_file() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        // First write
        {
            let mut config = EmitterConfig::new(&file_path).with_append(false);
            let mut emitter = EventEmitter::new(config).expect("Failed to create emitter");
            let event = Event::new(EventType::AgentStarted, EventData::agent("agent-1"));
            emitter.emit(event).expect("Failed to emit");
            emitter.flush().expect("Failed to flush");
        }
        
        // Second write with append
        {
            let mut config = EmitterConfig::new(&file_path).with_append(true);
            let mut emitter = EventEmitter::new(config).expect("Failed to create emitter");
            let event = Event::new(EventType::AgentStopped, EventData::agent("agent-1"));
            emitter.emit(event).expect("Failed to emit");
            emitter.flush().expect("Failed to flush");
        }
        
        let content = std::fs::read_to_string(&file_path).expect("Failed to read file");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn emitter_without_append_should_overwrite_file() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        // First write
        {
            let mut config = EmitterConfig::new(&file_path).with_append(false);
            let mut emitter = EventEmitter::new(config).expect("Failed to create emitter");
            let event = Event::new(EventType::AgentStarted, EventData::agent("agent-1"));
            emitter.emit(event).expect("Failed to emit");
            emitter.flush().expect("Failed to flush");
        }
        
        // Second write without append (overwrite)
        {
            let mut config = EmitterConfig::new(&file_path).with_append(false);
            let mut emitter = EventEmitter::new(config).expect("Failed to create emitter");
            let event = Event::new(EventType::AgentStopped, EventData::agent("agent-1"));
            emitter.emit(event).expect("Failed to emit");
            emitter.flush().expect("Failed to flush");
        }
        
        let content = std::fs::read_to_string(&file_path).expect("Failed to read file");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn emitter_validate_before_emit_should_reject_invalid_event() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        let mut config = EmitterConfig::new(&file_path).with_append(false);
        let mut emitter = EventEmitter::new(config).expect("Failed to create emitter");
        
        // Create event with invalid payload (empty agent_id)
        let event = Event::new(
            EventType::AgentStarted, 
            EventData::Agent { 
                agent_id: String::new(), 
                name: None, 
                metadata: None 
            }
        );
        
        let result = emitter.emit(event);
        assert!(result.is_err());
    }

    #[test]
    fn emitter_file_path_should_return_configured_path() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        let config = EmitterConfig::new(&file_path);
        let emitter = EventEmitter::new(config).expect("Failed to create emitter");
        
        assert_eq!(emitter.file_path(), &file_path);
    }

    #[test]
    fn emitter_config_default_should_have_sensible_defaults() {
        let config = EmitterConfig::default();
        
        assert_eq!(config.file_path, PathBuf::from("events.jsonl"));
        assert!(config.append);
        assert!(config.auto_flush);
    }

    #[test]
    fn emitter_config_builder_should_work_correctly() {
        let config = EmitterConfig::new("/custom/path/events.jsonl")
            .with_append(false)
            .with_auto_flush(false);
        
        assert_eq!(config.file_path, PathBuf::from("/custom/path/events.jsonl"));
        assert!(!config.append);
        assert!(!config.auto_flush);
    }

    #[test]
    fn emitter_events_written_should_return_estimate() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        let mut config = EmitterConfig::new(&file_path).with_append(false);
        let mut emitter = EventEmitter::new(config).expect("Failed to create emitter");
        
        let event = Event::new(EventType::AgentStarted, EventData::agent("agent-1"));
        emitter.emit(event).expect("Failed to emit");
        emitter.flush().expect("Failed to flush");
        
        let count = emitter.events_written().expect("Failed to get count");
        assert!(count >= 1);
    }

    #[test]
    fn emitter_write_and_read_roundtrip_should_preserve_event() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        // Create and emit event
        let original_event = Event::new(EventType::TaskCompleted, EventData::Task {
            task_id: "task-001".to_string(),
            name: Some("Test Task".to_string()),
            duration_ms: Some(1500),
            result: Some("success".to_string()),
        });
        
        {
            let mut emitter = EventEmitter::new(
                EmitterConfig::new(&file_path).with_append(false)
            ).expect("Failed to create emitter");
            emitter.emit(original_event.clone()).expect("Failed to emit");
            emitter.flush().expect("Failed to flush");
        }
        
        // Read and parse the event back
        let content = std::fs::read_to_string(&file_path).expect("Failed to read file");
        let lines: Vec<&str> = content.lines().collect();
        
        let parsed_event: Event = serde_json::from_str(lines[0]).expect("Failed to parse JSON");
        
        assert_eq!(original_event.id, parsed_event.id);
        assert_eq!(original_event.event_type, parsed_event.event_type);
    }
}
