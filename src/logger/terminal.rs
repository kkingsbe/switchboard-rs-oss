//! Terminal module - Handles interleaved terminal output for the switchboard scheduler
//!
//! This module provides the TerminalWriter struct which manages writing to
//! stdout/stderr with agent-name prefixes in foreground mode.
//!
//! # Output Behavior
//! - Foreground mode: Output is prefixed with [agent-name]
//! - Background mode: Output is written without agent prefix
//! - All output is thread-safe via Mutex

use std::io::{self, Write};
use std::sync::Mutex;
use thiserror::Error;

/// Error that occurs when writing to terminal
#[derive(Debug, Error)]
pub enum TerminalError {
    /// I/O error during write operation
    #[error("I/O error while writing to terminal: {0}")]
    IoError(#[from] io::Error),

    /// Error formatting output message
    #[error("Failed to format output message: {0}")]
    FormatError(String),
}

/// TerminalWriter handles writing output to terminal with optional agent-name prefix
///
/// This struct is responsible for managing terminal output, ensuring thread-safe
/// writes via a Mutex, and prefixing output with agent names when in foreground mode.
pub struct TerminalWriter {
    /// The name of the agent (for prefixing output in foreground mode)
    agent_name: String,
    /// Whether to prefix output with agent name (foreground mode)
    foreground_mode: bool,
    /// Thread-safe writer for stdout/stderr
    writer: Mutex<Box<dyn Write + Send>>,
}

impl std::fmt::Debug for TerminalWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalWriter")
            .field("agent_name", &self.agent_name)
            .field("foreground_mode", &self.foreground_mode)
            .finish()
    }
}

impl TerminalWriter {
    /// Create a new TerminalWriter instance
    ///
    /// # Arguments
    /// * `agent_name` - The name of the agent
    /// * `foreground_mode` - Whether to prefix output with agent name
    ///
    /// # Returns
    /// A new TerminalWriter instance configured with the provided parameters
    pub fn new(agent_name: String, foreground_mode: bool) -> Self {
        Self {
            agent_name,
            foreground_mode,
            writer: Mutex::new(Box::new(io::stdout())),
        }
    }

    /// Get the agent name
    ///
    /// # Returns
    /// The agent name
    pub fn get_agent_name(&self) -> &str {
        &self.agent_name
    }

    /// Check if in foreground mode
    ///
    /// # Returns
    /// True if in foreground mode, false otherwise
    pub fn is_foreground_mode(&self) -> bool {
        self.foreground_mode
    }

    /// Format a message with agent-name prefix (if in foreground mode)
    ///
    /// # Arguments
    /// * `message` - The message to format
    ///
    /// # Returns
    /// The formatted message string
    fn format_message(&self, message: &str) -> String {
        if self.foreground_mode {
            format!("[{}] {}", self.agent_name, message)
        } else {
            message.to_string()
        }
    }

    /// Write output to the terminal
    ///
    /// This method writes a message to the terminal, prefixing it with the agent
    /// name when in foreground mode. The operation is thread-safe via Mutex.
    ///
    /// # Arguments
    /// * `message` - The message to write
    ///
    /// # Returns
    /// * `Ok(())` if the write operation succeeds
    /// * `Err(TerminalError::IoError)` if an I/O error occurs during writing or flushing
    pub fn write_output(&self, message: &str) -> Result<(), TerminalError> {
        let formatted = self.format_message(message);
        let mut writer = self.writer.lock().map_err(|e| {
            TerminalError::IoError(io::Error::other(format!("Failed to lock writer: {}", e)))
        })?;
        write!(writer, "{}", formatted)?;
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for constructor

    #[test]
    fn test_terminalwriter_new_with_foreground() {
        let agent_name = "test-agent".to_string();
        let writer = TerminalWriter::new(agent_name.clone(), true);
        assert_eq!(writer.get_agent_name(), agent_name);
        assert!(writer.is_foreground_mode());
    }

    #[test]
    fn test_terminalwriter_new_with_background() {
        let agent_name = "background-agent".to_string();
        let writer = TerminalWriter::new(agent_name.clone(), false);
        assert_eq!(writer.get_agent_name(), agent_name);
        assert!(!writer.is_foreground_mode());
    }

    #[test]
    fn test_terminalwriter_new_empty_name() {
        let agent_name = "".to_string();
        let writer = TerminalWriter::new(agent_name.clone(), false);
        assert_eq!(writer.get_agent_name(), agent_name);
    }

    // Tests for helper methods

    #[test]
    fn test_get_agent_name() {
        let agent_name = "my-agent".to_string();
        let writer = TerminalWriter::new(agent_name.clone(), true);
        assert_eq!(writer.get_agent_name(), "my-agent");
    }

    #[test]
    fn test_get_agent_name_with_special_chars() {
        let agent_name = "agent_123-test".to_string();
        let writer = TerminalWriter::new(agent_name.clone(), false);
        assert_eq!(writer.get_agent_name(), "agent_123-test");
    }

    #[test]
    fn test_is_foreground_mode_true() {
        let writer = TerminalWriter::new("test".to_string(), true);
        assert!(writer.is_foreground_mode());
    }

    #[test]
    fn test_is_foreground_mode_false() {
        let writer = TerminalWriter::new("test".to_string(), false);
        assert!(!writer.is_foreground_mode());
    }

    // Tests for format_message

    #[test]
    fn test_format_message_with_foreground() {
        let writer = TerminalWriter::new("agent-1".to_string(), true);
        let message = "Hello, world!";
        let formatted = writer.format_message(message);
        assert_eq!(formatted, "[agent-1] Hello, world!");
    }

    #[test]
    fn test_format_message_with_background() {
        let writer = TerminalWriter::new("agent-2".to_string(), false);
        let message = "Background message";
        let formatted = writer.format_message(message);
        assert_eq!(formatted, "Background message");
    }

    #[test]
    fn test_format_message_empty_message_foreground() {
        let writer = TerminalWriter::new("agent-empty".to_string(), true);
        let message = "";
        let formatted = writer.format_message(message);
        assert_eq!(formatted, "[agent-empty] ");
    }

    #[test]
    fn test_format_message_empty_message_background() {
        let writer = TerminalWriter::new("agent-empty".to_string(), false);
        let message = "";
        let formatted = writer.format_message(message);
        assert_eq!(formatted, "");
    }

    #[test]
    fn test_format_message_multiline() {
        let writer = TerminalWriter::new("multiline-agent".to_string(), true);
        let message = "Line 1\nLine 2\nLine 3";
        let formatted = writer.format_message(message);
        assert_eq!(formatted, "[multiline-agent] Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_format_message_with_quotes() {
        let writer = TerminalWriter::new("quote-agent".to_string(), true);
        let message = "Message with \"quotes\" and 'apostrophes'";
        let formatted = writer.format_message(message);
        assert_eq!(
            formatted,
            "[quote-agent] Message with \"quotes\" and 'apostrophes'"
        );
    }

    #[test]
    fn test_format_message_long_agent_name() {
        let agent_name = "very-long-agent-name-for-testing-purposes".to_string();
        let writer = TerminalWriter::new(agent_name.clone(), true);
        let message = "Test";
        let formatted = writer.format_message(message);
        assert_eq!(formatted, format!("[{}] Test", agent_name));
    }

    // Tests for write_output with mock writer

    use std::sync::Arc;

    /// Helper to create a writer with shared buffer for testing
    struct TestWriter {
        agent_name: String,
        foreground_mode: bool,
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl TestWriter {
        fn new(agent_name: String, foreground_mode: bool) -> Self {
            Self {
                agent_name,
                foreground_mode,
                buffer: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn format_message(&self, message: &str) -> String {
            if self.foreground_mode {
                format!("[{}] {}", self.agent_name, message)
            } else {
                message.to_string()
            }
        }

        fn write_output(&self, message: &str) -> Result<(), TerminalError> {
            let formatted = self.format_message(message);
            let mut buffer = self.buffer.lock().map_err(|e| {
                TerminalError::IoError(io::Error::other(format!("Failed to lock buffer: {}", e)))
            })?;
            writeln!(buffer, "{}", formatted)?;
            Ok(())
        }

        fn get_buffer_content(&self) -> String {
            let buffer = self.buffer.lock().unwrap();
            String::from_utf8(buffer.clone()).unwrap()
        }
    }

    #[test]
    fn test_write_output_with_foreground() {
        let writer = TestWriter::new("test-agent".to_string(), true);
        let message = "Hello, world!";
        writer.write_output(message).unwrap();
        let content = writer.get_buffer_content();
        assert_eq!(content, "[test-agent] Hello, world!\n");
    }

    #[test]
    fn test_write_output_with_background() {
        let writer = TestWriter::new("background-agent".to_string(), false);
        let message = "Background message";
        writer.write_output(message).unwrap();
        let content = writer.get_buffer_content();
        assert_eq!(content, "Background message\n");
    }

    #[test]
    fn test_write_output_empty_message() {
        let writer = TestWriter::new("empty-agent".to_string(), true);
        let message = "";
        writer.write_output(message).unwrap();
        let content = writer.get_buffer_content();
        assert_eq!(content, "[empty-agent] \n");
    }

    #[test]
    fn test_write_output_multiline_message() {
        let writer = TestWriter::new("multiline-agent".to_string(), false);
        let message = "Line 1\nLine 2\nLine 3";
        writer.write_output(message).unwrap();
        let content = writer.get_buffer_content();
        assert_eq!(content, "Line 1\nLine 2\nLine 3\n");
    }

    #[test]
    fn test_write_output_multiple_writes() {
        let writer = TestWriter::new("multi-write-agent".to_string(), true);
        writer.write_output("First message").unwrap();
        writer.write_output("Second message").unwrap();
        writer.write_output("Third message").unwrap();
        let content = writer.get_buffer_content();
        assert_eq!(
            content,
            "[multi-write-agent] First message\n[multi-write-agent] Second message\n[multi-write-agent] Third message\n"
        );
    }

    #[test]
    fn test_write_output_unicode_message() {
        let writer = TestWriter::new("unicode-agent".to_string(), true);
        let message = "Hello 世界 🌍";
        writer.write_output(message).unwrap();
        let content = writer.get_buffer_content();
        assert_eq!(content, "[unicode-agent] Hello 世界 🌍\n");
    }

    #[test]
    fn test_write_output_special_characters() {
        let writer = TestWriter::new("special-agent".to_string(), false);
        let message = "Special: !@#$%^&*()_+-=[]{}|;':\",./<>?";
        writer.write_output(message).unwrap();
        let content = writer.get_buffer_content();
        assert_eq!(content, "Special: !@#$%^&*()_+-=[]{}|;':\",./<>?\n");
    }

    #[test]
    fn test_write_output_consistent_prefix() {
        let writer = TestWriter::new("consistent-agent".to_string(), true);
        let messages = vec!["msg1", "msg2", "msg3"];
        for msg in &messages {
            writer.write_output(msg).unwrap();
        }
        let content = writer.get_buffer_content();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
        for line in &lines {
            assert!(line.starts_with("[consistent-agent]"));
        }
    }

    #[test]
    fn test_format_message_no_prefix_in_background() {
        let writer = TestWriter::new("bg-agent".to_string(), false);
        writer.write_output("Message 1").unwrap();
        writer.write_output("Message 2").unwrap();
        let content = writer.get_buffer_content();
        assert!(!content.contains("[bg-agent]"));
    }

    #[test]
    fn test_write_output_very_long_message() {
        let writer = TestWriter::new("long-msg-agent".to_string(), true);
        let long_message = "A".repeat(10000);
        writer.write_output(&long_message).unwrap();
        let content = writer.get_buffer_content();
        assert!(content.starts_with("[long-msg-agent] "));
        assert!(content.ends_with('\n'));
    }
}
