//! Cross-platform process control for gateway management.
//!
//! This module provides a unified interface for process control operations
//! that work on both Unix and Windows systems.

use std::process::Command;
use thiserror::Error;

#[cfg(windows)]
use windows::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE, PROCESS_VM_READ,
};

/// Error types for process control operations.
#[derive(Debug, Error)]
pub enum ProcessError {
    /// Failed to query process status.
    #[error("Failed to query process: {0}")]
    QueryFailed(String),

    /// Failed to terminate process.
    #[error("Failed to terminate process: {0}")]
    TerminateFailed(String),

    /// Process does not exist.
    #[error("Process does not exist")]
    NotFound,
}

/// Cross-platform process control.
/// This enum provides a unified interface for process control operations.
pub enum ProcessController {
    /// Unix process control.
    Unix(UnixProcess),
    /// Windows process control.
    Windows(WindowsProcess),
}

impl ProcessController {
    /// Check if a process is running.
    pub fn is_running(&self, pid: u32) -> bool {
        match self {
            ProcessController::Unix(_) => UnixProcess::is_running(pid),
            ProcessController::Windows(_) => WindowsProcess::is_running(pid),
        }
    }

    /// Send terminate signal to a process.
    pub fn terminate(&self, pid: u32) -> Result<(), ProcessError> {
        match self {
            ProcessController::Unix(_) => UnixProcess::terminate(pid),
            ProcessController::Windows(_) => WindowsProcess::terminate(pid),
        }
    }

    /// Force kill a process (Unix only).
    #[cfg(unix)]
    pub fn force_kill(&self, pid: u32) -> Result<(), ProcessError> {
        match self {
            ProcessController::Unix(_) => UnixProcess::force_kill(pid),
            ProcessController::Windows(_) => {
                // On Windows, terminate is already forceful
                Ok(())
            }
        }
    }
}

/// Unix process controller.
pub struct UnixProcess;

impl UnixProcess {
    /// Check if a process is running (Unix).
    pub fn is_running(pid: u32) -> bool {
        // Use kill -0 to check if process exists without sending a signal
        let result = Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output();

        match result {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    /// Send terminate signal to a process (Unix).
    pub fn terminate(pid: u32) -> Result<(), ProcessError> {
        // First try graceful SIGTERM
        let result = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .output();

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(ProcessError::TerminateFailed(e.to_string())),
        }
    }

    /// Force kill a process (Unix).
    pub fn force_kill(pid: u32) -> Result<(), ProcessError> {
        let result = Command::new("kill")
            .arg("-9")
            .arg(pid.to_string())
            .output();

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(ProcessError::TerminateFailed(e.to_string())),
        }
    }
}

/// Windows process controller.
pub struct WindowsProcess;

#[cfg(windows)]
impl WindowsProcess {
    /// Check if a process is running (Windows).
    pub fn is_running(pid: u32) -> bool {
        unsafe {
            // Try to open the process with QUERY_INFORMATION access
            let handle = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                false,
                pid,
            );

            match handle {
                Ok(h) => {
                    // Process exists, close the handle
                    let _ = CloseHandle(h);
                    true
                }
                Err(_) => false,
            }
        }
    }

    /// Terminate a process (Windows).
    pub fn terminate(pid: u32) -> Result<(), ProcessError> {
        unsafe {
            // Open process with terminate access
            let handle = OpenProcess(PROCESS_TERMINATE, false, pid);

            match handle {
                Ok(h) => {
                    // Try to terminate the process
                    let result = TerminateProcess(h, 1);
                    let _ = CloseHandle(h);

                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(ProcessError::TerminateFailed(e.to_string())),
                    }
                }
                Err(e) => {
                    if e.code().0 == 87 {
                        // ERROR_INVALID_PARAMETER - process doesn't exist
                        Err(ProcessError::NotFound)
                    } else {
                        Err(ProcessError::TerminateFailed(e.to_string()))
                    }
                }
            }
        }
    }
}

#[cfg(not(windows))]
impl WindowsProcess {
    /// Check if a process is running (non-Windows stub).
    pub fn is_running(_pid: u32) -> bool {
        false
    }

    /// Terminate a process (non-Windows stub).
    pub fn terminate(_pid: u32) -> Result<(), ProcessError> {
        Err(ProcessError::TerminateFailed(
            "Windows process control not available on this platform".to_string(),
        ))
    }
}

/// Get the appropriate process controller for the current platform.
pub fn get_process_control() -> ProcessController {
    #[cfg(unix)]
    {
        ProcessController::Unix(UnixProcess)
    }

    #[cfg(not(unix))]
    {
        ProcessController::Windows(WindowsProcess)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_control_exists() {
        // This is just a compile-time test to ensure the trait is implemented
        let _unix = UnixProcess;
        let _windows = WindowsProcess;
    }
}
