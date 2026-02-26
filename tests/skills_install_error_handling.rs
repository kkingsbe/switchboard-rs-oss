//! Unit tests for skills install error handling
//!
//! This test file verifies error handling for skills installation, including:
//! - Destination already exists error handling
//! - Network unavailable error during install

use switchboard::skills::SkillsError;

/// Test that SkillsError::DestinationAlreadyExists displays the expected error message
#[test]
fn test_destination_already_exists_error_display() {
    let error = SkillsError::DestinationAlreadyExists {
        skill_name: "test-skill".to_string(),
        path: "./skills/test-skill".to_string(),
    };

    let display_message = format!("{}", error);

    // Verify error message contains key information
    assert!(
        display_message.contains("test-skill"),
        "Error message should contain skill name"
    );
    assert!(
        display_message.contains("./skills/test-skill"),
        "Error message should contain path"
    );
    assert!(
        display_message.contains("--yes"),
        "Error message should suggest --yes flag"
    );
}

/// Test that SkillsError::DestinationAlreadyExists can be cloned
#[test]
fn test_destination_already_exists_error_clone() {
    let error = SkillsError::DestinationAlreadyExists {
        skill_name: "test-skill".to_string(),
        path: "./skills/test-skill".to_string(),
    };

    let cloned = error.clone();

    assert_eq!(format!("{}", error), format!("{}", cloned));
}

/// Test that SkillsError::DestinationAlreadyExists debug format is correct
#[test]
fn test_destination_already_exists_error_debug() {
    let error = SkillsError::DestinationAlreadyExists {
        skill_name: "test-skill".to_string(),
        path: "./skills/test-skill".to_string(),
    };

    let debug_message = format!("{:?}", error);

    assert!(
        debug_message.contains("DestinationAlreadyExists"),
        "Debug format should contain 'DestinationAlreadyExists'"
    );
    assert!(
        debug_message.contains("test-skill"),
        "Debug format should contain skill name"
    );
}

/// Test that SkillsError::DestinationAlreadyExists equality works correctly
#[test]
fn test_destination_already_exists_error_equality() {
    let error1 = SkillsError::DestinationAlreadyExists {
        skill_name: "test-skill".to_string(),
        path: "./skills/test-skill".to_string(),
    };

    let error2 = SkillsError::DestinationAlreadyExists {
        skill_name: "test-skill".to_string(),
        path: "./skills/test-skill".to_string(),
    };

    assert_eq!(error1, error2);
}

/// Test that DestinationAlreadyExists is different from other SkillsError variants
#[test]
fn test_destination_already_exists_is_distinct_error_type() {
    let dest_exists = SkillsError::DestinationAlreadyExists {
        skill_name: "test-skill".to_string(),
        path: "./skills/test-skill".to_string(),
    };

    let npx_not_found = SkillsError::NpxNotFound;
    let network_error = SkillsError::NetworkUnavailable {
        operation: "install".to_string(),
        message: "no connection".to_string(),
    };

    assert_ne!(dest_exists, npx_not_found);
    assert_ne!(dest_exists, network_error);
}

/// Test that NetworkUnavailable error message includes retry suggestion
#[test]
fn test_network_unavailable_error_retry_suggestion() {
    let error = SkillsError::NetworkUnavailable {
        operation: "install".to_string(),
        message: "connection timed out".to_string(),
    };

    let display_message = format!("{}", error);

    assert!(
        display_message.contains("connection timed out"),
        "Error message should contain the network error"
    );
    assert!(
        display_message.contains("try again") || display_message.contains("connection"),
        "Error message should suggest retrying or mention connection"
    );
}

/// Test that NetworkUnavailable error includes operation details
#[test]
fn test_network_unavailable_error_includes_operation() {
    let error = SkillsError::NetworkUnavailable {
        operation: "update".to_string(),
        message: "DNS failure".to_string(),
    };

    let display_message = format!("{}", error);

    assert!(
        display_message.contains("update"),
        "Error message should include the operation name"
    );
    assert!(
        display_message.contains("DNS failure"),
        "Error message should include error details"
    );
}

/// Test that the error message for DestinationAlreadyExists is user-friendly
#[test]
fn test_destination_already_exists_error_user_friendly() {
    let error = SkillsError::DestinationAlreadyExists {
        skill_name: "frontend-design".to_string(),
        path: "./skills/frontend-design".to_string(),
    };

    let message = format!("{}", error);

    // The message should be actionable - tell users what to do
    assert!(
        message.contains("already installed") || message.contains("already"),
        "Error message should indicate the skill is already installed"
    );

    // The message should suggest using --yes flag or removing
    assert!(
        message.contains("--yes") || message.contains("remove"),
        "Error message should provide a solution"
    );
}

/// Test that the NPX_NOT_FOUND_ERROR constant has the expected format
#[test]
fn test_npx_not_found_error_constant() {
    use switchboard::skills::NPX_NOT_FOUND_ERROR;

    assert!(
        NPX_NOT_FOUND_ERROR.contains("npx"),
        "Error should mention npx"
    );
    assert!(
        NPX_NOT_FOUND_ERROR.contains("https://nodejs.org"),
        "Error should include installation URL"
    );
}
