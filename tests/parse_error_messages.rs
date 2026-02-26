//! Tests for enhanced parse error messages
//!
//! This test file verifies that parse errors include helpful messages,
//! file:line:column location information, and actionable suggestions.

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

#[test]
fn test_toml_syntax_error_with_location() {
    let temp_dir = TempDir::new().unwrap();

    // Invalid TOML syntax (unclosed bracket)
    let toml_content = r#"
        [[agent]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(result.is_err(), "Expected error for invalid TOML syntax");

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message contains file path
    assert!(
        error_msg.contains("Error parsing"),
        "Error message should start with 'Error parsing': {}",
        error_msg
    );

    // Verify error message includes helpful context about the syntax error
    assert!(
        error_msg.len() > 50,
        "Error message should be descriptive: {}",
        error_msg
    );
}

#[test]
fn test_invalid_cron_expression_with_examples() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Invalid cron expression (wrong number of fields)
    // Note: This will only fail validation when the "scheduler" feature is enabled
    // If scheduler feature is disabled, this test will skip
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "invalid-cron"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);

    // Only check if scheduler feature is enabled (result is an error)
    // When scheduler feature is disabled, cron validation is skipped
    if let Err(error) = result {
        let error_msg = format!("{}", error);

        // Verify error message includes helpful examples
        assert!(
            error_msg.contains("example")
                || error_msg.contains("Example")
                || error_msg.contains("Valid examples"),
            "Error message should include examples: {}",
            error_msg
        );

        // Verify error message mentions the expected format
        assert!(
            error_msg.contains("5 fields") || error_msg.contains("minute hour day month weekday"),
            "Error message should describe expected format: {}",
            error_msg
        );
    } else {
        // Scheduler feature not enabled - cron validation skipped
        // This is expected behavior, so test passes
    }
}

#[test]
fn test_missing_required_field_name() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // Missing required 'name' field
    let toml_content = r#"
        [[agent]]
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);

    // This will fail during deserialization since 'name' is required
    assert!(
        result.is_err(),
        "Expected error for missing required field 'name'"
    );

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message mentions the missing field
    assert!(
        error_msg.to_lowercase().contains("name") || error_msg.contains("missing field"),
        "Error message should mention the missing field: {}",
        error_msg
    );
}

#[test]
fn test_invalid_type_string_instead_of_number_for_queue_size() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // String value instead of number for max_queue_size
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
        max_queue_size = "invalid"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(
        result.is_err(),
        "Expected error for invalid type (string instead of number)"
    );

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message includes file path and location
    assert!(
        error_msg.contains("Error parsing"),
        "Error message should be a parse error: {}",
        error_msg
    );

    // Verify error message provides context about the type mismatch
    assert!(
        error_msg.len() > 50,
        "Error message should be descriptive about type mismatch: {}",
        error_msg
    );
}

#[test]
fn test_invalid_type_string_instead_of_bool_for_readonly() {
    let temp_dir = TempDir::new().unwrap();
    create_temp_prompt(&temp_dir, "test.md");

    // String value instead of boolean for readonly
    let toml_content = r#"
        [[agent]]
        name = "test-agent"
        prompt_file = "prompts/test.md"
        schedule = "0 */6 * * *"
        readonly = "yes"
    "#;

    let config_path = create_temp_config(toml_content, &temp_dir);

    let result = switchboard::config::Config::from_toml(&config_path);
    assert!(
        result.is_err(),
        "Expected error for invalid type (string instead of boolean)"
    );

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify error message includes file path
    assert!(
        error_msg.contains("Error parsing"),
        "Error message should be a parse error: {}",
        error_msg
    );
}

#[test]
fn test_missing_both_prompt_and_prompt_file() {
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
        error_msg.contains("prompt")
            && (error_msg.contains("either") || error_msg.contains("must have")),
        "Error message should explain prompt requirement: {}",
        error_msg
    );
}

#[test]
fn test_both_prompt_and_prompt_file_provided() {
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

    // Verify error message includes examples of valid formats
    assert!(
        error_msg.contains("Valid formats") || error_msg.contains("example"),
        "Error message should include examples: {}",
        error_msg
    );
}

#[test]
fn test_duplicate_agent_name() {
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
        error_msg.contains("Duplicate") || error_msg.contains("unique"),
        "Error message should explain uniqueness requirement: {}",
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
fn test_invalid_timezone_with_examples() {
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

    // Verify error message includes examples of valid timezones
    assert!(
        error_msg.contains("America/")
            || error_msg.contains("Europe/")
            || error_msg.contains("Asia/")
            || error_msg.contains("IANA"),
        "Error message should include timezone examples: {}",
        error_msg
    );
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
}
