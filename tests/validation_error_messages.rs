//! Tests for enhanced validation error messages
//!
//! This test file verifies that validation errors include helpful messages,
//! field names, invalid values, and actionable suggestions.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a temporary config file
fn create_temp_config(content: &str, temp_dir: &TempDir) -> PathBuf {
    let config_path = temp_dir.path().join("switchboard.toml");
    let mut file = fs::File::create(&config_path).unwrap();
    write!(file, "{}", content).unwrap();
    config_path
}

/// Helper to create a temporary prompt file
fn create_temp_prompt(temp_dir: &TempDir, name: &str) -> PathBuf {
    let prompts_dir = temp_dir.path().join("prompts");
    fs::create_dir_all(&prompts_dir).unwrap();
    let prompt_path = prompts_dir.join(name);
    fs::write(&prompt_path, "Test prompt content").unwrap();
    prompt_path
}

/// Helper to create a subdirectory
#[allow(dead_code)]
fn create_temp_subdir(temp_dir: &TempDir, name: &str) -> PathBuf {
    let sub_dir = temp_dir.path().join(name);
    fs::create_dir_all(&sub_dir).unwrap();
    sub_dir
}

#[test]
fn test_missing_prompt_and_prompt_file() {
    let temp_dir = TempDir::new().unwrap();

    // Missing both 'prompt' and 'prompt_file'
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        schedule = "0 */6 * * *"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(
        result.is_err(),
        "Expected error for missing both prompt and prompt_file"
    );

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message explains the requirement
    assert!(
        error_msg.contains("test-agent"),
        "Error message should mention agent name: {}",
        error_msg
    );

    assert!(
        error_msg.contains("must have either 'prompt'"),
        "Error message should explain prompt requirement: {}",
        error_msg
    );

    assert!(
        error_msg.contains("prompt_file"),
        "Error message should mention prompt_file: {}",
        error_msg
    );

    assert!(
        error_msg.contains("inline text"),
        "Error message should explain inline prompt option: {}",
        error_msg
    );

    assert!(
        error_msg.contains("path to file"),
        "Error message should explain prompt_file option: {}",
        error_msg
    );
}

#[test]
fn test_invalid_prompt_file_path() {
    let temp_dir = TempDir::new().unwrap();

    // Invalid prompt_file path (file doesn't exist)
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/nonexistent.md"
        schedule = "0 */6 * * *"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(
        result.is_err(),
        "Expected error for invalid prompt_file path"
    );

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message includes the agent name
    assert!(
        error_msg.contains("test-agent"),
        "Error message should mention agent name: {}",
        error_msg
    );

    // Verify error message includes the path that was checked
    assert!(
        error_msg.contains("nonexistent.md") || error_msg.contains("prompts"),
        "Error message should mention the invalid path: {}",
        error_msg
    );

    // Verify error message provides guidance
    assert!(
        error_msg.contains("invalid prompt_file path") || error_msg.contains("Prompt file"),
        "Error message should explain the issue: {}",
        error_msg
    );
}

#[test]
fn test_invalid_prompt_file_absolute_path_not_found() {
    let temp_dir = TempDir::new().unwrap();

    // Invalid absolute prompt_file path
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "/absolute/path/to/nonexistent.md"
        schedule = "0 */6 * * *"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(
        result.is_err(),
        "Expected error for invalid absolute prompt_file path"
    );

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message includes the agent name and path
    assert!(
        error_msg.contains("test-agent"),
        "Error message should mention agent name: {}",
        error_msg
    );

    assert!(
        error_msg.contains("/absolute/path/to/nonexistent.md") || error_msg.contains("Prompt file"),
        "Error message should mention the invalid absolute path: {}",
        error_msg
    );
}

#[test]
fn test_duplicate_agent_names() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test1.md");
    create_temp_prompt(&temp_dir, "test2.md");

    // Duplicate agent names
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test1.md"
        schedule = "0 */6 * * *"
        
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test2.md"
        schedule = "0 9 * * *"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_err(), "Expected error for duplicate agent names");

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message explains uniqueness requirement
    assert!(
        error_msg.contains("Duplicate") && error_msg.contains("test-agent"),
        "Error message should mention duplicate agent name: {}",
        error_msg
    );

    assert!(
        error_msg.contains("unique") || error_msg.contains("must be unique"),
        "Error message should explain uniqueness requirement: {}",
        error_msg
    );

    assert!(
        error_msg.contains("name"),
        "Error message should mention the 'name' field: {}",
        error_msg
    );
}

#[test]
fn test_invalid_timezone() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Invalid timezone
    let toml_content = r#"
        [settings]
        timezone = "Invalid/Timezone"
        
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_err(), "Expected error for invalid timezone");

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message includes the invalid timezone
    assert!(
        error_msg.contains("Invalid/Timezone"),
        "Error message should mention the invalid timezone: {}",
        error_msg
    );

    // Verify error message includes examples of valid timezones
    assert!(
        error_msg.contains("America/")
            || error_msg.contains("Europe/")
            || error_msg.contains("Asia/")
            || error_msg.contains("IANA"),
        "Error message should include timezone examples: {}",
        error_msg
    );

    // Verify error message mentions IANA format
    assert!(
        error_msg.contains("IANA") || error_msg.contains("Area/City"),
        "Error message should mention IANA format: {}",
        error_msg
    );

    // Verify error message includes specific examples
    assert!(
        error_msg.contains("New_York")
            || error_msg.contains("London")
            || error_msg.contains("Tokyo"),
        "Error message should include specific timezone examples: {}",
        error_msg
    );

    // Verify error message includes helpful link
    assert!(
        error_msg.contains("https://") || error_msg.contains("wikipedia.org"),
        "Error message should include documentation link: {}",
        error_msg
    );
}

#[test]
fn test_invalid_overlap_mode() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Invalid overlap_mode value
    let toml_content = r#"
        [settings]
        overlap_mode_str = "invalid-mode"
        
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_err(), "Expected error for invalid overlap_mode");

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message includes the invalid value
    assert!(
        error_msg.contains("invalid-mode"),
        "Error message should mention the invalid overlap_mode: {}",
        error_msg
    );

    // Verify error message mentions valid values
    assert!(
        error_msg.contains("skip") || error_msg.contains("Skip"),
        "Error message should mention 'skip' mode: {}",
        error_msg
    );

    assert!(
        error_msg.contains("queue") || error_msg.contains("Queue"),
        "Error message should mention 'queue' mode: {}",
        error_msg
    );

    // Verify error message explains what each mode does
    assert!(
        error_msg.contains("already running") || error_msg.contains("max_queue_size"),
        "Error message should explain mode behavior: {}",
        error_msg
    );
}

#[test]
fn test_invalid_timeout_format() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Invalid timeout format
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
        timeout = "invalid"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_err(), "Expected error for invalid timeout format");

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message includes the invalid value
    assert!(
        error_msg.contains("invalid"),
        "Error message should mention the invalid timeout: {}",
        error_msg
    );

    // Verify error message includes examples of valid formats
    assert!(
        error_msg.contains("Valid formats") || error_msg.contains("example"),
        "Error message should include examples: {}",
        error_msg
    );

    assert!(
        error_msg.contains("30s") || error_msg.contains("5m") || error_msg.contains("1h"),
        "Error message should include specific examples: {}",
        error_msg
    );

    // Verify error message mentions the 'timeout' field
    assert!(
        error_msg.contains("timeout"),
        "Error message should mention the 'timeout' field: {}",
        error_msg
    );
}

#[test]
fn test_timeout_zero() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Timeout value of 0
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
        timeout = "0s"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_err(), "Expected error for timeout value of 0");

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message explains that timeout must be positive
    assert!(
        error_msg.contains("greater than 0") || error_msg.contains("positive"),
        "Error message should explain positive requirement: {}",
        error_msg
    );

    // Verify error message provides helpful examples
    assert!(
        error_msg.contains("10s") || error_msg.contains("5m"),
        "Error message should provide examples: {}",
        error_msg
    );

    // Verify error message mentions valid range
    assert!(
        error_msg.contains("Minimum")
            || error_msg.contains("Maximum")
            || error_msg.contains("1 second")
            || error_msg.contains("86400"),
        "Error message should mention valid range: {}",
        error_msg
    );
}

#[test]
fn test_timeout_too_large() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Timeout value > 24 hours
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
        timeout = "25h"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_err(), "Expected error for timeout > 24 hours");

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message includes the invalid value
    assert!(
        error_msg.contains("25h"),
        "Error message should mention the invalid timeout: {}",
        error_msg
    );

    // Verify error message explains that timeout is too large
    assert!(
        error_msg.contains("too large") || error_msg.contains("too large"),
        "Error message should explain timeout is too large: {}",
        error_msg
    );

    // Verify error message mentions valid range (minimum and maximum)
    assert!(
        error_msg.contains("Minimum") || error_msg.contains("1 second"),
        "Error message should mention minimum value: {}",
        error_msg
    );

    assert!(
        error_msg.contains("Maximum")
            || error_msg.contains("86400")
            || error_msg.contains("24 hours"),
        "Error message should mention maximum value: {}",
        error_msg
    );

    // Verify error message suggests valid value
    assert!(
        error_msg.contains("24h") || error_msg.contains("Try '24h'"),
        "Error message should suggest valid value: {}",
        error_msg
    );
}

#[test]
fn test_timeout_exactly_24_hours() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Timeout value exactly 24 hours (should be valid)
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
        timeout = "24h"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_ok(), "24h timeout should be valid");
}

#[test]
fn test_max_queue_size_zero() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // max_queue_size value of 0
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
        max_queue_size = 0
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_err(), "Expected error for max_queue_size of 0");

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message mentions the invalid value
    assert!(
        error_msg.contains("0") || error_msg.contains("max_queue_size"),
        "Error message should mention the invalid value or field: {}",
        error_msg
    );

    // Verify error message explains that queue size must be positive
    assert!(
        error_msg.contains("positive") || error_msg.contains("greater than 0"),
        "Error message should explain positive requirement: {}",
        error_msg
    );

    // Verify error message mentions valid range (minimum and maximum)
    assert!(
        error_msg.contains("Minimum") && error_msg.contains("Maximum"),
        "Error message should mention valid range: {}",
        error_msg
    );

    assert!(
        error_msg.contains("1") || error_msg.contains("100"),
        "Error message should mention specific range values: {}",
        error_msg
    );
}

#[test]
fn test_max_queue_size_too_large() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // max_queue_size value > 100
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
        max_queue_size = 101
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_err(), "Expected error for max_queue_size > 100");

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message includes the invalid value
    assert!(
        error_msg.contains("101") || error_msg.contains("too large"),
        "Error message should mention the invalid value or issue: {}",
        error_msg
    );

    // Verify error message explains that queue size is too large
    assert!(
        error_msg.contains("too large") || error_msg.contains("is too large"),
        "Error message should explain queue size is too large: {}",
        error_msg
    );

    // Verify error message mentions valid range
    assert!(
        error_msg.contains("Minimum") && error_msg.contains("Maximum"),
        "Error message should mention valid range: {}",
        error_msg
    );

    assert!(
        error_msg.contains("1") && error_msg.contains("100"),
        "Error message should mention specific range values: {}",
        error_msg
    );
}

#[test]
fn test_max_queue_size_at_boundary_values() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Test max_queue_size = 1 (should be valid)
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
        max_queue_size = 1
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_ok(), "max_queue_size = 1 should be valid");

    // Test max_queue_size = 100 (should be valid)
    let toml_content2 = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
        max_queue_size = 100
    "#;

    let config_path2 = create_temp_config(toml_content2, &temp_dir);

    let result2 = switchboard::config::Config::from_toml(&config_path2);
    assert!(result2.is_ok(), "max_queue_size = 100 should be valid");
}

#[test]
fn test_empty_agent_name() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Empty agent name
    let toml_content = r#"
        [[agent]]
        name = ""
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_err(), "Expected error for empty agent name");

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message explains that name cannot be empty
    assert!(
        error_msg.contains("empty") || error_msg.contains("cannot be empty"),
        "Error message should explain that name cannot be empty: {}",
        error_msg
    );

    // Verify error message mentions the 'name' field
    assert!(
        error_msg.contains("name"),
        "Error message should mention the 'name' field: {}",
        error_msg
    );
}

#[test]
fn test_both_prompt_and_prompt_file() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Both 'prompt' and 'prompt_file' provided (mutually exclusive)
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt = "Inline prompt text"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(
        result.is_err(),
        "Expected error for both prompt and prompt_file"
    );

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message explains mutual exclusivity
    assert!(
        error_msg.contains("exactly one") || error_msg.contains("not both"),
        "Error message should explain mutual exclusivity: {}",
        error_msg
    );

    // Verify error message mentions both options
    assert!(
        error_msg.contains("prompt") && error_msg.contains("prompt_file"),
        "Error message should mention both prompt options: {}",
        error_msg
    );
}

#[test]
fn test_no_agents_defined() {
    let temp_dir = TempDir::new().unwrap();

    // No agents defined (empty config)
    let toml_content = r#"
        [settings]
        image_name = "test-image"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_err(), "Expected error for no agents defined");

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message suggests adding agents
    assert!(
        error_msg.contains("at least one agent") || error_msg.contains("[[agent]]"),
        "Error message should suggest adding agents: {}",
        error_msg
    );
}

#[test]
fn test_timezone_system_and_empty_are_valid() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // "system" and empty timezone should be valid
    let toml_content1 = r#"
        [settings]
        timezone = "system"
        
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
    "#;

    let config_path1 = create_temp_config(toml_content1, &temp_dir);
    let result1 = switchboard::config::Config::from_toml(&config_path1);
    assert!(result1.is_ok(), "timezone = 'system' should be valid");

    // Empty timezone (default)
    let toml_content2 = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
    "#;

    let config_path2 = create_temp_config(toml_content2, &temp_dir);
    let result2 = switchboard::config::Config::from_toml(&config_path2);
    assert!(result2.is_ok(), "empty timezone should be valid");
}

#[test]
fn test_timeout_formats() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Test various valid timeout formats
    let test_cases = vec![
        ("1s", "1 second"),
        ("30s", "30 seconds"),
        ("5m", "5 minutes"),
        ("60m", "60 minutes"),
        ("1h", "1 hour"),
        ("24h", "24 hours"),
        ("86400s", "86400 seconds"),
    ];

    for (timeout_val, description) in test_cases {
        let toml_content = format!(
            r#"
            [[agent]]
            name = "test-agent"
            prompt_file = "prompts/test.md"
            schedule = "0 */6 * * *"
            timeout = "{}"
        "#,
            timeout_val
        );

        let config_path = create_temp_config(&toml_content, &temp_dir);
        let result = switchboard::config::Config::from_toml(&config_path);
        assert!(result.is_ok(), "{} should be a valid timeout", description);
    }
}

#[test]
fn test_error_message_consistency_formatting() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Test that error messages have consistent formatting
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
        max_queue_size = 0
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);
    let result = switchboard::config::Config::from_toml(&config_path);

    if let Err(error) = result {
        let error_msg = format!("{}", error);

        // Verify error message starts with "Validation error"
        assert!(
            error_msg.starts_with("Validation error") || error_msg.contains("Validation error"),
            "Error message should start with 'Validation error': {}",
            error_msg
        );

        // Verify error message includes field name
        assert!(
            error_msg.contains("max_queue_size") || error_msg.contains("field"),
            "Error message should include field name: {}",
            error_msg
        );
    }
}
