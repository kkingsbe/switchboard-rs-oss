# Switchboard Workspace Profile

## Project Type

**Existing** - This is an established Rust project (v0.1.0) with extensive production-ready codebase for scheduling and running AI coding agents.

---

## Tech Stack

### Languages
- **Rust** (edition 2021) - Primary language for the entire codebase

### Build Tools
- **Cargo** - Rust package manager and build tool
- **cargo-llvm-cov** - Code coverage instrumentation

### Key Frameworks & Dependencies
| Category | Frameworks/Libraries |
|----------|---------------------|
| Async Runtime | `tokio` (1.40) with full features |
| HTTP/WebSocket | `axum` (0.7), `tower`, `tower-http` |
| Container Runtime | `bollard` (0.18) - Docker API client |
| Scheduling | `tokio-cron-scheduler` (0.15), `cron` (0.12) |
| Discord Integration | `twilight-gateway`, `twilight-http`, `twilight-model` (0.17) |
| Serialization | `serde`, `serde_json`, `toml`, `serde_yaml` |
| Observability | `tracing`, `tracing-subscriber`, `tracing-appender` |
| API Documentation | `utoipa`, `utoipa-swagger-ui` |
| Error Handling | `anyhow`, `thiserror`, `async-trait` |

---

## Project Structure

| Directory | Purpose |
|-----------|---------|
| [`src/main.rs`](src/main.rs) | Application entry point |
| [`src/lib.rs`](src/lib.rs) | Library root with public exports |
| [`src/cli/`](src/cli/) | CLI framework and command processing |
| [`src/api/`](src/api/) | REST API handlers, router, state management |
| [`src/scheduler/`](src/scheduler/) | Cron-based scheduling engine |
| [`src/docker/`](src/docker/) | Docker container lifecycle management |
| [`src/discord/`](src/discord/) | Discord bot and Gateway client |
| [`src/gateway/`](src/gateway/) | Discord Gateway WebSocket server |
| [`src/skills/`](src/skills/) | Skills package management system |
| [`src/workflows/`](src/workflows/) | Workflow management and GitHub integration |
| [`src/metrics/`](src/metrics/) | Metrics collection and storage |
| [`src/logger/`](src/logger/) | File and terminal logging |
| [`src/config/`](src/config/) | Configuration loading and environment handling |
| [`src/commands/`](src/commands/) | User-facing CLI commands (build, list, logs, etc.) |
| [`src/architect/`](src/architect/) | Git executor for code analysis |
| [`docs/`](docs/) | Documentation files |
| [`examples/`](examples/) | Example configurations and prompts |
| [`scripts/`](scripts/) | Build and test automation scripts |
| [`.github/workflows/`](.github/workflows/) | CI/CD pipelines |

---

## Existing Capabilities

Switchboard is a CLI tool for managing KiloCode agents and Docker containers with the following capabilities:

1. **Cron-based Agent Scheduling** - Schedule AI agents using cron expressions with timezone support
2. **Docker Container Isolation** - Run each agent in isolated Docker containers with configurable timeouts
3. **Overlap Handling** - Skip or queue concurrent runs based on configuration
4. **Discord Bot Integration** - Conversational AI bot for Discord servers with LLM-powered responses
5. **Skills System** - Reusable, versioned agent configurations stored in `skills/` directory
6. **REST API Server** - HTTP API for monitoring and control (axum-based)
7. **Process Management** - Detached mode, PID tracking, status checks
8. **Logging** - Structured logging with file rotation
9. **Metrics Collection** - Execution metrics tracking
10. **Project Scaffolding** - Initialize new Switchboard projects and workflows

---

## Available Tooling

### Test Runners
- **`cargo test`** - Standard Rust test runner
- **Built-in test modules** - Integration tests in `src/api/tests/`, `src/api/handlers/*_tests.rs`
- **`serial_test`** - For tests requiring sequential execution

### Coverage Tools
- **`cargo-llvm-cov`** - LLVM-based code coverage (`[dev-dependencies]` in Cargo.toml)
- **Scripts**:
  - [`scripts/generate-coverage.sh`](scripts/generate-coverage.sh) - Generate coverage reports
  - [`scripts/check-coverage-ratchet.sh`](scripts/check-coverage-ratchet.sh) - Enforce minimum coverage thresholds
  - [`scripts/check-new-code-coverage.sh`](scripts/check-new-code-coverage.sh) - Check new code coverage

### Linters & Formatters
- **`cargo clippy`** - Rust linter (recommended for Rust projects)
- **`cargo fmt`** - Rust code formatter

### CI/CD
- **[`.github/workflows/ci.yml`](.github/workflows/ci.yml)** - Build and test pipeline
- **[`.github/workflows/release.yml`](.github/workflows/release.yml)** - Release automation

### Platform Testing
- **[`scripts/test-platform-compatibility.sh`](scripts/test-platform-compatibility.sh)** - Cross-platform compatibility testing

---

## Relevant Context for Goals

### Switchboard Observability (`goals.md`)

This workspace is the **target implementation** for the Switchboard Observability feature defined in [`goals.md`](goals.md). The goals.md document specifies:

1. **Event Log Architecture** - JSONL-based event logging at `.switchboard/events/events.jsonl`
2. **Event Types** - `scheduler.started`, `scheduler.stopped`, `container.started`, `container.exited`, `container.skipped`, `container.queued`, `git.diff`
3. **Log Rotation** - 10MB threshold with 30-day retention
4. **Derived Metrics** - Throughput, velocity, and reliability metrics computed from events
5. **Configuration** - `[settings.observability]` TOML section

### Implementation Alignment

The observability implementation requires:
- Adding [`src/logger/`](src/logger/) extensions for structured event emission
- Modifying [`src/scheduler/`](src/scheduler/) to emit lifecycle events
- Enhancing [`src/docker/`](src/docker/) to emit container events
- Adding git diff capture in [`src/architect/git_executor.rs`](src/architect/git_executor.rs)
- Creating event log rotation logic

### Workspace State

The `.switchboard/state/` directory already contains:
- Sprint tracking and project management files
- Development TODO lists
- Review queues and blockers tracking
- Knowledge base journals

This workspace profile should be updated as the observability feature is implemented.
