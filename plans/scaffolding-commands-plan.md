# Scaffolding Commands Implementation Plan

## 1. Overview

This plan outlines the implementation of two new CLI commands for the Switchboard project:

- **`switchboard project init`** - Scaffolds a new Switchboard project with the standard directory structure and minimal configuration
- **`switchboard workflow init`** - Scaffolds a new Switchboard workflow with the manifest structure and prompt templates

These commands follow the existing CLI architecture patterns in the codebase, using clap derive macros for argument parsing and a modular command implementation structure.

---

## 2. Command Interface

### 2.1 `switchboard project init`

```bash
# Create project in current directory
switchboard project init

# Create project in specified directory
switchboard project init --path ./my-project

# Create project with specific name (affects config)
switchboard project init --name my-project

# Skip confirmation if directory exists
switchboard project init --force

# Create minimal project (no examples, no prompts)
switchboard project init --minimal
```

**Arguments:**

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--path` | `Option<String>` | Current directory | Target directory for the new project |
| `--name` | `Option<String>` | Directory name | Project name (used in config) |
| `--force` | `bool` | `false` | Overwrite if directory exists |
| `--minimal` | `bool` | `false` | Create minimal config without examples |

**Exit Codes:**
- `0` - Success
- `1` - Error (directory exists without --force, permission denied, etc.)

### 2.2 `switchboard workflow init`

```bash
# Create workflow in current directory (must be inside a project)
switchboard workflow init

# Create workflow with name
switchboard workflow init --name my-workflow

# Create workflow with specific agents
switchboard workflow init --name my-workflow --agents architect developer

# Create workflow with schedule
switchboard workflow init --name my-workflow --schedule "0 9 * * *"
```

**Arguments:**

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--name` | `Option<String>` | Required | Workflow name (must be valid identifier) |
| `--agents` | `Vec<String>` | `["developer"]` | List of agent roles to scaffold |
| `--schedule` | `Option<String>` | `"0 9 * * *"` | Default cron schedule |
| `--path` | `Option<String>` | `.switchboard/workflows/<name>` | Target directory |

**Exit Codes:**
- `0` - Success
- `1` - Error (not in project directory, workflow exists, invalid name, etc.)

---

## 3. File Structure Changes

### 3.1 New Files to Create

```
src/
├── cli/
│   ├── commands/
│   │   ├── project.rs          # NEW: Project command dispatcher
│   │   └── workflow_init.rs    # NEW: Workflow init command handler
│   └── mod.rs                  # MODIFY: Add Project/WorkflowInit to Commands enum
├── commands/
│   ├── project/                # NEW: Project command module
│   │   ├── mod.rs
│   │   ├── init.rs             # Implementation of project init
│   │   └── types.rs            # CLI argument types
│   └── workflow_init/          # NEW: Workflow init module
│       ├── mod.rs
│       ├── init.rs              # Implementation of workflow init
│       └── types.rs             # CLI argument types
```

### 3.2 Template Files (Embedded in Rust)

**Project Templates:**

```
<PROJECT_ROOT>/
├── switchboard.toml             # Minimal configuration
├── .switchboard/                # Local data directory
│   ├── logs/                    # Log directory (empty)
│   └── .gitkeep
├── skills/                     # Project-level skills directory (empty)
│   └── .gitkeep
├── prompts/                    # Project-level prompts (empty)
│   └── .gitkeep
├── .gitignore                  # Standard gitignore for switchboard
└── README.md                   # Project readme with getting started
```

**Workflow Templates:**

```
.switchboard/workflows/<WORKFLOW_NAME>/
├── manifest.toml               # Workflow definition
├── prompts/                    # Agent prompt files
│   └── <AGENT_NAME>.md         # Template prompts for each agent
└── .gitkeep
```

---

## 4. Implementation Steps

### Step 1: Add CLI Command Types

**File:** `src/commands/project/types.rs`

Create the argument types for the project command:

```rust
use clap::{Args, Parser};

#[derive(Parser, Debug)]
#[clap(name = "project", about = "Manage Switchboard projects")]
pub struct ProjectCommand {
    #[clap(subcommand)]
    pub subcommand: ProjectSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ProjectSubcommand {
    /// Initialize a new Switchboard project
    Init(ProjectInit),
}

#[derive(Args, Debug)]
pub struct ProjectInit {
    /// Path where to create the project
    #[arg(short, long, value_name = "PATH")]
    pub path: Option<String>,
    
    /// Name of the project (defaults to directory name)
    #[arg(short, long, value_name = "NAME")]
    pub name: Option<String>,
    
    /// Force creation if directory exists
    #[arg(short, long)]
    pub force: bool,
    
    /// Create minimal configuration without examples
    #[arg(long)]
    pub minimal: bool,
}
```

**File:** `src/commands/workflow_init/types.rs`

Create the argument types for the workflow init command:

```rust
use clap::{Args, Parser};

#[derive(Parser, Debug)]
#[clap(name = "workflow", about = "Manage Switchboard workflows")]
pub struct WorkflowCommand {
    #[clap(subcommand)]
    pub subcommand: WorkflowSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum WorkflowSubcommand {
    /// Initialize a new workflow scaffold
    Init(WorkflowInit),
}

#[derive(Args, Debug)]
pub struct WorkflowInit {
    /// Name of the workflow to create
    #[arg(short, long, value_name = "NAME")]
    pub name: String,
    
    /// Agent roles to include in the workflow
    #[arg(short, long, value_name = "ROLE", num_args = 1..)]
    pub agents: Vec<String>,
    
    /// Default cron schedule for agents
    #[arg(long, value_name = "CRON")]
    pub schedule: Option<String>,
    
    /// Path where to create the workflow
    #[arg(short, long, value_name = "PATH")]
    pub path: Option<String>,
}
```

### Step 2: Implement Project Init Command

**File:** `src/commands/project/init.rs`

```rust
use crate::commands::project::{ProjectInit, ExitCode};
use crate::config::Config;
use std::fs;
use std::path::Path;

/// Run the `switchboard project init` command
pub async fn run_project_init(args: ProjectInit) -> ExitCode {
    // 1. Determine target directory
    let target_dir = match args.path {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir().unwrap_or_default(),
    };
    
    // 2. Check if directory exists
    if target_dir.exists() && !args.force {
        // Check if it's a valid switchboard project
        if target_dir.join("switchboard.toml").exists() {
            eprintln!("Error: Directory already contains a Switchboard project. Use --force to overwrite.");
            return ExitCode::Error;
        }
        if !is_empty_dir(&target_dir) {
            eprintln!("Error: Directory is not empty. Use --force to overwrite.");
            return ExitCode::Error;
        }
    }
    
    // 3. Create directory structure
    create_project_structure(&target_dir, &args).await
}
```

### Step 3: Implement Workflow Init Command

**File:** `src/commands/workflow_init/init.rs`

```rust
use crate::commands::workflow_init::{WorkflowInit, ExitCode};
use crate::config::Config;
use std::fs;
use std::path::Path;

/// Run the `switchboard workflow init` command
pub async fn run_workflow_init(args: WorkflowInit) -> ExitCode {
    // 1. Validate workflow name (must be valid identifier)
    if !is_valid_identifier(&args.name) {
        eprintln!("Error: '{}' is not a valid workflow name. Use only letters, numbers, and hyphens.", args.name);
        return ExitCode::Error;
    }
    
    // 2. Determine target directory
    let target_dir = args.path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".switchboard/workflows").join(&args.name));
    
    // 3. Check we're in a project (look for switchboard.toml)
    if !Path::new("switchboard.toml").exists() {
        eprintln!("Error: Not in a Switchboard project directory. Run 'switchboard project init' first.");
        return ExitCode::Error;
    }
    
    // 4. Create workflow structure
    create_workflow_structure(&target_dir, &args).await
}
```

### Step 4: Add Command Dispatch

**File:** `src/cli/commands/project.rs`

```rust
use crate::commands::project::{run_project_init, ProjectCommand};

pub async fn run_project(
    args: ProjectCommand,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    match args.subcommand {
        ProjectSubcommand::Init(init_args) => {
            run_project_init(init_args).await;
        }
    }
    Ok(())
}
```

**File:** `src/cli/commands/workflow_init.rs`

```rust
use crate::commands::workflow_init::{run_workflow_init, WorkflowCommand};

pub async fn run_workflow_init_cmd(
    args: WorkflowCommand,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    match args.subcommand {
        WorkflowSubcommand::Init(init_args) => {
            run_workflow_init(init_args).await;
        }
    }
    Ok(())
}
```

### Step 5: Update CLI Command Enum

**File:** `src/cli/mod.rs`

Add to the `Commands` enum:

```rust
/// Manage Switchboard projects
Project(ProjectCommand),

/// Initialize a new workflow
WorkflowInit(WorkflowInitCommand),
```

Add to the `run()` function dispatch:

```rust
Commands::Project(args) => commands::project::run_project(args, cli.config).await,
Commands::WorkflowInit(args) => commands::workflow_init::run_workflow_init_cmd(args, cli.config).await,
```

---

## 5. Template Definitions

### 5.1 Project Template Files.toml (Minimal):**
```toml
#

**switchboard Switchboard Project Configuration
# Generated by switchboard project init

[settings]
# Docker image name for agent containers
image_name = "switchboard-agent"
image_tag = "latest"

# Log directory
log_dir = ".switchboard/logs"

# Timezone for schedules
timezone = "system"

# =============================================================================
# AGENT CONFIGURATION
# =============================================================================

# Example agent - uncomment and customize
# [[agent]]
# name = "example-agent"
# schedule = "0 9 * * *"
# prompt = "Your agent prompt here"
```

**switchboard.toml (Full - without --minimal):**
```toml
# Switchboard Project Configuration
# Generated by switchboard project init

[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "system"
overlap_mode = "skip"

# =============================================================================
# EXAMPLE AGENT
# =============================================================================

[[agent]]
name = "example-agent"
schedule = "0 9 * * *"
prompt = """
This is an example agent. Edit this prompt to customize your agent's behavior.

The agent will run daily at 9:00 AM (according to the configured timezone).
"""

# =============================================================================
# GETTING STARTED
# =============================================================================
#
# 1. Edit this configuration file to define your agents
# 2. Add prompts to the prompts/ directory
# 3. Run 'switchboard up' to start the scheduler
# 4. Run 'switchboard list' to see your configured agents
#
# For more information, see the docs:
# https://github.com/switchboard/switchboard-rs-oss/docs
```

**.gitignore:**
```
# Switchboard
.switchboard/
*.log

# Editor
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db
```

**README.md:**
```markdown
# Switchboard Project

This is a Switchboard project. It contains AI coding agents that run on scheduled intervals.

## Getting Started

1. Edit `switchboard.toml` to configure your agents
2. Add prompts to the `prompts/` directory
3. Run `switchboard up` to start the scheduler

## Project Structure

```
.
├── switchboard.toml    # Main configuration file
├── .switchboard/       # Local data (logs, PID files)
├── skills/            # Project-level skills
└── prompts/           # Agent prompt files
```

## Available Commands

```bash
# Start the scheduler
switchboard up

# List configured agents
switchboard list

# View logs
switchboard logs

# Stop the scheduler
switchboard down
```

For more information, see the [documentation](https://github.com/switchboard/switchboard-rs-oss/docs).
```

### 5.2 Workflow Template Files

**manifest.toml:**
```toml
name = "{{WORKFLOW_NAME}}"
description = "{{DESCRIPTION}}"
version = "0.1.0"

[defaults]
schedule = "{{SCHEDULE}}"
timeout = "30m"
readonly = false
overlap_mode = "skip"

# Skills required by this workflow
# skills = []

[[prompts]]
name = "{{AGENT_NAME}}.md"
description = "{{AGENT_DESCRIPTION}}"
role = "{{AGENT_ROLE}}"

[[agents]]
name = "{{AGENT_NAME}}"
prompt_file = "{{AGENT_NAME}}.md"
schedule = "{{SCHEDULE}}"
timeout = "30m"
```

**prompts/developer.md:**
```markdown
# Developer Agent

You are a software developer agent. Your role is to implement features and fixes based on the project context.

## Responsibilities

- Implement new features as described in tasks
- Write clean, maintainable code
- Follow project conventions and best practices
- Create appropriate tests for new functionality

## Guidelines

1. Always read existing code before making changes
2. Ask clarifying questions if requirements are unclear
3. Report any bugs or issues you discover
4. Keep your changes focused and minimal

## Context

{{ADD CONTEXT ABOUT YOUR PROJECT HERE}}

## Current Task

{{THE SPECIFIC TASK TO WORK ON}}
```

**prompts/architect.md:**
```markdown
# Architect Agent

You are a software architect agent. Your role is to design systems and plan implementations.

## Responsibilities

- Design system architecture and component interactions
- Identify technical requirements and constraints
- Create technical specifications and documentation
- Review proposed solutions for feasibility

## Guidelines

1. Consider scalability and maintainability
2. Balance complexity with practical constraints
3. Document design decisions and rationale
4. Provide clear specifications for implementation
```

---

## 6. Error Handling

### 6.1 Project Init Error Cases

| Error | Message | Handling |
|-------|---------|----------|
| Directory not empty | `"Error: Directory is not empty. Use --force to overwrite."` | Exit with error code |
| Directory exists with project | `"Error: Directory already contains a Switchboard project."` | Exit with error code |
| Permission denied | `"Error: Cannot create directory: permission denied"` | Exit with error code |
| Invalid path | `"Error: Invalid path: ..."` | Exit with error code |
| Creation failed | `"Error: Failed to create project structure: ..."` | Clean up partial creation, exit with error |

### 6.2 Workflow Init Error Cases

| Error | Message | Handling |
|-------|---------|----------|
| Not in project | `"Error: Not in a Switchboard project directory."` | Exit with error code |
| Invalid workflow name | `"Error: 'name' is not a valid workflow name."` | Exit with error code |
| Workflow exists | `"Error: Workflow 'name' already exists."` | Suggest different name or --force |
| Invalid agent role | `"Error: Unknown agent role: 'role'. Valid roles: architect, developer, code_reviewer, scrum_master"` | Exit with error code |
| Invalid schedule | `"Error: Invalid cron schedule: ..."` | Exit with error code |
| Creation failed | `"Error: Failed to create workflow: ..."` | Clean up partial creation, exit with error |

### 6.3 Validation Functions

```rust
/// Check if directory is empty (only contains .gitkeep, .gitignore)
fn is_empty_dir(path: &Path) -> bool {
    // Implementation
}

/// Check if name is valid identifier (alphanumeric, hyphens, underscores)
fn is_valid_identifier(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Validate cron schedule format
fn is_valid_cron(schedule: &str) -> bool {
    // Basic validation or use cron crate
}
```

---

## 7. Testing Strategy

### 7.1 Unit Tests

**Test File:** `src/commands/project/init.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("my-workflow"));
        assert!(is_valid_identifier("workflow_123"));
        assert!(!is_valid_identifier("123-workflow"));
        assert!(!is_valid_identifier(""));
    }
    
    #[test]
    fn test_is_empty_dir() {
        // Test cases for empty/non-empty directories
    }
}
```

### 7.2 Integration Tests

**Test scenarios:**

1. **Project Init Tests:**
   - Create project in new directory
   - Create project with custom path
   - Create project with custom name
   - Create minimal project
   - Test --force on empty directory
   - Test --force on non-empty directory (should fail)
   - Test --force on existing project
   - Verify all files created correctly

2. **Workflow Init Tests:**
   - Create workflow in project directory
   - Create workflow with custom agents
   - Create workflow with custom schedule
   - Test error when not in project
   - Test error for invalid name
   - Test error for existing workflow
   - Verify manifest.toml is valid
   - Verify prompt files created

### 7.3 Manual Testing Checklist

```bash
# Project init
cd /tmp && rm -rf test-project && switchboard project init test-project
ls -la test-project/
cat test-project/switchboard.toml

# Workflow init
cd test-project && switchboard workflow init --name my-workflow --agents architect developer
ls -la .switchboard/workflows/my-workflow/
cat .switchboard/workflows/my-workflow/manifest.toml

# Validation
switchboard validate
```

---

## 8. Implementation Dependencies

### 8.1 External Crates

No new external dependencies required. The implementation uses:
- `clap` - Already in use for CLI
- `tokio` - Already in use for async
- `std::fs` - Standard library for file operations

### 8.2 Code References

- [`src/commands/skills/install.rs`](src/commands/skills/install.rs) - Similar file creation pattern
- [`src/commands/skills/types.rs`](src/commands/skills/types.rs) - CLI argument type patterns
- [`src/cli/commands/skills.rs`](src/cli/commands/skills.rs) - Command dispatcher pattern

---

## 9. Summary of Changes

| File | Change |
|------|--------|
| `src/cli/mod.rs` | Add `Project` and `WorkflowInit` to Commands enum, add dispatch in `run()` |
| `src/commands/mod.rs` | Add `pub mod project;` and `pub mod workflow_init;` |
| `src/commands/project/mod.rs` | Create module with init command |
| `src/commands/project/types.rs` | Create CLI argument types |
| `src/commands/project/init.rs` | Implement project scaffolding logic |
| `src/commands/workflow_init/mod.rs` | Create module with init command |
| `src/commands/workflow_init/types.rs` | Create CLI argument types |
| `src/commands/workflow_init/init.rs` | Implement workflow scaffolding logic |
| `src/cli/commands/project.rs` | Create command dispatcher |
| `src/cli/commands/workflow_init.rs` | Create command dispatcher |

---

## 10. Future Considerations

After initial implementation, consider adding:

1. **Interactive mode** - `switchboard project init --interactive` for guided setup
2. **Template registry** - Download project/workflow templates from registry
3. **Custom templates** - Support for user-defined templates
4. **Workflow validation** - Ensure workflow manifest is valid before creation
5. **Auto-install skills** - Automatically install required skills for workflow
