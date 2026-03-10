# Story 004-08: CLI `gateway up` Command

> Epic: epic-04 — Discord Gateway Phase 1
> Points: 3
> Sprint: 8
> Type: feature
> Risk: low
> Created: 2026-03-03

## User Story

As a user, I want to start the gateway from the CLI, So that I can easily run the gateway service.

## Acceptance Criteria

1. CLI has `gateway` subcommand with `up` action - `switchboard gateway up --help` shows usage
   - **Test:** Run `cargo run -- --help` and verify `gateway` subcommand appears. Run `cargo run -- gateway up --help` and verify usage is displayed.

2. Command starts gateway with config from `gateway.toml` - Gateway starts and connects to Discord
   - **Test:** Run gateway with valid config and verify it connects to Discord (check logs for "Connected to Discord")

3. Support `--config` flag for custom config path - Custom config file loads correctly
   - **Test:** Run with `--config custom.toml` and verify it uses that config file

4. Support `--detach` flag to run in background (future) - Not required for MVP
   - **Test:** N/A - defer to future sprint

## Technical Context

### Architecture Reference
(Extract from architecture.md - Discord gateway module)
- The gateway uses twilight-gateway for Discord WebSocket connections
- Configuration loaded from gateway.toml
- Gateway server runs on configurable port

### Project Conventions
(From project-context.md)
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`

### Existing Code Context
Current CLI structure in `src/cli/mod.rs`:
- Has subcommands: skills, up, list, build, logs, validate, metrics
- Uses clap for CLI parsing
- See `src/cli/commands/up.rs` for example of how commands are implemented

```
src/cli/
├── mod.rs (main CLI setup with subcommands)
└── commands/
    ├── mod.rs
    ├── up.rs (existing 'up' command for docker)
    ├── list.rs
    └── skills.rs
```

## Implementation Plan

1. **Create** `src/cli/commands/gateway.rs` - New CLI module for gateway commands
2. **Modify** `src/cli/mod.rs` - Add `gateway` subcommand group
3. **Create** gateway implementation that:
   - Loads config from gateway.toml (or custom path)
   - Starts the gateway server
   - Handles graceful shutdown
4. **Write tests** in a new test module
5. **Run** `cargo build --features "discord gateway"` and verify compilation
6. **Run** `cargo test --lib` and verify all tests pass

### Skills to Read
- `skills/rust-engineer/SKILL.md` — Core Rust patterns
- `skills/rust-engineer/references/async.md` — Async/await with tokio
- `skills/rust-engineer/references/error-handling.md` — Error handling patterns

### Dependencies
- story-004-07 (Discord Gateway) — Must be complete first

## Scope Boundaries

### This Story Includes
- `switchboard gateway up` command implementation
- `--config` flag support
- Basic gateway startup

### This Story Does NOT Include
- `--detach` flag (defer to future)
- Gateway process management (PID file)
- Advanced configuration options

### Files in Scope
- `src/cli/commands/gateway.rs` — create
- `src/cli/mod.rs` — modify (add subcommand)
- `gateway.toml` — read (config file)

### Files NOT in Scope
- `src/gateway/server.rs` — don't modify (use existing)
- `src/discord/gateway.rs` — don't modify
