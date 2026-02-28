//! Tool definitions module.
//!
//! Contains type definitions for tools including the Tool enum,
//! ToolError enum, and JSON schema generation.

use serde_json::{json, Value};
use thiserror::Error;

/// Maximum character limit for file reads.
pub const MAX_FILE_SIZE: usize = 3000;

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
