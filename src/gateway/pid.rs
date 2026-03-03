//! PID file management for the gateway server.
//!
//! This module provides functionality to manage a PID file that tracks
//! whether the gateway is currently running.

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Error types for PID file operations.
#[derive(Debug, Error)]
pub enum PidFileError {
    /// IO error while reading or writing the PID file.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Failed to parse the PID from the file.
    #[error("Failed to parse PID file")]
    ParseError,

    /// Another gateway process is already running.
    #[error("Gateway is already running (PID: {0})")]
    AlreadyRunning(u32),
}

/// Default path for the PID file.
pub const DEFAULT_PID_PATH: &str = ".switchboard/gateway.pid";

/// PID file manager.
///
/// This struct provides methods to create, check, and clean up PID files
/// for the gateway server.
pub struct PidFile;

impl PidFile {
    /// Write the current process ID to a file.
    ///
    /// Creates the PID file at the specified path containing the current
    /// process ID. If the parent directory doesn't exist, it will be created.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the PID file
    ///
    /// # Returns
    ///
    /// * `Ok(())` - PID file created successfully
    /// * `Err(PidFileError)` - If the file cannot be created or written
    ///
    /// # Example
    ///
    /// ```ignore
    /// use switchboard::gateway::pid::PidFile;
    ///
    /// PidFile::write_pid(Path::new(".switchboard/gateway.pid"))?;
    /// ```
    pub fn write_pid(path: &Path) -> Result<(), PidFileError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create or truncate the PID file
        let mut file = File::create(path)?;

        // Write the current process ID
        let pid = std::process::id();
        writeln!(file, "{}", pid)?;

        info!("Created PID file at {} with PID {}", path.display(), pid);
        debug!("PID file written successfully");

        Ok(())
    }

    /// Check if another gateway process is already running.
    ///
    /// Reads the PID file and checks if a process with that PID is still alive.
    /// Returns an error if the file exists and the process is running.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the PID file
    ///
    /// # Returns
    ///
    /// * `Ok(())` - No other gateway is running
    /// * `Err(PidFileError::AlreadyRunning)` - If another gateway is running
    /// * `Err(PidFileError::ParseError)` - If the PID file is malformed
    /// * `Err(PidFileError::IoError)` - If the file cannot be read
    ///
    /// # Example
    ///
    /// ```ignore
    /// use switchboard::gateway::pid::PidFile;
    ///
    /// match PidFile::check_existing(Path::new(".switchboard/gateway.pid")) {
    ///     Ok(()) => println!("No other gateway running"),
    ///     Err(PidFileError::AlreadyRunning(pid)) => {
    ///         println!("Gateway already running with PID {}", pid);
    ///     }
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn check_existing(path: &Path) -> Result<(), PidFileError> {
        // If the file doesn't exist, no other gateway is running
        if !path.exists() {
            debug!("No existing PID file found at {}", path.display());
            return Ok(());
        }

        // Read the PID from the file
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let pid_str = lines
            .next()
            .ok_or(PidFileError::ParseError)? // Empty file
            .map_err(|_| PidFileError::ParseError)?; // IO error

        let pid: u32 = pid_str
            .trim()
            .parse()
            .map_err(|_| PidFileError::ParseError)?;

        // Check if the process is still running
        // On Unix, we can check if the process exists by sending signal 0
        if pid_exists(pid) {
            warn!("Gateway already running with PID {}", pid);
            return Err(PidFileError::AlreadyRunning(pid));
        }

        // Process with that PID is not running, so we can proceed
        // The stale PID file will be overwritten
        debug!("Stale PID file found (PID {} no longer running)", pid);

        Ok(())
    }

    /// Remove the PID file on shutdown.
    ///
    /// This should be called during graceful shutdown to clean up the PID file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the PID file
    ///
    /// # Returns
    ///
    /// * `Ok(())` - PID file removed successfully or didn't exist
    /// * `Err(PidFileError)` - If the file cannot be removed
    ///
    /// # Example
    ///
    /// ```ignore
    /// use switchboard::gateway::pid::PidFile;
    ///
    /// PidFile::cleanup(Path::new(".switchboard/gateway.pid"))?;
    /// ```
    pub fn cleanup(path: &Path) -> Result<(), PidFileError> {
        if path.exists() {
            fs::remove_file(path)?;
            info!("Removed PID file at {}", path.display());
        } else {
            debug!("No PID file to clean up at {}", path.display());
        }

        Ok(())
    }

    /// Get the default PID file path.
    ///
    /// Returns a PathBuf with the default location for the PID file.
    ///
    /// # Returns
    ///
    /// * A PathBuf to `.switchboard/gateway.pid`
    ///
    /// # Example
    ///
    /// ```ignore
    /// use switchboard::gateway::pid::PidFile;
    ///
    /// let path = PidFile::default_path();
    /// ```
    pub fn default_path() -> std::path::PathBuf {
        std::path::PathBuf::from(DEFAULT_PID_PATH)
    }
}

/// Check if a process with the given PID exists.
///
/// On Unix systems, we send signal 0 to check if the process exists
/// without actually sending a signal.
#[cfg(unix)]
fn pid_exists(pid: u32) -> bool {
    // Use libc::kill with signal 0 to check if process exists
    // This doesn't actually send a signal, just checks if we have permission
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

/// Check if a process with the given PID exists.
///
/// On non-Unix systems (like Windows), we always return true to be conservative
/// and prevent potential issues.
#[cfg(not(unix))]
fn pid_exists(_pid: u32) -> bool {
    // On non-Unix platforms, we can't easily check if a process exists
    // Return true to be conservative and prevent starting multiple instances
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_temp_dir() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let pid_path = temp_dir.path().join("gateway.pid");
        (temp_dir, pid_path)
    }

    #[test]
    fn pid_file_creation_should_write_correct_pid() {
        let (_temp_dir, pid_path) = create_temp_dir();

        PidFile::write_pid(&pid_path).expect("Failed to write PID file");

        // Read the file and verify the PID
        let content = fs::read_to_string(&pid_path).expect("Failed to read PID file");
        let file_pid: u32 = content.trim().parse().expect("Failed to parse PID");
        let current_pid = std::process::id();

        assert_eq!(file_pid, current_pid);
    }

    #[test]
    fn check_existing_should_return_ok_for_nonexistent_file() {
        let (_temp_dir, pid_path) = create_temp_dir();

        // Path doesn't exist, should return Ok
        let result = PidFile::check_existing(&pid_path);
        assert!(result.is_ok());
    }

    #[test]
    fn check_existing_should_return_ok_for_stale_pid_file() {
        let (_temp_dir, pid_path) = create_temp_dir();

        // Write a PID that definitely doesn't exist (very high number)
        let fake_pid = 999999999u32;
        fs::write(&pid_path, format!("{}\n", fake_pid)).expect("Failed to write fake PID");

        // The process shouldn't exist, so check_existing should return Ok
        let result = PidFile::check_existing(&pid_path);
        // On Unix, if the PID doesn't exist, it should return Ok
        // On non-Unix, it may return AlreadyRunning due to conservative behavior
        #[cfg(unix)]
        assert!(result.is_ok());
    }

    #[test]
    fn cleanup_should_remove_existing_pid_file() {
        let (_temp_dir, pid_path) = create_temp_dir();

        // Create a PID file
        PidFile::write_pid(&pid_path).expect("Failed to write PID file");
        assert!(pid_path.exists());

        // Clean it up
        PidFile::cleanup(&pid_path).expect("Failed to cleanup PID file");
        assert!(!pid_path.exists());
    }

    #[test]
    fn cleanup_should_not_fail_for_nonexistent_file() {
        let (_temp_dir, pid_path) = create_temp_dir();

        // Path doesn't exist, cleanup should still succeed
        let result = PidFile::cleanup(&pid_path);
        assert!(result.is_ok());
    }

    #[test]
    fn default_path_should_return_correct_value() {
        let default_path = PidFile::default_path();
        assert_eq!(default_path, PathBuf::from(DEFAULT_PID_PATH));
    }
}
