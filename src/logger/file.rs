//! File module - Handles log file operations for the switchboard scheduler
//!
//! This module provides the FileWriter struct which manages log file paths and
//! operations for both scheduler and agent logs.
//!
//! # Log File Organization
//! - Scheduler logs: `<log_dir>/switchboard.log`
//! - Agent logs: `<log_dir>/<agent-name>/<timestamp>.log`

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Maximum number of log files to keep per agent
const MAX_LOG_FILES_PER_AGENT: usize = 10;

/// Error that occurs when writing to a log file
#[derive(Debug, Error)]
#[error("Failed to write to log file: {0}")]
pub struct FileWriteError(#[from] io::Error);

/// Error that occurs during path construction
#[derive(Debug, Error)]
pub enum PathConstructionError {
    /// Invalid agent name provided
    #[error("Invalid agent name: '{0}'")]
    InvalidAgentName(String),

    /// Failed to construct path
    #[error("Failed to construct path: {0}")]
    PathConstructionFailed(String),
}

/// FileWriter handles log file path construction and file operations
///
/// This struct is responsible for managing the base log directory and constructing
/// appropriate paths for both scheduler and agent log files.
#[derive(Debug, Clone)]
pub struct FileWriter {
    /// The base directory where logs should be written (e.g., ".switchboard/logs")
    log_dir: PathBuf,
}

impl FileWriter {
    /// Create a new FileWriter instance
    ///
    /// # Arguments
    /// * `log_dir` - The base directory where logs should be written
    ///
    /// # Returns
    /// A new FileWriter instance
    pub fn new(log_dir: impl AsRef<Path>) -> Self {
        Self {
            log_dir: log_dir.as_ref().to_path_buf(),
        }
    }

    /// Get the path to the scheduler log file
    ///
    /// # Returns
    /// A PathBuf pointing to `<log_dir>/switchboard.log`
    fn get_scheduler_log_path(&self) -> PathBuf {
        self.log_dir.join("switchboard.log")
    }

    /// Get the directory for agent logs
    ///
    /// # Arguments
    /// * `agent_name` - The name of the agent
    ///
    /// # Returns
    /// A PathBuf pointing to `<log_dir>/<agent-name>/`
    ///
    /// # Errors
    /// Returns PathConstructionError if the agent name is invalid
    fn get_agent_log_dir(&self, agent_name: &str) -> Result<PathBuf, PathConstructionError> {
        // Validate agent name - reject empty strings and names with path separators
        if agent_name.is_empty() {
            return Err(PathConstructionError::InvalidAgentName(
                "Agent name cannot be empty".to_string(),
            ));
        }

        if agent_name.contains('/') || agent_name.contains("\\") {
            return Err(PathConstructionError::InvalidAgentName(format!(
                "Agent name contains invalid characters: '{}'",
                agent_name
            )));
        }

        Ok(self.log_dir.join(agent_name))
    }

    /// Create the agent log directory if it doesn't exist
    ///
    /// # Arguments
    /// * `agent_name` - The name of the agent
    ///
    /// # Returns
    /// Ok(()) on success, PathConstructionError on failure
    pub fn create_agent_log_directory(
        &self,
        agent_name: &str,
    ) -> Result<(), PathConstructionError> {
        let log_dir = self.get_agent_log_dir(agent_name)?;
        fs::create_dir_all(&log_dir).map_err(|e| {
            PathConstructionError::PathConstructionFailed(format!(
                "Failed to create directory '{}': {}",
                log_dir.display(),
                e
            ))
        })?;
        Ok(())
    }

    /// Get the path to an agent's log file with a timestamp
    ///
    /// # Arguments
    /// * `agent_name` - The name of the agent
    ///
    /// # Returns
    /// A PathBuf pointing to `<log_dir>/<agent-name>/<timestamp>.log`
    ///
    /// # Errors
    /// Returns PathConstructionError if path construction fails
    pub fn get_agent_log_path(&self, agent_name: &str) -> Result<PathBuf, PathConstructionError> {
        let log_dir = self.get_agent_log_dir(agent_name)?;
        let timestamp = generate_timestamp();
        Ok(log_dir.join(format!("{}.log", timestamp)))
    }

    /// Open or create a file for appending
    ///
    /// # Arguments
    /// * `path` - The path to the file
    ///
    /// # Returns
    /// A File handle opened for appending
    ///
    /// # Errors
    /// Returns FileWriteError if the file cannot be opened or created
    fn open_or_append_file(path: &Path) -> Result<fs::File, FileWriteError> {
        fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .map_err(FileWriteError::from)
    }

    /// Write a message to the scheduler log file
    ///
    /// # Arguments
    /// * `message` - The message to write
    ///
    /// # Returns
    /// Ok(()) on success, FileWriteError on failure
    ///
    /// # Errors
    /// Returns FileWriteError if the file cannot be opened or the message cannot be written
    pub fn write_scheduler_log(&self, message: &str) -> Result<(), FileWriteError> {
        let path = self.get_scheduler_log_path();
        let mut file = Self::open_or_append_file(&path)?;
        let mut content = message.to_string();
        content.push('\n');
        io::Write::write_all(&mut file, content.as_bytes()).map_err(FileWriteError::from)?;
        Ok(())
    }

    /// Write a message to an agent's log file
    ///
    /// # Arguments
    /// * `agent_name` - The name of the agent
    /// * `message` - The message to write
    ///
    /// # Returns
    /// Ok(()) on success, FileWriteError on failure
    ///
    /// # Errors
    /// Returns FileWriteError if the path cannot be constructed, the directory cannot be created,
    /// the file cannot be opened, or the message cannot be written
    pub fn write_agent_log(&self, agent_name: &str, message: &str) -> Result<(), FileWriteError> {
        // Ensure the agent log directory exists before attempting to write
        self.create_agent_log_directory(agent_name)
            .map_err(|e| FileWriteError::from(io::Error::new(io::ErrorKind::InvalidInput, e)))?;

        let path = self
            .get_agent_log_path(agent_name)
            .map_err(|e| FileWriteError::from(io::Error::new(io::ErrorKind::InvalidInput, e)))?;
        let mut file = Self::open_or_append_file(&path)?;
        let mut content = message.to_string();
        content.push('\n');
        io::Write::write_all(&mut file, content.as_bytes()).map_err(FileWriteError::from)?;

        // Rotate logs after writing
        if let Err(e) = self.rotate_logs(agent_name) {
            tracing::warn!("Failed to rotate logs for agent '{}': {}", agent_name, e);
            // Continue anyway - log rotation failure is not fatal
        }

        Ok(())
    }

    /// Rotate log files for an agent, keeping only the most recent ones
    ///
    /// # Arguments
    /// * `agent_name` - The name of the agent
    ///
    /// # Returns
    /// Ok(()) on success, FileWriteError on failure
    ///
    /// # Errors
    /// Returns FileWriteError if the log directory cannot be read or files cannot be removed
    pub fn rotate_logs(&self, agent_name: &str) -> Result<(), FileWriteError> {
        let log_dir = self
            .get_agent_log_dir(agent_name)
            .map_err(|e| FileWriteError::from(io::Error::new(io::ErrorKind::InvalidInput, e)))?;

        // If log directory doesn't exist, nothing to rotate
        if !log_dir.exists() {
            return Ok(());
        }

        // Read all entries in the log directory
        let entries = match fs::read_dir(&log_dir) {
            Ok(e) => e,
            Err(e) => {
                tracing::debug!(
                    "Failed to read log directory '{}': {}",
                    log_dir.display(),
                    e
                );
                return Ok(());
            }
        };

        // Collect log files with their metadata
        let mut log_files: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tracing::debug!("Failed to read directory entry: {}", e);
                    continue;
                }
            };

            let path = entry.path();
            // Only process .log files
            if path.extension().and_then(|s| s.to_str()) == Some("log") {
                let metadata = match fs::metadata(&path) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::debug!("Failed to get metadata for '{}': {}", path.display(), e);
                        continue;
                    }
                };

                let modified = match metadata.modified() {
                    Ok(t) => t,
                    Err(e) => {
                        tracing::debug!(
                            "Failed to get modified time for '{}': {}",
                            path.display(),
                            e
                        );
                        continue;
                    }
                };

                log_files.push((path, modified));
            }
        }

        // Sort by modification time (oldest first)
        log_files.sort_by_key(|(_, time)| *time);

        // Remove oldest logs if count exceeds MAX_LOG_FILES_PER_AGENT
        if log_files.len() > MAX_LOG_FILES_PER_AGENT {
            let num_to_remove = log_files.len() - MAX_LOG_FILES_PER_AGENT;
            tracing::debug!(
                "Rotating logs for agent '{}': {} files, removing {} oldest",
                agent_name,
                log_files.len(),
                num_to_remove
            );

            for (path, _) in log_files.into_iter().take(num_to_remove) {
                tracing::debug!("Removing old log file: {}", path.display());
                if let Err(e) = fs::remove_file(&path) {
                    tracing::warn!("Failed to remove old log file '{}': {}", path.display(), e);
                }
            }
        }

        Ok(())
    }
}

/// Generate an ISO 8601 timestamp for log file naming
///
/// # Returns
/// A string containing the current UTC time in ISO 8601-like format
/// with second precision but using underscores instead of colons
/// for cross-platform compatibility (e.g., "2026-02-12T05_40_21Z")
pub fn generate_timestamp() -> String {
    let now = chrono::Utc::now();
    // Use RFC 3339 format but replace colons with underscores
    // Colons are invalid characters in Windows file names
    now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        .replace(':', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Tests for path construction

    #[test]
    fn test_filewriter_new() {
        let log_dir = "/tmp/test_logs";
        let writer = FileWriter::new(log_dir);
        assert_eq!(writer.log_dir, PathBuf::from(log_dir));
    }

    #[test]
    fn test_get_scheduler_log_path() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();
        let writer = FileWriter::new(log_dir);
        let path = writer.get_scheduler_log_path();
        assert_eq!(path, log_dir.join("switchboard.log"));
    }

    #[test]
    fn test_get_agent_log_dir() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();
        let writer = FileWriter::new(log_dir);
        let agent_name = "test-agent";
        let path = writer.get_agent_log_dir(agent_name).unwrap();
        assert_eq!(path, log_dir.join(agent_name));
    }

    #[test]
    fn test_get_agent_log_dir_invalid_name() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();
        let writer = FileWriter::new(log_dir);

        // Test empty agent name
        let result = writer.get_agent_log_dir("");
        assert!(result.is_err());
        match result.unwrap_err() {
            PathConstructionError::InvalidAgentName(msg) => {
                assert!(msg.contains("Agent name cannot be empty"));
            }
            _ => panic!("Expected InvalidAgentName error"),
        }

        // Test agent name with forward slash
        let result = writer.get_agent_log_dir("agent/name");
        assert!(result.is_err());
        match result.unwrap_err() {
            PathConstructionError::InvalidAgentName(msg) => {
                assert!(msg.contains("contains invalid characters"));
            }
            _ => panic!("Expected InvalidAgentName error"),
        }

        // Test agent name with backslash
        let result = writer.get_agent_log_dir(r"agent\name");
        assert!(result.is_err());
        match result.unwrap_err() {
            PathConstructionError::InvalidAgentName(msg) => {
                assert!(msg.contains("contains invalid characters"));
            }
            _ => panic!("Expected InvalidAgentName error"),
        }
    }

    // Tests for directory creation

    #[test]
    fn test_create_agent_log_directory() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();
        let writer = FileWriter::new(log_dir);
        let agent_name = "test-agent";

        // Create the directory
        writer.create_agent_log_directory(agent_name).unwrap();

        // Verify the directory exists
        let expected_path = log_dir.join(agent_name);
        assert!(expected_path.exists());
        assert!(expected_path.is_dir());
    }

    // Tests for log writing

    #[test]
    fn test_write_scheduler_log() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();
        let writer = FileWriter::new(log_dir);
        let message = "Test scheduler log message";

        // Write the log
        writer.write_scheduler_log(message).unwrap();

        // Verify the file exists
        let log_path = writer.get_scheduler_log_path();
        assert!(log_path.exists());

        // Verify the file contains the message
        let contents = fs::read_to_string(&log_path).unwrap();
        assert!(contents.contains(message));
        assert!(contents.ends_with('\n'));
    }

    #[test]
    fn test_write_agent_log() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();
        let writer = FileWriter::new(log_dir);
        let agent_name = "test-agent";
        let message = "Test agent log message";

        // Write the log WITHOUT explicitly creating the directory first
        // The implementation should create the directory automatically
        writer.write_agent_log(agent_name, message).unwrap();

        // Verify the log directory was created and contains files
        let agent_dir = log_dir.join(agent_name);
        assert!(agent_dir.exists());

        // Find the created log file
        let entries = fs::read_dir(&agent_dir).unwrap();
        let log_files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        assert_eq!(log_files.len(), 1);

        // Verify the file contains the message
        let log_file = log_files[0].path();
        let contents = fs::read_to_string(&log_file).unwrap();
        assert!(contents.contains(message));
        assert!(contents.ends_with('\n'));
    }

    #[test]
    fn test_write_multiple_agent_logs() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();
        let writer = FileWriter::new(log_dir);
        let agent_name = "multi-agent";
        let messages = vec!["First message", "Second message", "Third message"];

        // Write multiple messages WITHOUT explicitly creating the directory first
        // The implementation should create the directory automatically
        for msg in &messages {
            writer.write_agent_log(agent_name, msg).unwrap();
        }

        // Verify messages are present in log files
        let agent_dir = log_dir.join(agent_name);
        let entries = fs::read_dir(&agent_dir).unwrap();
        let log_files: Vec<_> = entries.filter_map(|e| e.ok()).collect();

        // With second precision, messages written within the same second overwrite the same file
        // So we expect at least 1 file
        assert!(!log_files.is_empty(), "Should have at least 1 log file");

        // Sort files by path to ensure consistent ordering
        let mut log_files: Vec<_> = log_files;
        log_files.sort_by_key(|a| a.path());

        // Verify each message is present in at least one log file
        for msg in &messages {
            let mut msg_found = false;
            for log_file in &log_files {
                let contents = fs::read_to_string(log_file.path()).unwrap();
                if contents.contains(msg) {
                    msg_found = true;
                    break;
                }
            }
            assert!(msg_found, "Message '{}' not found in any log file", msg);
        }
    }

    // Tests for timestamp generation

    #[test]
    fn test_generate_timestamp_format() {
        let timestamp = generate_timestamp();

        // Verify basic structure - should contain key components
        assert!(
            timestamp.contains('T'),
            "Timestamp should contain 'T' separator"
        );
        assert!(
            timestamp.ends_with('Z') || timestamp.contains('+'),
            "Timestamp should end with 'Z' or contain timezone offset"
        );

        // Verify the timestamp format has a reasonable length (YYYY-MM-DDTHH-MM-SSZ)
        assert!(
            timestamp.len() >= 20,
            "Timestamp should be at least 20 characters"
        );

        // Verify timestamp does not contain colons (for Windows compatibility)
        assert!(
            !timestamp.contains(':'),
            "Timestamp should not contain colons for Windows file name compatibility"
        );
    }

    // Tests for error handling

    #[test]
    fn test_write_to_invalid_path() {
        // Create a FileWriter with an invalid path that we can't write to
        // We'll use a path with a component that cannot be created on most systems
        let invalid_path = "/nonexistent/protected/path/that/cannot/be/created";
        let writer = FileWriter::new(invalid_path);

        let message = "Test message";
        let result = writer.write_scheduler_log(message);

        // Should return an error
        assert!(result.is_err());
        // Verify it's a FileWriteError
        let _error = result.unwrap_err();
    }
}
