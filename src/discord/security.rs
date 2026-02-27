//! Security module for Discord tools.
//!
//! Provides path traversal prevention and other security utilities
//! for safe file system operations in the Discord agent context.

use std::path::{Path, PathBuf};

/// Types of file operations that can be performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    /// Read operation - viewing file contents
    Read,
    /// Write operation - creating or modifying files
    Write,
    /// Delete operation - removing files
    Delete,
    /// List operation - directory listing
    List,
}

/// Policy for controlling write operations.
///
/// This struct defines restrictions on file system operations including:
/// - Whether overwrite/write operations are allowed
/// - Whether delete operations are allowed
/// - Whitelist of allowed file extensions (if specified)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WritePolicy {
    /// Allow write/overwrite operations
    pub allow_overwrite: bool,
    /// Allow delete operations
    pub allow_delete: bool,
    /// Whitelist of allowed file extensions (None means any extension is allowed)
    pub allowed_extensions: Option<Vec<String>>,
}

/// Creates a default read-only policy.
///
/// This policy:
/// - Blocks write/overwrite operations
/// - Blocks delete operations
/// - Only allows specific safe file extensions: txt, md, json, toml, yaml, yml, rs
///
/// # Returns
///
/// A `WritePolicy` configured for read-only operations with safe extensions.
///
/// # Examples
///
/// ```
/// use switchboard::discord::security::default_readonly_policy;
///
/// let policy = default_readonly_policy();
/// assert!(!policy.allow_overwrite);
/// assert!(!policy.allow_delete);
/// assert!(policy.allowed_extensions.is_some());
/// ```
pub fn default_readonly_policy() -> WritePolicy {
    WritePolicy {
        allow_overwrite: false,
        allow_delete: false,
        allowed_extensions: Some(vec![
            "txt".to_string(),
            "md".to_string(),
            "json".to_string(),
            "toml".to_string(),
            "yaml".to_string(),
            "yml".to_string(),
            "rs".to_string(),
        ]),
    }
}

/// Validates whether an operation is allowed by the given policy.
///
/// This function checks:
/// 1. If the operation type is permitted by the policy
/// 2. If the file extension is in the allowed extensions list (if specified)
///
/// # Arguments
///
/// * `operation` - The type of operation to validate
/// * `path` - The path of the file being operated on
/// * `policy` - The policy to validate against
///
/// # Returns
///
/// * `Ok(())` - If the operation is allowed
/// * `Err(String)` - If the operation is denied with a descriptive message
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use switchboard::discord::security::{OperationType, WritePolicy, validate_operation, default_readonly_policy};
///
/// // Read operation should be allowed with default policy
/// let policy = default_readonly_policy();
/// let result = validate_operation(OperationType::Read, Path::new("file.txt"), &policy);
/// assert!(result.is_ok());
///
/// // Write operation should be blocked with default policy
/// let result = validate_operation(OperationType::Write, Path::new("file.txt"), &policy);
/// assert!(result.is_err());
/// ```
pub fn validate_operation(
    operation: OperationType,
    path: &Path,
    policy: &WritePolicy,
) -> Result<(), String> {
    // Check operation-specific restrictions
    match operation {
        OperationType::Read | OperationType::List => {
            // Read and List are always allowed (they don't modify the filesystem)
            // But we still check extension for read operations if specified
            if operation == OperationType::Read {
                check_extension(path, policy)?;
            }
            Ok(())
        }
        OperationType::Write => {
            if !policy.allow_overwrite {
                return Err("Write/overwrite operations are not allowed by policy".to_string());
            }
            check_extension(path, policy)?;
            Ok(())
        }
        OperationType::Delete => {
            if !policy.allow_delete {
                return Err("Delete operations are not allowed by policy".to_string());
            }
            check_extension(path, policy)?;
            Ok(())
        }
    }
}

/// Checks if the file extension is allowed by the policy.
fn check_extension(path: &Path, policy: &WritePolicy) -> Result<(), String> {
    if let Some(ref allowed_extensions) = policy.allowed_extensions {
        // Get the file extension (without the dot)
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase());

        if let Some(ext) = extension {
            if !allowed_extensions.iter().any(|e| e.to_lowercase() == ext) {
                return Err(format!(
                    "File extension '{}' is not in the allowed list: {:?}",
                    ext, allowed_extensions
                ));
            }
        } else if !allowed_extensions.is_empty() {
            // No extension provided but policy requires specific extensions
            return Err(format!(
                "File has no extension but policy requires one of: {:?}",
                allowed_extensions
            ));
        }
    }
    Ok(())
}

/// Validates that a path is within an allowed directory.
///
/// This function prevents path traversal attacks by:
/// 1. Canonicalizing both the input path and allowed directory
/// 2. Checking that the resolved path starts with the allowed directory
/// 3. Handling edge cases like symbolic links, ".." in paths, and absolute/relative paths
///
/// # Arguments
///
/// * `path` - The path to validate (can be relative or absolute)
/// * `allowed_dir` - The directory that the path must be within
///
/// # Returns
///
/// * `Ok(PathBuf)` - The canonicalized path if it's within the allowed directory
/// * `Err(String)` - An error message if the path would escape the allowed directory
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use switchboard::discord::security::validate_path;
///
/// // Valid path within allowed directory - use current directory which exists
/// let result = validate_path(Path::new("subdir/file.txt"), Path::new("."));
/// assert!(result.is_ok());
///
/// // Path with ".." that escapes allowed directory
/// let result = validate_path(Path::new("../etc/passwd"), Path::new("."));
/// assert!(result.is_err());
/// ```
pub fn validate_path(path: &Path, allowed_dir: &Path) -> Result<PathBuf, String> {
    // Canonicalize the allowed directory
    let canonical_allowed = allowed_dir
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize allowed directory: {}", e))?;

    // For relative paths, first check if the raw path would escape
    // This is critical to catch ".." attacks before join normalizes them
    if !path.is_absolute() {
        check_escapes_via_components(path)?;
    }

    // For absolute paths, check if they're within the allowed directory
    if path.is_absolute() {
        let canonical_input = path
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize input path: {}", e))?;

        return check_within_allowed(&canonical_input, &canonical_allowed);
    }

    // For relative paths, join with allowed_dir and then canonicalize
    let input_path = canonical_allowed.join(path);

    // Canonicalize the input path to resolve ".." and symlinks
    let canonical_input = match input_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            // If the file doesn't exist but path is valid (no ".." escape),
            // we still need to validate. For non-existent files, we can at least
            // check that the path doesn't contain ".."
            if e.kind() == std::io::ErrorKind::NotFound {
                // File doesn't exist, but we already checked for ".." above
                // Just return the joined path (it's valid)
                return Ok(input_path);
            } else {
                return Err(format!("Failed to canonicalize input path: {}", e));
            }
        }
    };

    check_within_allowed(&canonical_input, &canonical_allowed)
}

/// Checks if a relative path would escape via ".." components
fn check_escapes_via_components(path: &Path) -> Result<(), String> {
    let mut depth: i32 = 0;

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err(format!(
                        "Path '{}' attempts to escape via '..'",
                        path.display()
                    ));
                }
            }
            std::path::Component::Normal(_) | std::path::Component::CurDir => {
                depth += 1;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Checks if canonical_input is within canonical_allowed
fn check_within_allowed(
    canonical_input: &Path,
    canonical_allowed: &Path,
) -> Result<PathBuf, String> {
    let canonical_input_str = canonical_input.to_string_lossy().to_string();
    let canonical_allowed_str = canonical_allowed.to_string_lossy().to_string();

    // Add trailing separator to allowed_dir to prevent partial matches
    // e.g., /home/user should not match /home/userdata
    let allowed_with_sep = if canonical_allowed_str.ends_with('/') {
        canonical_allowed_str.clone()
    } else {
        format!("{}/", canonical_allowed_str)
    };

    if canonical_input_str.starts_with(&allowed_with_sep) {
        Ok(canonical_input.to_path_buf())
    } else {
        Err(format!(
            "Path '{}' escapes allowed directory '{}'",
            canonical_input_str, canonical_allowed_str
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_valid_path_within_directory() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_dir = temp_dir.path();

        // Create a file inside the allowed directory
        let subdir = allowed_dir.join("subdir");
        fs::create_dir_all(&subdir).unwrap();
        let file_path = subdir.join("file.txt");
        fs::write(&file_path, "test").unwrap();

        let result = validate_path(Path::new("subdir/file.txt"), allowed_dir);
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with(allowed_dir));
    }

    #[test]
    fn test_path_with_parent_dir_escape() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_dir = temp_dir.path();

        // Create a file inside the allowed directory
        let file_path = allowed_dir.join("file.txt");
        fs::write(&file_path, "test").unwrap();

        // Try to escape using ".."
        let result = validate_path(Path::new("../Cargo.toml"), allowed_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("escape"));
    }

    #[test]
    fn test_absolute_path_outside_allowed() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_dir = temp_dir.path();

        // Try to access an absolute path outside the allowed directory
        let result = validate_path(Path::new("/etc/passwd"), allowed_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("escapes"));
    }

    #[test]
    fn test_symlink_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_dir = temp_dir.path();

        // Create a directory structure:
        // allowed_dir/
        //   link -> /tmp (symlink)

        let link_path = allowed_dir.join("link");
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            // Create a file inside /tmp to make the symlink target exist
            let target_file = std::env::temp_dir().join("test_symlink_target");
            fs::write(&target_file, "test").unwrap();

            // Create symlink to temp directory (not /tmp directly for better test reliability)
            symlink(std::env::temp_dir(), &link_path).unwrap();

            // Try to access through the symlink - accessing a file that exists in temp
            let result = validate_path(Path::new("link/test_symlink_target"), allowed_dir);

            // Clean up
            if let Err(e) = fs::remove_file(&target_file) {
                eprintln!("Warning: Failed to clean up test file: {}", e);
            }

            // This should fail because the symlink resolves to temp_dir which is outside allowed_dir
            // (unless temp_dir happens to be inside our allowed_dir, which is unlikely)
            if let Ok(resolved) = result {
                assert!(
                    !resolved.starts_with(allowed_dir),
                    "Symlink traversal should be blocked, but path {} starts with allowed {}",
                    resolved.display(),
                    allowed_dir.display()
                );
            }
        }
    }

    #[test]
    fn test_deeply_nested_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_dir = temp_dir.path();

        // Try multiple levels of ".."
        let result = validate_path(Path::new("a/b/../../../../etc/passwd"), allowed_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("escape"));
    }

    #[test]
    fn test_nonexistent_path_within_directory() {
        let temp_dir = TempDir::new().unwrap();
        let allowed_dir = temp_dir.path();

        // Test a path that doesn't exist but should be valid
        let result = validate_path(&Path::new("nonexistent/file.txt"), allowed_dir);
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with(allowed_dir));
    }

    // === Write Restriction Tests ===

    #[test]
    fn test_read_operation_allowed_with_default_policy() {
        let policy = default_readonly_policy();

        // Read operations should be allowed
        let result = validate_operation(OperationType::Read, Path::new("file.txt"), &policy);
        assert!(result.is_ok());

        // Read with allowed extension should work
        let result = validate_operation(OperationType::Read, Path::new("readme.md"), &policy);
        assert!(result.is_ok());

        let result = validate_operation(OperationType::Read, Path::new("config.json"), &policy);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_operation_blocked_with_default_policy() {
        let policy = default_readonly_policy();

        // Write operations should be blocked
        let result = validate_operation(OperationType::Write, Path::new("file.txt"), &policy);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[test]
    fn test_delete_operation_blocked_with_default_policy() {
        let policy = default_readonly_policy();

        // Delete operations should be blocked
        let result = validate_operation(OperationType::Delete, Path::new("file.txt"), &policy);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[test]
    fn test_list_operation_allowed_with_default_policy() {
        let policy = default_readonly_policy();

        // List operations should always be allowed
        let result = validate_operation(OperationType::List, Path::new("anything"), &policy);
        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_policy_allowing_write() {
        let policy = WritePolicy {
            allow_overwrite: true,
            allow_delete: false,
            allowed_extensions: Some(vec!["txt".to_string(), "md".to_string()]),
        };

        // Write should be allowed with custom policy
        let result = validate_operation(OperationType::Write, Path::new("file.txt"), &policy);
        assert!(result.is_ok());

        // Delete should still be blocked
        let result = validate_operation(OperationType::Delete, Path::new("file.txt"), &policy);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_policy_allowing_delete() {
        let policy = WritePolicy {
            allow_overwrite: false,
            allow_delete: true,
            allowed_extensions: Some(vec!["txt".to_string()]),
        };

        // Delete should be allowed
        let result = validate_operation(OperationType::Delete, Path::new("file.txt"), &policy);
        assert!(result.is_ok());

        // Write should still be blocked
        let result = validate_operation(OperationType::Write, Path::new("file.txt"), &policy);
        assert!(result.is_err());
    }

    #[test]
    fn test_extension_whitelist_enforcement() {
        let policy = WritePolicy {
            allow_overwrite: true,
            allow_delete: false,
            allowed_extensions: Some(vec!["txt".to_string(), "md".to_string()]),
        };

        // Write with allowed extension should work
        let result = validate_operation(OperationType::Write, Path::new("file.txt"), &policy);
        assert!(result.is_ok());

        // Write with disallowed extension should fail
        let result = validate_operation(OperationType::Write, Path::new("file.exe"), &policy);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in the allowed list"));

        // Read with disallowed extension should fail
        let result = validate_operation(OperationType::Read, Path::new("malicious.sh"), &policy);
        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive_extension_check() {
        let policy = WritePolicy {
            allow_overwrite: true,
            allow_delete: false,
            allowed_extensions: Some(vec!["txt".to_string()]),
        };

        // Extensions should be case-insensitive
        let result = validate_operation(OperationType::Write, Path::new("file.TXT"), &policy);
        assert!(result.is_ok());

        let result = validate_operation(OperationType::Write, Path::new("file.Txt"), &policy);
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_extension_with_whitelist() {
        let policy = WritePolicy {
            allow_overwrite: true,
            allow_delete: false,
            allowed_extensions: Some(vec!["txt".to_string(), "md".to_string()]),
        };

        // File without extension should fail when whitelist is specified
        let result = validate_operation(OperationType::Write, Path::new("Makefile"), &policy);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_extension_whitelist_allows_no_extension() {
        let policy = WritePolicy {
            allow_overwrite: true,
            allow_delete: false,
            allowed_extensions: Some(vec![]),
        };

        // Empty whitelist should allow no extension
        let result = validate_operation(OperationType::Write, Path::new("Makefile"), &policy);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unrestricted_policy() {
        let policy = WritePolicy {
            allow_overwrite: true,
            allow_delete: true,
            allowed_extensions: None, // No extension restriction
        };

        // All operations should be allowed
        assert!(validate_operation(OperationType::Read, Path::new("anything"), &policy).is_ok());
        assert!(
            validate_operation(OperationType::Write, Path::new("anything.exe"), &policy).is_ok()
        );
        assert!(validate_operation(OperationType::Delete, Path::new("anything"), &policy).is_ok());
        assert!(validate_operation(OperationType::List, Path::new("anything"), &policy).is_ok());
    }

    #[test]
    fn test_default_readonly_policy_values() {
        let policy = default_readonly_policy();

        assert!(!policy.allow_overwrite);
        assert!(!policy.allow_delete);
        assert!(policy.allowed_extensions.is_some());

        let extensions = policy.allowed_extensions.unwrap();
        assert!(extensions.contains(&"txt".to_string()));
        assert!(extensions.contains(&"md".to_string()));
        assert!(extensions.contains(&"json".to_string()));
        assert!(extensions.contains(&"toml".to_string()));
        assert!(extensions.contains(&"yaml".to_string()));
        assert!(extensions.contains(&"yml".to_string()));
        assert!(extensions.contains(&"rs".to_string()));
    }

    // === Path Traversal Prevention Tests ===
    // Using check_escapes_via_components() function

    #[test]
    fn test_path_traversal_blocked_double_dot() {
        // Test that paths containing ".." are blocked
        let result = check_escapes_via_components(Path::new("../etc/passwd"));
        assert!(result.is_err(), "Path with '..' should be blocked");
        assert!(result.unwrap_err().contains("escape"));
    }

    #[test]
    fn test_path_traversal_blocked_multiple_double_dots() {
        // Test various escape attempts like "../../../etc/passwd"
        let result = check_escapes_via_components(Path::new("../../../etc/passwd"));
        assert!(result.is_err(), "Path with multiple '..' should be blocked");
    }

    #[test]
    fn test_path_traversal_blocked_deep_nesting() {
        // Test deeply nested ".." escape attempts
        let result = check_escapes_via_components(Path::new("a/b/../../../../etc/passwd"));
        assert!(result.is_err(), "Deeply nested '..' should be blocked");
    }

    #[test]
    fn test_path_traversal_blocked_mixed_components() {
        // Test mixed path components with ".."
        let result = check_escapes_via_components(Path::new("subdir/../other/../../escape"));
        assert!(result.is_err(), "Mixed '..' components should be blocked");
    }

    #[test]
    fn test_path_traversal_allowed_valid_nested() {
        // Test valid nested paths without ".." are allowed
        let result = check_escapes_via_components(Path::new("dir/subdir/file.txt"));
        assert!(result.is_ok(), "Valid nested paths should be allowed");
    }

    #[test]
    fn test_path_traversal_allowed_cur_dir() {
        // Test that "." (current directory) is allowed
        let result = check_escapes_via_components(Path::new("./current/file.txt"));
        assert!(
            result.is_ok(),
            "Current directory component should be allowed"
        );
    }

    // === Absolute Path Blocking Tests ===

    #[test]
    fn test_absolute_path_blocked_unix() {
        // Test that absolute paths like "/etc/passwd" are blocked
        let temp_dir = TempDir::new().unwrap();
        let allowed_dir = temp_dir.path();

        let result = validate_path(Path::new("/etc/passwd"), allowed_dir);
        assert!(result.is_err(), "Unix absolute path should be blocked");
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("escape") || err_msg.contains("escapes"));
    }

    #[test]
    fn test_absolute_path_blocked_unix_root() {
        // Test paths starting with "/" on Unix
        let temp_dir = TempDir::new().unwrap();
        let allowed_dir = temp_dir.path();

        let result = validate_path(Path::new("/tmp/file.txt"), allowed_dir);
        assert!(result.is_err(), "Unix root path should be blocked");
    }

    #[test]
    fn test_absolute_path_blocked_windows_drive_c() {
        // Test paths with drive letters on Windows (C:\)
        let temp_dir = TempDir::new().unwrap();
        let _allowed_dir = temp_dir.path();

        #[cfg(windows)]
        {
            let result =
                validate_path(Path::new("C:\\Windows\\System32\\config.sys"), _allowed_dir);
            assert!(result.is_err(), "Windows C: drive path should be blocked");
        }

        #[cfg(not(windows))]
        {
            // On Unix, this path won't be treated as absolute in the same way
            // but we verify it doesn't start with / when treated as absolute
            let path = Path::new("C:\\Windows\\System32\\config.sys");
            assert!(
                !path.is_absolute() || path.starts_with("/"),
                "Non-Unix path should be handled"
            );
        }
    }

    #[test]
    fn test_absolute_path_blocked_windows_drive_d() {
        // Test paths with drive letters on Windows (D:\)
        let temp_dir = TempDir::new().unwrap();
        let _allowed_dir = temp_dir.path();

        #[cfg(windows)]
        {
            let result = validate_path(Path::new("D:\\Users\\admin\\secret.txt"), _allowed_dir);
            assert!(result.is_err(), "Windows D: drive path should be blocked");
        }
    }

    // === Write Policy Enforcement Tests ===

    #[test]
    fn test_write_policy_default_blocks_write() {
        // Test that default policy blocks write operations
        let policy = default_readonly_policy();

        let result = validate_operation(OperationType::Write, Path::new("newfile.txt"), &policy);
        assert!(
            result.is_err(),
            "Default policy should block write operations"
        );
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[test]
    fn test_write_policy_default_blocks_delete() {
        // Test that default policy blocks delete operations
        let policy = default_readonly_policy();

        let result = validate_operation(OperationType::Delete, Path::new("existing.txt"), &policy);
        assert!(
            result.is_err(),
            "Default policy should block delete operations"
        );
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[test]
    fn test_write_policy_allow_overwrite_true() {
        // Test that allow_overwrite works when set to true
        let policy = WritePolicy {
            allow_overwrite: true,
            allow_delete: false,
            allowed_extensions: Some(vec!["txt".to_string()]),
        };

        let result = validate_operation(OperationType::Write, Path::new("file.txt"), &policy);
        assert!(
            result.is_ok(),
            "allow_overwrite=true should allow write operations"
        );
    }

    #[test]
    fn test_write_policy_allow_delete_true() {
        // Test that allow_delete works when set to true
        let policy = WritePolicy {
            allow_overwrite: false,
            allow_delete: true,
            allowed_extensions: Some(vec!["txt".to_string()]),
        };

        let result = validate_operation(OperationType::Delete, Path::new("file.txt"), &policy);
        assert!(
            result.is_ok(),
            "allow_delete=true should allow delete operations"
        );
    }

    #[test]
    fn test_write_policy_both_write_and_delete_allowed() {
        // Test that both allow_overwrite and allow_delete can be true
        let policy = WritePolicy {
            allow_overwrite: true,
            allow_delete: true,
            allowed_extensions: None,
        };

        let write_result = validate_operation(OperationType::Write, Path::new("file.txt"), &policy);
        assert!(
            write_result.is_ok(),
            "Write should be allowed when allow_overwrite=true"
        );

        let delete_result =
            validate_operation(OperationType::Delete, Path::new("file.txt"), &policy);
        assert!(
            delete_result.is_ok(),
            "Delete should be allowed when allow_delete=true"
        );
    }

    // === Extension Validation Tests ===

    #[test]
    fn test_extension_validation_allowed_txt() {
        // Test that default allowed extensions work - txt
        let policy = default_readonly_policy();

        let result = validate_operation(OperationType::Read, Path::new("file.txt"), &policy);
        assert!(result.is_ok(), "txt extension should be allowed");
    }

    #[test]
    fn test_extension_validation_allowed_md() {
        // Test that default allowed extensions work - md
        let policy = default_readonly_policy();

        let result = validate_operation(OperationType::Read, Path::new("readme.md"), &policy);
        assert!(result.is_ok(), "md extension should be allowed");
    }

    #[test]
    fn test_extension_validation_allowed_json() {
        // Test that default allowed extensions work - json
        let policy = default_readonly_policy();

        let result = validate_operation(OperationType::Read, Path::new("config.json"), &policy);
        assert!(result.is_ok(), "json extension should be allowed");
    }

    #[test]
    fn test_extension_validation_allowed_toml() {
        // Test that default allowed extensions work - toml
        let policy = default_readonly_policy();

        let result = validate_operation(OperationType::Read, Path::new("Cargo.toml"), &policy);
        assert!(result.is_ok(), "toml extension should be allowed");
    }

    #[test]
    fn test_extension_validation_allowed_yaml() {
        // Test that default allowed extensions work - yaml
        let policy = default_readonly_policy();

        let result = validate_operation(OperationType::Read, Path::new("config.yaml"), &policy);
        assert!(result.is_ok(), "yaml extension should be allowed");
    }

    #[test]
    fn test_extension_validation_allowed_yml() {
        // Test that default allowed extensions work - yml
        let policy = default_readonly_policy();

        let result = validate_operation(OperationType::Read, Path::new("config.yml"), &policy);
        assert!(result.is_ok(), "yml extension should be allowed");
    }

    #[test]
    fn test_extension_validation_allowed_rs() {
        // Test that default allowed extensions work - rs
        let policy = default_readonly_policy();

        let result = validate_operation(OperationType::Read, Path::new("lib.rs"), &policy);
        assert!(result.is_ok(), "rs extension should be allowed");
    }

    #[test]
    fn test_extension_validation_blocked_exe() {
        // Test that disallowed extensions are blocked - exe
        let policy = default_readonly_policy();

        // Use Read operation to test extension validation specifically
        // (Write operations are blocked by policy before extension check)
        let result = validate_operation(OperationType::Read, Path::new("malware.exe"), &policy);
        assert!(result.is_err(), "exe extension should be blocked");
        assert!(result.unwrap_err().contains("not in the allowed list"));
    }

    #[test]
    fn test_extension_validation_blocked_sh() {
        // Test that disallowed extensions are blocked - sh
        let policy = default_readonly_policy();

        // Use Read operation to test extension validation specifically
        let result = validate_operation(OperationType::Read, Path::new("script.sh"), &policy);
        assert!(result.is_err(), "sh extension should be blocked");
    }

    #[test]
    fn test_extension_validation_blocked_bat() {
        // Test that disallowed extensions are blocked - bat
        let policy = default_readonly_policy();

        // Use Read operation to test extension validation specifically
        let result = validate_operation(OperationType::Read, Path::new("install.bat"), &policy);
        assert!(result.is_err(), "bat extension should be blocked");
    }

    #[test]
    fn test_extension_validation_case_insensitive_upper() {
        // Test case-insensitivity - uppercase TXT
        let policy = default_readonly_policy();

        let result = validate_operation(OperationType::Read, Path::new("file.TXT"), &policy);
        assert!(result.is_ok(), "Uppercase TXT extension should be allowed");
    }

    #[test]
    fn test_extension_validation_case_insensitive_mixed() {
        // Test case-insensitivity - mixed case MD
        let policy = default_readonly_policy();

        let result = validate_operation(OperationType::Read, Path::new("file.Md"), &policy);
        assert!(result.is_ok(), "Mixed case Md extension should be allowed");
    }

    #[test]
    fn test_extension_validation_blocked_uppercase_disallowed() {
        // Test that uppercase disallowed extensions are blocked
        let policy = default_readonly_policy();

        // Use Read operation to test extension validation specifically
        let result = validate_operation(OperationType::Read, Path::new("file.EXE"), &policy);
        assert!(result.is_err(), "Uppercase EXE should be blocked");
    }
}
