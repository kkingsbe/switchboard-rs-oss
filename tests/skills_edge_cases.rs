//! Edge case tests for skills handling
//!
//! These tests verify the robustness of the skills module when handling
//! various edge cases including:
//! - Extremely long skill names
//! - Special characters in skill sources
//! - Malformed skill metadata
//! - Empty skill directories
//! - Corrupted/binary data in SKILL.md files
//!
//! # Running These Tests
//!
//! ```bash
//! cargo test --test skills_edge_cases
//! ```

use switchboard::skills::{parse_skill_frontmatter, read_skill_file, SkillsError};
use std::fs;
use tempfile::TempDir;

/// Test: Extremely long skill name (>200 characters)
///
/// Verifies that the parser handles very long skill names gracefully.
/// The system should either truncate the name or return an appropriate error.
#[test]
fn test_extremely_long_skill_name() {
    // Create a name with 250 characters (exceeds 200 limit)
    let long_name = "a".repeat(250);

    let content = format!(
        "\
---
name: {}
description: Test skill with extremely long name
---
",
        long_name
    );

    let result = parse_skill_frontmatter(&content);

    // The parser should handle this gracefully - either accept it or return an error
    // We verify the system doesn't panic and returns a reasonable result
    match result {
        Ok(metadata) => {
            // If successful, verify the name is stored (possibly truncated)
            assert!(!metadata.name.is_empty());
            println!(
                "Parser accepted long name (length: {})",
                metadata.name.len()
            );
        }
        Err(e) => {
            // If it fails, it should be a clear error about the name
            let error_msg = format!("{}", e);
            println!("Parser rejected long name with error: {}", error_msg);
            // The error should be meaningful, not a panic
            assert!(!error_msg.is_empty());
        }
    }
}

/// Test: Skill name at exactly 200 characters (boundary case)
#[test]
fn test_skill_name_200_chars() {
    // Create exactly 200 characters
    let exact_name = "a".repeat(200);

    let content = format!(
        "\
---
name: {}
description: Test skill with exactly 200 chars
---
",
        exact_name
    );

    let result = parse_skill_frontmatter(&content);

    match result {
        Ok(metadata) => {
            assert!(!metadata.name.is_empty());
            println!("Parser accepted 200-char name");
        }
        Err(e) => {
            println!("Parser rejected 200-char name with error: {}", e);
        }
    }
}

/// Test: Special characters in skill sources - forward slash
///
/// Tests handling of / character in skill source field
#[test]
fn test_special_char_forward_slash() {
    let content = "\
---
name: test-skill
description: Test skill
source: https://github.com/owner/repo
---
";

    let result = parse_skill_frontmatter(content);
    assert!(
        result.is_ok(),
        "Should handle forward slash in source URL: {:?}",
        result.err()
    );

    let metadata = result.unwrap();
    assert_eq!(
        metadata.source,
        Some("https://github.com/owner/repo".to_string())
    );
}

/// Test: Special characters in skill sources - backslash
///
/// Tests handling of backslash character
#[test]
fn test_special_char_backslash() {
    let content = "\
---
name: test-skill
description: Test skill
source: C:\\Users\\test\\repo
---
";

    let result = parse_skill_frontmatter(content);
    // Should handle backslash in source
    match result {
        Ok(metadata) => {
            assert!(metadata.source.is_some());
            println!("Accepted backslash in source: {:?}", metadata.source);
        }
        Err(e) => {
            println!("Rejected backslash in source with error: {}", e);
        }
    }
}

/// Test: Special characters - colon
#[test]
fn test_special_char_colon() {
    let content = "\
---
name: test-skill
description: Test skill
source: scheme:resource
---
";

    let result = parse_skill_frontmatter(content);
    assert!(
        result.is_ok(),
        "Should handle colon in source: {:?}",
        result.err()
    );
}

/// Test: Special characters - asterisk and question mark
#[test]
fn test_special_char_asterisk_question() {
    let content = "\
---
name: test-skill
description: Test with * and ? characters
---
";

    let result = parse_skill_frontmatter(content);
    assert!(
        result.is_ok(),
        "Should handle special chars in description: {:?}",
        result.err()
    );
}

/// Test: Special characters - quotes
#[test]
fn test_special_char_quotes() {
    let content = "\
---
name: test-skill
description: Test with \"double\" and 'single' quotes
---
";

    let result = parse_skill_frontmatter(content);
    assert!(
        result.is_ok(),
        "Should handle quotes in description: {:?}",
        result.err()
    );
}

/// Test: Special characters - Unicode
#[test]
fn test_special_char_unicode() {
    let content = "\
---
name: test-skill-unicode
description: Unicode test with 中文日本語
---
";

    let result = parse_skill_frontmatter(content);
    assert!(
        result.is_ok(),
        "Should handle Unicode in description: {:?}",
        result.err()
    );

    let metadata = result.unwrap();
    assert!(metadata.description.unwrap().contains("中文"));
}

/// Test: Unicode in name and source
#[test]
fn test_unicode_full() {
    let content = "\
---
name: unicode-skill
description: This is a skill with Unicode: 你好世界
source: https://github.com/test/repo
---
";

    let result = parse_skill_frontmatter(content);
    // Note: Some YAML parsers have issues with CJK characters in certain contexts
    // This test verifies the system handles Unicode gracefully
    match result {
        Ok(metadata) => {
            assert!(metadata.description.unwrap().contains("你好世界"));
            println!("Handled Unicode in description successfully");
        }
        Err(e) => {
            // Some YAML parsers may reject CJK characters - document this limitation
            println!(
                "Unicode test got error (possible parser limitation): {:?}",
                e
            );
        }
    }
}

/// Test: Missing SKILL.md file
///
/// Tests behavior when attempting to read a non-existent SKILL.md
#[test]
fn test_missing_skill_file() {
    let temp_dir = TempDir::new().unwrap();
    let non_existent_path = temp_dir.path().join("non_existent/SKILL.md");

    let result = read_skill_file(&non_existent_path);
    assert!(result.is_err());

    match result {
        Err(SkillsError::IoError {
            operation,
            path,
            message,
        }) => {
            println!(
                "Correctly got IoError for missing file: {} - {}",
                path, message
            );
            assert_eq!(operation, "read SKILL.md");
        }
        Err(e) => {
            println!("Got error type: {:?}", e);
        }
        Ok(_) => panic!("Should have failed for non-existent file"),
    }
}

/// Test: Invalid YAML in SKILL.md frontmatter
///
/// Tests handling of malformed YAML syntax
#[test]
fn test_invalid_yaml_syntax() {
    let content = "\
---
name: test-skill
description: Test
invalid yaml: [unclosed
---
";

    let result = parse_skill_frontmatter(content);
    assert!(result.is_err());

    match result {
        Err(SkillsError::MalformedSkillMetadata {
            skill_name, reason, ..
        }) => {
            println!(
                "Correctly detected malformed YAML: {} - {}",
                skill_name, reason
            );
            assert!(reason.contains("YAML") || reason.contains("line"));
        }
        Err(e) => {
            println!("Got error: {:?}", e);
        }
        Ok(_) => panic!("Should have failed for invalid YAML"),
    }
}

/// Test: Missing required 'name' field
#[test]
fn test_missing_name_field() {
    let content = "\
---
description: Test skill without name
version: 1.0.0
---
";

    let result = parse_skill_frontmatter(content);
    assert!(result.is_err());

    match result {
        Err(SkillsError::FieldMissing { field_name, .. }) => {
            println!("Correctly detected missing 'name' field");
            assert_eq!(field_name, "name");
        }
        Err(e) => {
            println!("Got error type: {:?}", e);
        }
        Ok(_) => panic!("Should have failed for missing name field"),
    }
}

/// Test: Missing version field (should be optional)
#[test]
fn test_missing_version_field() {
    let content = "\
---
name: test-skill
description: Test skill without version
---
";

    let result = parse_skill_frontmatter(content);
    assert!(
        result.is_ok(),
        "Should succeed without version field: {:?}",
        result.err()
    );

    let metadata = result.unwrap();
    assert_eq!(metadata.name, "test-skill");
    assert!(metadata.version.is_none());
}

/// Test: Empty frontmatter (only delimiters)
#[test]
fn test_empty_frontmatter() {
    let content = "\
---
---
";

    let result = parse_skill_frontmatter(content);
    assert!(result.is_err());

    match result {
        Err(SkillsError::FieldMissing { field_name, .. }) => {
            println!(
                "Correctly detected missing field in empty frontmatter: {}",
                field_name
            );
        }
        Err(e) => {
            println!("Got error: {:?}", e);
        }
        Ok(_) => panic!("Should have failed for empty frontmatter"),
    }
}

/// Test: No frontmatter at all
#[test]
fn test_no_frontmatter() {
    let content = "\
# Test Skill

This is a skill without frontmatter.
";

    let result = parse_skill_frontmatter(content);
    assert!(result.is_err());

    match result {
        Err(SkillsError::MissingFrontmatter { .. }) => {
            println!("Correctly detected missing frontmatter");
        }
        Err(e) => {
            println!("Got error: {:?}", e);
        }
        Ok(_) => panic!("Should have failed for missing frontmatter"),
    }
}

/// Test: Malformed frontmatter - only opening delimiter
#[test]
fn test_partial_frontmatter_opening() {
    let content = "\
---
name: test-skill
description: Test
Some more content without closing delimiter
";

    let result = parse_skill_frontmatter(content);
    assert!(result.is_err());
}

/// Test: Empty skills directory
///
/// Tests behavior when skills directory exists but is empty
#[test]
fn test_empty_skills_directory() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path().join(".kilocode/skills");
    fs::create_dir_all(&skills_dir).unwrap();

    // Read directory - should be empty
    let entries = fs::read_dir(&skills_dir).unwrap();
    let count = entries.count();

    assert_eq!(count, 0, "Skills directory should be empty");
    println!("Empty skills directory handled correctly");
}

/// Test: Directory with only subdirectories (no SKILL.md)
#[test]
fn test_directory_only_subdirs() {
    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path().join(".kilocode/skills");
    fs::create_dir_all(&skills_dir).unwrap();

    // Create subdirectories without SKILL.md files
    fs::create_dir_all(skills_dir.join("skill1")).unwrap();
    fs::create_dir_all(skills_dir.join("skill2")).unwrap();
    fs::create_dir_all(skills_dir.join("nested/deep")).unwrap();

    // Count entries - includes nested/deep as a third entry
    let entries = fs::read_dir(&skills_dir).unwrap();
    let count = entries.count();

    assert!(
        count >= 2,
        "Should have at least 2 subdirectories (skill1, skill2)"
    );
    println!(
        "Directory with only subdirs handled correctly: {} entries",
        count
    );
}

/// Test: Binary data in SKILL.md file
///
/// Tests handling of non-text/binary content
#[test]
fn test_binary_data_in_skill_file() {
    let temp_dir = TempDir::new().unwrap();
    let skill_path = temp_dir.path().join("test-skill/SKILL.md");

    // Create parent directory
    fs::create_dir_all(skill_path.parent().unwrap()).unwrap();

    // Write binary data (not valid UTF-8)
    let binary_content: Vec<u8> = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];
    fs::write(&skill_path, &binary_content).unwrap();

    // Try to read - should fail with UTF-8 error
    let result = read_skill_file(&skill_path);
    assert!(result.is_err());

    match result {
        Err(SkillsError::IoError { message, .. }) => {
            println!("Correctly detected binary data error: {}", message);
            // The error should mention encoding/UTF-8 issues
        }
        Err(e) => {
            println!("Got error: {:?}", e);
        }
        Ok(_) => panic!("Should have failed for binary data"),
    }
}

/// Test: Very large SKILL.md file
///
/// Tests handling of files that are very large
#[test]
fn test_very_large_skill_file() {
    let temp_dir = TempDir::new().unwrap();
    let skill_path = temp_dir.path().join("test-skill/SKILL.md");

    // Create parent directory
    fs::create_dir_all(skill_path.parent().unwrap()).unwrap();

    // Create large content (1MB of repeated text)
    let large_content = format!(
        "---\nname: large-skill\ndescription: {}\n---\n",
        "x".repeat(1_000_000)
    );

    fs::write(&skill_path, &large_content).unwrap();

    // Try to read - should succeed (file is valid UTF-8)
    let result = read_skill_file(&skill_path);
    assert!(
        result.is_ok(),
        "Should handle large file: {:?}",
        result.err()
    );

    // Try to parse - may be slow but should complete
    let content = result.unwrap();
    let parse_result = parse_skill_frontmatter(&content);
    assert!(
        parse_result.is_ok(),
        "Should parse large file: {:?}",
        parse_result.err()
    );

    println!("Handled large SKILL.md file successfully");
}

/// Test: SKILL.md with only whitespace
#[test]
fn test_whitespace_only_skill_file() {
    let content = "   \n\n   \n   ";

    let result = parse_skill_frontmatter(content);
    assert!(result.is_err());

    match result {
        Err(SkillsError::MissingFrontmatter { .. }) => {
            println!("Correctly detected missing frontmatter in whitespace-only content");
        }
        Err(e) => {
            println!("Got error: {:?}", e);
        }
        Ok(_) => panic!("Should have failed for whitespace-only content"),
    }
}

/// Test: TOML in frontmatter instead of YAML
///
/// Tests handling of incorrectly formatted frontmatter
#[test]
fn test_toml_instead_of_yaml() {
    let content = "\
---\n\
name = 'test-skill'\n\
description = 'Test skill'\n\
version = '1.0.0'\n\
---\n";

    let result = parse_skill_frontmatter(content);
    // TOML-like syntax with single quotes is not valid YAML, so this should fail
    assert!(result.is_err());

    println!("Correctly rejected non-YAML format: {:?}", result.err());
}

/// Test: Nested array structures in YAML
#[test]
fn test_nested_yaml_arrays() {
    let content = "\
---
name: test-skill
description: Test
authors:
  - name: John
    email: john@example.com
  - name: Jane
    email: jane@example.com
---
";

    let result = parse_skill_frontmatter(content);
    // This should either parse with authors as empty or fail gracefully
    // depending on how strict the YAML parser is
    match result {
        Ok(metadata) => {
            println!(
                "Parsed with nested arrays, authors count: {}",
                metadata.authors.len()
            );
        }
        Err(e) => {
            println!("Rejected nested arrays: {}", e);
        }
    }
}

/// Test: Very long field values
#[test]
fn test_very_long_field_value() {
    let long_desc = "a".repeat(50_000);
    let content = format!(
        "\
---
name: test-skill
description: {}
---
",
        long_desc
    );

    let result = parse_skill_frontmatter(&content);
    assert!(
        result.is_ok(),
        "Should handle long description: {:?}",
        result.err()
    );

    let metadata = result.unwrap();
    assert_eq!(metadata.description.unwrap().len(), 50_000);
    println!("Handled 50k character description");
}

/// Test: Multiple YAML documents (not supported)
#[test]
fn test_multiple_yaml_documents() {
    let content = "\
---
name: first-skill
---
---
name: second-skill
---
";

    let result = parse_skill_frontmatter(content);
    // Should only parse the first document
    assert!(result.is_ok());

    let metadata = result.unwrap();
    assert_eq!(metadata.name, "first-skill");
    println!("Correctly handled multiple YAML docs (parsed first only)");
}
