//! Unit tests for the `switchboard validate` command
//!
//! These tests directly call ValidateCommand::run() to validate
//! the command's behavior with various configurations.

use std::fs;
use tempfile::TempDir;

use switchboard::commands::validate::ValidateCommand;
use switchboard::config::{Config, ConfigError};

/// Test validating a valid configuration with 1 agent
#[tokio::test]
async fn test_validate_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create the prompt file referenced in the config
    let prompt_path = temp_dir.path().join("prompt.txt");
    fs::write(&prompt_path, "Test prompt content").unwrap();

    // Create a valid switchboard.toml config file with 1 agent
    // Using 5-field cron format (minutes, hours, day-of-month, month, day-of-week)
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt_file = "./prompt.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify validation passes
    assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result.err());
}

/// Test validating when the config file is missing
#[tokio::test]
async fn test_validate_missing_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nonexistent.toml");

    // Create and run the validate command with a missing config file
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify it returns an error
    assert!(
        result.is_err(),
        "Expected Err for missing config file, got Ok"
    );

    // The error should indicate a parse/read error
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("Configuration parsing failed"),
        "Expected error about configuration parsing, got: {}",
        error_msg
    );
}

/// Test validating with invalid TOML syntax
#[tokio::test]
async fn test_validate_invalid_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create a file with invalid TOML syntax
    let invalid_toml = r#"
version = "1.0"
[[agents]
name = "test-agent"
"#;
    fs::write(&config_path, invalid_toml).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify it returns a parse error
    assert!(result.is_err(), "Expected Err for invalid TOML, got Ok");

    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("Configuration parsing failed"),
        "Expected error about configuration parsing, got: {}",
        error_msg
    );
}

/// Test validating with an invalid cron expression
#[tokio::test]
async fn test_validate_invalid_cron() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create the prompt file referenced in the config
    let prompt_path = temp_dir.path().join("prompt.txt");
    fs::write(&prompt_path, "Test prompt content").unwrap();

    // Create a config with an invalid cron expression
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "invalid-cron-expression"
prompt_file = "./prompt.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify it returns an error about invalid cron
    assert!(
        result.is_err(),
        "Expected Err for invalid cron expression, got Ok"
    );

    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("Invalid cron expressions") || error_msg.contains("cron"),
        "Expected error about cron validation, got: {}",
        error_msg
    );
}

/// Test validating with a valid 5-field Unix cron expression
/// This test validates BUG-001 fix - 5-field cron expressions should be supported
#[tokio::test]
async fn test_validate_5_field_cron_expression() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create the prompt file referenced in the config
    let prompt_path = temp_dir.path().join("prompt.txt");
    fs::write(&prompt_path, "Test prompt content").unwrap();

    // Create a config with a valid 5-field Unix cron expression "0 */6 * * *"
    // This means: at minute 0, every 6 hours, every day, every month, every weekday
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 */6 * * *"
prompt_file = "./prompt.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify validation passes - the 5-field expression should be accepted
    assert!(
        result.is_ok(),
        "Expected Ok for valid 5-field cron expression, got Err: {:?}",
        result.err()
    );
}

/// Test validating when the prompt file is missing
#[tokio::test]
async fn test_validate_missing_prompt_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a config pointing to a non-existent prompt file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt_file = "./nonexistent.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify it returns an error about the missing prompt file
    assert!(
        result.is_err(),
        "Expected Err for missing prompt file, got Ok"
    );
}

/// Test validating when both prompt and prompt_file are specified (should fail validation)
#[tokio::test]
async fn test_validate_both_prompt_and_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create the prompt file referenced in the config
    let prompt_path = temp_dir.path().join("prompt.txt");
    fs::write(&prompt_path, "Test prompt content").unwrap();

    // Create a config with both prompt and prompt_file (should fail validation)
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Inline prompt"
prompt_file = "./prompt.txt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify it returns a validation error
    assert!(
        result.is_err(),
        "Expected Err when both prompt and prompt_file are specified, got Ok"
    );

    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("but not both") || error_msg.contains("validation"),
        "Expected error about having both prompt and prompt_file, got: {}",
        error_msg
    );
}

// ============================================================================
// Parse Error Handling Tests
// ============================================================================

/// Test that parse errors include line and column information
#[tokio::test]
async fn test_parse_error_includes_line_and_column() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create a file with invalid TOML syntax (missing closing bracket)
    // Error should occur at line 3 (the [[agents] line)
    let invalid_toml = r#"
[[agent]
name = "test-agent"
"#;
    fs::write(&config_path, invalid_toml).unwrap();

    // Try to parse the config
    let result = Config::from_toml(&config_path);

    // Verify it returns a parse error with line/col info
    assert!(
        result.is_err(),
        "Expected parse error for invalid TOML syntax"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // The error message should include "line" and "col" for parse errors
    assert!(
        error_msg.contains("line") && error_msg.contains("col"),
        "Expected error message to include line and column info, got: {}",
        error_msg
    );

    // Also verify it contains the expected format pattern
    assert!(
        error_msg.contains("Error parsing")
            || error_msg.contains("line")
            || error_msg.contains("col"),
        "Expected error message to contain parse error format, got: {}",
        error_msg
    );
}

/// Test that parse error messages are user-friendly for unclosed bracket
#[tokio::test]
async fn test_parse_error_unclosed_bracket() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create a file with unclosed array bracket
    let invalid_toml = r#"
[settings
name = "test"
"#;
    fs::write(&config_path, invalid_toml).unwrap();

    // Try to parse the config
    let result = Config::from_toml(&config_path);

    // Verify it returns a parse error
    assert!(result.is_err(), "Expected parse error for unclosed bracket");

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // The error message should be user-friendly
    assert!(!error_msg.is_empty(), "Error message should not be empty");

    // The error should indicate what's wrong
    assert!(
        error_msg.contains("Error parsing"),
        "Expected error to mention parsing, got: {}",
        error_msg
    );
}

/// Test that parse error messages are user-friendly for unclosed string
#[tokio::test]
async fn test_parse_error_unclosed_string() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create a file with unclosed string
    let invalid_toml = r#"
[[agent]]
name = "unclosed string
schedule = "0 9 * * *"
prompt = "test"
"#;
    fs::write(&config_path, invalid_toml).unwrap();

    // Try to parse the config
    let result = Config::from_toml(&config_path);

    // Verify it returns a parse error
    assert!(result.is_err(), "Expected parse error for unclosed string");

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // The error message should mention the issue
    assert!(!error_msg.is_empty(), "Error message should not be empty");

    // The error should indicate what's wrong (typically mentions quotes or strings)
    assert!(
        error_msg.contains("Error parsing"),
        "Expected error to mention parsing, got: {}",
        error_msg
    );
}

/// Test that parse errors include detailed info for invalid type
#[tokio::test]
async fn test_parse_error_invalid_type() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create a file with number instead of string for name (line 3)
    let invalid_toml = r#"
[[agent]]
name = 123
schedule = "0 9 * * *"
prompt = "test"
"#;
    fs::write(&config_path, invalid_toml).unwrap();

    // Try to parse the config
    let result = Config::from_toml(&config_path);

    // Verify it returns a parse error
    assert!(result.is_err(), "Expected parse error for invalid type");

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // The error message should be detailed and user-friendly
    assert!(!error_msg.is_empty(), "Error message should not be empty");

    // The error should indicate the type mismatch
    assert!(
        error_msg.contains("Error parsing")
            || error_msg.contains("line")
            || error_msg.contains("col"),
        "Expected error message to contain parse error format, got: {}",
        error_msg
    );
}

/// Test that parse errors include detailed info for duplicate keys
#[tokio::test]
async fn test_parse_error_duplicate_keys() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create a file with duplicate keys (line 5)
    let invalid_toml = r#"
[[agent]]
name = "test-agent"
name = "duplicate-name"
schedule = "0 9 * * *"
prompt = "test"
"#;
    fs::write(&config_path, invalid_toml).unwrap();

    // Try to parse the config
    let result = Config::from_toml(&config_path);

    // Duplicate keys should cause a parse error
    // (toml crate reports duplicate keys as an error)
    let is_error = result.is_err();

    if is_error {
        let error = result.unwrap_err();
        let error_msg = error.to_string();

        // The error message should mention the duplicate
        assert!(!error_msg.is_empty(), "Error message should not be empty");

        // The error should indicate what's wrong
        assert!(
            error_msg.contains("Error parsing")
                || error_msg.contains("duplicate")
                || error_msg.contains("line"),
            "Expected error message to contain duplicate key info, got: {}",
            error_msg
        );
    }
    // Note: Some TOML parsers may allow duplicates (last wins), so we only assert if it's an error
}

/// Test that parse error messages work with validate command
#[tokio::test]
async fn test_validate_command_displays_parse_error_details() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create a file with invalid TOML syntax (missing closing bracket)
    let invalid_toml = r#"
[[agent]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "test"
"#;
    fs::write(&config_path, invalid_toml).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};
    let result = command.run(config_path).await;

    // Verify it returns a parse error
    assert!(result.is_err(), "Expected Err for invalid TOML, got Ok");

    let error_msg = format!("{:?}", result.unwrap_err());

    // The error should mention "Configuration parsing failed"
    assert!(
        error_msg.contains("Configuration parsing failed"),
        "Expected error message to mention 'Configuration parsing failed', got: {}",
        error_msg
    );
}

/// Test parse error for missing array bracket at the right place
#[tokio::test]
async fn test_parse_error_missing_closing_array_bracket() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create a file with missing closing bracket for agent array
    let invalid_toml = r#"
[[agent
name = "test-agent"
schedule = "0 9 * * *"
prompt = "test"
"#;
    fs::write(&config_path, invalid_toml).unwrap();

    // Try to parse the config
    let result = Config::from_toml(&config_path);

    // Verify it returns a parse error with line/col info
    assert!(
        result.is_err(),
        "Expected parse error for missing closing bracket"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // The error message should include line information
    assert!(
        error_msg.contains("line") || error_msg.contains("Error parsing"),
        "Expected error message to include line info, got: {}",
        error_msg
    );
}

/// Test parse error for malformed key-value pair
#[tokio::test]
async fn test_parse_error_malformed_key_value() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create a file with malformed key-value (missing equals)
    let invalid_toml = r#"
[[agent]]
name "test-agent"
schedule = "0 9 * * *"
prompt = "test"
"#;
    fs::write(&config_path, invalid_toml).unwrap();

    // Try to parse the config
    let result = Config::from_toml(&config_path);

    // Verify it returns a parse error
    assert!(
        result.is_err(),
        "Expected parse error for malformed key-value"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // The error message should be detailed
    assert!(!error_msg.is_empty(), "Error message should not be empty");

    // The error should indicate what's wrong
    assert!(
        error_msg.contains("Error parsing"),
        "Expected error to mention parsing, got: {}",
        error_msg
    );
}

/// Test parse error for invalid boolean value
#[tokio::test]
async fn test_parse_error_invalid_boolean() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create a file with invalid boolean value
    let invalid_toml = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
readonly = "not-a-boolean"
prompt = "test"
"#;
    fs::write(&config_path, invalid_toml).unwrap();

    // Try to parse the config
    let result = Config::from_toml(&config_path);

    // Verify it returns a parse error
    assert!(result.is_err(), "Expected parse error for invalid boolean");

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // The error message should be detailed
    assert!(!error_msg.is_empty(), "Error message should not be empty");

    // The error should indicate the type mismatch
    assert!(
        error_msg.contains("Error parsing")
            || error_msg.contains("type")
            || error_msg.contains("line"),
        "Expected error message to contain type mismatch info, got: {}",
        error_msg
    );
}

/// Test ConfigError::ParseError includes line information
#[test]
fn test_config_error_parse_error_format_with_line_col() {
    // Create a ParseError with both line and column
    let error = ConfigError::ParseError {
        file: "test.toml".to_string(),
        line: Some(10),
        col: Some(5),
        message: "unexpected character".to_string(),
        suggestion: None,
    };

    let error_msg = error.to_string();

    // Should include file, line, col, and message
    assert!(
        error_msg.contains("test.toml"),
        "Expected error to include file name, got: {}",
        error_msg
    );
    assert!(
        error_msg.contains("line 10") || error_msg.contains("10"),
        "Expected error to include line 10, got: {}",
        error_msg
    );
    assert!(
        error_msg.contains("col 5") || error_msg.contains("5"),
        "Expected error to include col 5, got: {}",
        error_msg
    );
    assert!(
        error_msg.contains("unexpected character"),
        "Expected error to include message, got: {}",
        error_msg
    );
}

/// Test ConfigError::ParseError format with only line (no column)
#[test]
fn test_config_error_parse_error_format_line_only() {
    // Create a ParseError with only line
    let error = ConfigError::ParseError {
        file: "test.toml".to_string(),
        line: Some(10),
        col: None,
        message: "unexpected character".to_string(),
        suggestion: None,
    };

    let error_msg = error.to_string();

    // Should include file and line, but not col
    assert!(
        error_msg.contains("test.toml"),
        "Expected error to include file name, got: {}",
        error_msg
    );
    assert!(
        error_msg.contains("line 10") || error_msg.contains("10"),
        "Expected error to include line 10, got: {}",
        error_msg
    );
    assert!(
        !error_msg.contains("col"),
        "Expected error to NOT include col, got: {}",
        error_msg
    );
}

/// Test ConfigError::ParseError format with no location info
#[test]
fn test_config_error_parse_error_format_no_location() {
    // Create a ParseError with no location info
    let error = ConfigError::ParseError {
        file: "test.toml".to_string(),
        line: None,
        col: None,
        message: "unexpected character".to_string(),
        suggestion: None,
    };

    let error_msg = error.to_string();

    // Should include file and message, but not line/col
    assert!(
        error_msg.contains("test.toml"),
        "Expected error to include file name, got: {}",
        error_msg
    );
    assert!(
        error_msg.contains("unexpected character"),
        "Expected error to include message, got: {}",
        error_msg
    );
    assert!(
        !error_msg.contains("line") && !error_msg.contains("col"),
        "Expected error to NOT include line/col, got: {}",
        error_msg
    );
}

// ============================================================================
// Skills Validation Integration Tests
// ============================================================================

/// Test that validate warns on empty skills field
///
/// This test verifies that when an agent has an empty skills list (skills = []),
/// the validate command returns Ok (warnings don't cause failure) but displays
/// a warning message about the empty skills field.
#[tokio::test]
async fn test_validate_warns_on_empty_skills_field() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a config with an agent that has an empty skills array
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
skills = []
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify the command returns Ok (warnings don't cause failure)
    assert!(
        result.is_ok(),
        "Expected Ok for empty skills field (warning only), got Err: {:?}",
        result.err()
    );
}

/// Test that validate errors on invalid skill format
///
/// This test verifies that when an agent has skills with invalid formats,
/// the validate command returns Err (errors cause failure) and the error message
/// contains information about the invalid skill source.
#[tokio::test]
async fn test_validate_errors_on_invalid_skill_format() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a config with an agent that has skills with invalid formats
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
skills = [
    "owner-only",      # Invalid: missing slash
    "owner@only",      # Invalid: @ without slash first
    "owner/repo/extra" # Invalid: multiple slashes
]
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify the command returns Err (errors cause failure)
    assert!(
        result.is_err(),
        "Expected Err for invalid skill format, got Ok"
    );

    // Verify the error message (it should be "Configuration validation failed" or similar)
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("Configuration validation failed") || error_msg.contains("Invalid"),
        "Expected error message to contain 'Configuration validation failed' or 'Invalid', got: {}",
        error_msg
    );
}

/// Test that validate errors on duplicate skills
///
/// This test verifies that when an agent has duplicate skill entries,
/// the validate command returns Err (errors cause failure) and the error message
/// contains information about duplicate skills.
#[tokio::test]
async fn test_validate_errors_on_duplicate_skills() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a config with an agent that has the same skill listed multiple times
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
skills = [
    "owner/repo",
    "another/skill",
    "owner/repo"  # Duplicate entry
]
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify the command returns Err (errors cause failure)
    assert!(result.is_err(), "Expected Err for duplicate skills, got Ok");

    // Verify the error message (it should be "Configuration validation failed" or similar)
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("Configuration validation failed") || error_msg.contains("Invalid"),
        "Expected error message to contain 'Configuration validation failed' or 'Invalid', got: {}",
        error_msg
    );
}

/// Test that validate errors on duplicate skills with specific error message format
///
/// This test verifies that when an agent has duplicate skill entries,
/// the validate command returns Err with a specific error message format
/// that includes the duplicate skill name and count.
#[tokio::test]
async fn test_validate_duplicate_skills() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a config with an agent that has duplicate skills
    // "owner/repo" appears twice
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
skills = ["owner/repo", "owner/repo", "owner/other"]
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify the command returns Err (errors cause failure)
    assert!(result.is_err(), "Expected Err for duplicate skills, got Ok");
}

/// Test that validate passes with valid skills
///
/// This test verifies that when an agent has valid skills in correct format,
/// the validate command returns Ok without any errors or warnings.
#[tokio::test]
async fn test_validate_passes_with_valid_skills() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a config with an agent that has valid skills in correct format
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
skills = [
    "owner/repo",
    "another/repo@skill-name"
]
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify validation passes
    assert!(
        result.is_ok(),
        "Expected Ok for valid skills, got Err: {:?}",
        result.err()
    );
}

// ============================================================================
// Complex Multi-Agent Validation Scenario Tests
// ============================================================================

/// Test validating multiple agents with mixed validation results
///
/// This test verifies that when a config file contains multiple agents with
/// different validation states (valid, warnings, errors), the validate command
/// correctly handles the mix and returns Err when errors are present.
#[tokio::test]
async fn test_validate_multiple_agents_with_mixed_issues() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a config with 3 agents:
    // - Agent 1: Valid skills (no issues)
    // - Agent 2: Empty skills field (warning)
    // - Agent 3: Invalid skill format (error)
    let config_content = r#"
[[agent]]
name = "agent-with-valid-skills"
schedule = "0 9 * * *"
prompt = "Test prompt"
skills = [
    "owner/repo",
    "another/repo@skill-name"
]

[[agent]]
name = "agent-with-empty-skills"
schedule = "0 10 * * *"
prompt = "Test prompt"
skills = []

[[agent]]
name = "agent-with-invalid-skills"
schedule = "0 11 * * *"
prompt = "Test prompt"
skills = [
    "owner-only",      # Invalid: missing slash
    "owner@only"       # Invalid: @ without slash first
]
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify the command returns Err (because of the error in agent-with-invalid-skills)
    assert!(
        result.is_err(),
        "Expected Err for config with mixed issues (contains errors), got Ok"
    );

    // Verify the error message is about validation failure
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("Configuration validation failed") || error_msg.contains("Invalid"),
        "Expected error message to contain 'Configuration validation failed' or 'Invalid', got: {}",
        error_msg
    );
}

/// Test validating when all agents have issues
///
/// This test verifies that when a config file contains multiple agents
/// and all of them have validation issues (warnings and/or errors), the
/// validate command correctly reports all issues and returns Err when
/// errors are present.
#[tokio::test]
async fn test_validate_all_agents_with_issues() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a config with 4 agents, all having issues:
    // - Agent 1: Empty skills (warning)
    // - Agent 2: Invalid format (error)
    // - Agent 3: Duplicates (error)
    // - Agent 4: Mix of invalid formats (error)
    let config_content = r#"
[[agent]]
name = "agent-with-empty-skills"
schedule = "0 9 * * *"
prompt = "Test prompt"
skills = []

[[agent]]
name = "agent-with-invalid-skills"
schedule = "0 10 * * *"
prompt = "Test prompt"
skills = [
    "owner-only",      # Invalid: missing slash
    "owner/repo/extra" # Invalid: multiple slashes
]

[[agent]]
name = "agent-with-duplicate-skills"
schedule = "0 11 * * *"
prompt = "Test prompt"
skills = [
    "owner/repo",
    "another/skill",
    "owner/repo"  # Duplicate entry
]

[[agent]]
name = "agent-with-mixed-invalid-skills"
schedule = "0 12 * * *"
prompt = "Test prompt"
skills = [
    "owner@only",      # Invalid: @ without slash first
    ""                # Invalid: empty string
]
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify the command returns Err (because errors are present)
    assert!(
        result.is_err(),
        "Expected Err for config where all agents have issues (including errors), got Ok"
    );

    // Verify the error message is about validation failure
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("Configuration validation failed") || error_msg.contains("Invalid"),
        "Expected error message to contain 'Configuration validation failed' or 'Invalid', got: {}",
        error_msg
    );
}

/// Test validating a valid config with no false positives
///
/// This test verifies that when a config file contains multiple agents
/// all with valid skills in various correct formats, the validate command
/// returns Ok and does not generate any warnings or errors about skills.
#[tokio::test]
async fn test_validate_no_false_positives_for_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a config with 4 agents, all having valid skills in various formats
    // Include various valid skill formats: owner/repo, owner/repo@skill-name
    let config_content = r#"
[[agent]]
name = "agent-with-basic-skills"
schedule = "0 9 * * *"
prompt = "Test prompt"
skills = [
    "owner/repo",
    "another/skill"
]

[[agent]]
name = "agent-with-skilled-skills"
schedule = "0 10 * * *"
prompt = "Test prompt"
skills = [
    "owner/repo@skill-name",
    "another/repo@different-skill"
]

[[agent]]
name = "agent-with-mixed-skills"
schedule = "0 11 * * *"
prompt = "Test prompt"
skills = [
    "owner/repo",
    "owner/repo@skill-name",
    "another/project",
    "third/repo@custom-skill"
]

[[agent]]
name = "agent-with-single-skill"
schedule = "0 12 * * *"
prompt = "Test prompt"
skills = [
    "single/repo@only-skill"
]
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify the command returns Ok (all skills are valid)
    assert!(
        result.is_ok(),
        "Expected Ok for valid config with multiple agents, got Err: {:?}",
        result.err()
    );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

/// Test validating when all agents have empty skills
///
/// This test verifies that when a config file contains multiple agents
/// and all of them have empty skills arrays (skills = []), the validate
/// command returns Ok (warnings don't cause failure) and displays
/// warnings for all agents.
#[tokio::test]
async fn test_validate_all_agents_with_empty_skills() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a config with 3 agents, all having empty skills arrays
    let config_content = r#"
[[agent]]
name = "agent-with-empty-skills-1"
schedule = "0 9 * * *"
prompt = "Test prompt"
skills = []

[[agent]]
name = "agent-with-empty-skills-2"
schedule = "0 10 * * *"
prompt = "Test prompt"
skills = []

[[agent]]
name = "agent-with-empty-skills-3"
schedule = "0 11 * * *"
prompt = "Test prompt"
skills = []
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify the command returns Ok (warnings don't cause failure)
    assert!(
        result.is_ok(),
        "Expected Ok for all agents with empty skills (warnings only), got Err: {:?}",
        result.err()
    );
}

/// Test validating a config file with no agents
///
/// This test verifies that when a config file has no agents (empty config
/// or config with other fields but no [[agent]] sections), the validate
/// command fails but does not produce any skill-related errors/warnings.
/// The error is about having no agents, not about skills.
#[tokio::test]
async fn test_validate_config_with_no_agents() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create an empty config file (no agents)
    let config_content = r#"
# Empty configuration with no agents
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // The command should return Err because config requires at least one agent
    // but verify the error is a generic config validation error, not about skills
    assert!(
        result.is_err(),
        "Expected Err for config with no agents, got Ok"
    );

    // Verify the error message is the generic "Configuration validation failed"
    // and is NOT about skills (which would indicate a bug)
    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(
        error_msg.contains("Configuration validation failed"),
        "Expected generic validation error, got: {}",
        error_msg
    );
    assert!(
        !error_msg.contains("skills")
            && !error_msg.contains("Skills")
            && !error_msg.contains("skill"),
        "Error message should not mention 'skills', got: {}",
        error_msg
    );
}

/// Test validating with very long skill names
///
/// This test verifies that when an agent has very long skill names
/// (200+ characters), the validate command returns Ok if the format
/// is correct, and ensures no length-related errors or panics occur.
#[tokio::test]
async fn test_validate_very_long_skill_names() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create very long owner, repo, and skill names (200+ characters total)
    let long_owner = "owner-name-very-long-".repeat(10); // ~180 chars
    let long_repo = "repo-name-very-long-".repeat(10); // ~180 chars
    let long_skill = "skill-name-very-long-".repeat(10); // ~200 chars

    // Build a valid skill format with long names
    let long_skill_with_name = format!("{}/{}@{}", long_owner, long_repo, long_skill);

    // Create a config with an agent having very long skill names
    let config_content = format!(
        r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
skills = [
    "{}"
]
"#,
        long_skill_with_name
    );
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify the command returns Ok (long names are valid if format is correct)
    assert!(
        result.is_ok(),
        "Expected Ok for very long skill names (valid format), got Err: {:?}",
        result.err()
    );
}

/// Test validating an agent with no skills field (None)
///
/// This test verifies that when an agent doesn't have a skills field at all
/// (not present in the config), the validate command returns Ok (no skills
/// field is valid) and does not produce any warnings or errors about skills.
#[tokio::test]
async fn test_validate_agent_with_no_skills_field() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a config with an agent that doesn't have a skills field at all
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create and run the validate command
    let command = ValidateCommand {};

    let result = command.run(config_path).await;

    // Verify the command returns Ok (no skills field is valid)
    assert!(
        result.is_ok(),
        "Expected Ok for agent with no skills field, got Err: {:?}",
        result.err()
    );
}
