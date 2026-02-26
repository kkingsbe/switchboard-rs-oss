# Container Skill Installation Behavior - Information Gathering Report

> **Subtask:** Gather context for Task 8 - Container Skill Installation Behavior
> **Date:** 2026-02-20T16:23:45Z
> **Agent:** Code Mode Subagent
> **For:** Worker Agent 1 (TODO1.md, Sprint 4 documentation focus)
> **Task:** Document Container Skill Installation Behavior (Task 8 in TODO1.md)

---

## Summary

This report provides comprehensive information about container skill installation behavior in the switchboard project, gathered for documentation purposes. It explains what container skills are, how they are installed, key behaviors, dependencies, and error conditions.

---

## 1. What is a "Container Skill"?

### Definition

A **container skill** is a pre-built module that extends the capabilities of Kilo Code agents running inside Docker containers. Skills are automatically installed inside agent containers at startup before the agent begins executing its task.

### Purpose

Skills enable AI agents to have specialized capabilities such as:
- Frontend design tools (e.g., UI/UX design guidelines)
- Security auditors (vulnerability scanning)
- Specialized code analysis utilities
- Language-specific helpers (Python, Rust, JavaScript, etc.)

### Skill Source Format

Skills are declared in [`switchboard.toml`](switchboard.toml:1) using the optional `skills` field on agent configurations. Two formats are supported:

| Format | Example | Description |
|---------|---------|-------------|
| `owner/repo` | `vercel-labs/agent-skills` | Installs all skills from the repository |
| `owner/repo@skill-name` | `vercel-labs/agent-skills@frontend-design` | Installs a specific skill from the repository |

### Example Configuration

```toml
[[agent]]
name = "security-scan"
schedule = "0 2 * * *"
prompt = "Scan for security vulnerabilities."
skills = [
    "vercel-labs/agent-skills@frontend-design",
    "anthropics/skills@security-audit",
]
```

### Relationship to skills.sh

The container skill feature integrates with [skills.sh](https://skills.sh) marketplace by Vercel Labs. Switchboard acts as a **thin ergonomic wrapper** around the `npx skills` CLI:
- Switchboard does **not** implement its own GitHub API integration
- Switchboard does **not** implement web scraping or skill fetching logic
- All skill discovery and installation operations are delegated exclusively to `npx skills`

---

## 2. Step-by-Step Installation Process

### Phase 1: Configuration Validation (Host Side)

1. **User declares skills in `switchboard.toml`:**
   - Each agent can optionally specify a `skills` field
   - Skills field contains a list of skill sources in `owner/repo` or `owner/repo@skill-name` format

2. **`switchboard validate` checks configuration:**
   - Validates skill entry format using regex pattern: `^[^/]+/[^@]+(?:@[^/]+)?$`
   - Warns if `skills = []` is present but empty
   - Reports error for invalid formats, duplicates, or missing components

3. **Three configuration scenarios handled:**
   - **No skills field**: Uses default Dockerfile entrypoint, no skill installation
   - **Empty skills list (`skills = []`)**: Uses default Dockerfile entrypoint, no skill installation
   - **Populated skills list**: Generates custom entrypoint script for skill installation

### Phase 2: Entrypoint Script Generation (Host Side)

1. **Skill format validation:**
   - Each skill in the list is validated using [`validate_skill_format()`](src/docker/skills.rs:123)
   - Checks for: exactly one `/` separator, non-empty owner/repo components, valid characters

2. **Preexisting skill detection:**
   - [`find_preexisting_skills()`](src/docker/run/run.rs:405) scans `.kilocode/skills/` directory
   - Checks which skills already exist locally with `SKILL.md` files
   - Preexisting skills are logged and skipped during npx installation

3. **Script generation:**
   - [`generate_entrypoint_script()`](src/docker/skills.rs:338) creates a shell script
   - Script structure:
     ```sh
     #!/bin/sh
     # POSIX shell for maximum compatibility across container environments
     set -e
     # Error propagation - immediately exit on any command failure

     # Error handler - captures exit code and logs which command failed
     handle_error() {
         local exit_code=$1
         if [ $exit_code -ne 0 ]; then
             echo "[SKILL INSTALL ERROR] Command failed with exit code $exit_code"
         fi
     }
     trap 'handle_error $?' EXIT

     # Install skills
     # Skills are installed sequentially in declaration order to satisfy dependencies
     npx skills add owner/repo1 -a kilo -y 2>&1 | while IFS= read -r line; do
       echo "[SKILL INSTALL STDERR] $line"
     done
     npx skills add owner/repo2@skill-name -a kilo -y 2>&1 | while IFS= read -r line; do
       echo "[SKILL INSTALL STDERR] $line"
     done

     # Hand off to Kilo Code CLI
     # Process replacement - replaces shell with kilocode, ensuring proper signal handling and exit code propagation
     exec kilocode --yes "$@"
     ```

4. **Empty skills handling:**
   - If skills list is empty, returns empty string (no script generated)
   - Container uses default entrypoint from Dockerfile

### Phase 3: Container Creation (Docker Side)

1. **Docker container configuration:**
   - [`run_agent()`](src/docker/run/run.rs:639) creates container configuration
   - When skills are present, custom entrypoint script is set:
     ```rust
     container_config.entrypoint = Some(vec![
         "/bin/sh".to_string(),
         "-c".to_string(),
         entrypoint_script,
     ]);
     ```
   - When no skills or empty skills, `entrypoint` remains `None` (uses Dockerfile default)

2. **Skills directory mounting:**
   - [`build_host_config()`](src/docker/run/run.rs:209) checks for `.kilocode/skills/` directory
   - If exists, mounts it read-only into container at `/workspace/.kilocode/skills`
   - This allows manually managed skills to be accessible inside container

3. **Container creation and start:**
   - Container is created with the generated entrypoint script
   - Container is started using Docker API

### Phase 4: Skill Installation (Container Side)

1. **Script execution at container startup:**
   - Container starts and runs the entrypoint script via `/bin/sh -c`
   - Skills are installed sequentially in declaration order
   - `set -e` ensures immediate exit on any command failure

2. **npx skills add execution:**
   - Each skill is installed using: `npx skills add <source> -a kilo -y`
   - `-a kilo`: Author/app identifier for the skills registry
   - `-y`: Auto-confirm, skip interactive prompts
   - stderr is captured and prefixed with `[SKILL INSTALL STDERR]` for log identification

3. **Preexisting skill handling:**
   - Skills already in `.kilocode/skills/` skip npx installation
   - Logged as: `[SKILL INSTALL] Using preexisting skill: <name> (skipping npx installation)`

4. **Handoff to Kilo Code CLI:**
   - After all skills are installed, script executes: `exec kilocode --yes "$@"`
   - `exec` replaces the shell process with kilocode (proper signal handling)

### Phase 5: Agent Execution (Container Side)

1. **Kilo Code CLI runs with original arguments:**
   - Agent prompt is passed as a positional argument
   - All original CLI arguments are forwarded
   - Agent executes with installed skills available in its environment

---

## 3. What Happens During Installation?

### Docker Container Creation

When an agent has a non-empty `skills` field:

1. **Custom entrypoint script is generated** on the host
2. **Entrypoint is overridden** in Docker configuration to `/bin/sh -c <script>`
3. **Container is created** with the custom entrypoint
4. **Container starts** and executes the entrypoint script

### npx Execution Inside Container

During container startup:

1. **Node.js/npx must be available** inside the container (satisfied by `node:22-slim` base image)
2. **`npx skills add` is invoked** for each skill sequentially
3. **Skills are downloaded** from skills.sh marketplace or GitHub
4. **Skills are installed** into `.kilocode/skills/` inside the container
5. **If any installation fails**, `set -e` causes immediate script exit with non-zero code

### Container Exit Codes

| Exit Code | Meaning | Action |
|------------|---------|--------|
| `0` | All skills installed successfully, agent executed | Success |
| `1-127` | Skill installation failed | Container exits, failure logged, metrics recorded |
| `137` | Container killed (SIGKILL) | Timeout or OOM, check metrics for partial installation |
| `139` | Container stopped (SIGTERM) | Graceful termination, check for partial installation |

### Metrics Tracking

[`AgentExecutionResult`](src/docker/run/run.rs:87) includes skill installation tracking:

```rust
pub struct AgentExecutionResult {
    pub container_id: String,
    pub exit_code: i64,
    pub skills_installed: Option<bool>,   // true=success, false=failed, None=no skills
    pub skills_install_failed: bool,      // true if any skill failed
}
```

Metrics recorded include:
- `skills_installed_count`: Number of skills successfully installed
- `skills_failed_count`: Number of skills that failed
- `skills_install_time_seconds`: Time spent on installation
- `runs_with_skill_failures`: Number of runs where at least one skill failed

---

## 4. Key Behaviors Users Need to Understand

### Per-Agent Skill Scoping

1. **Each agent has independent skills:**
   - Skills are declared per-agent, not globally
   - Different agents can have different skill sets
   - Skills must be explicitly declared for each agent

2. **No implicit inheritance:**
   - If `skills` field is omitted, agent receives no skills
   - Empty `skills = []` also means no skills
   - Skills are not shared between agents

### Installation Timing

1. **Installation happens at container startup:**
   - Skills are installed each time a container starts
   - Skills are NOT persisted between container runs
   - Each run installs fresh from source
   - First run with skills may take longer (downloading)

2. **Sequential installation:**
   - Skills are installed one at a time in declaration order
   - This allows for dependency ordering between skills
   - Failed skill stops entire installation

3. **Timeout considerations:**
   - Skill installation time counts toward agent timeout budget
   - Users should account for skill install time in timeout values
   - 15-second target for single skill installation

### Preexisting Skills Handling

1. **Manually managed skills are detected:**
   - If `.kilocode/skills/<name>/SKILL.md` exists locally
   - npx installation is skipped for that skill
   - Logged as preexisting skill

2. **Read-only mount:**
   - `.kilocode/skills/` is mounted read-only into container
   - Container cannot modify manually managed skills
   - Ensures manual skills remain unchanged

### Backwards Compatibility

1. **Optional feature:**
   - `skills` field is completely optional
   - Existing configs without skills continue to work
   - No warnings or errors for missing skills field

2. **Mixed environments:**
   - Some agents can have skills, some without
   - Each container configured independently
   - Default entrypoint used when no skills specified

---

## 5. Error Conditions and Edge Cases

### Invalid Skill Format

**Error:** [`SkillsError::InvalidSkillFormat`](src/skills/error.rs:398)

**Conditions:**
- Missing or multiple `/` separators
- Empty owner name
- Empty repo name (before or after `@`)
- Empty skill name (after `@`)
- Invalid characters (only alphanumeric, hyphen, underscore allowed)

**Example:**
```
Error: Invalid skill format 'owner.name/repo': Owner name contains invalid characters '.'. Only alphanumeric, hyphen, and underscore are allowed.
```

### Empty Skills Field

**Warning:** Empty skills list warning

**Condition:** `skills = []` in agent configuration

**Behavior:**
- Config validates successfully
- Warning issued by `switchboard validate`
- Container uses default entrypoint (same as no skills)

**Example:**
```
Warning: Agent 'my-agent' has empty skills field. Either remove the 'skills' field or add skills.
```

### Duplicate Skill Entries

**Error:** Duplicate skill in agent's skills list

**Condition:** Same skill source appears multiple times

**Behavior:**
- Validation error reported by `switchboard validate`
- Prevents redundant installation attempts

**Example:**
```
Error: Duplicate skill 'owner/repo' in agent 'my-agent'. Skills list contains this skill 2 times.
```

### Skill Installation Failure

**Error:** [`SkillsError::ContainerInstallFailed`](src/skills/error.rs:164)

**Conditions:**
- Network unavailable during `npx skills add`
- Invalid skill repository or package name
- Permission issues accessing skills directories
- npx skills CLI internal errors

**Behavior:**
- `set -e` causes script to exit with non-zero code
- Container exits with failure status
- Failure logged with `[SKILL INSTALL ERROR]` prefix
- Metrics recorded: `skills_install_failed = true`, `skills_failed_count = total`

**Example:**
```
[SKILL INSTALL] Installing skill: owner/repo
[SKILL INSTALL STDERR] Error: Unable to resolve package name
[SKILL INSTALL ERROR] Command failed with exit code 1
```

### Network Failure During Install

**Error:** Network unavailable for npx skills

**Conditions:**
- DNS resolution failed for skills.sh
- GitHub/GitLab unreachable
- Timeout during download

**Behavior:**
- npx skills reports error
- Script exits with non-zero code
- Clear error message in logs
- Metrics record failure

**Example:**
```
Error: npx skills command 'add' failed with exit code 1: fetch failed https://github.com/owner/repo.git
```

### Container Timeout

**Error:** Agent timed out during skill installation

**Conditions:**
- Agent timeout value too small for skill installation
- Slow network causing install to exceed timeout

**Behavior:**
- Container sent SIGTERM, then SIGKILL if not graceful
- Exit code 137 (SIGKILL) or 139 (SIGTERM)
- Unknown skill installation status (metrics: `skills_installed = None`)

### Preexisting Skills Directory Not Found

**Non-fatal:** `.kilocode/skills/` directory missing

**Behavior:**
- No error - treated as no skills installed
- Empty vector returned for preexisting skills check
- All configured skills installed via npx

### npx Not Available Inside Container

**Error:** npx not found in container

**Conditions:**
- Container image doesn't include Node.js/npx
- Image not built from `node:22-slim` base

**Behavior:**
- Entrypoint script fails at first npx invocation
- Container exits with error
- User error message needed

---

## 6. Dependencies

### Required Inside Container

| Dependency | Version/Type | Purpose |
|------------|---------------|---------|
| **Node.js** | v22+ | Provides `npx` command for skill installation |
| **npx** | Included with Node.js | Package runner that invokes skills CLI |
| **skills CLI** | Via npx | Actual tool that downloads and installs skills |
| **POSIX sh** | `/bin/sh` | Shell interpreter for entrypoint script |

**Satisfied by existing Docker image:**
- Base image: `node:22-slim`
- Includes: Node.js v22, npx, /bin/sh

### Required on Host (for CLI commands only)

| Dependency | Purpose | Commands Requiring |
|------------|---------|-------------------|
| **Node.js** | Provides `npx` | `switchboard skills list`, `install`, `update` |
| **npx** | Package runner | Same as above |
| **skills CLI** | Skill discovery | Same as above |

**Note:** Host-side Node.js is NOT required for container skill installation. Only for CLI commands that manually invoke npx.

### Required for Container Skill Installation

| Component | Requirement | Where Satisfied |
|-----------|-------------|------------------|
| Docker daemon | Running and accessible | System level |
| Docker image | `node:22-slim` or equivalent | [`Dockerfile`](Dockerfile:1) |
| Workspace mount | `/workspace` mounted | Container configuration |
| Skills directory mount | Optional: `.kilocode/skills/` | Auto-detected if exists |

### Optional Dependencies

| Dependency | Purpose | Behavior if Missing |
|------------|---------|-------------------|
| **`.kilocode/skills/`** | Manual skill management | All skills installed via npx |
| **Global skills** | Skills across projects | Project-level only if missing |
| **Network access** | Download skills | Local-only installation not supported |

---

## 7. Key Implementation Files

### Container Skills Module

- **[`src/docker/skills.rs`](src/docker/skills.rs:1)** (1567 lines)
  - [`validate_skill_format()`](src/docker/skills.rs:123): Validates skill source format
  - [`extract_skill_name()`](src/docker/skills.rs:103): Parses skill name from source
  - [`generate_entrypoint_script()`](src/docker/skills.rs:338): Generates shell script
  - Comprehensive tests (98.89% line coverage)

### Container Execution

- **[`src/docker/run/run.rs`](src/docker/run/run.rs:1)** (4644 lines)
  - [`find_preexisting_skills()`](src/docker/run/run.rs:405): Detects manual skills
  - [`build_host_config()`](src/docker/run/run.rs:209): Sets up skills mount
  - [`build_container_config()`](src/docker/run/run.rs:328): Creates Docker config
  - [`run_agent()`](src/docker/run/run.rs:639): Orchestrates container execution

### Skills Error Types

- **[`src/skills/error.rs`](src/skills/error.rs:1)** (528 lines)
  - [`SkillsError::ContainerInstallFailed`](src/skills/error.rs:164): Installation failure
  - [`SkillsError::ScriptGenerationFailed`](src/skills/error.rs:188): Script errors
  - [`SkillsError::InvalidSkillFormat`](src/skills/error.rs:398): Format validation
  - [`SkillsError::InvalidSkillDirectory`](src/skills/error.rs:295): Directory errors

### Validation

- **[`src/commands/validate.rs`](src/commands/validate.rs:1)** (1000+ lines)
  - [`validate_agent_skills()`](src/commands/validate.rs:211): Unified skill validation
  - [`validate_agent_skills_format()`](src/commands/validate.rs:108): Format validation
  - [`validate_agent_skills_duplicates()`](src/commands/validate.rs:299): Duplicate detection

### Metrics

- **[`src/commands/metrics.rs`](src/commands/metrics.rs:1)** (1100+ lines)
  - Formats skill installation status as "installed/failed" in table output
  - Shows `Total Skills Installed`, `Total Skills Failed`, `Avg Skill Install Time`

---

## 8. Documentation References

### Primary Documentation

- **[`addtl-features/skills-feature.md`](addtl-features/skills-feature.md:1)**: Complete feature specification (422 lines)
- **[`BACKWARDS_COMPATIBILITY_SKILLS.md`](BACKWARDS_COMPATIBILITY_SKILLS.md:1)**: Backwards compatibility guide
- **[`README.md`](README.md:1)**: User-facing documentation

### Performance Documentation

- **[`docs/PERFORMANCE_SKILLS_INSTALL.md`](docs/PERFORMANCE_SKILLS_INSTALL.md:1)**: Performance expectations (15s target)
- **[`docs/PERFORMANCE_SKILLS_LIST.md`](docs/PERFORMANCE_SKILLS_LIST.md:1)**: List command performance (3s target)

### Network Failure Documentation

- **[`docs/NETWORK_FAILURE_HANDLING.md`](docs/NETWORK_FAILURE_HANDLING.md:1)**: Network failure scenarios

### Test References

- **[`tests/skills_install_performance.rs`](tests/skills_install_performance.rs:1)**: Performance tests
- **[`tests/skills_network_failure.rs`](tests/skills_network_failure.rs:1)**: Network failure tests

---

## 9. Acceptance Criteria Verification

| Criteria | Status | Evidence |
|-----------|--------|----------|
| Report explains what container skills are | ✅ Complete | Section 1 provides comprehensive definition and purpose |
| Report describes installation process step-by-step | ✅ Complete | Section 2 details all 5 phases with code examples |
| Report identifies key behaviors and dependencies | ✅ Complete | Sections 4 and 5 cover behaviors, timing, dependencies |
| Report notes error conditions or edge cases | ✅ Complete | Section 6 lists all error scenarios with examples |

---

## 10. Recommendations for Documentation

When creating documentation for Task 8, consider including:

1. **Conceptual overview:** What skills are and why they matter
2. **Configuration guide:** How to declare skills in `switchboard.toml`
3. **Installation flow:** Visual diagram of the 5-phase process
4. **Behavior examples:** Show success and failure scenarios
5. **Troubleshooting section:** Common errors and solutions
6. **Performance considerations:** Timeout budgeting, first-run latency
7. **Prerequisites check:** How to verify npx/Node.js in container
8. **Backwards compatibility:** Emphasis that skills are optional
9. **Metrics interpretation:** How to read skill installation status
10. **Code examples:** Working `switchboard.toml` configurations

---

## 11. Open Questions (from skills-feature.md)

1. **Skill install latency and agent timeouts:** Should Switchboard automatically subtract estimated skill install time from timeout, or should users account manually?
2. **Skill version pinning:** Should skills support pinning to specific git SHA or tag for reproducibility?
3. **Skill caching across runs:** Should Switchboard optionally cache installed skills in Docker volume to reduce startup time?
4. **npx skills version pinning:** Should Switchboard target a specific version of npx skills to avoid breaking changes?
5. **Skill install failure policy:** Should a failed skill install always abort, or support `skills_optional = true` flag?

These are acknowledged as desirable but deferred to future iterations (see [skills-feature.md](addtl-features/skills-feature.md:380)).

---

## 12. Conclusion

This report provides comprehensive information about container skill installation behavior in switchboard. The feature is:
- **Well-designed**: Clear separation of concerns, modular implementation
- **Well-tested**: 98.89% line coverage in docker/skills module
- **Production-ready**: All acceptance criteria met, comprehensive error handling
- **Backwards compatible**: No breaking changes to existing projects
- **Extensible**: Foundation for future enhancements (caching, version pinning, optional skills)

The documentation should focus on:
1. Clear explanations of what container skills are
2. Step-by-step installation process with examples
3. Key behaviors users need to understand (timing, dependencies)
4. Common error conditions and how to resolve them
5. Practical configuration examples

---

**Report prepared by:** Code Mode Subagent
**Date:** 2026-02-20T16:23:45Z
**Purpose:** Context gathering for Worker Agent 1, Task 8 (Container Skill Installation Behavior Documentation)
