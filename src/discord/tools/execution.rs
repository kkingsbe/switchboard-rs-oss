//! Tool execution module.
//!
//! Contains functions for executing tools, path validation,
//! and parsing LLM tool calls into Tool variants.

use std::fs;
use std::path::Path;

use chrono::Local;
use tracing::debug;

use crate::discord::tools::definitions::{Tool, ToolError, MAX_FILE_SIZE};

/// Validates that a path does not contain path traversal elements.
///
/// Returns an error if the path contains ".." or resolves outside the workspace.
pub fn validate_path(path: &str) -> Result<(), ToolError> {
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
    debug!(
        "execute_read_file called with path: '{}', cwd: {}",
        path,
        cwd.display()
    );
    eprintln!(
        "[DISCORD TOOL] execute_read_file: path='{}', cwd={}",
        path,
        cwd.display()
    );

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
    debug!(
        "execute_list_directory called with path: '{}', cwd: {}",
        path,
        cwd.display()
    );
    eprintln!(
        "[DISCORD TOOL] execute_list_directory: path='{}', cwd={}",
        path,
        cwd.display()
    );

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
pub fn slugify(s: &str) -> String {
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
    let args: serde_json::Value = serde_json::from_str(arguments).map_err(ToolError::Json)?;

    match name {
        "read_file" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolError::Execution("Missing required parameter: path".to_string())
                })?
                .to_string();
            Ok(Tool::ReadFile { path })
        }
        "list_directory" => {
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolError::Execution("Missing required parameter: path".to_string())
                })?
                .to_string();
            Ok(Tool::ListDirectory { path })
        }
        "get_status" => Ok(Tool::GetStatus),
        "list_inbox" => Ok(Tool::ListInbox),
        "read_outbox" => Ok(Tool::ReadOutbox),
        "read_todos" => {
            let agent = args
                .get("agent")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok(Tool::ReadTodos { agent })
        }
        "read_backlog" => Ok(Tool::ReadBacklog),
        "add_to_backlog" => {
            let item = args
                .get("item")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolError::Execution("Missing required parameter: item".to_string())
                })?
                .to_string();
            let tag = args
                .get("tag")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok(Tool::AddToBacklog { item, tag })
        }
        "file_task" => {
            let title = args
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolError::Execution("Missing required parameter: title".to_string())
                })?
                .to_string();
            let description = args
                .get("description")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolError::Execution("Missing required parameter: description".to_string())
                })?
                .to_string();
            let priority = args
                .get("priority")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok(Tool::FileTask {
                title,
                description,
                priority,
            })
        }
        "file_bug" => {
            let title = args
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolError::Execution("Missing required parameter: title".to_string())
                })?
                .to_string();
            let description = args
                .get("description")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolError::Execution("Missing required parameter: description".to_string())
                })?
                .to_string();
            let severity = args
                .get("severity")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Ok(Tool::FileBug {
                title,
                description,
                severity,
            })
        }
        _ => Err(ToolError::UnknownTool(name.to_string())),
    }
}
