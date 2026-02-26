//! Environment variable loader for switchboard configuration.
//!
//! This module provides functionality to:
//! - Load environment variables from a .env file
//! - Resolve ${VAR} and ${VAR:-default} syntax in configuration values
//! - Support shell-style default values for missing environment variables
//!
//! The .env file should contain KEY=value pairs, one per line.
//! Lines starting with # are treated as comments.
//! Empty lines are ignored.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use tracing::{debug, warn};

/// Global environment variables loaded from .env file
/// Using OnceLock to ensure we only load once
static ENV_CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();

/// Load environment variables from a .env file.
///
/// This function reads KEY=value pairs from the specified file.
/// System environment variables take precedence over values in the .env file.
///
/// # Arguments
///
/// * `path` - Path to the .env file
///
/// # Returns
///
/// * `Ok(HashMap<String, String>)` - The loaded environment variables
/// * `Err(String)` - If the file cannot be read (but not found is handled gracefully)
pub fn load_env_file(path: &Path) -> Result<HashMap<String, String>, String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("No .env file found at {:?}, using system environment variables only", path);
            return Ok(HashMap::new());
        }
        Err(e) => return Err(format!("Failed to read .env file: {}", e)),
    };

    let mut vars = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse KEY=value
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            let mut value = trimmed[eq_pos + 1..].trim().to_string();

            // Remove surrounding quotes if present
            if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                value = value[1..value.len() - 1].to_string();
            }

            if !key.is_empty() {
                vars.insert(key, value);
            }
        }
    }

    debug!("Loaded {} variables from .env file: {:?}", vars.len(), vars.keys().collect::<Vec<_>>());
    Ok(vars)
}

/// Get a resolved value for an environment variable reference.
///
/// Supports two syntaxes:
/// - `${VAR}` - Returns the value of VAR, or logs a warning if not found
/// - `${VAR:-default}` - Returns the value of VAR, or 'default' if not found
///
/// System environment variables take precedence over values loaded from .env file.
///
/// # Arguments
///
/// * `var_ref` - The environment variable reference (e.g., "${VAR}" or "${VAR:-default}")
/// * `env_file_vars` - Optional map of variables loaded from .env file
///
/// # Returns
///
/// The resolved value, or None if the variable is not found and no default is specified
pub fn resolve_env_var(var_ref: &str, env_file_vars: &HashMap<String, String>) -> Option<String> {
    // Parse the variable reference
    let var_ref = var_ref.trim();

    // Must start with ${ and end with }
    if !var_ref.starts_with("${") || !var_ref.ends_with('}') {
        return None;
    }

    // Extract the inner content
    let inner = &var_ref[2..var_ref.len() - 1];

    // Check for default value syntax ${VAR:-default}
    let (var_name, default_value) = if let Some(dash_pos) = inner.find(":-") {
        let name = inner[..dash_pos].to_string();
        let default = inner[dash_pos + 2..].to_string();
        (name, Some(default))
    } else {
        (inner.to_string(), None)
    };

    // First check system environment
    if let Ok(value) = env::var(&var_name) {
        if !value.is_empty() {
            return Some(value);
        }
    }

    // Then check .env file variables
    if let Some(value) = env_file_vars.get(&var_name) {
        if !value.is_empty() {
            return Some(value.clone());
        }
    }

    // Use default if available
    if let Some(default) = default_value {
        return Some(default);
    }

    // No value found - log warning
    warn!(
        "Environment variable '{}' not found and no default specified",
        var_name
    );
    None
}

/// Resolve all environment variable references in a string.
///
/// Scans the input string for ${VAR} and ${VAR:-default} patterns
/// and replaces them with their resolved values.
///
/// # Arguments
///
/// * `input` - The input string that may contain environment variable references
/// * `env_file_vars` - Map of variables loaded from .env file
///
/// # Returns
///
/// The input string with all environment variable references resolved
pub fn resolve_env_vars(input: &str, env_file_vars: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            // Found ${ pattern, read until }
            chars.next(); // consume '{'

            let mut var_name = String::new();
            let mut has_default = false;
            let mut default_value = String::new();

            while let Some(&c) = chars.peek() {
                match c {
                    '}' => {
                        chars.next(); // consume '}'
                        break;
                    }
                    ':' if !has_default => {
                        has_default = true;
                        chars.next(); // consume ':'
                        // Next char must be '-'
                        if chars.peek() == Some(&'-') {
                            chars.next(); // consume '-'
                        }
                    }
                    _ => {
                        if has_default {
                            default_value.push(c);
                        } else {
                            var_name.push(c);
                        }
                        chars.next();
                    }
                }
            }

            // Try to resolve the variable
            if var_name.is_empty() {
                // Empty variable name, keep as-is
                result.push_str(&format!("${{{}}}", if has_default { format!(":-{}", default_value) } else { String::new() }));
            } else {
                let env_ref = format!("${{{}}}", var_name);
                let resolved = resolve_env_var(&env_ref, env_file_vars);
                match resolved {
                    Some(value) => result.push_str(&value),
                    None if has_default => result.push_str(&default_value),
                    None => {
                        // Keep the original reference if not resolved
                        result.push_str(&format!("${{{}}}", var_name));
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Load and cache environment variables from switchboard.env in the project root.
///
/// The project root is determined by looking for switchboard.toml or switchboard.env
/// in the current working directory or parent directories.
///
/// This function is safe to call multiple times - the .env file is only loaded once.
///
/// # Returns
///
/// A reference to the cached environment variables
pub fn get_env_vars() -> &'static HashMap<String, String> {
    ENV_CACHE.get_or_init(|| {
        // Try to find the project root by looking for switchboard.toml or switchboard.env
        let project_root = find_project_root();

        let env_path = project_root.join("switchboard.env");

        match load_env_file(&env_path) {
            Ok(vars) => vars,
            Err(e) => {
                warn!("Failed to load switchboard.env: {}", e);
                HashMap::new()
            }
        }
    })
}

/// Find the project root directory by looking for switchboard.toml or switchboard.env
fn find_project_root() -> std::path::PathBuf {
    // Start from current working directory
    let cwd = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    // Walk up the directory tree looking for switchboard.toml or switchboard.env
    let mut current = cwd.as_path();
    loop {
        let switchboard_toml = current.join("switchboard.toml");
        let switchboard_env = current.join("switchboard.env");

        if switchboard_toml.exists() || switchboard_env.exists() {
            return current.to_path_buf();
        }

        // Go up one directory
        match current.parent() {
            Some(parent) => current = parent,
            None => return cwd, // Return current working directory if not found
        }
    }
}

/// Resolve an environment variable reference from a TOML config value.
///
/// This function is used to resolve values like `${DISCORD_TOKEN}` or
/// `${OPENROUTER_API_KEY:-}` in configuration files.
///
/// # Arguments
///
/// * `value` - The configuration value that may contain environment variable references
///
/// # Returns
///
/// The resolved value with environment variable references replaced
pub fn resolve_config_value(value: &str) -> String {
    let env_vars = get_env_vars();
    resolve_env_vars(value, env_vars)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_env_var_simple() {
        let mut vars = HashMap::new();
        vars.insert("TEST_VAR".to_string(), "test_value".to_string());

        let result = resolve_env_var("${TEST_VAR}", &vars);
        assert_eq!(result, Some("test_value".to_string()));
    }

    #[test]
    fn test_resolve_env_var_with_default() {
        let vars = HashMap::new();

        let result = resolve_env_var("${MISSING_VAR:-default_value}", &vars);
        assert_eq!(result, Some("default_value".to_string()));
    }

    #[test]
    fn test_resolve_env_var_missing_no_default() {
        let vars = HashMap::new();

        let result = resolve_env_var("${MISSING_VAR}", &vars);
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_env_vars_in_string() {
        let mut vars = HashMap::new();
        vars.insert("DISCORD_TOKEN".to_string(), "my_token".to_string());
        vars.insert("API_KEY".to_string(), "secret_key".to_string());

        let input = "token = ${DISCORD_TOKEN}, key = ${API_KEY}";
        let result = resolve_env_vars(input, &vars);

        assert_eq!(result, "token = my_token, key = secret_key");
    }

    #[test]
    fn test_resolve_env_vars_with_defaults() {
        let vars = HashMap::new();

        let input = "token = ${DISCORD_TOKEN:-default_token}, key = ${API_KEY:-default_key}";
        let result = resolve_env_vars(input, &vars);

        assert_eq!(result, "token = default_token, key = default_key");
    }

    #[test]
    fn test_load_env_file() {
        let temp_dir = TempDir::new().unwrap();
        let env_file = temp_dir.path().join(".env");

        fs::write(
            &env_file,
            r#"
# Comment line
TEST_KEY=test_value
ANOTHER_KEY="quoted_value"
EMPTY_KEY=
"#,
        )
        .unwrap();

        let vars = load_env_file(&env_file).unwrap();

        assert_eq!(vars.get("TEST_KEY"), Some(&"test_value".to_string()));
        assert_eq!(vars.get("ANOTHER_KEY"), Some(&"quoted_value".to_string()));
        // Empty values are preserved but treated as empty strings
        assert_eq!(vars.get("EMPTY_KEY"), Some(&"".to_string()));
    }

    #[test]
    fn test_load_env_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let env_file = temp_dir.path().join("nonexistent.env");

        let result = load_env_file(&env_file);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_resolve_config_value() {
        // Set a test environment variable
        env::set_var("TEST_RESOLVE_VAR", "resolved_value");

        let result = resolve_config_value("${TEST_RESOLVE_VAR}");
        assert_eq!(result, "resolved_value");

        env::remove_var("TEST_RESOLVE_VAR");
    }

    #[test]
    fn test_resolve_config_value_with_default() {
        let result = resolve_config_value("${NONEXISTENT_VAR:-fallback}");
        assert_eq!(result, "fallback");
    }

    #[test]
    fn test_mixed_text_and_vars() {
        let mut vars = HashMap::new();
        vars.insert("HOST".to_string(), "localhost".to_string());
        vars.insert("PORT".to_string(), "8080".to_string());

        let input = "Server: ${HOST}:${PORT}";
        let result = resolve_env_vars(input, &vars);

        assert_eq!(result, "Server: localhost:8080");
    }
}
