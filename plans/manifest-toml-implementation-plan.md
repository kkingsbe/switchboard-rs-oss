# manifest.toml Implementation Plan

## Overview

The `manifest.toml` feature enables workflows in the switchboard-workflows registry to declare their configuration in a standardized format. This allows the Switchboard CLI to:

- Automatically discover workflow configuration during installation
- Validate workflow manifests before use
- Generate `switchboard.toml` entries from workflow manifests
- Provide a seamless "apply" or "add" workflow experience

Each workflow in the registry will include a `manifest.toml` file defining:
- Default agent configurations
- Available prompts and their purposes
- Default settings (timeout, schedule, etc.)
- Required environment variables

---

## Data Structures

### Core Manifest Structs

The following Rust structs will be defined in a new module `src/workflows/manifest.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ManifestDefaults defines default configuration values for agents
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManifestDefaults {
    /// Default cron schedule for agents (e.g., "0 9 * * *")
    #[serde(default)]
    pub schedule: Option<String>,
    /// Default timeout for agent runs (e.g., "30m", "2h")
    #[serde(default)]
    pub timeout: Option<String>,
    /// Default read-only mode
    #[serde(default)]
    pub readonly: Option<bool>,
    /// Default overlap mode ("skip" or "queue")
    #[serde(default)]
    pub overlap_mode: Option<String>,
    /// Default max queue size for queue mode
    #[serde(default)]
    pub max_queue_size: Option<usize>,
    /// Default environment variables
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    /// Default skills available to agents
    #[serde(default)]
    pub skills: Option<Vec<String>>,
}

/// ManifestPrompt represents a single prompt file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManifestPrompt {
    /// Filename of the prompt (e.g., "ARCHITECT.md")
    pub name: String,
    /// Human-readable description of what this prompt does
    pub description: Option<String>,
    /// Role or purpose of this prompt (e.g., "architect", "developer")
    pub role: Option<String>,
}

/// ManifestAgent defines an agent configuration from the manifest
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManifestAgent {
    /// Agent name (will be prefixed with workflow name)
    pub name: String,
    /// Prompt file to use (reference to prompts array)
    pub prompt_file: String,
    /// Agent-specific schedule override
    #[serde(default)]
    pub schedule: Option<String>,
    /// Agent-specific timeout override
    #[serde(default)]
    pub timeout: Option<String>,
    /// Agent-specific readonly override
    #[serde(default)]
    pub readonly: Option<bool>,
    /// Agent-specific overlap mode override
    #[serde(default)]
    pub overlap_mode: Option<String>,
    /// Agent-specific max queue size override
    #[serde(default)]
    pub max_queue_size: Option<usize>,
    /// Agent-specific environment variables
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    /// Agent-specific skills
    #[serde(default)]
    pub skills: Option<Vec<String>>,
}

/// ManifestConfig represents the complete manifest.toml structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManifestConfig {
    /// Workflow name (matches directory name)
    #[serde(default)]
    pub name: Option<String>,
    /// Human-readable workflow description
    #[serde(default)]
    pub description: Option<String>,
    /// Version of the workflow manifest
    #[serde(default)]
    pub version: Option<String>,
    /// Default configuration values
    #[serde(default)]
    pub defaults: Option<ManifestDefaults>,
    /// Available prompt files
    #[serde(default)]
    pub prompts: Vec<ManifestPrompt>,
    /// Agent configurations
    #[serde(default)]
    pub agents: Vec<ManifestAgent>,
}
```

### Error Types

```rust
/// Errors that can occur when parsing or using manifest.toml
#[derive(Debug, Error)]
pub enum ManifestError {
    /// Failed to read manifest file
    #[error("Failed to read manifest file: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Failed to parse TOML
    #[error("Failed to parse manifest.toml: {0}")]
    ParseError(String),
    
    /// Validation error in manifest
    #[error("Manifest validation error: {0}")]
    ValidationError(String),
    
    /// Referenced prompt file not found
    #[error("Prompt file '{0}' referenced in manifest not found")]
    PromptNotFound(String),
    
    /// Invalid overlap mode value
    #[error("Invalid overlap mode '{0}': must be 'skip' or 'queue'")]
    InvalidOverlapMode(String),
}
```

### Conversion to Config Agent

A conversion trait will transform `ManifestAgent` to the application's `Agent` struct:

```rust
impl ManifestAgent {
    /// Convert manifest agent to Config Agent
    /// Applies workflow defaults and prefixes agent name with workflow name
    pub fn to_agent(&self, workflow_name: &str, defaults: &ManifestDefaults) -> Agent {
        let full_name = format!("{}_{}", workflow_name, self.name);
        
        Agent {
            name: full_name,
            prompt: None,
            prompt_file: Some(format!("prompts/{}", self.prompt_file)),
            schedule: self.schedule.clone().or_else(|| defaults.schedule.clone())
                .unwrap_or_else(|| "0 9 * * *".to_string()),
            env: self.env.clone().or_else(|| defaults.env.clone()),
            readonly: self.readonly.or(defaults.readonly),
            timeout: self.timeout.clone().or_else(|| defaults.timeout.clone()),
            overlap_mode: None, // Will be resolved after parsing
            max_queue_size: self.max_queue_size.or(defaults.max_queue_size),
            skills: self.skills.clone().or_else(|| defaults.skills.clone()),
        }
    }
}
```

---

## Module Changes

### Files to Create

| File | Purpose |
|------|---------|
| `src/workflows/manifest.rs` | Manifest parsing and validation logic |
| `src/commands/workflows/validate.rs` | Validate workflow manifest command |
| `src/commands/workflows/apply.rs` | Apply/generate switchboard.toml entries |

### Files to Modify

| File | Changes |
|------|---------|
| `src/workflows/mod.rs` | Add `pub mod manifest;` and export `ManifestConfig`, `ManifestError` |
| `src/commands/workflows/mod.rs` | Add `pub mod validate;`, `pub mod apply;` and new subcommands |
| `src/commands/workflows/types.rs` | Add `WorkflowsValidate`, `WorkflowsApply` argument structs |
| `src/cli/commands/workflows.rs` | Add handlers for new subcommands |

---

## Implementation Phases

### Phase 1: Add manifest parsing module

**Objective**: Create the core manifest.toml parsing infrastructure

**Tasks**:

1. Create `src/workflows/manifest.rs`
   - Define `ManifestDefaults`, `ManifestPrompt`, `ManifestAgent`, `ManifestConfig` structs
   - Implement `ManifestError` enum
   - Implement `ManifestConfig::from_path(path: &Path) -> Result<ManifestConfig, ManifestError>`
   - Implement validation: check all referenced prompt files exist
   - Implement conversion: `ManifestAgent::to_agent()`

2. Add module exports in `src/workflows/mod.rs`
   ```rust
   pub mod manifest;
   pub use manifest::{ManifestConfig, ManifestDefaults, ManifestAgent, ManifestError};
   ```

3. Add unit tests for manifest parsing
   - Test valid manifest parsing
   - Test missing required fields error
   - Test prompt file validation
   - Test conversion to Agent struct

**Estimated Effort**: 2-3 days

---

### Phase 2: Modify workflow install

**Objective**: to fetch manifest.toml Automatically download and store manifest.toml when installing a workflow

**Tasks**:

1. Modify `src/workflows/github.rs`
   - Add `download_manifest(workflow_name: &str) -> Result<ManifestConfig, WorkflowsError>` method
   - Fetch `manifest.toml` from workflow directory

2. Modify `src/commands/workflows/install.rs`
   - After downloading workflow files, attempt to download manifest.toml
   - Store manifest.toml in workflow directory alongside prompts/
   - Log warning if manifest.toml is not present (backward compatibility)

3. Add helper function to load manifest from installed workflow:
   ```rust
   pub fn load_workflow_manifest(workflow_name: &str) -> Option<ManifestConfig>
   ```

**Estimated Effort**: 1-2 days

---

### Phase 3: Add validate command for manifest.toml

**Objective**: Provide `switchboard workflows validate <workflow-name>` command

**Tasks**:

1. Add CLI types in `src/commands/workflows/types.rs`:
   ```rust
   #[derive(Parser, Debug)]
   pub struct WorkflowsValidate {
       /// Name of the workflow to validate
       #[arg(value_name = "WORKFLOW_NAME")]
       pub workflow_name: String,
   }
   ```

2. Create `src/commands/workflows/validate.rs`:
   ```rust
   pub async fn run_workflows_validate(args: WorkflowsValidate, config: &Config) -> ExitCode
   ```

3. Register new subcommand in `src/commands/workflows/mod.rs`:
   - Add `WorkflowsSubcommand::Validate(WorkflowsValidate)` variant
   - Add handler in `run_workflows()`

4. Validation checks:
   - Verify manifest.toml exists and is parseable
   - Verify all referenced prompt files exist in prompts/
   - Verify cron schedule format is valid
   - Verify overlap_mode values are valid ("skip" or "queue")
   - Verify timeout format is valid

**Estimated Effort**: 1-2 days

---

### Phase 4: Add "apply" or "add" command

**Objective**: Generate switchboard.toml entries from workflow manifest

**Tasks**:

1. Add CLI types in `src/commands/workflows/types.rs`:
   ```rust
   #[derive(Parser, Debug)]
   pub struct WorkflowsApply {
       /// Name of the workflow to apply
       #[arg(value_name = "WORKFLOW_NAME")]
       pub workflow_name: String,
       /// Agent name prefix (optional, defaults to workflow name)
       #[arg(long, short)]
       pub prefix: Option<String>,
       /// Output file (default: switchboard.toml)
       #[arg(long, short, value_name = "FILE")]
       pub output: Option<String>,
       /// Append to existing switchboard.toml
       #[arg(long, short = 'a')]
       pub append: bool,
       /// Skip confirmation prompt
       #[arg(long)]
       pub yes: bool,
   }
   ```

2. Create `src/commands/workflows/apply.rs`:
   ```rust
   pub async fn run_workflows_apply(args: WorkflowsApply, config: &Config) -> ExitCode
   ```

3. Implementation logic:
   - Load manifest.toml for the workflow
   - Load existing switchboard.toml (if `--append`) or create new
   - Convert manifest agents to Config agents using defaults
   - Merge agents into Config (handle name conflicts with prefix)
   - Write output file

4. Support for `--dry-run` flag to preview changes:
   ```rust
   #[derive(Parser, Debug)]
   pub struct WorkflowsApply {
       // ... existing fields
       /// Preview changes without writing
       #[arg(long)]
       pub dry_run: bool,
   }
   ```

**Estimated Effort**: 2-3 days

---

### Phase 5: Integration tests

**Objective**: End-to-end testing of manifest workflow

**Tasks**:

1. Create integration tests:
   - Test full workflow: install → validate → apply
   - Test backward compatibility (workflows without manifest.toml)
   - Test error handling for invalid manifests

2. Create test workflow in repository:
   - Add manifest.toml to existing workflow in switchboard-workflows
   - Verify it passes validation
   - Verify generated config is valid

3. CLI integration tests:
   - Test `workflows validate` command
   - Test `workflows apply` command with various options
   - Test error messages are helpful

**Estimated Effort**: 1-2 days

---

## Backward Compatibility

### Handling Workflows Without manifest.toml

The implementation must gracefully handle workflows that don't have a manifest.toml file:

1. **During install**: Log a warning message
   ```
   Warning: Workflow 'bmad' does not include manifest.toml.
   Some features may not be available. Consider updating the workflow.
   ```

2. **During validate**: Show informative message
   ```
   Error: Workflow 'bmad' does not have a manifest.toml file.
   This workflow may not be compatible with the latest features.
   ```

3. **During apply**: Error with suggestion
   ```
   Error: Workflow 'bmad' does not have manifest.toml.
   Cannot auto-generate switchboard.toml entries.
   
   Manual configuration required. See: https://docs.switchboard.dev/workflows
   ```

4. **Existing functionality unchanged**: 
   - `workflows list` continues to work
   - `workflows install` continues to work (just downloads files)
   - `workflows remove` continues to work
   - Prompt files are still accessible even without manifest

### Feature Detection

```rust
pub fn has_manifest(workflow_name: &str) -> bool {
    let manifest_path = PathBuf::from(WORKFLOWS_DIR)
        .join(workflow_name)
        .join("manifest.toml");
    manifest_path.exists()
}
```

---

## CLI Commands

### New Commands

#### `switchboard workflows validate`

Validate a workflow's manifest.toml file.

```
switchboard workflows validate <workflow-name>
```

**Options**:
- `<workflow-name>` - Name of the workflow to validate (required)

**Output**:
- Success: "✓ Manifest is valid"
- Failure: Detailed error messages with line numbers

#### `switchboard workflows apply`

Generate switchboard.toml entries from workflow manifest.

```
switchboard workflows apply <workflow-name> [OPTIONS]
```

**Options**:
| Flag | Description |
|------|-------------|
| `<workflow-name>` | Name of the workflow to apply (required) |
| `-o, --output <FILE>` | Output file (default: switchboard.toml) |
| `-a, --append` | Append to existing switchboard.toml |
| `-p, --prefix <PREFIX>` | Agent name prefix |
| `--dry-run` | Preview changes without writing |
| `-y, --yes` | Skip confirmation prompt |

**Examples**:
```bash
# Apply workflow to new switchboard.toml
switchboard workflows apply bmad

# Append to existing config
switchboard workflows apply bmad --append

# Preview what would be generated
switchboard workflows apply bmad --dry-run

# Custom output file
switchboard workflows apply bmad -o my-config.toml
```

### Updated Commands

#### `switchboard workflows install`

Enhanced to download and store manifest.toml.

```
switchboard workflows install <workflow-name> [OPTIONS]
```

**New behavior**:
- Downloads manifest.toml along with prompts/
- Logs warning if manifest.toml is not present

#### `switchboard workflows list`

Enhanced to show manifest information.

```
switchboard workflows list [OPTIONS]
```

**New columns**:
- `Has Manifest` - Shows ✓ or ✗

---

## File Structure Summary

```
src/
├── workflows/
│   ├── mod.rs              # Add: pub mod manifest;
│   ├── manifest.rs         # NEW: Manifest parsing
│   ├── github.rs           # Modify: Add download_manifest()
│   └── metadata.rs
├── commands/
│   └── workflows/
│       ├── mod.rs           # Modify: Add Validate, Apply subcommands
│       ├── types.rs         # Modify: Add WorkflowsValidate, WorkflowsApply
│       ├── install.rs       # Modify: Fetch manifest.toml
│       ├── validate.rs     # NEW: Validate command
│       └── apply.rs        # NEW: Apply command
└── cli/
    └── commands/
        └── workflows.rs     # Modify: Add handlers
```

---

## Example manifest.toml

```toml
# Example: .switchboard/workflows/bmad/manifest.toml

name = "bmad"
description = "BMAD multi-agent development workflow"
version = "1.0.0"

[defaults]
schedule = "0 9 * * *"      # Daily at 9 AM
timeout = "30m"
readonly = false
overlap_mode = "skip"

[defaults.env]
OPENAI_MODEL = "gpt-4"

[[prompts]]
name = "ARCHITECT.md"
description = "System architect role - designs architecture and technical decisions"
role = "architect"

[[prompts]]
name = "DEV_PARALLEL.md"
description = "Parallel developer role - implements code in parallel"
role = "developer"

[[agents]]
name = "architect"
prompt_file = "ARCHITECT.md"

[[agents]]
name = "developer"
prompt_file = "DEV_PARALLEL.md"
# Override default schedule for developer
schedule = "0 10 * * *"
```

---

## Migration Path

1. **Phase 1-2**: Core infrastructure (manifest parsing, download)
2. **Phase 3**: Validate command for workflow maintainers
3. **Phase 4**: Apply command for users
4. **Phase 5**: Testing and documentation
5. **Post-launch**: Update existing workflows in registry to include manifest.toml

---

## Dependencies

- **No new dependencies** - Uses existing `toml` crate for parsing
- Reuses existing:
  - `WorkflowsError` from `src/workflows/mod.rs`
  - `Agent` struct from `src/config/mod.rs`
  - `Config` struct from `src/config/mod.rs`
  - CLI patterns from existing workflow commands

---

## Success Criteria

1. ✅ Manifest.toml files can be parsed and validated
2. ✅ Workflows without manifest.toml continue to work (backward compatible)
3. ✅ `workflows validate` command reports clear errors
4. ✅ `workflows apply` generates valid switchboard.toml
5. ✅ All existing workflows commands remain functional
