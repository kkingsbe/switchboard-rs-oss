//! Test fixtures for Switchboard tests.
//! 
//! Provides reusable test data structures and helper functions for creating
//! consistent test scenarios.

use std::path::PathBuf;

/// Creates a temporary directory path for testing.
/// 
/// Note: This is a simple path builder. For actual temp directory management,
/// consider using the `tempfile` crate in your tests.
pub fn temp_dir() -> PathBuf {
    PathBuf::from("/tmp/switchboard_test")
}

/// Sample skill configuration for testing.
pub fn sample_skill_config() -> &'static str {
    r#"[[skills]]
name = "test-skill"
owner = "test-owner"
repo = "test-repo"
version = "1.0.0"
"#}

/// Sample invalid skill configuration (missing owner).
pub fn invalid_skill_missing_owner() -> &'static str {
    r#"[[skills]]
name = "test-skill"
repo = "test-repo"
version = "1.0.0"
"#}

/// Sample invalid skill configuration (missing repo).
pub fn invalid_skill_missing_repo() -> &'static str {
    r#"[[skills]]
name = "test-skill"
owner = "test-owner"
version = "1.0.0"
"#}

/// Sample empty skills configuration.
pub fn empty_skills_config() -> &'static str {
    r#"# Empty skills configuration
"#}

/// Sample switchboard.toml with minimal valid configuration.
pub fn minimal_switchboard_config() -> &'static str {
    r#"version = "1.0"
"#}

/// Sample switchboard.toml with full configuration.
pub fn full_switchboard_config() -> &'static str {
    r#"version = "1.0"

[agent]
name = "test-agent"
max_concurrent_tasks = 5
timeout_seconds = 300

[logging]
level = "debug"
format = "json"

[skills]
auto_install = true
registry_url = "https://github.com"
"#}

/// Creates a test workspace path.
pub fn test_workspace_path() -> PathBuf {
    PathBuf::from("/tmp/switchboard_test_workspace")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_skill_config_parses() {
        // Basic sanity check that the fixture is valid TOML
        toml::from_str::<toml::Value>(sample_skill_config()).expect("Valid TOML");
    }

    #[test]
    fn test_full_switchboard_config_parses() {
        toml::from_str::<toml::Value>(full_switchboard_config()).expect("Valid TOML");
    }
}
