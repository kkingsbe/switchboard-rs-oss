# Error Handling Audit Report

**Date:** 2026-02-15  
**Task:** Comprehensive Error Messages - auditing current state  
**Scope:** src/config, src/docker, src/scheduler, src/cli modules  

## Executive Summary

This audit documents the current error handling implementation across the Racing Sim Overlay project. The codebase has a solid foundation with well-structured error types, but there are several opportunities for improvement in error message clarity, consistency, and helpfulness.

### Key Findings

| Module | Error Types | Status | Assessment |
|---------|-------------|---------|------------|
| [`src/config/mod.rs`](src/config/mod.rs) | 3 variants | ✅ Good | Well-structured, comprehensive |
| [`src/docker/mod.rs`](src/docker/mod.rs) | 3 variants | ⚠️ Needs improvement | Basic structure, missing specific error types |
| [`src/scheduler/mod.rs`](src/scheduler/mod.rs) | 0 variants | ❌ Missing | Uses generic errors |
| [`src/cli/mod.rs`](src/cli/mod.rs) | 0 variants | ❌ Missing | Uses generic errors |

### Overall Assessment

- **Strengths:** Config module has excellent error structure with detailed location information
- **Weaknesses:** Scheduler and CLI modules lack dedicated error types
- **Gaps:** Missing error types for specific Docker failures, scheduler operations, and CLI validation

---

## 1. Config Module (`src/config/mod.rs`)

### Error Type Definition

**Location:** [`src/config/mod.rs:24-50`](src/config/mod.rs:24-50)

```rust
pub enum ConfigError {
    ParseError {
        file: String,
        line: Option<usize>,
        col: Option<usize>,
        message: String,
    },
    ValidationError {
        message: String,
        agent_name: Option<String>,
        field_name: Option<String>,
        line: Option<usize>,
        col: Option<usize>,
    },
    PromptFileNotFound {
        agent_name: String,
        prompt_file: String,
    },
}
```

### Display Implementation

**Location:** [`src/config/mod.rs:52-111`](src/config/mod.rs:52-111)

### Error Instances by Category

#### 1.1 Parse Errors

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| File read error | [`src/config/mod.rs:355-360`](src/config/mod.rs:355-360) | `Error parsing {file}: Failed to read file: {error}` | ✅ Good - includes file path and reason |
| TOML parse error | [`src/config/mod.rs:362-386`](src/config/mod.rs:362-386) | `Error parsing {file}:line {line}, col {col}: {message}` | ✅ Good - includes precise location |
| Prompt file read error | [`src/config/mod.rs:282-287`](src/config/mod.rs:282-287) | `Error parsing {file}: Failed to read prompt file: {error}` | ✅ Good - includes file path |

#### 1.2 Validation Errors

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| No agents defined | [`src/config/mod.rs:392-400`](src/config/mod.rs:392-400) | `Validation error: Configuration must define at least one agent. Add [[agent]] sections to your switchboard.toml` | ✅ Excellent - actionable advice |
| Duplicate agent name | [`src/config/mod.rs:405-416`](src/config/mod.rs:405-416) | `Validation error in field 'name': Duplicate agent name: '{name}'. Agent names must be unique across all [[agent]] sections` | ✅ Excellent - clear and helpful |
| Empty agent name | [`src/config/mod.rs:424-432`](src/config/mod.rs:424-432) | `Validation error in field 'name': Agent name cannot be empty. Each [[agent]] section must have a non-empty 'name' field` | ✅ Excellent - clear requirement |
| Missing prompt/prompt_file | [`src/config/mod.rs:436-447`](src/config/mod.rs:436-447) | `Validation error in agent '{name}': Agent '{name}' must have either 'prompt' (inline text) or 'prompt_file' (path to file) specified` | ✅ Excellent - explains both options |
| Both prompt and prompt_file | [`src/config/mod.rs:448-459`](src/config/mod.rs:448-459) | `Validation error in agent '{name}': Agent '{name}' must have exactly one of 'prompt' or 'prompt_file' specified, not both` | ✅ Excellent - clear mutual exclusivity |
| Invalid prompt_file path | [`src/config/mod.rs:467-478`](src/config/mod.rs:467-478) | `Validation error in agent '{name}' field 'prompt_file': Agent '{name}' has invalid prompt_file path: '{path}'. Check that the file path is absolute or relative to the config file` | ✅ Excellent - path guidance |
| Invalid cron expression (field count) | [`src/config/mod.rs:543-555`](src/config/mod.rs:543-555) | `Validation error in field 'schedule': Invalid cron expression '{schedule}': expected 5 fields (minute hour day month weekday), got {count}. Example: '0 */6 * * *' (runs every 6 hours)` | ✅ Excellent - includes example |
| Invalid cron expression (parse) | [`src/config/mod.rs:577-586`](src/config/mod.rs:577-586) | `Validation error in field 'schedule': Invalid cron expression '{schedule}': {error}` | ⚠️ Good - could include example |
| Invalid overlap_mode | [`src/config/mod.rs:500-511`](src/config/mod.rs:500-511) | `Validation error in field 'overlap_mode': Invalid overlap_mode '{mode}'. Must be one of: Skip (default, skip if already running), Queue (queue up to max_queue_size runs)` | ✅ Excellent - explains both options |
| Invalid timezone | [`src/config/mod.rs:605-616`](src/config/mod.rs:605-616) | `Validation error in field 'timezone': Invalid timezone '{tz}'. Use IANA timezone format (e.g., 'America/New_York', 'Europe/London', 'Asia/Tokyo'). See: https://en.wikipedia.org/wiki/List_of_tz_database_time_zones` | ✅ Excellent - examples and documentation link |
| Invalid timeout format | [`src/config/mod.rs:649-657`](src/config/mod.rs:649-657) | `Validation error in field 'timeout': Invalid timeout value: '{value}'. Valid formats: '30s' (30 seconds), '5m' (5 minutes), '1h' (1 hour). {error}` | ✅ Excellent - examples and formats |
| Timeout value zero | [`src/config/mod.rs:660-670`](src/config/mod.rs:660-670) | `Validation error in field 'timeout': Timeout value must be greater than 0. Use a positive value like '10s' or '5m'` | ✅ Excellent - provides examples |

#### 1.3 PromptFileNotFound Errors

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| Prompt file not found | [`src/config/mod.rs:480-484`](src/config/mod.rs:480-484) | `Prompt file '{prompt_file}' not found for agent '{agent_name}'` | ✅ Good - includes both file and agent |

### Config Module Assessment

**Overall:** ✅ **Excellent** - The config module has the best error handling in the codebase.

**Strengths:**
- Comprehensive error categories (Parse, Validate, PromptFileNotFound)
- Rich location information (file, line, col)
- Contextual error messages with agent_name and field_name
- Actionable suggestions and examples
- Good Display implementation

**Recommendations:**
- Consider adding examples to all cron expression errors (not just field count errors)
- Already very good - minimal improvements needed

---

## 2. Docker Module (`src/docker/mod.rs`)

### Error Type Definition

**Location:** [`src/docker/mod.rs:23-44`](src/docker/mod.rs:23-44)

```rust
#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    #[error("Docker connection error: {0}")]
    ConnectionError(String),

    #[error("Docker daemon unavailable: {reason}\n{suggestion}")]
    DockerUnavailable {
        reason: String,
        suggestion: String,
    },

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}
```

### Error Instances by Category

#### 2.1 Connection Errors

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| Permission denied | [`src/docker/mod.rs:221-258`](src/docker/mod.rs:221-258) | Detailed message with fix: `sudo usermod -aG docker $USER` | ✅ Good - includes actionable fix |
| Connection refused | [`src/docker/mod.rs:235-246`](src/docker/mod.rs:235-246) | Detailed message with platform-specific fix | ✅ Good - platform guidance |
| General connection | [`src/docker/mod.rs:247-256`](src/docker/mod.rs:247-256) | General guidance for Docker daemon | ✅ Good - covers multiple scenarios |
| Container creation failed | [`src/docker/run/run.rs:435-438`](src/docker/run/run.rs:435-438) | `Failed to create container '{name}': {error}` | ⚠️ Basic - could include more context |
| Container start failed | [`src/docker/run/run.rs:429-432`](src/docker/run/run.rs:429-432) | `Failed to start container '{id}': {error}` | ⚠️ Basic - could include more context |

#### 2.2 DockerUnavailable Errors

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| Ping failed | [`src/docker/mod.rs:261-270`](src/docker/mod.rs:261-270) | `Docker daemon appears to be running but is not responding: {error}` + suggestion | ✅ Good - includes suggestion |
| Timeout on check_available | [`src/docker/mod.rs:284-299`](src/docker/mod.rs:284-299) | `Docker daemon is not running or is not responding` + suggestion | ✅ Good - clear and actionable |

#### 2.3 IoError Errors

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| Timeout string empty | [`src/docker/run/wait/timeout.rs:67-69`](src/docker/run/wait/timeout.rs:67-69) | `Timeout string is empty` | ⚠️ Basic - could be more helpful |
| Invalid timeout value | [`src/docker/run/wait/timeout.rs:74-76`](src/docker/run/wait/timeout.rs:74-76) | `Invalid timeout value: '{value}'` | ⚠️ Basic - missing examples |
| Invalid timeout unit | [`src/docker/run/wait/timeout.rs:82-87`](src/docker/run/wait/timeout.rs:82-87) | `Invalid timeout unit: '{unit}' (use s, m, or h)` | ⚠️ OK - could include examples |
| Inspect container failed | [`src/docker/run/wait/timeout.rs:139-142`](src/docker/run/wait/timeout.rs:139-142) | `Failed to inspect container '{id}': {error}` | ⚠️ Basic - missing context |
| SIGTERM send failed | [`src/docker/run/wait/timeout.rs:184-189`](src/docker/run/wait/timeout.rs:184-189) | `Failed to send SIGTERM to container '{id}': {error}` | ⚠️ Basic - missing context |
| SIGKILL send failed | [`src/docker/run/wait/timeout.rs:207-212`](src/docker/run/wait/timeout.rs:207-212) | `Failed to send SIGKILL to container '{id}': {error}` | ⚠️ Basic - missing context |

#### 2.4 NotImplemented Errors

None currently used in the codebase.

### Docker Module Assessment

**Overall:** ⚠️ **Needs Improvement** - Basic structure but missing specific error types.

**Strengths:**
- Uses `thiserror` for clean derive implementation
- `DockerUnavailable` provides good structured errors with suggestions
- Connection errors include helpful platform-specific guidance

**Weaknesses:**
- Missing specific error types for:
  - Build failures (currently uses `anyhow::anyhow`)
  - Container creation failures (wrapped in generic errors)
  - Container stop/start failures (wrapped in generic errors)
  - Image pull/push failures
  - Volume mount errors
  - Resource limit errors
- IoError is overly broad - could be more specific
- No error codes or severity levels

**Missing Error Types Needed:**

```rust
// Suggested additions to DockerError
BuildError {
    image_name: String,
    image_tag: String,
    stage: String, // e.g., "fetching base image", "copying files", "running commands"
    reason: String,
},
ContainerCreationError {
    container_name: String,
    reason: String,
    suggestion: Option<String>,
},
ContainerStartError {
    container_id: String,
    reason: String,
},
ContainerStopError {
    container_id: String,
    reason: String,
},
ImagePullError {
    image_name: String,
    reason: String,
},
ImageNotFoundError {
    image_name: String,
    image_tag: String,
    suggestion: String,
},
VolumeMountError {
    workspace_path: String,
    reason: String,
},
PermissionError {
    operation: String, // e.g., "create container", "start container"
    reason: String,
    suggestion: String,
},
```

---

## 3. Scheduler Module (`src/scheduler/mod.rs`)

### Error Type Definition

**Status:** ❌ **No dedicated error type defined**

The scheduler module does not define a `SchedulerError` enum. It uses:

- `Box<dyn std::error::Error>` for public methods
- `anyhow::Error` for internal operations
- `Result<(), Box<dyn std::error::Error>>` return types

### Error Instances by Category

#### 3.1 Agent Registration Errors

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| Scheduler creation failed | [`src/scheduler/mod.rs:509-524`](src/scheduler/mod.rs:509-524) | Generic Box<dyn Error> | ❌ Missing - no specific error |
| Agent registration failed | [`src/scheduler/mod.rs:552-632`](src/scheduler/mod.rs:552-632) | Generic Box<dyn Error> | ❌ Missing - no specific error |
| Timezone resolution failed | [`src/scheduler/mod.rs:642-666`](src/scheduler/mod.rs:642-666) | `Invalid timezone: {timezone}` | ⚠️ Generic - could be more specific |
| Next run calculation failed | [`src/scheduler/mod.rs:677-703`](src/scheduler/mod.rs:677-703) | `Invalid cron schedule '{schedule}': {error}` | ⚠️ Generic - could be a specific variant |

#### 3.2 Scheduler Lifecycle Errors

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| Scheduler start failed | [`src/scheduler/mod.rs:714-724`](src/scheduler/mod.rs:714-724) | Generic Box<dyn Error> | ❌ Missing - no specific error |
| Scheduler stop failed | [`src/scheduler/mod.rs:735-739`](src/scheduler/mod.rs:735-739) | Generic Box<dyn Error> | ❌ Missing - no specific error |

#### 3.3 Agent Execution Errors

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| Prompt file not found | [`src/scheduler/mod.rs:320-324`](src/scheduler/mod.rs:320-324) | `Prompt file not found: {prompt_file}` | ⚠️ Basic - uses ConfigError |
| Prompt file read failed | [`src/scheduler/mod.rs:325-329`](src/scheduler/mod.rs:325-329) | `Failed to read prompt file: {error}` | ⚠️ Generic - uses anyhow |
| Agent must have prompt | [`src/scheduler/mod.rs:331-335`](src/scheduler/mod.rs:331-335) | `Agent must have either 'prompt' or 'prompt_file' specified` | ⚠️ Generic - should be ConfigError |
| Docker connection failed | [`src/scheduler/mod.rs:351-357`](src/scheduler/mod.rs:351-357) | `Docker connection failed: {error}` | ✅ Good - uses DockerError |
| Agent execution failed | [`src/scheduler/mod.rs:409-414`](src/scheduler/mod.rs:409-414) | `Failed to run agent: {error}` | ⚠️ Generic - uses anyhow |
| Metrics update failed | [`src/scheduler/mod.rs:451-462`](src/scheduler/mod.rs:451-462) | `Failed to update metrics with queued start time: {error}` | ⚠️ Generic - uses tracing::error |

#### 3.4 Queue/Overlap Errors

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| Agent already running (Skip) | [`src/scheduler/mod.rs:151-156`](src/scheduler/mod.rs:151-156) | Warning: `Agent '{name}' is already running (container_id: {id}), overlap_mode=skip, skipping new run` | ✅ Good - clear warning |
| Agent already running (Queue - full) | [`src/scheduler/mod.rs:163-169`](src/scheduler/mod.rs:163-169) | Warning: `Agent '{name}' queue is full (max {size}), skipping scheduled run` | ✅ Good - clear warning |
| Agent queued | [`src/scheduler/mod.rs:183-188`](src/scheduler/mod.rs:183-188) | Info: `Agent '{name}' is running, queued run ({position}/{max})` | ✅ Good - informative |
| Queued run execution failed | [`src/scheduler/mod.rs:297-303`](src/scheduler/mod.rs:297-303) | Error: `Error executing queued run for agent '{name}': {error}` | ⚠️ Generic - could be specific |
| Agent configuration not found | [`src/scheduler/mod.rs:304-309`](src/scheduler/mod.rs:304-309) | Warning: `Agent configuration not found for queued run: '{name}'` | ⚠️ Basic - could explain why |

### Scheduler Module Assessment

**Overall:** ❌ **Needs Dedicated Error Type** - The scheduler module lacks proper error handling structure.

**Strengths:**
- Good logging for overlap scenarios (Skip/Queue modes)
- Uses DockerError appropriately for Docker operations

**Weaknesses:**
- No `SchedulerError` enum defined
- Uses generic `Box<dyn std::error::Error>` return types
- No structured error information
- Errors from underlying libraries (tokio-cron-scheduler) are not wrapped in a specific type
- Missing error types for:
  - Scheduler already running
  - Scheduler not running
  - Agent not found by name
  - Invalid overlap mode
  - Queue full scenarios
  - Cron expression parsing errors
  - Timezone parsing errors
  - Job registration failures

**Missing Error Types Needed:**

```rust
// Suggested SchedulerError enum
pub enum SchedulerError {
    /// Agent not found in scheduler
    AgentNotFound {
        agent_name: String,
        available_agents: Vec<String>,
    },

    /// Scheduler is already running
    AlreadyRunning {
        pid: u32,
    },

    /// Scheduler is not running
    NotRunning,

    /// Invalid overlap mode
    InvalidOverlapMode {
        mode: String,
        valid_modes: Vec<String>,
    },

    /// Queue is full
    QueueFull {
        agent_name: String,
        current_size: usize,
        max_size: usize,
    },

    /// Cron expression parsing failed
    InvalidCronExpression {
        expression: String,
        reason: String,
        examples: Vec<String>,
    },

    /// Timezone parsing failed
    InvalidTimezone {
        timezone: String,
        reason: String,
        examples: Vec<String>,
    },

    /// Job registration failed
    JobRegistrationFailed {
        agent_name: String,
        reason: String,
    },

    /// Scheduler start failed
    StartFailed {
        reason: String,
        suggestion: String,
    },

    /// Scheduler stop failed
    StopFailed {
        reason: String,
    },
}
```

---

## 4. CLI Module (`src/cli/mod.rs`)

### Error Type Definition

**Status:** ❌ **No dedicated error type defined**

The CLI module does not define a `CliError` enum. It uses:

- `Box<dyn std::error::Error>` for all command handlers
- Direct `eprintln!` for error output
- `std::process::exit(1)` for fatal errors

### Error Instances by Category

#### 4.1 Config Loading Errors (Up Command)

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| Config parse failed | [`src/cli/mod.rs:290-293`](src/cli/mod.rs:290-293) | `✗ Configuration parsing failed: {error}` | ✅ Good - matches ConfigError pattern |
| Config validation failed | [`src/cli/mod.rs:294-297`](src/cli/mod.rs:294-297) | `✗ Configuration validation failed` | ⚠️ Basic - could include more detail |
| Prompt file not found | [`src/cli/mod.rs:298-304`](src/cli/mod.rs:298-304) | `✗ Prompt file not found: {file}` | ✅ Good - clear and actionable |
| Scheduler already running | [`src/cli/mod.rs:319-326`](src/cli/mod.rs:319-326) | `✗ Scheduler is already running (PID: {pid}). Use 'switchboard list' to see active agents or 'switchboard down' to stop it first` | ✅ Excellent - provides next steps |
| Workspace path invalid | [`src/cli/mod.rs:353-357`](src/cli/mod.rs:353-357) | `✗ Workspace path '{path}' does not exist or is not a directory. Check your switchboard.toml configuration or create the directory.` | ✅ Excellent - clear explanation |
| Scheduler creation failed | [`src/cli/mod.rs:369-376`](src/cli/mod.rs:369-376) | `⚠ Warning: Failed to create scheduler: {error}` | ✅ Good - warning not fatal |
| Agent registration failed | [`src/cli/mod.rs:402-409`](src/cli/mod.rs:402-409) | `⚠ Warning: Failed to register agent '{name}': {error}` | ✅ Good - warning not fatal |
| Docker connection failed | [`src/cli/mod.rs:426-431`](src/cli/mod.rs:426-431) | `⚠ Warning: Docker is not available` | ✅ Good - continues without Docker |
| Image check failed | [`src/cli/mod.rs:444-448`](src/cli/mod.rs:444-448) | `⚠ Warning: Failed to check image availability: {error}` | ✅ Good - continues anyway |
| Scheduler start failed | [`src/cli/mod.rs:471-474`](src/cli/mod.rs:471-474) | `✗ Failed to start scheduler: {error}` | ✅ Good - fatal error |
| .switchboard directory creation failed | [`src/cli/mod.rs:508-512`](src/cli/mod.rs:508-512) | `⚠ Warning: Failed to create .switchboard directory: {error}` | ✅ Good - warning not fatal |
| PID file write failed | [`src/cli/mod.rs:516-519`](src/cli/mod.rs:516-519) | `⚠ Warning: Failed to write PID file: {error}` | ✅ Good - warning not fatal |

#### 4.2 Config Loading Errors (Run Command)

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| Config parse failed | [`src/cli/mod.rs:636-639`](src/cli/mod.rs:636-639) | `✗ Configuration parsing failed: {error}` | ✅ Good - consistent with up command |
| Config validation failed | [`src/cli/mod.rs:640-643`](src/cli/mod.rs:640-643) | `✗ Configuration validation failed` | ⚠️ Basic - could include more detail |
| Prompt file not found | [`src/cli/mod.rs:644-650`](src/cli/mod.rs:644-650) | `✗ Prompt file not found: {file}` | ✅ Good - clear |
| Agent not found | [`src/cli/mod.rs:658-667`](src/cli/mod.rs:658-667) | `✗ Agent '{name}' not found in configuration. Available agents: {list}` | ✅ Excellent - shows alternatives |
| Workspace path invalid | [`src/cli/mod.rs:686-690`](src/cli/mod.rs:686-690) | `✗ Workspace path '{path}' does not exist or is not a directory. Check your switchboard.toml configuration or create the directory.` | ✅ Excellent - clear explanation |
| Prompt file not found (read) | [`src/cli/mod.rs:699-702`](src/cli/mod.rs:699-702) | `✗ Prompt file not found: {file}` | ✅ Good - clear |
| Prompt file read failed | [`src/cli/mod.rs:703-706`](src/cli/mod.rs:703-706) | `✗ Failed to read prompt file: {error}` | ✅ Good - includes error |
| Agent must have prompt | [`src/cli/mod.rs:709-712`](src/cli/mod.rs:709-712) | `✗ Agent must have either 'prompt' or 'prompt_file' specified` | ✅ Good - clear requirement |
| Docker connection failed | [`src/cli/mod.rs:730-734`](src/cli/mod.rs:730-734) | `✗ Docker connection failed: {error}` | ✅ Good - clear |
| Agent execution failed | [`src/cli/mod.rs:780-783`](src/cli/mod.rs:780-783) | `✗ Failed to run agent: {error}` | ⚠️ Basic - could include more context |

#### 4.3 Config Loading Errors (List Command)

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| Config parse failed | [`src/cli/mod.rs:813-816`](src/cli/mod.rs:813-816) | `✗ Configuration parsing failed: {error}` | ✅ Good - consistent |
| Config validation failed | [`src/cli/mod.rs:817-820`](src/cli/mod.rs:817-820) | `✗ Configuration validation failed` | ⚠️ Basic - could include more detail |
| Prompt file not found | [`src/cli/mod.rs:821-827`](src/cli/mod.rs:821-827) | `✗ Prompt file not found: {file}` | ✅ Good - clear |

#### 4.4 Config Loading Errors (Metrics Command)

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| Config parse failed | [`src/cli/mod.rs:858-861`](src/cli/mod.rs:858-861) | `✗ Configuration parsing failed: {error}` | ✅ Good - consistent |
| Config validation failed | [`src/cli/mod.rs:862-865`](src/cli/mod.rs:862-865) | `✗ Configuration validation failed` | ⚠️ Basic - could include more detail |
| Prompt file not found | [`src/cli/mod.rs:866-872`](src/cli/mod.rs:866-872) | `✗ Prompt file not found: {file}` | ✅ Good - clear |

#### 4.5 Down Command Errors

| Error | Location | Message Format | Assessment |
|--------|----------|----------------|------------|
| PID file not found | [`src/cli/mod.rs:906-909`](src/cli/mod.rs:906-909) | `No scheduler running. Use 'switchboard up' to start the scheduler` | ✅ Excellent - provides next step |
| PID parse failed | [`src/cli/mod.rs:915-918`](src/cli/mod.rs:915-918) | `Error: Failed to parse PID: {error}` | ✅ Good - includes error |
| Scheduler stop failed | [`src/cli/mod.rs:941-944`](src/cli/mod.rs:941-944) | `✗ Failed to stop scheduler (exit code: {code})` | ⚠️ Basic - could include more context |
| Scheduler kill failed | [`src/cli/mod.rs:951-954`](src/cli/mod.rs:951-954) | `✗ Failed to stop scheduler: {error}` | ⚠️ Basic - could be more specific |
| Docker connection failed | [`src/cli/mod.rs:964-969`](src/cli/mod.rs:964-969) | `⚠ Warning: Failed to connect to Docker: {error}` | ✅ Good - continues anyway |
| Container list failed | [`src/cli/mod.rs:981-986`](src/cli/mod.rs:981-986) | `⚠ Warning: Failed to list containers: {error}` | ✅ Good - continues anyway |
| Container has no ID | [`src/cli/mod.rs:1003-1007`](src/cli/mod.rs:1003-1007) | `⚠ Warning: Container has no ID, skipping` | ✅ Good - informative warning |
| Container stop failed | [`src/cli/mod.rs:1031-1034`](src/cli/mod.rs:1031-1034) | `✗ Failed to stop container {id}: {error}` | ⚠️ Basic - could include agent name |
| PID file removal failed | [`src/cli/mod.rs:1053-1056`](src/cli/mod.rs:1053-1056) | `✗ Failed to remove PID file: {error}` | ✅ Good - clear |
| .switchboard directory removal failed | [`src/cli/mod.rs:1067-1070`](src/cli/mod.rs:1067-1070) | `✗ Failed to remove .switchboard directory: {error}` | ✅ Good - clear |

### CLI Module Assessment

**Overall:** ⚠️ **Needs Dedicated Error Type** - Good error messages but no structured error types.

**Strengths:**
- Excellent use of ✗/⚠/✓ symbols for error/warning/success
- Very clear and actionable error messages
- Consistent formatting across commands
- Good distinction between warnings (⚠) and fatal errors (✗)
- Provides next steps in many error messages

**Weaknesses:**
- No `CliError` enum defined
- Uses `Box<dyn std::error::Error>` for all error paths
- Some errors use `std::process::exit(1)` which bypasses error handling
- Inconsistent error handling between commands (some return errors, some print and exit)
- Missing error types for:
  - Invalid command arguments
  - Missing required arguments
  - Invalid flag combinations
  - Permission errors
  - Network errors
  - Timeout errors

**Missing Error Types Needed:**

```rust
// Suggested CliError enum
pub enum CliError {
    /// Configuration file not found
    ConfigNotFound {
        path: String,
        suggestion: String,
    },

    /// Invalid command usage
    InvalidUsage {
        command: String,
        reason: String,
        usage_example: String,
    },

    /// Missing required argument
    MissingArgument {
        argument: String,
        command: String,
    },

    /// Agent not found
    AgentNotFound {
        agent_name: String,
        available_agents: Vec<String>,
    },

    /// Scheduler not running
    SchedulerNotRunning {
        suggestion: String,
    },

    /// Scheduler already running
    SchedulerAlreadyRunning {
        pid: u32,
        suggestion: String,
    },

    /// Docker not available
    DockerNotAvailable {
        reason: String,
        suggestion: String,
    },

    /// Permission denied
    PermissionDenied {
        operation: String,
        resource: String,
        suggestion: String,
    },
}
```

---

## 5. Cross-Cutting Patterns

### 5.1 Error Message Quality

| Module | Clarity | Actionability | Consistency | Examples | Location Info |
|---------|-----------|---------------|--------------|-----------|---------------|
| Config | ✅ Excellent | ✅ Excellent | ✅ Excellent | ✅ Excellent | ✅ Excellent |
| Docker | ⚠️ Mixed | ⚠️ Mixed | ⚠️ Mixed | ⚠️ Limited | ❌ None |
| Scheduler | ❌ None | ❌ None | N/A | ❌ None | ❌ None |
| CLI | ✅ Excellent | ✅ Excellent | ✅ Excellent | ✅ Good | ⚠️ Limited |

### 5.2 Common Patterns

**Good Patterns (seen in Config and CLI):**
- ✅ Using visual indicators (✗/⚠/✓)
- ✅ Providing actionable suggestions
- ✅ Including examples
- ✅ Showing available alternatives (e.g., agent names)
- ✅ Providing links to documentation
- ✅ Platform-specific guidance

**Missing Patterns:**
- ❌ Error codes/severity levels
- ❌ Structured error metadata
- ❌ Error recovery suggestions
- ❌ Stack traces in debug mode
- ❌ Error aggregation (multiple errors)

### 5.3 Inconsistencies

1. **Return Type Inconsistency:**
   - Config: `Result<T, ConfigError>` ✅
   - Docker: `Result<T, DockerError>` ✅
   - Scheduler: `Result<T, Box<dyn Error>>` ❌
   - CLI: `Result<T, Box<dyn Error>>` ❌

2. **Error Display Inconsistency:**
   - Config: Custom Display impl ✅
   - Docker: thiserror derive ✅
   - Scheduler: Uses underlying error's Display ⚠️
   - CLI: Manual eprintln! with custom formatting ⚠️

3. **Exit Strategy Inconsistency:**
   - Config: Returns errors ✅
   - Docker: Returns errors ✅
   - Scheduler: Returns errors ✅
   - CLI: Mix of returning errors and std::process::exit(1) ⚠️

---

## 6. Specific Gaps and Recommendations

### 6.1 High Priority Gaps

| Gap | Module | Impact | Recommendation |
|-----|---------|--------|---------------|
| No SchedulerError type | Scheduler | High | Create dedicated error enum |
| No CliError type | CLI | High | Create dedicated error enum |
| Build error uses anyhow | Docker | High | Add BuildError variant |
| Container errors generic | Docker | High | Add specific error variants |
| Scheduler already running check | Scheduler | High | Add AlreadyRunning variant |
| Scheduler not running check | Scheduler | High | Add NotRunning variant |

### 6.2 Medium Priority Gaps

| Gap | Module | Impact | Recommendation |
|-----|---------|--------|---------------|
| Missing timeout examples | Docker | Medium | Add examples to timeout errors |
| Missing container context | Docker | Medium | Include agent_name in errors |
| Inconsistent exit handling | CLI | Medium | Standardize on returning errors |
| No error codes | All modules | Medium | Add error codes for programmatic handling |
| No error severity | All modules | Medium | Add severity levels |

### 6.3 Low Priority Gaps

| Gap | Module | Impact | Recommendation |
|-----|---------|--------|---------------|
| Missing stack traces | All modules | Low | Add in debug builds |
| No error aggregation | All modules | Low | Consider for bulk operations |
| Limited i18n support | All modules | Low | Consider future i18n needs |

---

## 7. Recommended Error Type Definitions

### 7.1 Complete SchedulerError Definition

```rust
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    /// Agent not found in scheduler
    #[error("Agent '{agent_name}' not found. Available agents: {available_agents:?}")]
    AgentNotFound {
        agent_name: String,
        available_agents: Vec<String>,
    },

    /// Scheduler is already running
    #[error("Scheduler is already running (PID: {pid}). Use 'switchboard down' to stop it first")]
    AlreadyRunning {
        pid: u32,
    },

    /// Scheduler is not running
    #[error("Scheduler is not running. Use 'switchboard up' to start the scheduler")]
    NotRunning,

    /// Invalid overlap mode
    #[error("Invalid overlap mode '{mode}'. Valid modes: {valid_modes:?}")]
    InvalidOverlapMode {
        mode: String,
        valid_modes: Vec<String>,
    },

    /// Queue is full
    #[error("Agent '{agent_name}' queue is full ({current_size}/{max_size}). Scheduled run skipped")]
    QueueFull {
        agent_name: String,
        current_size: usize,
        max_size: usize,
    },

    /// Cron expression parsing failed
    #[error("Invalid cron expression '{expression}': {reason}. Example: '0 */6 * * *'")]
    InvalidCronExpression {
        expression: String,
        reason: String,
    },

    /// Timezone parsing failed
    #[error("Invalid timezone '{timezone}': {reason}. Example: 'America/New_York'")]
    InvalidTimezone {
        timezone: String,
        reason: String,
    },

    /// Job registration failed
    #[error("Failed to register job for agent '{agent_name}': {reason}")]
    JobRegistrationFailed {
        agent_name: String,
        reason: String,
    },

    /// Scheduler start failed
    #[error("Failed to start scheduler: {reason}. Check that no other scheduler instance is running")]
    StartFailed {
        reason: String,
    },

    /// Scheduler stop failed
    #[error("Failed to stop scheduler: {reason}")]
    StopFailed {
        reason: String,
    },
}
```

### 7.2 Complete DockerError Enhancement

```rust
#[derive(Debug, thiserror::Error)]
pub enum DockerError {
    #[error("Docker connection error: {0}")]
    ConnectionError(String),

    #[error("Docker daemon unavailable: {reason}\n{suggestion}")]
    DockerUnavailable {
        reason: String,
        suggestion: String,
    },

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Build error with stage information
    #[error("Docker build failed for image {image_name}:{image_tag}\n  Stage: {stage}\n  Reason: {reason}")]
    BuildError {
        image_name: String,
        image_tag: String,
        stage: String,
        reason: String,
    },

    /// Container creation error
    #[error("Failed to create container '{container_name}': {reason}")]
    ContainerCreationError {
        container_name: String,
        reason: String,
    },

    /// Container start error
    #[error("Failed to start container '{container_id}': {reason}")]
    ContainerStartError {
        container_id: String,
        reason: String,
    },

    /// Container stop error
    #[error("Failed to stop container '{container_id}': {reason}")]
    ContainerStopError {
        container_id: String,
        reason: String,
    },

    /// Image not found
    #[error("Docker image '{image_name}:{image_tag}' not found locally. Run: docker pull {image_name}:{image_tag}")]
    ImageNotFoundError {
        image_name: String,
        image_tag: String,
    },

    /// Image pull error
    #[error("Failed to pull image '{image_name}:{image_tag}': {reason}")]
    ImagePullError {
        image_name: String,
        image_tag: String,
        reason: String,
    },

    /// Volume mount error
    #[error("Failed to mount workspace '{workspace_path}': {reason}")]
    VolumeMountError {
        workspace_path: String,
        reason: String,
    },

    /// Permission error with suggestion
    #[error("Permission denied for {operation}: {reason}\n{suggestion}")]
    PermissionError {
        operation: String,
        reason: String,
        suggestion: String,
    },
}
```

### 7.3 Complete CliError Definition

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// Configuration file not found
    #[error("Configuration file not found: {path}\n  Suggestion: {suggestion}")]
    ConfigNotFound {
        path: String,
        suggestion: String,
    },

    /// Invalid command usage
    #[error("Invalid usage of '{command}' command: {reason}\n  Example: {usage_example}")]
    InvalidUsage {
        command: String,
        reason: String,
        usage_example: String,
    },

    /// Missing required argument
    #[error("Missing required argument '{argument}' for command '{command}'")]
    MissingArgument {
        argument: String,
        command: String,
    },

    /// Agent not found
    #[error("Agent '{agent_name}' not found. Available agents: {available_agents:?}")]
    AgentNotFound {
        agent_name: String,
        available_agents: Vec<String>,
    },

    /// Scheduler not running
    #[error("Scheduler is not running. {suggestion}")]
    SchedulerNotRunning {
        suggestion: String,
    },

    /// Scheduler already running
    #[error("Scheduler is already running (PID: {pid}). {suggestion}")]
    SchedulerAlreadyRunning {
        pid: u32,
        suggestion: String,
    },

    /// Docker not available
    #[error("Docker is not available. {reason}\n  Suggestion: {suggestion}")]
    DockerNotAvailable {
        reason: String,
        suggestion: String,
    },

    /// Permission denied
    #[error("Permission denied for {operation} on {resource}. {suggestion}")]
    PermissionDenied {
        operation: String,
        resource: String,
        suggestion: String,
    },
}
```

---

## 8. Implementation Roadmap

### Phase 1: Core Error Types (High Priority)

1. **Implement SchedulerError enum** in [`src/scheduler/mod.rs`](src/scheduler/mod.rs)
   - Define enum with all variants
   - Add Display impl or use thiserror
   - Replace Box<dyn Error> return types
   - Update all error sites

2. **Implement CliError enum** in [`src/cli/mod.rs`](src/cli/mod.rs)
   - Define enum with all variants
   - Add Display impl or use thiserror
   - Replace Box<dyn Error> return types
   - Update all error sites
   - Remove std::process::exit(1) calls

3. **Enhance DockerError enum** in [`src/docker/mod.rs`](src/docker/mod.rs)
   - Add BuildError variant
   - Add ContainerCreationError variant
   - Add ContainerStartError variant
   - Add ContainerStopError variant
   - Add ImageNotFoundError variant
   - Add PermissionError variant

### Phase 2: Error Message Improvements (Medium Priority)

4. **Add examples to timeout errors** in Docker
   - Include format examples
   - Show valid values

5. **Include agent_name in container errors**
   - Helps identify which agent failed
   - Improves debugging

6. **Standardize error formatting**
   - Consistent use of ✗/⚠/✓
   - Consistent message structure

### Phase 3: Advanced Error Handling (Low Priority)

7. **Add error codes**
   - Programmatic error handling
   - Error rate limiting
   - Error categorization

8. **Add error severity levels**
   - Error vs warning vs info
   - User-controlled verbosity

9. **Add debug mode enhancements**
   - Stack traces
   - Detailed error context
   - Error aggregation

---

## 9. Test Coverage

### Current Test Coverage

| Module | Error Tests | Coverage |
|---------|-------------|----------|
| Config | ~15 tests | ✅ Good |
| Docker | ~3 tests | ⚠️ Limited |
| Scheduler | 0 error tests | ❌ Missing |
| CLI | 0 error tests | ❌ Missing |

### Recommended Test Coverage

**Config Module:** Already has good test coverage. Maintain.

**Docker Module:** Add tests for:
- ConnectionError scenarios
- DockerUnavailable scenarios
- IoError scenarios (timeout parsing)
- New error variants (once implemented)

**Scheduler Module:** Add tests for:
- AgentNotFound error
- AlreadyRunning error
- NotRunning error
- InvalidCronExpression error
- InvalidTimezone error
- QueueFull error

**CLI Module:** Add tests for:
- ConfigNotFound error
- AgentNotFound error
- InvalidUsage error
- MissingArgument error
- SchedulerNotRunning error

---

## 10. Documentation Recommendations

### 10.1 Code Documentation

Add doc comments to all error variants:

```rust
/// Agent not found in the scheduler
///
/// This error occurs when attempting to execute an agent
/// that has not been registered with the scheduler.
#[error(...)]
AgentNotFound { ... }
```

### 10.2 User Documentation

Create a new user guide section on error handling:

**File:** `docs/errors.md`

Topics to cover:
- Common error messages and solutions
- Troubleshooting guide
- Error message format explanation
- How to report bugs with error information

### 10.3 Developer Documentation

Add error handling guide to ARCHITECTURE.md:

Topics to cover:
- When to create new error types
- Error message best practices
- Error propagation patterns
- Testing error handling

---

## 11. Conclusion

### Summary of Findings

1. **Config module** is exemplary with comprehensive, well-structured error handling
2. **Docker module** has a good foundation but needs specific error variants
3. **Scheduler module** lacks dedicated error types entirely
4. **CLI module** has excellent error messages but no structured error types

### Priority Recommendations

**Immediate (Sprint 4):**
- Implement `SchedulerError` enum
- Implement `CliError` enum
- Enhance `DockerError` with missing variants

**Short-term (Sprint 5):**
- Add examples to timeout errors
- Include agent_name in container errors
- Standardize error formatting across modules

**Long-term (Future):**
- Add error codes for programmatic handling
- Add error severity levels
- Implement debug mode enhancements

### Impact Assessment

Implementing these recommendations will:
- ✅ Improve user experience with clearer error messages
- ✅ Make errors more actionable and helpful
- ✅ Enable better error handling in calling code
- ✅ Improve testability with specific error types
- ✅ Provide consistent error patterns across the codebase

---

**Audit completed:** 2026-02-15T14:44:00Z  
**Auditor:** Worker 4 (TODO4.md - Comprehensive Error Messages)  
**Status:** Ready for implementation planning
