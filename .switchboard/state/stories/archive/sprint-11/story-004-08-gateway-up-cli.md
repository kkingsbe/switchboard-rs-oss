# Story 004-08: CLI `gateway up` Command

> Epic: epic-04 — Discord Gateway Phase 1
> Points: 3
> Sprint: 10
> Type: feature
> Risk: low
> Created: 2026-03-03
> Status: in-progress
> Assigned To: dev-1

## User Story

As a user, I want to start the gateway from the CLI, So that I can easily run the gateway service.

## Acceptance Criteria

1. CLI has `gateway` subcommand with `up` action - `switchboard gateway up --help` shows usage
   - **Test:** Run `cargo run --features "discord gateway" -- --help` and verify `gateway` subcommand appears. Run `cargo run --features "discord gateway" -- gateway up --help` and verify usage is displayed.

2. Command starts gateway with config from `gateway.toml` - Gateway starts and connects to Discord
   - **Test:** Run gateway with valid config and verify it connects to Discord (check logs for "Connected to Discord")

3. Support `--config` flag for custom config path - Custom config file loads correctly
   - **Test:** Run with `--config custom.toml` and verify it uses that config file

4. Support `--detach` flag to run in background (future) - Not required for MVP
   - **Test:** N/A - defer to future sprint

## Technical Context

### Architecture Reference

From architecture.md:
- The gateway uses twilight-gateway for Discord WebSocket connections
- Configuration loaded from gateway.toml
- Gateway server runs on configurable port (default 9745)
- HTTP + WebSocket for project communication

### Project Conventions

From project-context.md:
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Use `thiserror` for error types
- Use `tracing` for logging
- Never use `unwrap()` in production

### Existing Code Context

Current CLI structure in `src/cli/mod.rs`:
- Has subcommands: skills, up, list, build, logs, validate, metrics, gateway
- Uses clap for CLI parsing

```
src/cli/
├── mod.rs (main CLI setup with subcommands)
└── commands/
    ├── mod.rs
    ├── up.rs (existing 'up' command for docker)
    ├── gateway.rs (EXISTS - needs completion)
    ├── list.rs
    └── skills.rs
```

**Existing gateway.rs content:**
```rust
// Current file exists at src/cli/commands/gateway.rs
// Contains partial implementation of gateway subcommand
// Needs: up subcommand implementation
```

**Existing gateway server:**
```rust
// src/gateway/server.rs exists
// GatewayServer::run().await - main entry point
// Loads config from GatewayConfig
```

## Implementation Plan

1. **Examine** `src/cli/commands/gateway.rs` - Understand what's already implemented
2. **Complete** the `up` subcommand:
   - Parse `--config` flag
   - Load gateway config from file or default path
   - Call `GatewayServer::new(config)` and `run().await`
   - Handle graceful shutdown on SIGINT
3. **Register** the gateway subcommand in `src/cli/mod.rs` if not already done
4. **Run** `cargo build --features "discord gateway"` and verify compilation
5. **Run** `cargo test --lib` and verify all tests pass

### Skills to Read
- `./skills/rust-engineer/SKILL.md` — Core Rust patterns
- `./skills/rust-engineer/references/async.md` — Async/await with tokio
- `./skills/rust-engineer/references/error-handling.md` — Error handling patterns with thiserror

### Dependencies
- story-004-07 (Discord Gateway) — Must be complete first (server.rs exists)
- story-007-03 (PID file management) — Should be complete (pid.rs exists)

## Scope Boundaries

### This Story Includes
- `switchboard gateway up` command implementation
- `--config` flag support
- Basic gateway startup with config loading

### This Story Does NOT Include
- `--detach` flag (defer to future)
- Gateway process management (PID file already done in story-007-03)
- Gateway status command (story-007-01)
- Gateway down command (story-007-02)

### Files in Scope
- `src/cli/commands/gateway.rs` — modify (complete implementation)
- `src/cli/mod.rs` — modify (register subcommand if needed)
- `gateway.toml` — read (config file)
- `src/gateway/server.rs` — use (don't modify)

### Files NOT in Scope
- `src/gateway/server.rs` — don't modify (use existing API)
- `src/discord/gateway.rs` — don't modify
- `src/gateway/pid.rs` — already implemented
