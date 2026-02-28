//! Skills module - manages skill discovery and installation via npx skills CLI
//!
//! This module acts as a thin ergonomic wrapper around npx skills CLI.
//! All skill discovery and installation operations are delegated to npx skills.
//! Switchboard does not implement HTTP/GitHub API code directly.

mod error;
pub mod lockfile;
pub mod manager;
pub mod metadata;
pub mod validate;

pub use error::SkillsError;
pub use validate::{extract_skill_name, validate_skill_format};

// Re-export from manager module
pub use manager::{create_npx_command, run_npx_command, run_npx_skills_update, SkillsManager};

// Re-export from lockfile module
pub use lockfile::{
    add_skill_to_lockfile, default_lockfile, load_lockfile, read_lockfile,
    remove_skill_from_lockfile, save_lockfile, sync_skills_to_lockfile, write_lockfile,
    LockfileSkill, LockfileStruct, SkillLockEntry, SkillsLockfile, LOCKFILE_FILENAME,
};

// Re-export from metadata module
pub use metadata::{
    find_skill_directory, get_agents_using_skill, load_skill_metadata, parse_skill_frontmatter,
    read_skill_file, remove_skill_directory, scan_global_skills, scan_project_skills,
    scan_skill_directory, skills_sh_search, SkillMetadata, SkillSearchResult, SkillsSearchResponse,
};

/// Error message for when npx is not available
pub const NPX_NOT_FOUND_ERROR: &str =
    "Error: npx is required for this command. Install Node.js from https://nodejs.org";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::SkillsError;
    use std::fs;

    /// Test that verifies check_npx_available returns NpxNotFound error with installation instructions
    /// when npx is not available in PATH.
    ///
    /// This test validates:
    /// 1. The error type is SkillsError::NpxNotFound
    /// 2. The error message includes installation instructions (URL to nodejs.org)
    #[test]
    fn test_check_npx_available_error_contains_installation_instructions() {
        // Test the error type and message directly
        let error = SkillsError::NpxNotFound;
        let error_message = format!("{}", error);

        // Verify error type is correct
        assert_eq!(
            error,
            SkillsError::NpxNotFound,
            "Error type should be NpxNotFound"
        );

        // Verify error message includes installation instructions
        assert!(
            error_message.contains("https://nodejs.org"),
            "Error message should include installation URL, got: {}",
            error_message
        );
        assert!(
            error_message.contains("Install"),
            "Error message should contain 'Install' instruction, got: {}",
            error_message
        );

        // Verify the NPX_NOT_FOUND_ERROR constant also has installation instructions
        assert!(
            NPX_NOT_FOUND_ERROR.contains("https://nodejs.org"),
            "NPX_NOT_FOUND_ERROR should include installation URL"
        );
    }

    #[test]
    fn test_check_npx_available_when_npx_exists() {
        // This test assumes npx is available in the test environment
        let mut manager = SkillsManager::new(None);

        // The result depends on whether npx is installed in the test environment
        let result = manager.check_npx_available();

        if let Err(e) = result {
            assert_eq!(e, SkillsError::NpxNotFound);
            assert!(
                !manager.npx_available,
                "npx_available should be false when check fails"
            );
        } else {
            assert!(
                manager.npx_available,
                "npx_available should be true when check succeeds"
            );
        }
    }

    #[test]
    fn test_check_npx_available_sets_flag_correctly() {
        let mut manager = SkillsManager::new(None);

        let _ = manager.check_npx_available();

        // The flag should be set to match the check result
        let result = manager.check_npx_available();
        if result.is_ok() {
            assert!(manager.npx_available);
        } else {
            assert!(!manager.npx_available);
        }
    }

    #[tokio::test]
    async fn test_run_npx_command_invalid_executable() {
        // This should fail because npx may not be available in all environments
        let result = run_npx_command("nonexistent-cmd", &["--version"], None, None).await;

        // We expect either success (if somehow this works) or NpxCommandFailed error
        // The important thing is that it returns a proper Result type
        assert!(
            result.is_ok() || matches!(result.unwrap_err(), SkillsError::NpxCommandFailed { .. })
        );
    }

    #[test]
    fn test_npx_error_message_constant() {
        assert_eq!(
            NPX_NOT_FOUND_ERROR,
            "Error: npx is required for this command. Install Node.js from https://nodejs.org"
        );
    }

    #[test]
    fn test_skills_error_display_npx_not_found() {
        let error = SkillsError::NpxNotFound;
        let display = format!("{}", error);
        assert!(display.contains("npx is required"));
        assert!(display.contains("https://nodejs.org"));
    }

    #[test]
    fn test_skills_error_display_npx_command_failed() {
        let error = SkillsError::NpxCommandFailed {
            command: "skills add owner/repo".to_string(),
            exit_code: 1,
            stderr: "Skill not found".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("failed with exit code 1"));
        assert!(display.contains("skills add owner/repo"));
        assert!(display.contains("Skill not found"));
    }

    #[test]
    fn test_skills_error_display_skill_not_found() {
        let error = SkillsError::SkillNotFound {
            skill_source: "owner/repo@skill-name".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("owner/repo@skill-name"));
    }

    #[test]
    fn test_skills_error_display_malformed_skill_metadata() {
        let error = SkillsError::MalformedSkillMetadata {
            skill_name: "test-skill".to_string(),
            path: "/path/to/SKILL.md".to_string(),
            reason: "Missing required field 'description'".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("test-skill"));
        assert!(display.contains("/path/to/SKILL.md"));
        assert!(display.contains("Missing required field 'description'"));
    }

    #[test]
    fn test_skills_error_display_network_unavailable() {
        let error = SkillsError::NetworkUnavailable {
            operation: "list".to_string(),
            message: "Connection timeout".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("list"));
        assert!(display.contains("Connection timeout"));
    }

    #[test]
    fn test_skills_error_display_skill_name_collision() {
        let error = SkillsError::SkillNameCollision {
            skill_name: "test-skill".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("test-skill"));
        assert!(display.contains("project and global"));
    }

    #[test]
    fn test_skills_error_display_container_install_failed() {
        let error = SkillsError::ContainerInstallFailed {
            skill_source: "owner/repo@skill-name".to_string(),
            agent_name: "test-agent".to_string(),
            exit_code: 127,
            stderr: "command not found".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("test-agent"));
        assert!(display.contains("owner/repo@skill-name"));
        assert!(display.contains("code 127"));
    }

    #[test]
    fn test_skills_error_display_empty_skills_list() {
        let error = SkillsError::EmptySkillsList {
            agent_name: "test-agent".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("test-agent"));
        assert!(display.contains("empty skills list"));
    }

    #[test]
    fn test_skills_error_display_invalid_skills_entry_format() {
        let error = SkillsError::InvalidSkillsEntryFormat {
            entry: "invalid-entry".to_string(),
            agent_name: "test-agent".to_string(),
            reason: "Missing '/' separator".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("invalid-entry"));
        assert!(display.contains("test-agent"));
        assert!(display.contains("Missing '/' separator"));
    }

    #[test]
    fn test_skills_error_display_skills_directory_not_found() {
        let error = SkillsError::SkillsDirectoryNotFound {
            path: "/nonexistent/path".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("/nonexistent/path"));
    }

    #[test]
    fn test_skills_error_display_io_error() {
        let error = SkillsError::IoError {
            operation: "read".to_string(),
            path: "/path/to/file".to_string(),
            message: "Permission denied".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("read"));
        assert!(display.contains("/path/to/file"));
        assert!(display.contains("Permission denied"));
    }

    #[test]
    fn test_skills_error_debug() {
        let error = SkillsError::NpxNotFound;
        let debug = format!("{:?}", error);
        assert!(debug.contains("NpxNotFound"));
    }

    #[test]
    fn test_skills_error_clone() {
        let error = SkillsError::NpxNotFound;
        let cloned = error.clone();
        assert_eq!(format!("{}", error), format!("{}", cloned));
    }

    #[test]
    fn test_skills_manager_new() {
        let manager = SkillsManager::new(None);
        assert_eq!(manager.skills_dir, PathBuf::from("./skills"));
        // global_skills_dir now uses ./skills
        assert_eq!(manager.global_skills_dir, PathBuf::from("./skills"));
        assert!(!manager.npx_available);
    }

    #[test]
    fn test_skills_manager_with_skills_dir() {
        let custom_dir = PathBuf::from("/custom/skills");
        let manager = SkillsManager::with_skills_dir(custom_dir.clone(), None);
        assert_eq!(manager.skills_dir, custom_dir);
        // global_skills_dir now uses ./skills
        assert_eq!(manager.global_skills_dir, PathBuf::from("./skills"));
        assert!(!manager.npx_available);
    }

    #[test]
    fn test_skills_manager_skills_dir_getter() {
        let manager = SkillsManager::new(None);
        assert_eq!(manager.skills_dir(), &PathBuf::from("./skills"));
    }

    #[test]
    fn test_skills_manager_global_skills_dir_getter() {
        let manager = SkillsManager::new(None);
        // global_skills_dir now uses ./skills
        assert_eq!(manager.global_skills_dir(), &PathBuf::from("./skills"));
    }

    #[test]
    fn test_parse_skill_frontmatter_with_complete_metadata() {
        let content = r#"---
name: test-skill
description: A test skill for unit testing
source: https://github.com/owner/repo
version: 1.0.0
---
 "#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.name, "test-skill");
        assert_eq!(
            metadata.description,
            Some("A test skill for unit testing".to_string())
        );
        assert_eq!(
            metadata.source,
            Some("https://github.com/owner/repo".to_string())
        );
        assert_eq!(metadata.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_skill_frontmatter_with_minimal_metadata() {
        let content = r#"---
name: minimal-skill
---
 "#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.name, "minimal-skill");
        assert_eq!(metadata.description, None);
        assert_eq!(metadata.source, None);
        assert_eq!(metadata.version, None);
    }

    #[test]
    fn test_parse_skill_frontmatter_missing_frontmatter_delimiters() {
        let content = r#"name: test-skill
description: A test skill
 "#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SkillsError::MissingFrontmatter { .. }
        ));
    }

    #[test]
    fn test_parse_skill_frontmatter_missing_closing_delimiter() {
        let content = r#"---
name: test-skill
description: A test skill
 "#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SkillsError::MissingFrontmatter { .. }
        ));
    }

    #[test]
    fn test_parse_skill_frontmatter_empty_frontmatter() {
        let content = r#"---
---
 "#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SkillsError::FieldMissing { .. }
        ));
    }

    #[test]
    fn test_parse_skill_frontmatter_invalid_yaml() {
        let content = r#"---
name: [invalid yaml
description: test
---
"#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SkillsError::MalformedSkillMetadata { .. }
        ));
    }

    #[test]
    fn test_parse_skill_frontmatter_missing_name_field() {
        let content = r#"---
description: A test skill
---
"#;

        let result = parse_skill_frontmatter(content);
        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), SkillsError::FieldMissing { field_name, .. } if field_name == "name")
        );
    }

    #[test]
    fn test_load_skill_metadata_fallback_to_directory_name() {
        // Create a temporary directory structure
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");

        // Write a SKILL.md file without name field
        let content = r#"---
description: A test skill
---
"#;
        fs::write(&skill_file, content).unwrap();

        // Load metadata - should return error because parsing failed
        // scan_skill_directory() will handle the error and create fallback
        let result = load_skill_metadata(&skill_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_skill_directory_single_skill() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");

        let content = r#"---
name: test-skill
description: A test skill
---
"#;
        fs::write(&skill_file, content).unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_scan_skill_directory_multiple_skills() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create first skill
        let skill1_dir = temp_dir.path().join("skill1");
        fs::create_dir_all(&skill1_dir).unwrap();
        let skill1_file = skill1_dir.join("SKILL.md");
        let content1 = r#"---
name: skill1
description: First skill
---
"#;
        fs::write(&skill1_file, content1).unwrap();

        // Create second skill
        let skill2_dir = temp_dir.path().join("skill2");
        fs::create_dir_all(&skill2_dir).unwrap();
        let skill2_file = skill2_dir.join("SKILL.md");
        let content2 = r#"---
name: skill2
description: Second skill
---
"#;
        fs::write(&skill2_file, content2).unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();
        assert_eq!(skills.len(), 2);
        let skill_names: Vec<_> = skills.iter().map(|s| s.name.clone()).collect();
        assert!(skill_names.contains(&"skill1".to_string()));
        assert!(skill_names.contains(&"skill2".to_string()));
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_scan_skill_directory_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();
        assert_eq!(skills.len(), 0);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_scan_skill_directory_mixed_valid_invalid() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create valid skill
        let valid_dir = temp_dir.path().join("valid-skill");
        fs::create_dir_all(&valid_dir).unwrap();
        let valid_file = valid_dir.join("SKILL.md");
        let valid_content = r#"---
name: valid-skill
description: A valid skill
---
"#;
        fs::write(&valid_file, valid_content).unwrap();

        // Create directory without SKILL.md (should be skipped)
        let invalid_dir = temp_dir.path().join("no-skill");
        fs::create_dir_all(&invalid_dir).unwrap();

        // Create file (not a directory, should be skipped)
        let not_dir = temp_dir.path().join("not-a-dir.md");
        fs::write(&not_dir, "not a directory").unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "valid-skill");
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_get_agents_using_skill_with_repo_at_skill_format() {
        use crate::config::{Agent, Config};

        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec!["owner/repo@skill-name".to_string()]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["owner/repo@other-skill".to_string()]),
                ..Default::default()
            },
            Agent {
                name: "agent3".to_string(),
                skills: Some(vec![
                    "owner/repo@skill-name".to_string(),
                    "other/repo@another".to_string(),
                ]),
                ..Default::default()
            },
            Agent {
                name: "agent4".to_string(),
                skills: None,
                ..Default::default()
            },
        ];

        let agents = get_agents_using_skill("skill-name", &config);
        assert_eq!(agents.len(), 2);
        assert!(agents.contains(&"agent1".to_string()));
        assert!(agents.contains(&"agent3".to_string()));
    }

    #[test]
    fn test_get_agents_using_skill_with_skill_name_only() {
        use crate::config::{Agent, Config};

        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec!["skill-name".to_string()]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["other-skill".to_string()]),
                ..Default::default()
            },
        ];

        let agents = get_agents_using_skill("skill-name", &config);
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0], "agent1");
    }

    #[test]
    fn test_get_agents_using_skill_empty_result() {
        use crate::config::{Agent, Config};

        let mut config = Config::default();
        config.agents = vec![Agent {
            name: "agent1".to_string(),
            skills: Some(vec!["owner/repo@skill-name".to_string()]),
            ..Default::default()
        }];

        let agents = get_agents_using_skill("non-existent-skill", &config);
        assert_eq!(agents.len(), 0);
    }

    #[test]
    fn test_get_agents_using_skill_empty_skills_lists() {
        use crate::config::{Agent, Config};

        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec![]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: None,
                ..Default::default()
            },
        ];

        let agents = get_agents_using_skill("skill-name", &config);
        assert_eq!(agents.len(), 0);
    }

    #[test]
    fn test_scan_skill_directory_missing_frontmatter() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");

        // Write a SKILL.md file with NO frontmatter delimiters
        let content = "name: test-skill\n";
        fs::write(&skill_file, content).unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();

        // Assert that skills contains 1 skill with the directory name as fallback
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert_eq!(skills[0].description, None);
        assert_eq!(skills[0].version, None);
        // Warnings should be generated for parsing failure
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Warning"));
        assert!(warnings[0].contains("test-skill"));
    }

    #[test]
    fn test_scan_skill_directory_malformed_yaml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let skill_dir = temp_dir.path().join("test-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let skill_file = skill_dir.join("SKILL.md");

        // Write a SKILL.md file with invalid YAML
        let content = "---\nname: test-skill\ndescription: [unclosed bracket\n---";
        fs::write(&skill_file, content).unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();

        // Assert that skills contains 1 skill with the directory name as fallback
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "test-skill");
        assert_eq!(skills[0].description, None);
        assert_eq!(skills[0].version, None);
        // Warnings should be generated for parsing failure
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Warning"));
        assert!(warnings[0].contains("test-skill"));
    }

    #[test]
    fn test_scan_skill_directory_multiple_malformed_skills() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create first skill with malformed SKILL.md (missing frontmatter)
        let skill1_dir = temp_dir.path().join("skill1");
        fs::create_dir_all(&skill1_dir).unwrap();
        let skill1_file = skill1_dir.join("SKILL.md");
        let content1 = "name: skill1\n";
        fs::write(&skill1_file, content1).unwrap();

        // Create second skill with malformed SKILL.md (invalid YAML)
        let skill2_dir = temp_dir.path().join("skill2");
        fs::create_dir_all(&skill2_dir).unwrap();
        let skill2_file = skill2_dir.join("SKILL.md");
        let content2 = "---\nname: skill2\ndescription: [invalid\n---";
        fs::write(&skill2_file, content2).unwrap();

        let result = scan_skill_directory(temp_dir.path());
        assert!(result.is_ok());
        let (skills, warnings) = result.unwrap();

        // Assert that skills contains 2 skills (both with directory names as fallback)
        assert_eq!(skills.len(), 2);
        let skill_names: Vec<_> = skills.iter().map(|s| s.name.clone()).collect();
        assert!(skill_names.contains(&"skill1".to_string()));
        assert!(skill_names.contains(&"skill2".to_string()));
        // Warnings should be generated for both parsing failures
        assert_eq!(warnings.len(), 2);
        assert!(warnings.iter().any(|w| w.contains("skill1")));
        assert!(warnings.iter().any(|w| w.contains("skill2")));
    }

    // ========================================================================
    // Mock ProcessExecutor for testing
    // ========================================================================

    use crate::traits::{ExitStatus, ProcessError, ProcessExecutorTrait, ProcessOutput};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;

    /// Mock executor that returns predefined responses for testing
    #[derive(Debug)]
    struct MockProcessExecutor {
        /// If Some, return this output on execute. If None, return error.
        output: Option<ProcessOutput>,
        /// If true, return error instead of output
        should_error: bool,
    }

    impl MockProcessExecutor {
        fn with_success_output() -> Self {
            Self {
                output: Some(ProcessOutput {
                    stdout: "10.0.0".to_string(),
                    stderr: String::new(),
                    status: ExitStatus::Code(0),
                }),
                should_error: false,
            }
        }

        fn with_failure_output() -> Self {
            Self {
                output: Some(ProcessOutput {
                    stdout: String::new(),
                    stderr: "command not found".to_string(),
                    status: ExitStatus::Code(127),
                }),
                should_error: false,
            }
        }

        fn with_error() -> Self {
            Self {
                output: None,
                should_error: true,
            }
        }
    }

    impl ProcessExecutorTrait for MockProcessExecutor {
        fn execute(&self, _program: &str, _args: &[String]) -> Result<ProcessOutput, ProcessError> {
            if self.should_error {
                Err(ProcessError::ProgramNotFound {
                    program: "npx".to_string(),
                    suggestion: "Install npx or provide skills that don't require it".to_string(),
                })
            } else {
                self.output.clone().ok_or(ProcessError::ProgramNotFound {
                    program: "npx".to_string(),
                    suggestion: "Install npx or provide skills that don't require it".to_string(),
                })
            }
        }

        fn execute_with_env(
            &self,
            _program: &str,
            _args: &[String],
            _env: HashMap<String, String>,
            _working_dir: Option<PathBuf>,
        ) -> Result<ProcessOutput, ProcessError> {
            self.execute(_program, _args)
        }
    }

    // ========================================================================
    // SkillsManager with Mock Executor Tests
    // ========================================================================

    #[test]
    fn test_skills_manager_construction_with_custom_executor() {
        let mock_executor: Arc<dyn ProcessExecutorTrait> =
            Arc::new(MockProcessExecutor::with_success_output());
        let manager = SkillsManager::new(Some(mock_executor.clone()));

        // Verify the executor was injected correctly
        assert!(Arc::ptr_eq(&manager.executor, &mock_executor));
        // Verify default values
        assert_eq!(manager.skills_dir, PathBuf::from("./skills"));
        assert!(!manager.npx_available);
    }

    #[tokio::test]
    async fn test_check_npx_available_with_mock_success() {
        let mock_executor: Arc<dyn ProcessExecutorTrait> =
            Arc::new(MockProcessExecutor::with_success_output());
        let mut manager = SkillsManager::new(Some(mock_executor));

        // Execute check_npx_available with mocked successful response
        let result = manager.check_npx_available();

        // Should succeed because mock returns exit code 0
        assert!(
            result.is_ok(),
            "Expected success when mock returns exit code 0"
        );
        assert!(
            manager.npx_available,
            "npx_available should be true after successful check"
        );
    }

    #[tokio::test]
    async fn test_check_npx_available_with_mock_failure_exit_code() {
        let mock_executor: Arc<dyn ProcessExecutorTrait> =
            Arc::new(MockProcessExecutor::with_failure_output());
        let mut manager = SkillsManager::new(Some(mock_executor));

        // Execute check_npx_available with mocked failure response (exit code 127)
        let result = manager.check_npx_available();

        // Should fail because mock returns non-zero exit code
        assert!(
            result.is_err(),
            "Expected error when mock returns non-zero exit code"
        );
        assert!(
            !manager.npx_available,
            "npx_available should be false after failed check"
        );
    }

    #[tokio::test]
    async fn test_check_npx_available_with_mock_error() {
        let mock_executor: Arc<dyn ProcessExecutorTrait> =
            Arc::new(MockProcessExecutor::with_error());
        let mut manager = SkillsManager::new(Some(mock_executor));

        // Execute check_npx_available with mocked error
        let result = manager.check_npx_available();

        // Should fail because mock returns error
        assert!(result.is_err(), "Expected error when mock returns error");
        assert!(
            !manager.npx_available,
            "npx_available should be false after error"
        );
    }

    #[test]
    fn test_skills_manager_with_skills_dir_with_custom_executor() {
        let custom_skills_dir = PathBuf::from("/custom/path/skills");
        let mock_executor: Arc<dyn ProcessExecutorTrait> =
            Arc::new(MockProcessExecutor::with_success_output());
        let manager =
            SkillsManager::with_skills_dir(custom_skills_dir.clone(), Some(mock_executor));

        // Verify the custom skills_dir was set
        assert_eq!(manager.skills_dir, custom_skills_dir);
        // Verify default values
        assert!(!manager.npx_available);
    }

    // Tests for write_lockfile function
    use tempfile::TempDir;

    #[test]
    fn test_write_lockfile_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let lockfile = default_lockfile();

        let result = write_lockfile(&lockfile, temp_dir.path());
        assert!(result.is_ok());

        // Verify the file was created
        let lockfile_path = temp_dir.path().join(LOCKFILE_FILENAME);
        assert!(lockfile_path.exists());
    }

    #[test]
    fn test_write_lockfile_creates_directory_if_missing() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("nested").join("dir");
        let lockfile = default_lockfile();

        let result = write_lockfile(&lockfile, &nested_dir);
        assert!(result.is_ok());

        // Verify the nested directory and file were created
        let lockfile_path = nested_dir.join(LOCKFILE_FILENAME);
        assert!(lockfile_path.exists());
    }

    #[test]
    fn test_write_and_read_lockfile_roundtrip() {
        let temp_dir = TempDir::new().unwrap();

        // Create a lockfile with some skills
        let mut lockfile = default_lockfile();
        lockfile.skills.insert(
            "test-skill".to_string(),
            SkillLockEntry {
                source: "owner/test-repo".to_string(),
                skill_name: "test-skill".to_string(),
                installed_at: "2026-02-22T14:30:00Z".to_string(),
                version: None,
            },
        );

        // Write the lockfile
        let result = write_lockfile(&lockfile, temp_dir.path());
        assert!(result.is_ok());

        // Read it back
        let read_lockfile = read_lockfile(temp_dir.path());
        assert!(read_lockfile.is_ok());

        let loaded = read_lockfile.unwrap();
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.skills.len(), 1);
        assert!(loaded.skills.contains_key("test-skill"));
    }

    #[test]
    fn test_write_lockfile_pretty_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let lockfile = default_lockfile();

        write_lockfile(&lockfile, temp_dir.path()).unwrap();

        // Read the raw file content to verify pretty printing
        let lockfile_path = temp_dir.path().join(LOCKFILE_FILENAME);
        let content = fs::read_to_string(&lockfile_path).unwrap();

        // Verify the JSON is pretty-printed (contains newlines and indentation)
        assert!(content.contains('\n'));
        assert!(content.contains("  "));
    }

    #[test]
    fn test_add_skill_to_lockfile() {
        let temp_dir = TempDir::new().unwrap();

        // Add a skill
        let result = add_skill_to_lockfile(temp_dir.path(), "test-skill", "owner/repo");
        assert!(result.is_ok());

        // Verify the skill was added
        let lockfile = read_lockfile(temp_dir.path()).unwrap();
        assert!(lockfile.skills.contains_key("test-skill"));

        let skill = lockfile.skills.get("test-skill").unwrap();
        assert_eq!(skill.skill_name, "test-skill");
        assert_eq!(skill.source, "owner/repo");
        // Verify installed_at is a valid ISO 8601 timestamp
        assert!(chrono::DateTime::parse_from_rfc3339(&skill.installed_at).is_ok());
    }

    #[test]
    fn test_add_skill_to_lockfile_existing_skills() {
        let temp_dir = TempDir::new().unwrap();

        // Add first skill
        add_skill_to_lockfile(temp_dir.path(), "skill-one", "owner/repo1").unwrap();

        // Add second skill
        add_skill_to_lockfile(temp_dir.path(), "skill-two", "owner/repo2").unwrap();

        // Verify both skills exist
        let lockfile = read_lockfile(temp_dir.path()).unwrap();
        assert_eq!(lockfile.skills.len(), 2);
        assert!(lockfile.skills.contains_key("skill-one"));
        assert!(lockfile.skills.contains_key("skill-two"));
    }

    #[test]
    fn test_remove_skill_from_lockfile() {
        let temp_dir = TempDir::new().unwrap();

        // Add a skill first
        add_skill_to_lockfile(temp_dir.path(), "test-skill", "owner/repo").unwrap();

        // Verify it exists
        let lockfile = read_lockfile(temp_dir.path()).unwrap();
        assert!(lockfile.skills.contains_key("test-skill"));

        // Remove the skill
        let result = remove_skill_from_lockfile(temp_dir.path(), "test-skill");
        assert!(result.is_ok());

        // Verify it's gone
        let lockfile = read_lockfile(temp_dir.path()).unwrap();
        assert!(!lockfile.skills.contains_key("test-skill"));
    }

    #[test]
    fn test_remove_skill_from_lockfile_nonexistent() {
        let temp_dir = TempDir::new().unwrap();

        // Create an empty lockfile
        write_lockfile(&default_lockfile(), temp_dir.path()).unwrap();

        // Try to remove a non-existent skill
        let result = remove_skill_from_lockfile(temp_dir.path(), "nonexistent");
        assert!(result.is_ok()); // Should not fail, just no-op
    }

    #[test]
    fn test_lockfile_version() {
        let lockfile = default_lockfile();
        assert_eq!(lockfile.version, 1);
    }
}
