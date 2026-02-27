//! Unit tests for npx not found error handling
//!
//! This test file verifies that when npx is not available on the system,
//! the error handling properly displays informative messages and provides
//! actionable remediation steps for users.
//!
//! These are UNIT tests (not integration tests) that verify the error
//! message format, error type behavior, and proper error handling without
//! requiring actual npx execution.

use switchboard::skills::SkillsError;

/// Test that SkillsError::NpxNotFound displays the expected error message
///
/// Verifies that the Display implementation for NpxNotFound includes:
/// - The fact that npx is required
/// - The URL to install Node.js
#[test]
fn test_npx_not_found_error_display_message() {
    let error = SkillsError::NpxNotFound;
    let display_message = format!("{}", error);

    // Verify error message contains key information
    assert!(
        display_message.contains("npx"),
        "Error message should mention npx"
    );
    assert!(
        display_message.contains("required"),
        "Error message should indicate npx is required"
    );
    assert!(
        display_message.contains("https://nodejs.org"),
        "Error message should include Node.js installation URL"
    );
}

/// Test that SkillsError::NpxNotFound can be cloned correctly
///
/// Verifies that the NpxNotFound error variant can be cloned
/// without losing any information.
#[test]
fn test_npx_not_found_error_clone() {
    let error = SkillsError::NpxNotFound;
    let cloned = error.clone();

    // Cloned error should produce the same display message
    assert_eq!(format!("{}", error), format!("{}", cloned));
}

/// Test that SkillsError::NpxNotFound debug format is correct
///
/// Verifies that the Debug implementation includes the variant name.
#[test]
fn test_npx_not_found_error_debug() {
    let error = SkillsError::NpxNotFound;
    let debug_message = format!("{:?}", error);

    // Debug format should contain the variant name
    assert!(
        debug_message.contains("NpxNotFound"),
        "Debug format should contain 'NpxNotFound'"
    );
}

/// Test that SkillsError::NpxNotFound equality works correctly
///
/// Verifies that two NpxNotFound errors are equal to each other.
#[test]
fn test_npx_not_found_error_equality() {
    let error1 = SkillsError::NpxNotFound;
    let error2 = SkillsError::NpxNotFound;

    assert_eq!(error1, error2);
}

/// Test that npx not found error message is user-friendly
///
/// Verifies that the error message provides clear guidance on how
/// to resolve the issue (installing Node.js).
#[test]
fn test_npx_not_found_error_user_friendly() {
    let error = SkillsError::NpxNotFound;
    let message = format!("{}", error);

    // The message should be actionable - tell users what to do
    assert!(
        message.contains("Install"),
        "Error message should suggest installing Node.js"
    );

    // The message should not be too technical or confusing
    assert!(
        !message.contains("ENOENT") || !message.contains("No such file"),
        "Error message should be user-friendly, not show raw OS errors"
    );
}

/// Test that the NPX_NOT_FOUND_ERROR constant matches expected format
///
/// Verifies that the constant used throughout the codebase has the
/// correct format that users will see.
#[test]
fn test_npx_not_found_error_constant_format() {
    // The expected message from SkillsError::NpxNotFound Display impl
    let expected_message =
        "npx is required for this command. Install Node.js from https://nodejs.org";

    // Create the error and format it
    let error = SkillsError::NpxNotFound;
    let actual_message = format!("{}", error);

    // Verify the message matches expected format
    assert_eq!(
        actual_message, expected_message,
        "NpxNotFound error message should match the expected format"
    );
}

/// Test error message contains "command" to indicate which operation needs npx
///
/// Verifies that the error message indicates this is related to a command
/// that requires npx, not just that npx is missing generally.
#[test]
fn test_npx_not_found_error_mentions_command() {
    let error = SkillsError::NpxNotFound;
    let message = format!("{}", error);

    assert!(
        message.contains("command"),
        "Error message should mention it's needed for a command"
    );
}

/// Test error message provides installation guidance
///
/// Verifies that the error message provides a specific URL where
/// users can download Node.js.
#[test]
fn test_npx_not_found_error_provides_install_url() {
    let error = SkillsError::NpxNotFound;
    let message = format!("{}", error);

    // Verify a valid URL is provided
    assert!(
        message.contains("https://nodejs.org"),
        "Error message should provide the Node.js download URL"
    );
}

/// Test that NpxNotFound is different from other SkillsError variants
///
/// Verifies that NpxNotFound is distinct from other error types like
/// NpxCommandFailed or NetworkUnavailable.
#[test]
fn test_npx_not_found_is_distinct_error_type() {
    let npx_not_found = SkillsError::NpxNotFound;
    let npx_command_failed = SkillsError::NpxCommandFailed {
        command: "skills list".to_string(),
        exit_code: 1,
        stderr: "error".to_string(),
    };
    let network_error = SkillsError::NetworkUnavailable {
        operation: "list".to_string(),
        message: "no connection".to_string(),
    };

    // NpxNotFound should not equal other error types
    assert_ne!(npx_not_found, npx_command_failed);
    assert_ne!(npx_not_found, network_error);
}
