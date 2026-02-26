# Log Prefix Formats

This document defines the log prefix formats used throughout the switchboard system, enabling easy filtering and identification of different types of log messages.

## Overview

The switchboard system uses distinct log prefixes to separate different types of log output, making it easy to filter and analyze logs using standard command-line tools like `grep`. This design ensures that skill installation logs are clearly distinguishable from agent execution logs, and that errors are easily identifiable.

**Key Design Principle:** Log prefixes provide clear, machine-readable markers that enable precise filtering without parsing complex log formats.

## Log Prefix Categories

### 1. Skill Installation Logs

Skill installation logs use the `[SKILL INSTALL]` prefix family to mark all messages related to the skill installation phase.

#### 1.1 `[SKILL INSTALL]` - General Skill Installation Messages

**Purpose:** Marks informational messages about skill installation progress and status.

**When Used:**
- Before starting skill installation for an agent
- When logging individual skill installation attempts
- When reporting successful skill installation
- When noting preexisting skills that skip npx installation
- When providing remediation steps or context

**Examples:**

```
[SKILL INSTALL] Installing skills for agent 'my-agent'
[SKILL INSTALL] Installing skill: owner/repo
[SKILL INSTALL] Installing skill: owner/repo@skill-name
[SKILL INSTALL] Using preexisting skill: cool-skill (skipping npx installation)
[SKILL INSTALL] Found 2 preexisting skill(s) that will skip npx installation
[SKILL INSTALL] Skills installed successfully for agent 'my-agent'
[SKILL INSTALL] Installed: owner/repo, another/repo@skill-name
[SKILL INSTALL] Skills being installed: owner/repo, another/repo
[SKILL INSTALL] Exit code: 1
[SKILL INSTALL] Error: Skill installation failed for agent 'my-agent'
[SKILL INSTALL] Remediation steps:
 - Check if the skill exists: switchboard skills list
 - Verify the skill format: owner/repo or owner/repo@skill-name
 - Check network connectivity (npx needs internet access)
 - Review [SKILL INSTALL STDERR] lines above for detailed error information
[SKILL INSTALL] The agent did not execute. Fix the skill installation issues before retrying.
```

**Location in Code:**
- Generated entrypoint script: [`src/docker/skills.rs:380-420`](../src/docker/skills.rs:380-420)
- Log messages in [`src/docker/run/run.rs:720-752`](../src/docker/run/run.rs:720-752) (installation start)
- Error messages in [`src/docker/run/run.rs:1147-1187`](../src/docker/run/run.rs:1147-1187) (failure)
- Success messages in [`src/docker/run/run.rs:1206-1226`](../src/docker/run/run.rs:1206-1226) (success)

#### 1.2 `[SKILL INSTALL ERROR]` - Skill Installation Errors

**Purpose:** Marks error messages that occur during skill installation.

**When Used:**
- When the npx command exits with a non-zero code
- When container execution fails during skill installation

**Examples:**

```
[SKILL INSTALL ERROR] Command failed with exit code 1
```

**Location in Code:**
- Error handler in generated entrypoint script: [`src/docker/skills.rs:412-417`](../src/docker/skills.rs:412-417)

#### 1.3 `[SKILL INSTALL STDERR]` - Captured Stderr Output

**Purpose:** Marks stderr output from npx commands during skill installation, making it easy to identify technical error details.

**When Used:**
- For each line of stderr output from the `npx skills add` command

**Examples:**

```
[SKILL INSTALL STDERR] npm ERR! code ENOTFOUND
[SKILL INSTALL STDERR] npm ERR! errno ENOTFOUND
[SKILL INSTALL STDERR] npm ERR! network request failed
[SKILL INSTALL STDERR] fetch failed: unable to access 'https://github.com/owner/repo'
```

**Location in Code:**
- Stderr capture loop in generated entrypoint script: [`src/docker/skills.rs:392-398`](../src/docker/skills.rs:392-398)

### 2. Agent Execution Logs

**Purpose:** Raw output from the agent execution phase (container stdout/stderr).

**Prefix:** **None** - Agent execution logs have no special prefix.

**When Used:**
- All output after the agent starts executing (`exec kilocode`)
- Any output from the kilocode CLI during agent execution
- Any stdout/stderr from the containerized agent process

**Examples:**

```
Starting agent execution...
Agent: test-agent is running...
Thinking about the task...
Executing step 1...
Agent execution completed successfully.
```

**Important:** Agent execution logs are written directly via [`streams.rs`](../src/streams.rs) without modification or prefixing. This ensures that the raw output from the agent is preserved exactly as it appears.

**Location in Code:**
- Container stdout/stderr stream handling: [`src/streams.rs`](../src/streams.rs)
- Agent execution starts after skill installation in generated entrypoint script

## Log Prefix Summary Table

| Prefix | Purpose | Example | Filter Command |
|--------|---------|---------|----------------|
| `[SKILL INSTALL]` | General skill installation messages | `[SKILL INSTALL] Installing skill: owner/repo` | `grep "\[SKILL INSTALL\]"` |
| `[SKILL INSTALL ERROR]` | Skill installation errors | `[SKILL INSTALL ERROR] Command failed with exit code 1` | `grep "\[SKILL INSTALL ERROR\]"` |
| `[SKILL INSTALL STDERR]` | Captured stderr from npx | `[SKILL INSTALL STDERR] npm ERR! code ENOTFOUND` | `grep "\[SKILL INSTALL STDERR\]"` |
| *(No prefix)* | Agent execution logs | `Starting agent execution...` | `grep -v "\[SKILL INSTALL\]"` |

## Filtering Logs by Prefix

### View All Skill Installation Logs

```bash
# View all skill installation related logs
switchboard logs my-agent | grep "\[SKILL INSTALL\]"

# View with context lines (5 lines before and after each match)
switchboard logs my-agent | grep -C 5 "\[SKILL INSTALL\]"
```

### View Only Skill Installation Errors

```bash
# View only skill installation errors
switchboard logs my-agent | grep "\[SKILL INSTALL ERROR\]"

# View errors with context
switchboard logs my-agent | grep -C 10 "\[SKILL INSTALL ERROR\]"
```

### View Only Stderr Output from Skill Installation

```bash
# View stderr output for technical debugging
switchboard logs my-agent | grep "\[SKILL INSTALL STDERR\]"
```

### View Agent Execution Logs Only

```bash
# View only agent execution logs (exclude all skill installation logs)
switchboard logs my-agent | grep -v "\[SKILL INSTALL\]"

# Or view logs after skill installation phase
switchboard logs my-agent | tail -n +$(grep -n "exec kilocode" <(switchboard logs my-agent) | cut -d: -f1 | head -1)
```

### Exclude Stderr from Normal Output

```bash
# View skill installation logs without the verbose stderr output
switchboard logs my-agent | grep "\[SKILL INSTALL\]" | grep -v "\[SKILL INSTALL STDERR\]"
```

### View Combined Error Context

```bash
# View both ERROR and STDERR lines for complete error context
switchboard logs my-agent | grep -E "\[SKILL INSTALL (ERROR|STDERR)\]"
```

## Log Phase Structure

Logs from an agent execution follow a structured sequence:

```
[Phase 1: Skill Installation - all prefixed with [SKILL INSTALL]]
[SKILL INSTALL] Installing skills for agent 'my-agent'
[SKILL INSTALL] Installing skill: owner/repo
[SKILL INSTALL STDERR] npm output...
[SKILL INSTALL] Skills installed successfully for agent 'my-agent'

[Phase 2: Agent Execution - no prefix]
Starting agent execution...
Agent output...
```

This structure enables:
1. Easy identification of which phase a log message belongs to
2. Simple filtering to focus on skill installation or agent execution
3. Clear separation of system-generated installation logs vs. agent-generated content

## Benefits of Distinct Log Prefixes

### 1. Easy Filtering

Log prefixes enable quick filtering with standard tools:

```bash
# Show only installation-related logs
grep "\[SKILL INSTALL\]" agent.log

# Show only errors
grep "\[SKILL INSTALL ERROR\]" agent.log

# Exclude installation logs to see only agent output
grep -v "\[SKILL INSTALL\]" agent.log
```

### 2. Clear Problem Identification

When issues occur, prefixes make it immediately clear whether the problem is:
- During skill installation (check `[SKILL INSTALL ERROR]` and `[SKILL INSTALL STDERR]`)
- During agent execution (check unprefixed logs)

### 3. Debugging Efficiency

- `[SKILL INSTALL STDERR]` provides technical details from npx for debugging installation issues
- `[SKILL INSTALL ERROR]` provides high-level error summaries
- Unprefixed agent logs preserve the exact output from the agent for debugging agent behavior

### 4. Tool Integration

The prefix format is compatible with:
- Log aggregation systems (ELK stack, Splunk, etc.)
- Log monitoring tools (grep, awk, sed)
- Alerting systems that can trigger on specific error patterns

### 5. Testability

The distinct prefixes enable comprehensive test coverage:
- Tests verify that skill installation logs use correct prefixes
- Tests verify that agent logs do NOT use skill installation prefixes
- Tests can assert on specific prefix formats for automated verification

See [`tests/skills_log_prefix.rs`](../tests/skills_log_prefix.rs) for examples of prefix-related tests.

## Implementation Details

### Generated Entrypoint Script

The entrypoint script (generated by [`generate_entrypoint_script`](../src/docker/skills.rs)) is responsible for:
1. Outputting `[SKILL INSTALL]` prefixed messages for each skill installation step
2. Capturing stderr from npx and prefixing with `[SKILL INSTALL STDERR]`
3. Using `[SKILL INSTALL ERROR]` for command failures
4. Handing off to agent execution with `exec kilocode` (no further prefixing)

### Log Writing in Rust Code

The [`run_agent`](../src/docker/run/run.rs) function writes additional `[SKILL INSTALL]` prefixed messages:
- Before starting installation
- After successful installation
- On failure with detailed error context

These are written via the logger (see [`logger.rs`](../src/logger.rs)) to both the log file and terminal (in foreground mode).

### Agent Output Handling

Agent output is handled by [`streams.rs`](../src/streams.rs), which:
- Attaches to container stdout/stderr
- Writes raw output directly to the log file
- Does **not** modify or prefix agent output

## Testing Log Prefixes

The test suite includes comprehensive verification of log prefix behavior:

- [`tests/skills_log_prefix.rs`](../tests/skills_log_prefix.rs) - Tests for prefix correctness
- [`tests/skill_install_error_handling.rs`](../tests/skill_install_error_handling.rs) - Tests for error message prefixes
- [`src/docker/run/run.rs`](../src/docker/run/run.rs) (tests within) - Integration tests for log prefixes

### Running Log Prefix Tests

```bash
# Run all log prefix tests
cargo test --test skills_log_prefix

# Run specific test
cargo test --test skills_log_prefix test_skill_install_logs_have_prefix
```

## Related Documentation

- [Network Failure Handling](NETWORK_FAILURE_HANDLING.md) - Error handling during skill installation
- [Performance Skills List](PERFORMANCE_SKILLS_LIST.md) - Performance expectations for skills commands
- [Skill Error Handling](../tests/skill_install_error_handling.rs) - Test coverage for error scenarios

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-20 | Initial documentation of log prefix formats |

---

**Last Updated:** 2026-02-20  
**Maintainer:** Development Team  
**Related Sprint:** Sprint 4, Task 5
