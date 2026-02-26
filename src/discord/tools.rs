//! Discord agent tools module.
//!
//! Provides tool definitions and execution for the Discord concierge integration,
//! allowing AI agents to perform file operations and system queries through
//! OpenAI function-calling format.

#[cfg(test)]
use serial_test::serial;

use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, error, info};

#[allow(unused_imports)]
use chrono::Local;

/// Maximum character limit for file reads.
const MAX_FILE_SIZE: usize = 3000;

/// Tool error types.
#[derive(Error, Debug)]
pub enum ToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Path traversal attempt blocked: {0}")]
    PathTraversal(String),

    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Tool not found: {0}")]
    UnknownTool(String),

    #[error("Tool execution failed: {0}")]
    Execution(String),
}

/// Available tools for agent execution.
#[derive(Debug, Clone)]
pub enum Tool {
    /// Read a file from the project repository.
    ReadFile { path: String },

    /// List files and subdirectories in a project directory.
    ListDirectory { path: String },

    /// Check the current status of all agents.
    GetStatus,

    /// List all pending items in the agent inbox.
    ListInbox,

    /// Read pending agent messages from comms/outbox/, relay them, and archive.
    ReadOutbox,

    /// Read TODO file progress for a specific agent or all agents.
    ReadTodos { agent: Option<String> },

    /// Read the current project backlog.
    ReadBacklog,

    /// File a task or feature request into the agent inbox.
    FileTask {
        title: String,
        description: String,
        priority: Option<String>,
    },

    /// Add a new item to the project backlog.
    AddToBacklog { item: String, tag: Option<String> },

    /// File a bug report into the agent inbox for triage and fixing.
    FileBug {
        title: String,
        description: String,
        severity: Option<String>,
    },
}

/// Returns the JSON schema for all available tools in OpenAI function-calling format.
pub fn tools_schema() -> Value {
    json!([
        {
            "name": "read_file",
            "description": "Read a file from the project repository",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path from repo root"
                    }
                },
                "required": ["path"]
            }
        },
        {
            "name": "list_directory",
            "description": "List files and subdirectories in a project directory",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path from repo root (use '.' for root)"
                    }
                },
                "required": ["path"]
            }
        },
        {
            "name": "get_status",
            "description": "Check the current status of all agents — signal files, TODO progress, inbox/outbox counts",
            "parameters": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "list_inbox",
            "description": "List all pending items in the agent inbox",
            "parameters": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "read_outbox",
            "description": "Read pending agent messages from comms/outbox/, relay them, and archive",
            "parameters": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "read_todos",
            "description": "Read TODO file progress for a specific agent or all agents",
            "parameters": {
                "type": "object",
                "properties": {
                    "agent": {
                        "type": "string",
                        "description": "Agent name/number, or 'all'"
                    }
                }
            }
        },
        {
            "name": "read_backlog",
            "description": "Read the current project backlog",
            "parameters": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "file_task",
            "description": "File a task or feature request into the agent inbox",
            "parameters": {
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Short task title"
                    },
                    "description": {
                        "type": "string",
                        "description": "Detailed description"
                    },
                    "priority": {
                        "type": "string",
                        "enum": ["high", "medium", "low"],
                        "description": "Task priority"
                    }
                },
                "required": ["title", "description"]
            }
        },
        {
            "name": "add_to_backlog",
            "description": "Add a new item to the backlog",
            "parameters": {
                "type": "object",
                "properties": {
                    "item": {
                        "type": "string",
                        "description": "The backlog item description"
                    },
                    "tag": {
                        "type": "string",
                        "enum": ["FEATURE", "BUG", "CHORE", "IDEA"],
                        "description": "Optional tag for the backlog item"
                    }
                },
                "required": ["item"]
            }
        },
        {
            "name": "file_bug",
            "description": "File a bug report into the agent inbox for triage and fixing",
            "parameters": {
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Short bug title"
                    },
                    "description": {
                        "type": "string",
                        "description": "Detailed bug description"
                    },
                    "severity": {
                        "type": "string",
                        "enum": ["critical", "high", "medium", "low"],
                        "description": "Bug severity level"
                    }
                },
                "required": ["title", "description"]
            }
        }
    ])
}

/// Validates that a path does not contain path traversal elements.
///
/// Returns an error if the path contains ".." or resolves outside the workspace.
fn validate_path(path: &str) -> Result<(), ToolError> {
    // Reject paths containing ".."
    if path.contains("..") {
        return Err(ToolError::PathTraversal(format!(
            "Path traversal attempt blocked: {}",
            path
        )));
    }

    // Resolve the path and check it's within the workspace
    let resolved = Path::new(path);
    if resolved.is_absolute() {
        return Err(ToolError::PathTraversal(format!(
            "Absolute paths not allowed: {}",
            path
        )));
    }

    Ok(())
}

/// Execute the read_file tool.
///
/// Reads a file from the repository, with a maximum of 3000 characters.
/// Rejects paths containing ".." to prevent path traversal attacks.
pub fn execute_read_file(path: &str) -> Result<String, ToolError> {
    // Debug: Log the current working directory and requested path
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("unknown"));
    debug!("execute_read_file called with path: '{}', cwd: {}", path, cwd.display());
    eprintln!("[DISCORD TOOL] execute_read_file: path='{}', cwd={}", path, cwd.display());
    
    validate_path(path)?;

    let file_path = Path::new(path);

    // Check if file exists
    if !file_path.exists() {
        return Err(ToolError::NotFound(path.to_string()));
    }

    // Read file contents
    let content = fs::read_to_string(file_path)?;

    // Truncate if too long
    if content.len() > MAX_FILE_SIZE {
        let truncated = &content[..MAX_FILE_SIZE];
        Ok(format!(
            "{}\n\n[Truncated - {} characters total]",
            truncated,
            content.len()
        ))
    } else {
        Ok(content)
    }
}

/// Execute the list_directory tool.
///
/// Lists all files and subdirectories in the specified directory.
pub fn execute_list_directory(path: &str) -> Result<String, ToolError> {
    // Debug: Log the current working directory and requested path
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("unknown"));
    debug!("execute_list_directory called with path: '{}', cwd: {}", path, cwd.display());
    eprintln!("[DISCORD TOOL] execute_list_directory: path='{}', cwd={}", path, cwd.display());
    
    validate_path(path)?;

    let dir_path = Path::new(path);

    // Check if directory exists
    if !dir_path.exists() {
        return Err(ToolError::NotFound(format!(
            "Directory not found: {}",
            path
        )));
    }

    if !dir_path.is_dir() {
        return Err(ToolError::NotFound(format!("Not a directory: {}", path)));
    }

    // Read directory contents
    let mut entries: Vec<String> = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_type = if entry.file_type()?.is_dir() {
            "[DIR]"
        } else {
            "[FILE]"
        };
        entries.push(format!("{} {}", file_type, file_name));
    }

    entries.sort();
    Ok(entries.join("\n"))
}

/// Execute the get_status tool.
///
/// Checks the current status of all agents including:
/// - Signal files (.agent_done_*)
/// - TODO progress
/// - Inbox/outbox counts
pub fn get_status() -> Result<String, ToolError> {
    // Debug: Log the current working directory
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("unknown"));
    debug!("get_status called, cwd: {}", cwd.display());
    eprintln!("[DISCORD TOOL] get_status: cwd={}", cwd.display());
    
    let mut status = String::new();

    // Check for agent done files
    status.push_str("=== Agent Status ===\n");
    let workspace = Path::new(".");

    if let Ok(entries) = fs::read_dir(workspace) {
        let mut done_agents: Vec<String> = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(".agent_done_") {
                done_agents.push(name);
            }
        }
        done_agents.sort();
        if done_agents.is_empty() {
            status.push_str("No agent completion signals found.\n");
        } else {
            for agent in &done_agents {
                status.push_str(&format!("  {}\n", agent));
            }
        }
    }

    // Check TODO files
    status.push_str("\n=== TODO Files ===\n");
    let todo_files = ["TODO1.md", "TODO2.md", "TODO3.md", "TODO4.md", "TODO5.md"];
    for todo_file in todo_files {
        let path = Path::new(todo_file);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                let lines: Vec<&str> = content.lines().collect();
                let total_lines = lines.len();
                let checked = lines.iter().filter(|l| l.starts_with("- [x]")).count();
                status.push_str(&format!(
                    "  {}: {}/{} completed\n",
                    todo_file, checked, total_lines
                ));
            }
        }
    }

    // Check inbox count
    status.push_str("\n=== Communication Status ===\n");
    let inbox_path = Path::new("comms/inbox");
    let inbox_count = if inbox_path.exists() && inbox_path.is_dir() {
        fs::read_dir(inbox_path)?.count()
    } else {
        0
    };
    status.push_str(&format!("  Inbox: {} pending\n", inbox_count));

    // Check outbox count
    let outbox_path = Path::new("comms/outbox");
    let outbox_count = if outbox_path.exists() && outbox_path.is_dir() {
        fs::read_dir(outbox_path)?.count()
    } else {
        0
    };
    status.push_str(&format!("  Outbox: {} pending\n", outbox_count));

    Ok(status)
}

/// Execute the list_inbox tool.
///
/// Lists all pending items in the agent inbox.
pub fn execute_list_inbox() -> Result<String, ToolError> {
    let inbox_path = Path::new("comms/inbox");

    if !inbox_path.exists() {
        return Ok("Inbox is empty (directory does not exist)".to_string());
    }

    if !inbox_path.is_dir() {
        return Ok("Inbox is empty (not a directory)".to_string());
    }

    let mut entries: Vec<String> = Vec::new();

    for entry in fs::read_dir(inbox_path)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_type = if entry.file_type()?.is_dir() {
            "[DIR]"
        } else {
            "[FILE]"
        };
        entries.push(format!("{} {}", file_type, file_name));
    }

    entries.sort();

    if entries.is_empty() {
        Ok("Inbox is empty".to_string())
    } else {
        Ok(entries.join("\n"))
    }
}

/// Execute the read_outbox tool.
///
/// Reads pending agent messages from comms/outbox/ and archives them.
pub fn execute_read_outbox() -> Result<String, ToolError> {
    let outbox_path = Path::new("comms/outbox");
    let archive_path = Path::new("comms/archive");

    if !outbox_path.exists() {
        return Ok("Outbox is empty (directory does not exist)".to_string());
    }

    if !outbox_path.is_dir() {
        return Ok("Outbox is empty (not a directory)".to_string());
    }

    let mut messages: Vec<String> = Vec::new();

    // Read all files from outbox
    let entries: Vec<_> = fs::read_dir(outbox_path)?.collect();

    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let content = fs::read_to_string(entry.path())?;

            messages.push(format!("=== {} ===\n{}", file_name, content));

            // Archive the file
            if archive_path.exists() {
                let archive_file = archive_path.join(&file_name);
                // Only archive if not already there
                if !archive_file.exists() {
                    fs::copy(entry.path(), archive_file)?;
                }
                // Remove from outbox after archiving
                fs::remove_file(entry.path())?;
            }
        }
    }

    messages.sort();

    if messages.is_empty() {
        Ok("Outbox is empty".to_string())
    } else {
        Ok(messages.join("\n\n"))
    }
}

/// Execute the read_todos tool.
///
/// Reads TODO file progress for a specific agent or all agents.
pub fn execute_read_todos(agent: Option<&str>) -> Result<String, ToolError> {
    let agent = agent.unwrap_or("all");

    let todo_files: Vec<String> = if agent == "all" {
        vec![
            "TODO1.md".to_string(),
            "TODO2.md".to_string(),
            "TODO3.md".to_string(),
            "TODO4.md".to_string(),
            "TODO5.md".to_string(),
        ]
    } else {
        // Map agent name/number to TODO file
        let agent_num = agent
            .trim_start_matches("agent")
            .trim_start_matches('a')
            .parse::<usize>()
            .ok();

        match agent_num {
            Some(n) if (1..=5).contains(&n) => vec![format!("TODO{}.md", n)],
            _ => vec![format!("TODO{}.md", agent)],
        }
    };

    let mut results: Vec<String> = Vec::new();

    for todo_file in &todo_files {
        let path = Path::new(todo_file);
        if path.exists() {
            let content = fs::read_to_string(path)?;
            results.push(format!("=== {} ===\n{}", todo_file, content));
        } else if agent == "all" {
            // Only show missing file info when listing all
            results.push(format!("{} (not found)", todo_file));
        }
    }

    if results.is_empty() {
        Ok("No TODO files found".to_string())
    } else {
        Ok(results.join("\n\n"))
    }
}

/// Execute the read_backlog tool.
///
/// Reads the current project backlog from BACKLOG.md.
pub fn execute_read_backlog() -> Result<String, ToolError> {
    let backlog_path = Path::new("BACKLOG.md");

    if !backlog_path.exists() {
        return Err(ToolError::NotFound("BACKLOG.md not found".to_string()));
    }

    let content = fs::read_to_string(backlog_path)?;

    // Truncate if too long
    if content.len() > MAX_FILE_SIZE {
        let truncated = &content[..MAX_FILE_SIZE];
        Ok(format!(
            "{}\n\n[Truncated - {} characters total]",
            truncated,
            content.len()
        ))
    } else {
        Ok(content)
    }
}

/// Execute the add_to_backlog tool.
///
/// Appends a new item to the project backlog (BACKLOG.md).
pub fn execute_add_to_backlog(item: &str, tag: Option<&str>) -> Result<String, ToolError> {
    let backlog_path = Path::new("BACKLOG.md");

    // Read existing content (or create empty string if doesn't exist)
    let existing_content = if backlog_path.exists() {
        fs::read_to_string(backlog_path)?
    } else {
        String::new()
    };

    // Format the new entry with tag prefix
    let tag_prefix = match tag {
        Some(t) => format!("[{}] ", t.to_uppercase()),
        None => String::new(),
    };

    // Append the new entry with a newline
    let new_content = if existing_content.is_empty() {
        format!("{}{}", tag_prefix, item)
    } else {
        format!("\n{}{}", tag_prefix, item)
    };

    // Write back to BACKLOG.md
    fs::write(backlog_path, format!("{}{}", existing_content, new_content))?;

    Ok(format!("Added to BACKLOG.md: {}{}", tag_prefix, item))
}

/// Slugify a string for use in filenames.
///
/// Converts to lowercase and replaces spaces/special chars with hyphens.
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Execute the file_task tool.
///
/// Files a task or feature request into the agent inbox.
pub fn execute_file_task(
    title: &str,
    description: &str,
    priority: Option<&str>,
) -> Result<String, ToolError> {
    let inbox_path = Path::new("comms/inbox");

    // Ensure the inbox directory exists
    if !inbox_path.exists() {
        fs::create_dir_all(inbox_path)?;
    }

    // Get current date and time for filename
    let now = Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let time_str = now.format("%H%M%S").to_string();

    // Create slug from title
    let slug = slugify(title);

    // Build filename
    let filename = format!("{}_{}_discord_task_{}.md", date_str, time_str, slug);

    // Determine priority (default to "medium")
    let priority = priority.unwrap_or("medium");
    let valid_priority = match priority {
        "high" | "medium" | "low" => priority,
        _ => "medium",
    };

    // Build file content
    let content = format!(
        "# Task: {}\n\n\
         **Source:** Discord\n\
         **Priority:** {}\n\
         **Date:** {}\n\n\
         ## Description\n\
         {}",
        title, valid_priority, date_str, description
    );

    // Write the file
    let file_path = inbox_path.join(&filename);
    fs::write(&file_path, content)?;

    Ok(format!("Task filed successfully: {}", file_path.display()))
}

/// Execute the file_bug tool.
///
/// Files a bug report into the agent inbox for triage and fixing.
pub fn execute_file_bug(
    title: &str,
    description: &str,
    severity: Option<&str>,
) -> Result<String, ToolError> {
    let inbox_path = Path::new("comms/inbox");

    // Ensure the inbox directory exists
    if !inbox_path.exists() {
        fs::create_dir_all(inbox_path)?;
    }

    // Get current date and time for filename
    let now = Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let time_str = now.format("%H%M%S").to_string();

    // Create slug from title using existing slugify function
    let slug = slugify(title);

    // Determine severity (default to "medium")
    let severity = severity.unwrap_or("medium");
    let valid_severity = match severity {
        "critical" | "high" | "medium" | "low" => severity,
        _ => "medium",
    };

    // Build filename
    let filename = format!("{}_{}_discord_bug_{}.md", date_str, time_str, slug);

    // Build file content
    let content = format!(
        "# Bug Report: {}\n\n\
         > Filed via: Discord Concierge\n\
         > Date: {}\n\
         > Severity: {}\n\n\
         ## Description\n\
         {}\n\n\
         ## Status\n\
         - **Status:** Triage Pending\n\
         - **Priority:** {}\n\
         - **Assignee:** Unassigned\n",
        title, date_str, valid_severity, description, valid_severity
    );

    // Write the file
    let file_path = inbox_path.join(&filename);
    fs::write(&file_path, content)?;

    Ok(format!("Bug report filed: {}", file_path.display()))
}

/// Execute a tool and return the result.
///
/// Dispatches to the appropriate execution function based on the tool variant.
pub fn execute_tool(tool: Tool) -> Result<String, ToolError> {
    match tool {
        Tool::ReadFile { path } => execute_read_file(&path),
        Tool::ListDirectory { path } => execute_list_directory(&path),
        Tool::GetStatus => get_status(),
        Tool::ListInbox => execute_list_inbox(),
        Tool::ReadOutbox => execute_read_outbox(),
        Tool::ReadTodos { agent } => execute_read_todos(agent.as_deref()),
        Tool::ReadBacklog => execute_read_backlog(),
        Tool::AddToBacklog { item, tag } => execute_add_to_backlog(&item, tag.as_deref()),
        Tool::FileTask {
            title,
            description,
            priority,
        } => execute_file_task(&title, &description, priority.as_deref()),
        Tool::FileBug {
            title,
            description,
            severity,
        } => execute_file_bug(&title, &description, severity.as_deref()),
    }
}

/// Parse a tool name and JSON arguments into a Tool variant.
///
/// This is used by the ToolExecutor to convert LLM tool calls into executable Tool variants.
pub fn parse_tool_from_llm(name: &str, arguments: &str) -> Result<Tool, ToolError> {
    let args: serde_json::Value = serde_json::from_str(arguments)
        .map_err(|e| ToolError::Json(e))?;

    match name {
        "read_file" => {
            let path = args.get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::Execution("Missing required parameter: path".to_string()))?
                .to_string();
            Ok(Tool::ReadFile { path })
        }
        "list_directory" => {
            let path = args.get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::Execution("Missing required parameter: path".to_string()))?
                .to_string();
            Ok(Tool::ListDirectory { path })
        }
        "get_status" => Ok(Tool::GetStatus),
        "list_inbox" => Ok(Tool::ListInbox),
        "read_outbox" => Ok(Tool::ReadOutbox),
        "read_todos" => {
            let agent = args.get("agent")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok(Tool::ReadTodos { agent })
        }
        "read_backlog" => Ok(Tool::ReadBacklog),
        "add_to_backlog" => {
            let item = args.get("item")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::Execution("Missing required parameter: item".to_string()))?
                .to_string();
            let tag = args.get("tag")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok(Tool::AddToBacklog { item, tag })
        }
        "file_task" => {
            let title = args.get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::Execution("Missing required parameter: title".to_string()))?
                .to_string();
            let description = args.get("description")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::Execution("Missing required parameter: description".to_string()))?
                .to_string();
            let priority = args.get("priority")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok(Tool::FileTask { title, description, priority })
        }
        "file_bug" => {
            let title = args.get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::Execution("Missing required parameter: title".to_string()))?
                .to_string();
            let description = args.get("description")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::Execution("Missing required parameter: description".to_string()))?
                .to_string();
            let severity = args.get("severity")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok(Tool::FileBug { title, description, severity })
        }
        _ => Err(ToolError::UnknownTool(name.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    #[serial]
    fn test_add_to_backlog_with_tag() {
        // Use temp directory to avoid polluting project directory
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        // Change to temp dir and ensure we restore on drop
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Use scope to ensure cwd is restored before any assertions
        let result = {
            let backlog_path = Path::new("BACKLOG.md");
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
    #[serial]
    fn test_add_to_backlog_without_tag() {
        // Use temp directory to avoid polluting project directory
        let temp_dir = tempfile::tempdir().unwrap();
        let original_cwd = std::env::current_dir().unwrap();

        // Change to temp dir
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Execute in a scope so result is available after cwd restore
        let result = {
            let backlog_path = Path::new("BACKLOG.md");
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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

    #[test]
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
    #[serial]
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
