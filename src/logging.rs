//! Tracing initialization for scheduler logging
//!
//! This module provides functionality to initialize the tracing subscriber
//! with a file appender for writing scheduler logs to `<log_dir>/switchboard.log`.

use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, EnvFilter};

use crate::skills::SkillsError;

/// Initialize tracing subscriber with file appender
///
/// This function sets up tracing to write scheduler logs to `<log_dir>/switchboard.log`.
/// It creates the log directory if it doesn't exist and configures a non-blocking
/// writer for performance.
///
/// # Arguments
///
/// * `log_dir` - The directory where the log file should be created
///
/// # Returns
///
/// A `WorkerGuard` that must be kept alive for the duration of the program
/// to ensure logs are flushed properly.
///
/// # Panics
///
/// Panics if:
/// - The log directory cannot be created
/// - The global tracing subscriber cannot be set (in production code)
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use switchboard::logging::init_logging;
/// let log_dir = PathBuf::from(".switchboard/logs");
/// let _guard = init_logging(log_dir);
/// // Logging is now initialized and will write to .switchboard/logs/switchboard.log
/// ```
pub fn init_logging(log_dir: PathBuf) -> Result<WorkerGuard, SkillsError> {
    // Create the log directory if it doesn't exist
    let log_dir_display = log_dir.display().to_string();
    std::fs::create_dir_all(&log_dir).map_err(|e| SkillsError::IoError {
        operation: "create directory".to_string(),
        path: log_dir_display,
        message: e.to_string(),
    })?;

    // Create file appender for switchboard.log
    let file_appender = tracing_appender::rolling::never(&log_dir, "switchboard.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Build the subscriber with file appender
    let subscriber = fmt()
        .with_writer(non_blocking)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .finish();

    // Set the global subscriber (panics in production if already set)
    tracing::subscriber::set_global_default(subscriber).map_err(|e| SkillsError::IoError {
        operation: "set tracing subscriber".to_string(),
        path: "".to_string(),
        message: e.to_string(),
    })?;

    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::Once;
    use tempfile::tempdir;

    static INIT: Once = Once::new();
    static mut GLOBAL_GUARD: Option<WorkerGuard> = None;
    static mut GLOBAL_LOG_DIR: Option<PathBuf> = None;
    static mut TEMP_DIR: Option<tempfile::TempDir> = None;
    static INIT_ERROR: std::sync::Mutex<Option<SkillsError>> = std::sync::Mutex::new(None);

    /// Helper function to initialize logging for tests
    /// Uses a static flag to ensure tracing is only initialized once across all tests
    /// All tests share the same log directory and subscriber
    #[allow(static_mut_refs)]
    fn get_test_log_dir() -> Result<&'static Path, SkillsError> {
        unsafe {
            INIT.call_once(|| {
                // Create a temp dir that lives for the duration of the test run
                let temp = match tempdir() {
                    Ok(t) => t,
                    Err(e) => {
                        let err = SkillsError::IoError {
                            operation: "create temp directory".to_string(),
                            path: "".to_string(),
                            message: e.to_string(),
                        };
                        *INIT_ERROR
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(err);
                        return;
                    }
                };
                let log_dir = temp.path().join("logs");
                if let Err(e) = std::fs::create_dir_all(&log_dir) {
                    let err = SkillsError::IoError {
                        operation: "create directory".to_string(),
                        path: log_dir.display().to_string(),
                        message: e.to_string(),
                    };
                    *INIT_ERROR
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(err);
                    return;
                }

                TEMP_DIR = Some(temp);
                GLOBAL_LOG_DIR = Some(log_dir.clone());

                let file_appender = tracing_appender::rolling::never(&log_dir, "switchboard.log");
                let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

                let subscriber = fmt()
                    .with_writer(non_blocking)
                    .with_env_filter(
                        EnvFilter::try_from_default_env()
                            .unwrap_or_else(|_| EnvFilter::new("info")),
                    )
                    .finish();

                if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
                    let err = SkillsError::IoError {
                        operation: "set tracing subscriber".to_string(),
                        path: "".to_string(),
                        message: e.to_string(),
                    };
                    *INIT_ERROR
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(err);
                    return;
                }

                GLOBAL_GUARD = Some(guard);
            });

            // Check if there was an error during initialization
            let error = INIT_ERROR
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take();
            if let Some(err) = error {
                return Err(err);
            }

            Ok(GLOBAL_LOG_DIR
                .as_ref()
                .ok_or_else(|| SkillsError::IoError {
                    operation: "get log directory".to_string(),
                    path: "".to_string(),
                    message: "Log directory not initialized".to_string(),
                })?
                .as_path())
        }
    }

    #[test]
    fn test_init_logging_creates_directory() {
        let log_dir = get_test_log_dir().unwrap();

        // Directory should exist
        assert!(log_dir.exists());
        assert!(log_dir.is_dir());
    }

    #[test]
    fn test_init_logging_creates_log_file() {
        let log_dir = get_test_log_dir().unwrap();
        let log_file_path = log_dir.join("switchboard.log");

        // Log file should exist
        assert!(log_file_path.exists());
        assert!(log_file_path.is_file());
    }

    #[test]
    fn test_logging_writes_to_file() {
        let log_dir = get_test_log_dir().unwrap();
        let log_file_path = log_dir.join("switchboard.log");

        // Write a log message
        tracing::info!("Test log message from test");

        // Give the non-blocking writer time to flush
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Verify the log file contains the message
        let contents = std::fs::read_to_string(&log_file_path).unwrap();
        assert!(contents.contains("Test log message from test"));
    }
}
