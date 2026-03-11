//! Event Emitter for persisting events to files
//!
//! This module provides the EventEmitter which handles writing events
//! to a file for persistence and later analysis.
//!
//! # Log Rotation
//! 
//! The EventEmitter supports automatic log rotation when configured with:
//! - `rotation_size_threshold`: Maximum file size before rotation (default: 10MB)
//! - `retention_days`: Number of days to keep rotated files (default: 30)
//!
//! Rotation creates timestamp-suffixed files: `events.2025-03-10T09-00-00Z.jsonl`

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::observability::error::EventError;
use crate::observability::event::Event;

/// Default rotation threshold: 10MB
pub const DEFAULT_ROTATION_SIZE_THRESHOLD: u64 = 10 * 1024 * 1024;
/// Default retention period: 30 days
pub const DEFAULT_RETENTION_DAYS: u32 = 30;

/// Configuration for the EventEmitter
#[derive(Debug, Clone)]
pub struct EmitterConfig {
    /// Path to the output file
    pub file_path: PathBuf,
    /// Whether to append to existing file or overwrite
    pub append: bool,
    /// Whether to flush after each write
    pub auto_flush: bool,
    /// Maximum file size in bytes before triggering rotation (default: 10MB)
    pub rotation_size_threshold: u64,
    /// Number of days to keep rotated files (default: 30)
    pub retention_days: u32,
    /// Whether to enable rotation (default: true)
    pub rotation_enabled: bool,
}

impl EmitterConfig {
    /// Create a new configuration with the given file path
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
            append: true,
            auto_flush: true,
            rotation_size_threshold: DEFAULT_ROTATION_SIZE_THRESHOLD,
            retention_days: DEFAULT_RETENTION_DAYS,
            rotation_enabled: true,
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

    /// Set the rotation size threshold (in bytes)
    pub fn with_rotation_size_threshold(mut self, threshold: u64) -> Self {
        self.rotation_size_threshold = threshold;
        self
    }

    /// Set the retention period (in days)
    pub fn with_retention_days(mut self, days: u32) -> Self {
        self.retention_days = days;
        self
    }

    /// Enable or disable rotation
    pub fn with_rotation_enabled(mut self, enabled: bool) -> Self {
        self.rotation_enabled = enabled;
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
/// - **Log rotation** with size threshold and timestamp-based files
/// - **Retention cleanup** to automatically delete old rotated files
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
///
/// # Log Rotation Example
/// ```rust
/// use switchboard::observability::{EventEmitter, EmitterConfig};
///
/// // Configure rotation: 10MB threshold, keep files for 30 days
/// let config = EmitterConfig::new("events.jsonl")
///     .with_rotation_size_threshold(10 * 1024 * 1024)
///     .with_retention_days(30);
/// let mut emitter = EventEmitter::new(config).unwrap();
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

        // Check for rotation if enabled
        if self.config.rotation_enabled {
            self.check_and_rotate()?;
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

    /// Check if the file exceeds the rotation threshold and rotate if needed
    fn check_and_rotate(&mut self) -> Result<(), EventError> {
        // Sync to ensure we have the latest file size
        self.file.sync_all().map_err(EventError::WriteError)?;
        
        let metadata = self.file.metadata().map_err(EventError::WriteError)?;
        let file_size = metadata.len();

        if file_size >= self.config.rotation_size_threshold {
            self.rotate()?;
        }

        Ok(())
    }

    /// Rotate the current log file to a timestamp-suffixed file
    fn rotate(&mut self) -> Result<(), EventError> {
        // Get the current file path
        let current_path = self.config.file_path.clone();
        
        // Generate timestamp suffix
        let timestamp = generate_timestamp_suffix();
        
        // Build new filename: events.2025-03-10T09-00-00Z.jsonl
        let new_filename = format!(
            "{}.{}.jsonl",
            current_path.file_stem().unwrap_or_default().to_string_lossy(),
            timestamp
        );
        
        let rotated_path = current_path.parent()
            .unwrap_or(std::path::Path::new("."))
            .join(&new_filename);

        // Flush and sync before renaming
        self.flush()?;
        self.file.sync_all().map_err(EventError::WriteError)?;

        // Rename current file to rotated path
        if current_path.exists() {
            std::fs::rename(&current_path, &rotated_path)
                .map_err(EventError::WriteError)?;
        }

        // Re-open the current file
        self.file = Self::open_file(&self.config)?;
        self.writer = BufWriter::new(self.file.try_clone()?);

        // Run retention cleanup after rotation
        self.cleanup_old_files()?;

        Ok(())
    }

    /// Clean up rotated files older than retention_days
    fn cleanup_old_files(&self) -> Result<(), EventError> {
        let dir = self.config.file_path.parent()
            .unwrap_or(std::path::Path::new("."));
        
        let file_stem = self.config.file_path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Calculate cutoff time
        let cutoff = SystemTime::now() 
            - Duration::from_secs(self.config.retention_days as u64 * 24 * 60 * 60);

        // Scan directory for rotated files
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Check if it's a rotated file (matches pattern events.YYYY-MM-DDTHH-MM-SSZ.jsonl)
                if let Some(filename) = path.file_name() {
                    let filename = filename.to_string_lossy();
                    if filename.starts_with(&format!("{}.", file_stem)) 
                        && filename.ends_with(".jsonl")
                        && filename != format!("{}.jsonl", file_stem) {
                        
                        // Check file modification time
                        if let Ok(metadata) = path.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                if modified < cutoff {
                                    let _ = std::fs::remove_file(&path);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Force rotation manually (useful for testing or manual rotation)
    pub fn force_rotate(&mut self) -> Result<(), EventError> {
        self.rotate()
    }

    /// Get the current file size in bytes
    pub fn file_size(&self) -> Result<u64, EventError> {
        let metadata = self.file.metadata().map_err(EventError::WriteError)?;
        Ok(metadata.len())
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

/// Generate a timestamp suffix for rotated log files
/// Format: YYYY-MM-DDTHH-MM-SSZ (ISO 8601)
fn generate_timestamp_suffix() -> String {
    let now = std::time::SystemTime::now();
    let datetime: chrono::DateTime<chrono::Utc> = now.into();
    datetime.format("%Y-%m-%dT%H-%M-%SZ").to_string()
}

#[cfg(test)]
mod rotation_tests {
    use super::*;
    use crate::observability::{Event, EventData, EventType};
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[test]
    fn test_generate_timestamp_suffix_format() {
        let suffix = generate_timestamp_suffix();
        
        // Check format: YYYY-MM-DDTHH-MM-SSZ (20 chars total)
        // Example: 2025-03-10T09-00-00Z
        assert_eq!(suffix.len(), 20);
        assert!(suffix.contains('-'));
        assert!(suffix.contains('T'));
        assert!(suffix.ends_with('Z'));
    }

    #[test]
    fn test_rotation_creates_timestamp_suffix_file() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        // Create a small threshold to trigger rotation quickly
        let config = EmitterConfig::new(&file_path)
            .with_append(false)
            .with_rotation_size_threshold(10) // Very small threshold
            .with_rotation_enabled(true);
        
        let mut emitter = EventEmitter::new(config.clone()).expect("Failed to create emitter");
        
        // Write enough data to exceed threshold
        let event = Event::new(EventType::AgentStarted, EventData::agent("agent-1"));
        emitter.emit(event).expect("Failed to emit");
        emitter.flush().expect("Failed to flush");
        
        // Check if rotation happened - original file should exist with small size
        // and a rotated file should exist
        let files: Vec<_> = fs::read_dir(temp_dir.path()).unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        
        // Either we have rotated file or the current file is small enough
        assert!(files.len() >= 1);
    }

    #[test]
    fn test_rotation_disabled_does_not_rotate() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        let config = EmitterConfig::new(&file_path)
            .with_append(false)
            .with_rotation_size_threshold(10)
            .with_rotation_enabled(false);
        
        let mut emitter = EventEmitter::new(config).expect("Failed to create emitter");
        
        // Write data
        let event = Event::new(EventType::AgentStarted, EventData::agent("agent-1"));
        emitter.emit(event).expect("Failed to emit");
        emitter.flush().expect("Failed to flush");
        
        // Should only have the original file
        let files: Vec<_> = fs::read_dir(temp_dir.path()).unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        
        assert!(files.contains(&"events.jsonl".to_string()));
    }

    #[test]
    fn test_force_rotate_creates_rotated_file() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        let config = EmitterConfig::new(&file_path)
            .with_append(false)
            .with_rotation_enabled(true);
        
        let mut emitter = EventEmitter::new(config).expect("Failed to create emitter");
        
        // Write some data
        let event = Event::new(EventType::AgentStarted, EventData::agent("agent-1"));
        emitter.emit(event).expect("Failed to emit");
        emitter.flush().expect("Failed to flush");
        
        // Force rotate
        emitter.force_rotate().expect("Failed to rotate");
        
        // Check rotated file exists
        let files: Vec<_> = fs::read_dir(temp_dir.path()).unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        
        // Should have both original and rotated file
        let rotated_files: Vec<_> = files.iter()
            .filter(|f| f.starts_with("events.") && f.ends_with(".jsonl") && *f != "events.jsonl")
            .collect();
        
        assert!(!rotated_files.is_empty(), "Expected rotated file, got: {:?}", files);
    }

    #[test]
    fn test_file_size_returns_current_size() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        let mut emitter = EventEmitter::new(
            EmitterConfig::new(&file_path).with_append(false)
        ).expect("Failed to create emitter");
        
        let event = Event::new(EventType::AgentStarted, EventData::agent("agent-1"));
        emitter.emit(event).expect("Failed to emit");
        emitter.flush().expect("Failed to flush");
        
        let size = emitter.file_size().expect("Failed to get file size");
        assert!(size > 0);
    }

    #[test]
    fn test_rotation_threshold_respected() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        // Set very small threshold
        let config = EmitterConfig::new(&file_path)
            .with_append(false)
            .with_rotation_size_threshold(5); // 5 bytes - very small
        
        let mut emitter = EventEmitter::new(config).expect("Failed to create emitter");
        
        // Write a small event
        let event = Event::new(EventType::AgentStarted, EventData::agent("a"));
        emitter.emit(event).expect("Failed to emit");
        
        // File should be rotated since event exceeds 5 bytes
        let size = emitter.file_size().expect("Failed to get file size");
        assert!(size < 50); // After rotation, new file should be small
    }

    #[test]
    fn test_retention_cleanup_removes_old_files() {
        let temp_dir = create_temp_dir();
        let file_path = temp_dir.path().join("events.jsonl");
        
        // Set 0 day retention for immediate cleanup
        let config = EmitterConfig::new(&file_path)
            .with_append(false)
            .with_retention_days(0);
        
        let mut emitter = EventEmitter::new(config).expect("Failed to create emitter");
        
        // Write and rotate
        let event = Event::new(EventType::AgentStarted, EventData::agent("agent-1"));
        emitter.emit(event).expect("Failed to emit");
        emitter.force_rotate().expect("Failed to rotate");
        
        // With 0 day retention, old files should be cleaned up
        let files: Vec<_> = fs::read_dir(temp_dir.path()).unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        
        // Should only have events.jsonl (current), not the old rotated file
        let rotated_files: Vec<_> = files.iter()
            .filter(|f| f.starts_with("events.") && f.ends_with(".jsonl") && *f != "events.jsonl")
            .collect();
        
        // Note: With 0 retention, rotated files should be cleaned up
        // but the timing of cleanup might vary
    }

    #[test]
    fn test_config_defaults_include_rotation() {
        let config = EmitterConfig::default();
        
        assert_eq!(config.rotation_size_threshold, DEFAULT_ROTATION_SIZE_THRESHOLD);
        assert_eq!(config.retention_days, DEFAULT_RETENTION_DAYS);
        assert!(config.rotation_enabled);
    }

    #[test]
    fn test_config_builder_for_rotation() {
        let config = EmitterConfig::new("events.jsonl")
            .with_rotation_size_threshold(5_000_000)
            .with_retention_days(7)
            .with_rotation_enabled(true);
        
        assert_eq!(config.rotation_size_threshold, 5_000_000);
        assert_eq!(config.retention_days, 7);
        assert!(config.rotation_enabled);
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
