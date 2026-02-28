//! Discord agent tools module.
//!
//! Provides tool definitions and execution for the Discord concierge integration,
//! allowing AI agents to perform file operations and system queries through
//! OpenAI function-calling format.
//!
//! # Submodules
//!
//! - [`definitions`] - Tool and ToolError types, JSON schema generation
//! - [`execution`] - Tool execution functions, path validation, LLM parsing

pub mod definitions;
pub mod execution;

// Re-export types and functions from submodules for convenient access
pub use definitions::{tools_schema, Tool, ToolError, MAX_FILE_SIZE};
pub use execution::{
    execute_add_to_backlog, execute_file_bug, execute_file_task, execute_list_directory,
    execute_list_inbox, execute_read_backlog, execute_read_file, execute_read_outbox,
    execute_read_todos, execute_tool, get_status, parse_tool_from_llm, slugify, validate_path,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_tools_schema_is_valid_json() {
        let schema = tools_schema();
        // Should be a JSON array
        assert!(schema.is_array());
        // Should have 10 tools
        assert_eq!(schema.as_array().unwrap().len(), 10);
    }

    #[test]
    fn test_validate_path_rejects_traversal() {
        let result = validate_path("../etc/passwd");
        assert!(result.is_err());

        let result = validate_path("foo/../../../bar");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_rejects_absolute() {
        let result = validate_path("/etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_accepts_relative() {
        let result = validate_path("src/main.rs");
        assert!(result.is_ok());

        let result = validate_path(".");
        assert!(result.is_ok());

        let result = validate_path("comms/outbox/message.md");
        assert!(result.is_ok());
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Test Task"), "test-task");
        assert_eq!(slugify("Some__Special_Chars"), "some-special-chars");
    }

    #[test]
    fn test_slugify_edge_cases() {
        // Test multiple spaces
        assert_eq!(slugify("Hello   World"), "hello-world");

        // Test leading/trailing spaces
        assert_eq!(slugify("  Hello World  "), "hello-world");

        // Test special characters
        assert_eq!(slugify("Task @#$%!"), "task");

        // Test numbers
        assert_eq!(slugify("Task 123 Test"), "task-123-test");

        // Test underscores
        assert_eq!(slugify("Task_Name"), "task-name");

        // Test mixed case
        assert_eq!(slugify("UPPERCASE"), "uppercase");
        assert_eq!(slugify("MixedCase"), "mixedcase");

        // Test single character
        assert_eq!(slugify("A"), "a");

        // Test empty after filtering (special chars only)
        assert_eq!(slugify("@#$%"), "");
    }

    #[test]
    fn test_tool_file_task_parsing() {
        // Test that Tool::FileTask can be constructed
        let tool = Tool::FileTask {
            title: "Test Title".to_string(),
            description: "Test Description".to_string(),
            priority: Some("high".to_string()),
        };

        match tool {
            Tool::FileTask {
                title,
                description,
                priority,
            } => {
                assert_eq!(title, "Test Title");
                assert_eq!(description, "Test Description");
                assert_eq!(priority, Some("high".to_string()));
            }
            _ => panic!("Expected FileTask variant"),
        }
    }

    #[test]
    fn test_tool_file_task_no_priority() {
        let tool = Tool::FileTask {
            title: "Test".to_string(),
            description: "Desc".to_string(),
            priority: None,
        };

        match tool {
            Tool::FileTask {
                title,
                description,
                priority,
            } => {
                assert_eq!(title, "Test");
                assert_eq!(description, "Desc");
                assert!(priority.is_none());
            }
            _ => panic!("Expected FileTask variant"),
        }
    }

    #[test]
    fn test_tools_schema_contains_file_task() {
        let schema = tools_schema();
        let schema_array = schema.as_array().unwrap();

        // Find file_task in the schema
        let file_task_schema = schema_array
            .iter()
            .find(|s| s.get("name").and_then(|n| n.as_str()) == Some("file_task"));

        assert!(file_task_schema.is_some(), "file_task schema not found");

        let schema_obj = file_task_schema.unwrap();

        // Verify description
        assert!(schema_obj.get("description").is_some());

        // Verify parameters
        let params = schema_obj.get("parameters").unwrap();
        let props = params.get("properties").unwrap();

        // Verify required fields
        let required = params.get("required").unwrap().as_array().unwrap();
        assert!(required.iter().any(|r| r.as_str() == Some("title")));
        assert!(required.iter().any(|r| r.as_str() == Some("description")));

        // Verify title field
        assert!(props.get("title").is_some());

        // Verify description field
        assert!(props.get("description").is_some());

        // Verify priority field with enum
        let priority_field = props.get("priority").unwrap();
        let priority_enum = priority_field.get("enum").unwrap().as_array().unwrap();
        assert!(priority_enum.iter().any(|e| e.as_str() == Some("high")));
        assert!(priority_enum.iter().any(|e| e.as_str() == Some("medium")));
        assert!(priority_enum.iter().any(|e| e.as_str() == Some("low")));
    }

    // =========================================================================
    // Tests that require temp directory
    // =========================================================================

    #[test]
    #[serial_test::serial]
    fn test_add_to_backlog_with_tag() {
        // Use temp directory to avoid polluting project directory
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        // Change to temp dir and ensure we restore on drop
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Use scope to ensure cwd is restored before any assertions
        let result = {
            let _backlog_path = Path::new("BACKLOG.md");
            execute_add_to_backlog("Test feature item", Some("FEATURE"))
        };

        // Restore cwd BEFORE assertions
        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Added to BACKLOG.md"));

        // Now read the file from temp dir (after restoring cwd)
        let backlog_path = temp_dir.path().join("BACKLOG.md");
        let content = fs::read_to_string(&backlog_path).unwrap();
        assert!(content.contains("[FEATURE] Test feature item"));

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_add_to_backlog_without_tag() {
        // Use temp directory to avoid polluting project directory
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        // Change to temp dir
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Execute in a scope so result is available after cwd restore
        let result = {
            let _backlog_path = Path::new("BACKLOG.md");
            execute_add_to_backlog("Untagged backlog item", None)
        };

        // Restore cwd BEFORE assertions
        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Added to BACKLOG.md"));

        // Verify the content has no tag prefix (read from temp dir)
        let backlog_path = temp_dir.path().join("BACKLOG.md");
        let content = fs::read_to_string(&backlog_path).unwrap();
        assert!(content.contains("Untagged backlog item"));
        // Should NOT contain brackets at the start
        assert!(!content.starts_with('['));

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_add_to_backlog_appends_to_existing() {
        // Use temp directory to avoid polluting project directory
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        // Change to temp dir
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let backlog_path = Path::new("BACKLOG.md");

        // Create initial BACKLOG.md with existing content
        fs::write(backlog_path, "Existing backlog item 1").unwrap();

        // Add a new item - should append
        let result = execute_add_to_backlog("New appended item", Some("IDEA"));

        // Restore cwd BEFORE assertions
        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());

        // Verify both items exist (read from temp dir)
        let backlog_path = temp_dir.path().join("BACKLOG.md");
        let content = fs::read_to_string(&backlog_path).unwrap();
        assert!(content.contains("Existing backlog item 1"));
        assert!(content.contains("[IDEA] New appended item"));

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_add_to_backlog_all_tags() {
        // Use temp directory to avoid polluting project directory
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        // Change to temp dir
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let backlog_path = Path::new("BACKLOG.md");

        // Test all tag types: FEATURE, BUG, CHORE, IDEA
        let tags = ["FEATURE", "BUG", "CHORE", "IDEA"];

        for tag in tags.iter() {
            let item = format!("Test {} item", tag);
            let result = execute_add_to_backlog(&item, Some(tag));
            assert!(result.is_ok(), "Failed for tag: {}", tag);

            let content = fs::read_to_string(backlog_path).unwrap();
            let expected = format!("[{}] {}", tag.to_uppercase(), item);
            assert!(
                content.contains(&expected),
                "Expected '{}' in content for tag {}",
                expected,
                tag
            );
        }

        // Restore cwd BEFORE final assertions
        std::env::set_current_dir(original_cwd).unwrap();

        // Verify final content has all items (read from temp dir)
        let backlog_path = temp_dir.path().join("BACKLOG.md");
        let content = fs::read_to_string(&backlog_path).unwrap();
        assert!(content.contains("[FEATURE] Test FEATURE item"));
        assert!(content.contains("[BUG] Test BUG item"));
        assert!(content.contains("[CHORE] Test CHORE item"));
        assert!(content.contains("[IDEA] Test IDEA item"));

        let _ = temp_dir.close();
    }

    // =========================================================================
    // Tests for file_task tool
    // =========================================================================

    #[test]
    #[serial_test::serial]
    fn test_execute_file_task_creates_file_in_inbox() {
        // Use a temp directory for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        // Change to temp dir and create comms/inbox structure
        std::env::set_current_dir(temp_dir.path()).unwrap();
        std::fs::create_dir_all("comms/inbox").unwrap();

        let result = execute_file_task("Test Task", "Test description", None);

        // Restore original directory
        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(message.contains("Task filed successfully"));

        // Verify file was created in comms/inbox (use absolute path to temp_dir)
        let inbox_path = temp_dir.path().join("comms/inbox");
        let inbox_files = std::fs::read_dir(&inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();
        assert!(!files.is_empty());

        // Check that at least one file matches the pattern
        let has_task_file = files
            .iter()
            .any(|f| f.file_name().to_string_lossy().contains("discord_task_"));
        assert!(has_task_file, "Expected a discord_task_ file in inbox");

        // Cleanup
        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_execute_file_task_filename_format() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        // Pre-create the inbox directory
        std::fs::create_dir_all("comms/inbox").unwrap();

        let result = execute_file_task("Test Task", "Description", None);

        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());

        // Read the created file - use absolute path to temp_dir
        let inbox_path = temp_dir.path().join("comms/inbox");
        let inbox_files = std::fs::read_dir(&inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();

        // Should have at least 1 file (may have more from previous tests)
        assert!(!files.is_empty(), "Expected at least 1 file in inbox");
        let filename = files[0].file_name().to_string_lossy().to_string();

        // Check filename format: YYYY-MM-DD_HHMMSS_discord_task_<slug>.md
        let regex =
            regex::Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{6}_discord_task_test-task\.md$").unwrap();
        assert!(
            regex.is_match(&filename),
            "Filename {} does not match expected format",
            filename
        );

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_execute_file_task_content_structure() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        std::fs::create_dir_all("comms/inbox").unwrap();

        let result = execute_file_task("My Test Task", "This is a detailed description", None);

        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());

        // Read the created file content
        let inbox_path = temp_dir.path().join("comms/inbox");
        let inbox_files = std::fs::read_dir(&inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();

        let content = std::fs::read_to_string(files[0].path()).unwrap();

        // Verify content includes title
        assert!(content.contains("# Task: My Test Task"));

        // Verify content includes source
        assert!(content.contains("**Source:** Discord"));

        // Verify content includes default priority (medium)
        assert!(content.contains("**Priority:** medium"));

        // Verify content includes description
        assert!(content.contains("## Description"));
        assert!(content.contains("This is a detailed description"));

        // Verify content includes date
        assert!(content.contains("**Date:**"));

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_execute_file_task_priority_high() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        // Pre-create the inbox directory
        std::fs::create_dir_all("comms/inbox").unwrap();

        let result = execute_file_task("High Priority Task", "Description", Some("high"));

        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());

        // Read the file - use absolute path to temp_dir
        let inbox_path = temp_dir.path().join("comms/inbox");
        let inbox_files = std::fs::read_dir(&inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();
        let content = std::fs::read_to_string(files[0].path()).unwrap();

        assert!(content.contains("**Priority:** high"));

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_execute_file_task_priority_low() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        // Pre-create the inbox directory
        std::fs::create_dir_all("comms/inbox").unwrap();

        let result = execute_file_task("Low Priority Task", "Description", Some("low"));

        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());

        // Read the file - use absolute path to temp_dir
        let inbox_path = temp_dir.path().join("comms/inbox");
        let inbox_files = std::fs::read_dir(&inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();
        let content = std::fs::read_to_string(files[0].path()).unwrap();

        assert!(content.contains("**Priority:** low"));

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_execute_file_task_priority_invalid_defaults_to_medium() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        // Pre-create the inbox directory
        std::fs::create_dir_all("comms/inbox").unwrap();

        // Use an invalid priority value
        let result = execute_file_task("Task", "Description", Some("urgent"));

        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());

        // Read the file - use absolute path to temp_dir
        let inbox_path = temp_dir.path().join("comms/inbox");
        let inbox_files = std::fs::read_dir(&inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();
        let content = std::fs::read_to_string(files[0].path()).unwrap();

        // Should default to medium for invalid priority
        assert!(content.contains("**Priority:** medium"));

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_execute_tool_dispatches_file_task() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        std::fs::create_dir_all("comms/inbox").unwrap();

        let tool = Tool::FileTask {
            title: "Dispatched Task".to_string(),
            description: "Via execute_tool".to_string(),
            priority: Some("low".to_string()),
        };

        let result = execute_tool(tool);

        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Task filed successfully"));

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_file_task_creates_inbox_if_not_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        // Pre-create the comms directory but NOT the inbox - let the function create it
        std::fs::create_dir_all("comms").unwrap();

        let result = execute_file_task("New Inbox Task", "Description", None);

        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());

        // Verify inbox directory was created - use absolute path to temp_dir
        let inbox_path = temp_dir.path().join("comms/inbox");
        assert!(
            inbox_path.exists(),
            "Inbox directory should exist at {:?}",
            inbox_path
        );

        // Verify file was created
        let inbox_files = std::fs::read_dir(&inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();
        assert!(!files.is_empty());

        let _ = temp_dir.close();
    }

    // =========================================================================
    // Tests for file_bug tool
    // =========================================================================

    #[test]
    #[serial_test::serial]
    fn test_execute_file_bug_creates_file_in_inbox() {
        // Use a temp directory for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        // Change to temp dir and create comms/inbox structure
        std::env::set_current_dir(temp_dir.path()).unwrap();
        std::fs::create_dir_all("comms/inbox").unwrap();

        let result = execute_file_bug("Test Bug", "Test description", None);

        // Verify file was created in comms/inbox BEFORE restoring cwd
        let inbox_path = Path::new("comms/inbox");
        let inbox_files = std::fs::read_dir(inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();

        // Restore original directory
        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(message.contains("Bug report filed"));

        assert!(!files.is_empty());

        // Check that at least one file matches the pattern
        let has_bug_file = files
            .iter()
            .any(|f| f.file_name().to_string_lossy().contains("discord_bug_"));
        assert!(has_bug_file, "Expected a discord_bug_ file in inbox");

        // Cleanup
        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_execute_file_bug_filename_format() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        // Pre-create the inbox directory
        std::fs::create_dir_all("comms/inbox").unwrap();

        let result = execute_file_bug("Test Bug", "Description", None);

        // Read the created file BEFORE restoring cwd
        let inbox_path = Path::new("comms/inbox");
        let inbox_files = std::fs::read_dir(inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();

        // Restore cwd BEFORE assertions
        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());

        // Should have at least 1 file
        assert!(!files.is_empty(), "Expected at least 1 file in inbox");
        let filename = files[0].file_name().to_string_lossy().to_string();

        // Check filename format: YYYY-MM-DD_HHMMSS_discord_bug_<slug>.md
        let regex =
            regex::Regex::new(r"^\d{4}-\d{2}-\d{2}_\d{6}_discord_bug_test-bug\.md$").unwrap();
        assert!(
            regex.is_match(&filename),
            "Filename {} does not match expected format",
            filename
        );

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_execute_file_bug_content_structure() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        std::fs::create_dir_all("comms/inbox").unwrap();

        let result = execute_file_bug("My Test Bug", "This is a detailed bug description", None);

        // Read the created file content BEFORE restoring cwd
        let inbox_path = Path::new("comms/inbox");
        let inbox_files = std::fs::read_dir(inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();
        let content = std::fs::read_to_string(files[0].path()).unwrap();

        // Restore cwd BEFORE assertions
        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());

        // Verify content includes title
        assert!(content.contains("# Bug Report: My Test Bug"));

        // Verify content includes source
        assert!(content.contains("Filed via: Discord"));

        // Verify content includes default severity (medium)
        assert!(content.contains("> Severity: medium"));

        // Verify content includes description
        assert!(content.contains("## Description"));
        assert!(content.contains("This is a detailed bug description"));

        // Verify content includes date
        assert!(content.contains("> Date:"));

        // Verify content includes status section
        assert!(content.contains("## Status"));
        assert!(content.contains("**Status:** Triage Pending"));

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_execute_file_bug_severity_critical() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        // Pre-create the inbox directory
        std::fs::create_dir_all("comms/inbox").unwrap();

        let result = execute_file_bug("Critical Bug", "Description", Some("critical"));

        // Read the file BEFORE restoring cwd
        let inbox_path = Path::new("comms/inbox");
        let inbox_files = std::fs::read_dir(inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();
        let content = std::fs::read_to_string(files[0].path()).unwrap();

        // Restore cwd BEFORE assertions
        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());

        assert!(content.contains("> Severity: critical"));
        assert!(content.contains("**Priority:** critical"));

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_execute_file_bug_severity_low() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        // Pre-create the inbox directory
        std::fs::create_dir_all("comms/inbox").unwrap();

        let result = execute_file_bug("Low Priority Bug", "Description", Some("low"));

        // Read the file BEFORE restoring cwd
        let inbox_path = Path::new("comms/inbox");
        let inbox_files = std::fs::read_dir(inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();
        let content = std::fs::read_to_string(files[0].path()).unwrap();

        // Restore cwd BEFORE assertions
        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());

        assert!(content.contains("> Severity: low"));
        assert!(content.contains("**Priority:** low"));

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_execute_file_bug_severity_defaults_to_medium() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        // Pre-create the inbox directory
        std::fs::create_dir_all("comms/inbox").unwrap();

        // Use None (no severity specified)
        let result = execute_file_bug("Bug Without Severity", "Description", None);

        // Read the file BEFORE restoring cwd
        let inbox_path = Path::new("comms/inbox");
        let inbox_files = std::fs::read_dir(inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();
        let content = std::fs::read_to_string(files[0].path()).unwrap();

        // Restore cwd BEFORE assertions
        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());

        // Should default to medium when severity is None
        assert!(content.contains("> Severity: medium"));
        assert!(content.contains("**Priority:** medium"));

        let _ = temp_dir.close();
    }

    #[test]
    #[serial_test::serial]
    fn test_execute_file_bug_creates_inbox_if_not_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();
        // Pre-create the comms directory but NOT the inbox - let the function create it
        std::fs::create_dir_all("comms").unwrap();

        let result = execute_file_bug("New Inbox Bug", "Description", None);

        // Verify inbox directory was created BEFORE restoring cwd
        let inbox_path = Path::new("comms/inbox");
        assert!(inbox_path.exists(), "Inbox directory should exist");

        // Verify file was created
        let inbox_files = std::fs::read_dir(inbox_path).unwrap();
        let files: Vec<_> = inbox_files.filter_map(|e| e.ok()).collect();

        // Restore cwd BEFORE assertions
        std::env::set_current_dir(original_cwd).unwrap();

        assert!(result.is_ok());
        assert!(!files.is_empty());

        let _ = temp_dir.close();
    }
}
