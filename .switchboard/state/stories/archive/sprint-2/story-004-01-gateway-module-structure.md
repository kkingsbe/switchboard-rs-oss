# Story 4.1: Create gateway module structure

> Epic: epic-04 — Discord Gateway Phase 1: Basic Gateway
> Points: 1
> Sprint: 2
> Type: infrastructure
> Risk: low
> Created: 2026-03-02

## User Story

**As a** developer,
**I want** a basic gateway module structure,
**So that** I can organize the gateway code.

## Acceptance Criteria

1. Create `src/gateway/mod.rs` with module declarations (Gateway)
   - **Test:** File exists and compiles with `cargo build --features gateway`

2. Add `pub mod gateway` to `src/lib.rs`
   - **Test:** `cargo build --features gateway` succeeds

3. Add feature flag `gateway` to Cargo.toml
   - **Test:** `cargo build --features gateway` compiles without errors

## Technical Context

### Architecture Reference

From architecture.md:
- NEW module structure: `src/gateway/` with submodules: config.rs, server.rs, protocol.rs, routing.rs, registry.rs, connections.rs, heartbeat.rs, ratelimit.rs, client.rs
- New dependencies: tokio-tungstenite 0.24, futures-util 0.3, axum 0.7, tower 0.4
- Phase 1 scope: HTTP server, WebSocket server, channel registry, project registration, CLI commands

### Project Conventions

From project-context.md:
- **Build:** `cargo build --features "discord gateway"`
- **Test:** `cargo test --lib`
- **Lint:** `cargo clippy -- -D warnings`
- **Error Handling:** Use thiserror - never anyhow in library code
- **No unwrap()** in production - use `?` or `.expect()`
- **Async:** tokio with patterns from `src/discord/gateway.rs`
- **Logging:** Use tracing - never println!
- **Tests:** Inline in module files, naming: `test_name_should_do_something()`

### Existing Code Context

Current `src/lib.rs` module declarations:
```
pub mod architect;
pub mod cli;
pub mod commands;
pub mod config;
pub mod discord;
pub mod docker;
pub mod logger;
pub mod logging;
pub mod metrics;
pub mod scheduler;
pub mod skills;
pub mod traits;
pub mod ui;
```

Pattern to follow from `src/discord/mod.rs`:
```rust
pub mod api;
pub mod config;
pub mod conversation;
pub mod gateway;
pub mod listener;
pub mod llm;
pub mod outbox;
pub mod security;
pub mod tools;
```

## Implementation Plan

1. Create directory `src/gateway/`
2. Create `src/gateway/mod.rs` with:
   - Module declarations for future submodules
   - Basic Gateway struct
   - Module-level documentation
3. Modify `src/lib.rs` to add: `pub mod gateway;`
4. Modify `Cargo.toml` to add:
   ```toml
   [features]
   default = ["discord"]
   discord = []
   gateway = []
   ```
5. Run `cargo build --features gateway` to verify
6. Run `cargo test --lib` to verify tests pass
7. Run `cargo clippy -- -D warnings` for lint check

### Skills to Read

- `./skills/rust-best-practices/SKILL.md` — Rust module organization and conventions
- `./skills/rust-engineer/SKILL.md` — Async Rust patterns with tokio
- `./skills/rust-engineer/references/testing.md` — Testing standards

### Dependencies

None. This story has no dependencies and can start immediately.

## Scope Boundaries

### This Story Includes
- Creating the `src/gateway/` directory
- Creating `src/gateway/mod.rs` with basic module structure
- Adding the module to `src/lib.rs`
- Adding the `gateway` feature flag to `Cargo.toml`

### This Story Does NOT Include
- Implementing any gateway functionality (config, HTTP server, WebSocket, etc.)
- Adding any dependencies to Cargo.toml beyond the feature flag
- Creating CLI commands
- Any Discord-specific functionality

### Files in Scope
- `src/gateway/mod.rs` — create
- `src/lib.rs` — modify
- `Cargo.toml` — modify

### Files NOT in Scope
- `src/gateway/config.rs` — not yet (story 4.2)
- `src/gateway/server.rs` — not yet (story 4.3)
- `src/gateway/protocol.rs` — not yet (story 4.5)
- Any CLI commands
