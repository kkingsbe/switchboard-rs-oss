//! Types for container exit status
//!
//! This module provides the ExitStatus struct for representing the result
//! of a Docker container execution, including:
//! - Exit code from the container
//! - Timeout status indicating if the container was killed due to timeout
//! - Termination signal indicating which signal was used to terminate the container
//!
//! Exit codes follow standard Unix conventions:
//! - 0: Success
//! - 137: SIGKILL (9) + 128 = 137 (typical for timeout kills)
//! - 143: SIGTERM (15) + 128 = 143 (graceful termination)
//! - Other non-zero values: Various error conditions

/// Represents the signal used to terminate a container
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationSignal {
    /// No signal - normal exit
    None,
    /// Container terminated via SIGTERM (graceful shutdown)
    SigTerm,
    /// Container terminated via SIGKILL (forceful kill)
    SigKill,
}

/// Represents the exit status of a container
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitStatus {
    /// Container exit code
    pub exit_code: i64,
    /// Whether the container was killed due to timeout
    pub timed_out: bool,
    /// Signal used to terminate the container
    pub termination_signal: TerminationSignal,
}

impl ExitStatus {
    /// Create a new ExitStatus
    pub fn new(exit_code: i64, timed_out: bool, termination_signal: TerminationSignal) -> Self {
        ExitStatus {
            exit_code,
            timed_out,
            termination_signal,
        }
    }

    /// Create an ExitStatus for a container that exited normally
    pub fn exited(exit_code: i64) -> Self {
        ExitStatus {
            exit_code,
            timed_out: false,
            termination_signal: TerminationSignal::None,
        }
    }

    /// Create an ExitStatus for a container that was killed due to timeout
    pub fn timed_out(termination_signal: Option<TerminationSignal>) -> Self {
        ExitStatus {
            exit_code: 137, // SIGKILL exit code
            timed_out: true,
            termination_signal: termination_signal.unwrap_or(TerminationSignal::SigKill),
        }
    }
}
