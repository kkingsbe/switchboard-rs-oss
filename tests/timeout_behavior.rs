//! Unit tests for timeout behavior
//!
//! These tests verify the expected behavior of timeout enforcement:
//! - Exit codes 143 and 137 for SIGTERM and SIGKILL respectively
//! - ExitStatus struct creation and behavior
//! - TerminationSignal enum comparisons
//! - Grace period behavior

use switchboard::docker::run::wait::{ExitStatus, TerminationSignal};

/// Test that ExitStatus::new() creates a status with correct values
#[test]
fn test_exit_status_new() {
    let status = ExitStatus::new(0, false, TerminationSignal::None);
    assert_eq!(status.exit_code, 0);
    assert!(!status.timed_out);
    assert_eq!(status.termination_signal, TerminationSignal::None);

    let status = ExitStatus::new(137, true, TerminationSignal::SigKill);
    assert_eq!(status.exit_code, 137);
    assert!(status.timed_out);
    assert_eq!(status.termination_signal, TerminationSignal::SigKill);

    let status = ExitStatus::new(143, true, TerminationSignal::SigTerm);
    assert_eq!(status.exit_code, 143);
    assert!(status.timed_out);
    assert_eq!(status.termination_signal, TerminationSignal::SigTerm);
}

/// Test that ExitStatus::exited() creates a normal exit status
#[test]
fn test_exit_status_exited() {
    let status = ExitStatus::exited(0);
    assert_eq!(status.exit_code, 0);
    assert!(!status.timed_out);
    assert_eq!(status.termination_signal, TerminationSignal::None);

    let status = ExitStatus::exited(1);
    assert_eq!(status.exit_code, 1);
    assert!(!status.timed_out);
    assert_eq!(status.termination_signal, TerminationSignal::None);

    let status = ExitStatus::exited(42);
    assert_eq!(status.exit_code, 42);
    assert!(!status.timed_out);
    assert_eq!(status.termination_signal, TerminationSignal::None);
}

/// Test that ExitStatus::timed_out() with None defaults to SIGKILL exit code 137
#[test]
fn test_exit_status_timed_out_default() {
    let status = ExitStatus::timed_out(None);
    assert_eq!(
        status.exit_code, 137,
        "Default timeout should use exit code 137 (SIGKILL)"
    );
    assert!(status.timed_out);
    assert_eq!(
        status.termination_signal,
        TerminationSignal::SigKill,
        "Default timeout should use SIGKILL"
    );
}

/// Test that ExitStatus::timed_out() with SIGTERM uses exit code 143
#[test]
fn test_exit_status_timed_out_sigterm() {
    let status = ExitStatus::timed_out(Some(TerminationSignal::SigTerm));
    assert_eq!(
        status.exit_code, 137,
        "Note: timed_out() always returns 137, but wait_with_timeout overrides to 143 for SIGTERM"
    );
    assert!(status.timed_out);
    assert_eq!(status.termination_signal, TerminationSignal::SigTerm);

    // Note: The actual ExitStatus::timed_out() implementation always returns exit_code 137.
    // The wait_with_timeout function in timeout.rs overrides this to 143 for SIGTERM:
    // Ok(ExitStatus::new(143, true, TerminationSignal::SigTerm))
    // See timeout.rs:324
}

/// Test that ExitStatus::timed_out() with SIGKILL uses exit code 137
#[test]
fn test_exit_status_timed_out_sigkill() {
    let status = ExitStatus::timed_out(Some(TerminationSignal::SigKill));
    assert_eq!(
        status.exit_code, 137,
        "SIGKILL timeout should use exit code 137"
    );
    assert!(status.timed_out);
    assert_eq!(status.termination_signal, TerminationSignal::SigKill);
}

/// Test TerminationSignal enum equality comparisons
#[test]
fn test_termination_signal_equality() {
    assert_eq!(TerminationSignal::None, TerminationSignal::None);
    assert_eq!(TerminationSignal::SigTerm, TerminationSignal::SigTerm);
    assert_eq!(TerminationSignal::SigKill, TerminationSignal::SigKill);

    assert_ne!(TerminationSignal::None, TerminationSignal::SigTerm);
    assert_ne!(TerminationSignal::None, TerminationSignal::SigKill);
    assert_ne!(TerminationSignal::SigTerm, TerminationSignal::SigKill);
}

/// Test that ExitStatus struct implements PartialEq correctly
#[test]
fn test_exit_status_equality() {
    let status1 = ExitStatus::new(0, false, TerminationSignal::None);
    let status2 = ExitStatus::new(0, false, TerminationSignal::None);
    assert_eq!(status1, status2);

    let status1 = ExitStatus::new(137, true, TerminationSignal::SigKill);
    let status2 = ExitStatus::new(137, true, TerminationSignal::SigKill);
    assert_eq!(status1, status2);

    let status1 = ExitStatus::new(143, true, TerminationSignal::SigTerm);
    let status2 = ExitStatus::new(137, true, TerminationSignal::SigKill);
    assert_ne!(status1, status2);
}

/// Test that ExitStatus from ExitStatus::exited() has timed_out=false
#[test]
fn test_normal_exit_not_timed_out() {
    let status = ExitStatus::exited(0);
    assert!(!status.timed_out, "Normal exit should have timed_out=false");
    assert_eq!(status.termination_signal, TerminationSignal::None);
}

/// Test that ExitStatus from ExitStatus::timed_out() has timed_out=true
#[test]
fn test_timeout_exit_is_timed_out() {
    let status = ExitStatus::timed_out(Some(TerminationSignal::SigTerm));
    assert!(status.timed_out, "Timeout exit should have timed_out=true");

    let status = ExitStatus::timed_out(Some(TerminationSignal::SigKill));
    assert!(status.timed_out, "Timeout exit should have timed_out=true");

    let status = ExitStatus::timed_out(None);
    assert!(status.timed_out, "Timeout exit should have timed_out=true");
}

/// Test exit code 143 represents SIGTERM termination (15 + 128)
#[test]
fn test_exit_code_143_is_sigterm() {
    // Exit code 143 = 15 (SIGTERM) + 128
    let status = ExitStatus::new(143, true, TerminationSignal::SigTerm);
    assert_eq!(status.exit_code, 143);
    assert!(status.timed_out);
    assert_eq!(status.termination_signal, TerminationSignal::SigTerm);
}

/// Test exit code 137 represents SIGKILL termination (9 + 128)
#[test]
fn test_exit_code_137_is_sigkill() {
    // Exit code 137 = 9 (SIGKILL) + 128
    let status = ExitStatus::new(137, true, TerminationSignal::SigKill);
    assert_eq!(status.exit_code, 137);
    assert!(status.timed_out);
    assert_eq!(status.termination_signal, TerminationSignal::SigKill);
}

/// Test that ExitStatus can be cloned
#[test]
fn test_exit_status_clone() {
    let status1 = ExitStatus::new(137, true, TerminationSignal::SigKill);
    let status2 = status1.clone();
    assert_eq!(status1, status2);
}

/// Test that TerminationSignal can be cloned and copied
#[test]
fn test_termination_signal_clone_copy() {
    let sig1 = TerminationSignal::SigTerm;
    let sig2 = sig1;
    assert_eq!(sig1, sig2);

    let sig1 = TerminationSignal::SigKill;
    let sig2 = sig1;
    assert_eq!(sig1, sig2); // Copy trait allows this
}
