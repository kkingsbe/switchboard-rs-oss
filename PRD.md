# Switchboard — Product Requirements Document

**Version:** 0.1.0
**Author:** Kyle
**Date:** February 11, 2026
**Status:** Draft

---

## 1. Overview

Switchboard is a Rust-based CLI tool that acts as a cron scheduler for AI coding agents. It reads a configuration file from a user's project repository, then orchestrates the execution of scheduled prompts ("Agents") by running each one inside its own isolated Docker container via the [Kilo Code CLI](https://www.npmjs.com/package/@kilocode/cli).

The project's workspace repository is bind-mounted into each container, allowing agents to read and write files in the user's actual project directory — just as if a developer were running Kilo Code locally.

---

## 2. Goals

- Provide a single CLI command that spins up all configured agents on their defined schedules.
- Isolate each agent in its own Docker container for safety and reproducibility.
- Mount the user's project workspace into each container so agents can operate on real files.
- Keep setup minimal: clone, `cargo install`, write a config, run.

---

## 3. User Flow

```
1. Clone the Switchboard repo
2. cargo install --path .          # installs `switchboard` binary globally
3. cd ~/my-project
4. Create a switchboard.toml config    # define agents, schedules, prompts
5. switchboard up                      # builds images, starts all scheduled agents
```

### 3.1 Detailed Steps

**Step 1 — Install.** The user clones the Switchboard repository and runs `cargo install --path .` (or `cargo install switchboard` once published to crates.io). This places the `switchboard` binary on their `$PATH`.

**Step 2 — Configure.** Inside any project repo, the user creates a `switchboard.toml` file (see §6) describing one or more agents — each with a name, a cron schedule, a prompt (inline or file path), and optional environment/config overrides.

**Step 3 — Run.** From the project repo root, the user runs `switchboard up`. Switchboard:

1. Reads `switchboard.toml`.
2. Builds (or reuses) the agent Docker image.
3. For each agent, schedules a container run according to its cron expression.
4. Each container bind-mounts the current working directory to `/workspace` inside the container.
5. The container executes the Kilo Code CLI with the agent's prompt, then exits.

---

## 4. Architecture

```
┌──────────────────────────────────────────────┐
│                  switchboard CLI                  │
│  ┌────────────┐  ┌────────────┐  ┌────────┐ │
│  │ Config      │  │ Scheduler  │  │ Docker │ │
│  │ Parser      │  │ (cron)     │  │ Client │ │
│  └────────────┘  └────────────┘  └────────┘ │
└──────────────────┬───────────────────────────┘
                   │ docker run -v $(pwd):/workspace
        ┌──────────┼──────────┐
        ▼          ▼          ▼
   ┌─────────┐ ┌─────────┐ ┌─────────┐
   │ Agent A │ │ Agent B │ │ Agent C │
   │container│ │container│ │container│
   └─────────┘ └─────────┘ └─────────┘
```

### 4.1 Core Components

| Component | Responsibility |
|---|---|
| **Config Parser** | Reads and validates `switchboard.toml`. Resolves prompt file paths relative to the project root. |
| **Scheduler** | Evaluates cron expressions and triggers container runs at the correct times. Runs as a foreground process (or optionally daemonized). |
| **Docker Client** | Builds the agent image (if not cached), creates and starts containers with the correct bind mounts, environment variables, and entrypoint arguments. |
| **Logger** | Aggregates stdout/stderr from all agent containers and writes to a unified log with agent-name prefixes. |

### 4.2 Docker Image

The agent container image is derived from the reference `Dockerfile.reference` and must preserve the following:

- **Base image:** `node:22-slim`
- **Kilo Code CLI:** `@kilocode/cli@0.26.0` installed globally via npm.
- **`.kilocode` directory:** Copied from the Switchboard repo into `/root/.kilocode` inside the image. This contains API keys, model configuration, and MCP server definitions needed by the CLI.
- **Rust toolchain:** Installed via rustup (needed by some Kilo Code extensions).
- **System dependencies:** `git`, `curl`, `build-essential`, `procps`, `file`, `sudo`.
- **Workspace mount point:** `/workspace` — the user's project repo is bind-mounted here at runtime.

The image build context is the Switchboard repo itself (so `.kilocode/` is available to `COPY`). The user's project repo is never baked into the image; it is only mounted at run time.

### 4.3 Container Execution Model

Each agent run is a single `docker run` invocation:

```bash
docker run --rm \
  -v /path/to/user/project:/workspace \
  -e AGENT_NAME=<name> \
  -e PROMPT="<prompt text or file>" \
  switchboard-agent:latest \
  kilo --prompt "<prompt>"
```

Key behaviors:

- The container starts, executes the Kilo Code CLI with the given prompt against `/workspace`, then exits.
- `--rm` ensures containers are cleaned up after each run.
- The workspace mount is read-write by default; a `readonly` flag can be set per-agent in the config.
- Multiple agents may run concurrently if their schedules overlap; each gets its own container.

---

## 5. CLI Interface

### 5.1 Commands

```
switchboard up [--config <path>] [--detach]
```
Build the agent image (if needed) and start the scheduler. Runs in the foreground by default; `--detach` backgrounds it.

```
switchboard run <agent-name> [--config <path>]
```
Immediately execute a single agent (ignoring its schedule). Useful for testing.

```
switchboard build [--config <path>] [--no-cache]
```
Build or rebuild the agent Docker image without starting the scheduler.

```
switchboard list [--config <path>]
```
Print all configured agents, their schedules, and their prompts.

```
switchboard logs [<agent-name>] [--follow] [--tail <n>]
```
View logs from agent runs. Optionally filter by agent name.

```
switchboard down
```
Stop the scheduler and any running agent containers.

```
switchboard validate [--config <path>]
```
Parse and validate the config file without running anything.

---

## 6. Configuration File

The configuration file is `switchboard.toml`, placed at the root of the user's project repository.

### 6.1 Schema

```toml
# switchboard.toml

[settings]
image_name = "switchboard-agent"        # Docker image name (default: "switchboard-agent")
image_tag = "latest"                # Docker image tag (default: "latest")
log_dir = ".switchboard/logs"           # Log output directory (default: ".switchboard/logs")
workspace_path = "."                # Path to mount into container (default: cwd)
timezone = "America/New_York"       # Timezone for cron evaluation (default: system tz)

[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"           # Every 6 hours
prompt = "Review all files changed in the last 6 hours. Flag bugs, security issues, and style violations."
readonly = false                    # Mount workspace read-write (default)
timeout = "30m"                     # Max runtime before the container is killed
env = { REVIEW_DEPTH = "thorough" } # Extra env vars passed to the container

[[agent]]
name = "doc-updater"
schedule = "0 2 * * 1"             # Mondays at 2 AM
prompt_file = "prompts/update-docs.md"  # Prompt loaded from a file relative to project root
timeout = "1h"

[[agent]]
name = "dependency-checker"
schedule = "0 9 * * *"             # Daily at 9 AM
prompt = "Check for outdated or vulnerable dependencies and open issues for any that need updating."
readonly = true                     # Read-only workspace mount
```

### 6.2 Field Reference

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `settings.image_name` | string | no | `"switchboard-agent"` | Docker image name |
| `settings.image_tag` | string | no | `"latest"` | Docker image tag |
| `settings.log_dir` | string | no | `".switchboard/logs"` | Directory for agent log files |
| `settings.workspace_path` | string | no | `"."` | Host path to mount into `/workspace` |
| `settings.timezone` | string | no | system | IANA timezone for cron evaluation |
| `agent[].name` | string | **yes** | — | Unique agent identifier |
| `agent[].schedule` | string | **yes** | — | Standard 5-field cron expression |
| `agent[].prompt` | string | one of prompt/prompt_file | — | Inline prompt text |
| `agent[].prompt_file` | string | one of prompt/prompt_file | — | Path to prompt file (relative to project root) |
| `agent[].readonly` | bool | no | `false` | Mount workspace as read-only |
| `agent[].timeout` | string | no | `"30m"` | Max container runtime (e.g. `"30m"`, `"2h"`) |
| `agent[].env` | table | no | `{}` | Additional environment variables |

---

## 7. Dockerfile

Switchboard ships a Dockerfile that is built once and reused across all agent runs. It is based on the existing reference Dockerfile with the following adaptations:

```dockerfile
FROM node:22-slim

WORKDIR /app

LABEL maintainer="Switchboard"
LABEL description="Switchboard agent container for scheduled Kilo Code CLI execution"

# System dependencies
RUN apt-get update && apt-get install -y \
    git curl build-essential procps file sudo \
    && rm -rf /var/lib/apt/lists/*

# Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Kilo Code CLI — pinned version
RUN npm i -g @kilocode/cli@0.26.0

# Kilo Code configuration (API keys, model config, MCP servers)
COPY .kilocode /root/.kilocode

# Workspace mount point
RUN mkdir -p /workspace
WORKDIR /workspace

# Default entrypoint runs the Kilo Code CLI
ENTRYPOINT ["kilo"]
```

**Key changes from the reference Dockerfile:**

- Removed the Node.js application layer (`package.json`, `lib/`, `run-*.js`) — Switchboard doesn't need the orchestration JS; it drives Kilo Code directly via the CLI.
- Removed `xvfb`, `libvulkan1`, `vulkan-tools` (not needed for headless CLI execution).
- Removed the non-root user setup — Kilo Code CLI expects `/root/.kilocode` and root-level access. Can be revisited in a security hardening pass.
- Changed `WORKDIR` to `/workspace` and set `ENTRYPOINT` to `kilo` so the CLI runs directly against the mounted workspace.
- Removed `HEALTHCHECK` — containers are ephemeral (run-and-exit), not long-lived.

**Preserved from reference:**

- `.kilocode` directory copied to `/root/.kilocode` ✓
- `@kilocode/cli@0.26.0` version pinned ✓
- `/workspace` as the mount target for the user's project ✓
- Rust toolchain installed ✓

---

## 8. Rust Crate Dependencies (Recommended)

| Crate | Purpose |
|---|---|
| `clap` | CLI argument parsing |
| `toml` / `serde` | Config file parsing and deserialization |
| `cron` or `tokio-cron-scheduler` | Cron expression parsing and job scheduling |
| `tokio` | Async runtime |
| `bollard` | Docker Engine API client (build, create, start, logs, remove) |
| `chrono` / `chrono-tz` | Timezone-aware scheduling |
| `tracing` / `tracing-subscriber` | Structured logging |
| `anyhow` / `thiserror` | Error handling |

---

## 9. Error Handling & Edge Cases

- **Docker not running:** `switchboard up` should check for Docker daemon availability and exit with a clear error message before attempting anything.
- **Missing `.kilocode` directory:** Fail at image build time with a message explaining the user needs to configure `.kilocode/` in the Switchboard repo with their API keys.
- **Overlapping runs:** If an agent is still running when its next scheduled execution fires, skip the new run and log a warning (configurable: skip or queue).
- **Container timeout:** Containers that exceed their `timeout` are forcibly killed. The exit is logged with the agent name and duration.
- **Invalid cron expression:** Caught at config validation time (`switchboard validate`), not at scheduler start.
- **Workspace path doesn't exist:** Fail with a clear error at startup.

---

## 10. Logging

Each agent run produces a log file at `<log_dir>/<agent-name>/<timestamp>.log` containing the full stdout/stderr from the container. The `switchboard up` process also writes a scheduler log to `<log_dir>/switchboard.log` with events like agent start, stop, timeout, and errors.

When running in the foreground, agent output is also interleaved on the terminal with `[agent-name]` prefixes, similar to `docker compose logs`.

---

## 11. Agent Metrics

### 11.1 Metrics to Track

Switchboard will track the following metrics for each agent:

| Metric | Description | Data Type |
|---|---|---|
| **Run Count** | Total number of times the agent has been executed | Integer |
| **Last Run Duration** | Duration of the most recent agent run | Duration (seconds) |
| **Average Run Duration** | Mean duration across all agent runs | Duration (seconds) |
| **First Run Timestamp** | Timestamp of the first recorded run | DateTime |
| **Last Run Timestamp** | Timestamp of the most recent run | DateTime |
| **Total Runtime** | Cumulative time spent executing this agent | Duration (seconds) |
| **Success Count** | Number of successful agent runs (exit code 0) | Integer |
| **Failure Count** | Number of failed agent runs (non-zero exit code) | Integer |

### 11.2 Data Collection

Metrics are automatically collected during each agent execution:

1. **Start Time:** Captured when the container is started
2. **End Time:** Captured when the container exits (whether normally or due to timeout)
3. **Exit Code:** Recorded to determine success/failure status
4. **Duration:** Calculated as End Time - Start Time

Metrics are persisted to a JSON file at `<log_dir>/metrics.json` with the following structure:

```json
{
  "agents": {
    "code-reviewer": {
      "run_count": 42,
      "success_count": 40,
      "failure_count": 2,
      "total_runtime_seconds": 3780.5,
      "first_run_timestamp": "2026-02-11T08:00:00Z",
      "last_run_timestamp": "2026-02-13T02:00:00Z",
      "last_run_duration_seconds": 85.3,
      "average_run_duration_seconds": 90.0
    },
    "doc-updater": {
      "run_count": 12,
      "success_count": 12,
      "failure_count": 0,
      "total_runtime_seconds": 245.0,
      "first_run_timestamp": "2026-02-11T02:00:00Z",
      "last_run_timestamp": "2026-02-13T02:00:00Z",
      "last_run_duration_seconds": 20.4,
      "average_run_duration_seconds": 20.4
    }
  }
}
```

### 11.3 CLI Query Interface

A new CLI command enables users to query and view agent metrics:

```
switchboard metrics [--config <path>] [--agent <agent-name>]
```

#### Usage Examples:

**View metrics for all agents:**
```bash
$ switchboard metrics
```

Output:
```
Agent            | Runs | Success | Fail | Avg Duration | Last Run | Status
-----------------|------|---------|------|--------------|----------|--------
code-reviewer    |   42 |      40 |    2 |        90.0s | 02:00    | ✓
doc-updater      |   12 |      12 |    0 |        20.4s | 02:00    | ✓
dependency-check |    5 |       5 |    0 |        45.2s | 09:00    | ✓
```

**View detailed metrics for a specific agent:**
```bash
$ switchboard metrics --agent code-reviewer
```

Output:
```
Agent: code-reviewer
Schedule: 0 */6 * * *
Total Runs: 42
Success Rate: 95.2% (40/42)
Total Runtime: 63m 0.5s
Average Duration: 90.0s
Last Run Duration: 85.3s
First Run: 2026-02-11 08:00:00 UTC
Last Run: 2026-02-13 02:00:00 UTC
```

**View metrics with custom config path:**
```bash
$ switchboard metrics --config /path/to/switchboard.toml
```

### 11.4 Metrics Storage and Integrity

- **Atomic Updates:** The metrics file is updated atomically to prevent corruption during concurrent writes.
- **Automatic Creation:** The metrics file and directory are created on first agent run if they don't exist.
- **Validation:** On startup, Switchboard validates the metrics file format and recreates it if corrupted.
- **Persistence:** Metrics survive scheduler restarts and system reboots.
- **No Cleanup:** Metrics are retained indefinitely to provide long-term historical data.

### 11.5 Error Handling

- **Missing Metrics File:** Treated as no historical data; new runs are tracked starting fresh.
- **Corrupted Metrics File:** Log a warning, back up the corrupted file, and start a new metrics file.
- **Concurrent Access:** Use file locking or atomic writes to prevent race conditions when multiple processes attempt to update metrics.

### 11.6 Architect Directive: Additional Metrics

The **architect agent** is tasked with determining **3 additional metrics** that will be crucial for improving the Switchboard system going forward. These metrics should:

1. **Fill gaps in observability** — Identify areas of system behavior that are not currently captured but are important for understanding performance, reliability, or user value.
2. **Enable data-driven improvements** — Provide actionable insights that can inform optimizations, feature development, or operational improvements.
3. **Align with product goals** — Support the core objectives defined in Section 2 (Goals) and the success criteria in Section 14 (Success Criteria).

**Requirements for the Architect Agent:**

- Analyze the current metrics (Section 11.1) and identify 3 meaningful additions that complement the existing suite.
- For each proposed metric, provide:
  - A clear name and description
  - The data type to be collected
  - The rationale for why this metric is valuable for system improvement
  - How the metric will be collected and stored (if it requires new mechanisms beyond the existing metrics infrastructure)
- Add the new metrics to the product specification by:
  - Updating Section 11.1 with the 3 new metrics in the metrics table
  - Ensuring Section 11.2 (Data Collection) describes how the new metrics are captured
  - Extending Section 11.3 (CLI Query Interface) if the new metrics should be displayed via `switchboard metrics`
  - Updating Section 11.2's JSON schema example to include the new fields
  - If new collection mechanisms are required, document them in the appropriate sections

The architect agent should prioritize metrics that are:
- Collectible within the existing system architecture
- Scalable as the number of agents and runs grows
- Meaningful to users operating and maintaining Switchboard deployments

---

## 12. Future Considerations

These are explicitly out of scope for v0.1 but worth noting:

- **Web UI / dashboard** for monitoring agent status and viewing logs.
- **Webhook triggers** in addition to cron schedules (e.g., on git push).
- **Agent chaining** — output of one agent feeds into the prompt of another.
- **Remote execution** — running agent containers on remote Docker hosts or Kubernetes.
- **Secret management** — integration with Vault, AWS Secrets Manager, etc. instead of baking `.kilocode` into the image.
- **Config hot-reload** — picking up `switchboard.toml` changes without restarting.
- **Notifications** — Slack/email/webhook alerts on agent success, failure, or timeout.
- **Per-agent Dockerfile overrides** — allowing agents to specify additional dependencies.

---

## 13. Code Coverage Requirements

### 13.1 Targets

| Metric | Minimum | Target | Enforcement |
|---|---|---|---|
| **Overall line coverage** | 70% | 85% | CI gate — builds fail below minimum |
| **Overall branch coverage** | 60% | 75% | CI gate — builds fail below minimum |
| **New code (per PR)** | 80% | 90% | PR check — merge blocked below minimum |

### 13.2 Per-Module Expectations

Not all code is equally testable. Coverage targets are tiered by module criticality and testability:

| Module | Min Coverage | Notes |
|---|---|---|
| **Config parser** | 90% | Pure logic, highly testable. Cover all valid/invalid TOML permutations, missing fields, type mismatches, and default resolution. |
| **Cron scheduler** | 80% | Test expression parsing, next-run calculation, timezone handling, and overlap/skip behavior. Time-dependent logic should use injectable clocks. |
| **Docker client** | 60% | Wraps `bollard` API calls. Unit test argument construction (mount paths, env vars, entrypoint args, image names). Integration tests require a running Docker daemon and are gated behind a `#[cfg(feature = "integration")]` flag. |
| **CLI layer** | 70% | Test argument parsing, config resolution, and command dispatch. Use `assert_cmd` for end-to-end CLI tests. |
| **Logger** | 70% | Test log file creation, rotation, agent-name prefixing, and interleaved terminal output. |

### 13.3 Tooling

- **Coverage engine:** [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) — source-based coverage via LLVM instrumentation. Preferred over `cargo-tarpaulin` for accuracy.
- **Report formats:** Generate both `lcov` (for CI upload) and `html` (for local inspection).
- **CI integration:** Coverage reports uploaded to Codecov or Coveralls on every push to `main` and on every PR.

```bash
# Run tests with coverage
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Generate HTML report for local review
cargo llvm-cov --all-features --workspace --html --output-dir coverage/
```

### 13.4 Testing Strategy

**Unit tests** live alongside source files in `#[cfg(test)]` modules and cover:

- Config deserialization (valid, invalid, partial, defaults)
- Cron expression parsing and next-fire-time calculation
- Docker run argument assembly (mount paths, env vars, entrypoint)
- Timeout duration parsing (`"30m"`, `"2h"`, invalid strings)
- Log path construction and file naming

**Integration tests** live in `tests/` and require Docker:

- Build the agent image from the shipped Dockerfile
- Run a trivial agent (`kilo --version`) with a mounted temp directory
- Verify container cleanup (`--rm` behavior)
- Verify read-only mount enforcement
- Verify container timeout and kill behavior

Integration tests are gated:

```bash
# Only run integration tests when Docker is available
cargo nextest run --features integration
```

**Coverage exclusions** (annotated with `#[cfg(not(tarpaulin_include))]` or `llvm_cov(off)`):

- `main()` function (thin wrapper over library entry point)
- Generated code (serde derives, clap derives)
- Panic/unreachable branches that exist only as defensive guards

### 13.5 Coverage Ratchet

Coverage must never decrease on `main`. The CI pipeline compares the incoming PR's coverage against the current `main` baseline. If overall coverage drops by more than 1%, the PR is flagged for review. This prevents gradual erosion as the codebase grows.

---

## 14. Success Criteria

Switchboard v0.1 is complete when:

1. A user can `cargo install` the CLI from the repo.
2. A `switchboard.toml` file with at least one agent can be validated and parsed.
3. `switchboard build` produces a working Docker image with Kilo Code CLI and `.kilocode` config.
4. `switchboard run <agent>` executes a single agent in a container with the workspace mounted and produces observable output.
5. `switchboard up` runs a scheduler that triggers agents on their cron schedules.
6. Agent logs are captured and viewable via `switchboard logs`.
7. `switchboard down` cleanly stops the scheduler and any running containers.