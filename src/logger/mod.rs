//! Logger - Capture and aggregate logs from all agent runs
//!
//! This module will handle:
//! - Log file management
//! - Terminal interleaved output with agent-name prefixes
//! - Agent log writing to `<log_dir>/<agent-name>/<timestamp>.log`
//! - Scheduler log writing to <log_dir>/switchboard.log
//!
//! **Current Status:** Logger struct with terminal writer integration implemented

pub mod file;
pub mod terminal;

use std::path::PathBuf;

/// Logger struct for capturing and aggregating logs from agent runs
///
/// This structure provides a unified logging interface for the Switchboard scheduler,
/// handling both terminal output and file-based log persistence. It manages:
///
/// - **Terminal Output**: Real-time interleaved output with agent-name prefixes in foreground mode
/// - **File Persistence**: Agent logs written to `<log_dir>/<agent-name>/<timestamp>.log`
/// - **Scheduler Logs**: Scheduler logs written to `<log_dir>/switchboard.log`
///
/// The logger supports both agent-specific logging (when `agent_name` is `Some`) and
/// scheduler logging (when `agent_name` is `None`).
///
/// # Fields
///
/// - `log_dir` - The base directory where all logs should be written
/// - `agent_name` - The name of the agent (None for scheduler logs)
/// - `foreground_mode` - Whether to output to terminal in real-time with agent-name prefixes
/// - `terminal_writer` - Optional terminal writer for formatted output with agent prefixes
/// - `file_writer` - File writer for persisting logs to disk
///
/// # Example
///
/// ```no_run
/// use switchboard::logger::Logger;
/// use std::path::PathBuf;
///
/// // Create a logger for an agent with foreground mode enabled
/// let agent_logger = Logger::new(
///     PathBuf::from("./logs"),
///     Some("agent1".to_string()),
///     true,  // foreground_mode
/// );
///
/// // Write to terminal (with agent prefix) and file
/// agent_logger.write_terminal_output("Agent started").unwrap();
/// agent_logger.write_agent_log("agent1", "Processing task").unwrap();
///
/// // Create a logger for the scheduler (no agent name, no foreground output)
/// let scheduler_logger = Logger::new(
///     PathBuf::from("./logs"),
///     None,  // no agent name
///     false, // no foreground mode
/// );
/// ```
pub struct Logger {
    /// The base directory where logs should be written
    pub log_dir: PathBuf,
    /// The name of the agent (None for scheduler logs)
    pub agent_name: Option<String>,
    /// Whether to output to terminal in real-time
    pub foreground_mode: bool,
    /// Terminal writer for outputting with agent-name prefixes
    terminal_writer: Option<terminal::TerminalWriter>,
    /// File writer for writing agent logs to disk
    file_writer: file::FileWriter,
}

impl Logger {
    /// Create a new Logger instance
    ///
    /// # Arguments
    /// * `log_dir` - The base directory where logs should be written
    /// * `agent_name` - The name of the agent (None for scheduler logs)
    /// * `foreground_mode` - Whether to output to terminal in real-time
    ///
    /// # Returns
    /// A new Logger instance configured with the provided parameters
    pub fn new(log_dir: PathBuf, agent_name: Option<String>, foreground_mode: bool) -> Self {
        let terminal_writer = agent_name
            .clone()
            .map(|name| terminal::TerminalWriter::new(name, foreground_mode));
        Self {
            log_dir: log_dir.clone(),
            agent_name,
            foreground_mode,
            terminal_writer,
            file_writer: file::FileWriter::new(log_dir),
        }
    }

    /// Write output to the terminal with agent-name prefix (if configured)
    ///
    /// This method writes a message to the terminal, prefixing it with the agent
    /// name when in foreground mode. If no terminal writer is configured (e.g., for
    /// scheduler logs), this method returns Ok(()) without writing anything.
    ///
    /// # Arguments
    /// * `message` - The message to write
    ///
    /// # Returns
    /// * `Ok(())` if the write operation succeeds or if no terminal writer is configured
    /// * `Err(terminal::TerminalError)` if an error occurs during writing
    pub fn write_terminal_output(&self, message: &str) -> Result<(), terminal::TerminalError> {
        if let Some(writer) = &self.terminal_writer {
            writer.write_output(message)?;
        }
        Ok(())
    }

    /// Write a log message to the agent's log file
    ///
    /// This method writes a message to the agent's log file located at
    /// `<log_dir>/<agent-name>/<timestamp>.log`. If the agent directory does
    /// not exist, it will be created.
    ///
    /// # Arguments
    /// * `agent_name` - The name of the agent
    /// * `message` - The message to write
    ///
    /// # Returns
    /// * `Ok(())` if the write operation succeeds
    /// * `Err(file::FileWriteError)` if an error occurs during writing
    pub fn write_agent_log(
        &self,
        agent_name: &str,
        message: &str,
    ) -> Result<(), file::FileWriteError> {
        self.file_writer.write_agent_log(agent_name, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn logger_write_agent_log_test() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();

        // Create a Logger instance with the temporary directory as log_dir
        let logger = Logger::new(temp_dir.path().to_path_buf(), None, false);

        // Call write_agent_log with test agent name and message
        let agent_name = "test-agent";
        let message = "Test log message";

        // Write the log WITHOUT explicitly creating the directory first
        // The implementation should create the directory automatically
        let result = logger.write_agent_log(agent_name, message);

        // Verify no error was returned
        assert!(
            result.is_ok(),
            "write_agent_log should not return an error, got: {:?}",
            result
        );

        // Verify the agent directory was created
        let agent_dir = temp_dir.path().join(agent_name);
        assert!(agent_dir.exists(), "Agent directory should exist");
        assert!(agent_dir.is_dir(), "Agent directory should be a directory");

        // Find the created log file
        let entries = fs::read_dir(&agent_dir).unwrap();
        let log_files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        assert_eq!(log_files.len(), 1, "There should be exactly one log file");

        // Verify the log file has the correct extension
        let log_file = log_files[0].path();
        assert!(
            log_file.extension().unwrap_or_default() == "log",
            "Log file should have .log extension"
        );

        // Verify the file content is exactly the message with a newline
        let contents = fs::read_to_string(&log_file).unwrap();
        assert_eq!(
            contents, "Test log message\n",
            "File content should be exactly the message with a newline"
        );

        // TempDir is automatically cleaned up when it goes out of scope
    }
}
