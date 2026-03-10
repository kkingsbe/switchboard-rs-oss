# Story 007-02: Gateway Down CLI

> Epic: epic-07 — Discord Gateway - CLI Integration & Monitoring
> Points: 2
> Sprint: 12
> Type: feature
> Risk: low
> Created: 2026-03-04
> Status: ready
> Assigned To: dev-2

## User Story

As a user, I want to stop the gateway, so that I can shut down the service.

## Acceptance Criteria

1. Gateway stops gracefully
   - **Test:** Clean shutdown with no errors - Verify gateway shuts down without errors

2. Connected projects notified of shutdown
   - **Test:** Projects receive shutdown message - Verify projects receive notification

3. Discord connection closed properly
   - **Test:** No stale connections - Verify Discord connections are properly closed

## Technical Context

### Architecture Reference
From architecture.md:
- Gateway module: `src/gateway/` — handles project connections, routing, WebSocket
- Protocol: `src/gateway/protocol.rs` - GatewayMessage enum for communication
- CLI: `src/cli/commands/gateway.rs` - gateway up/down commands

### Project Conventions
From project-context.md:
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Use `thiserror` for error types
- Use `tracing` for logging
- Never use `unwrap()` in production
- Use tokio for async

### Existing Code Context
See current implementations in:
- `src/gateway/protocol.rs` - existing message types
- `src/gateway/server.rs` - existing server setup
- `src/gateway/connections.rs` - connection management
- `src/cli/commands/gateway.rs` - existing gateway up command

## Implementation Plan

1. **Examine** existing `src/cli/commands/gateway.rs` - Understand current gateway up command
2. **Modify** `src/cli/commands/gateway.rs` — Add `gateway down` command implementation
3. **Modify** `src/gateway/server.rs` — Add graceful shutdown handler with project notification
4. **Ensure** Discord connection closure — Verify Discord WebSocket and API clients close properly
5. **Write** tests — Add integration tests for graceful shutdown
6. **Run** build + tests — Verify compilation and tests pass

### Skills to Read

- `./skills/rust-engineer/SKILL.md` — Core Rust patterns
- `./skills/rust-best-practices/SKILL.md` — Best practices

### Dependencies

Stories that must be complete:
- story-004-08 (gateway up CLI) — Must be complete first
- story-006-01 (project connections) — Must be complete first
- story-006-02 (heartbeat) — Should be complete first
- story-007-01 (gateway status) — Should be complete first

## Scope Boundaries

### This Story Includes
- CLI command to stop gateway
- Graceful shutdown with timeout
- Notification to connected projects
- Proper cleanup of all resources

### This Story Does NOT Include
- Starting the gateway (story-004-08)
- Gateway status monitoring (story-007-01)
- PID file management (story-007-03)
- Gateway logging (story-007-04)

### Files in Scope

- `src/gateway/protocol.rs` — modify (add shutdown message if needed)
- `src/gateway/server.rs` — modify
- `src/gateway/connections.rs` — modify (if needed)
- `src/cli/commands/gateway.rs` — modify

### Files NOT in Scope

- `src/discord/` — do not modify (but may need to call existing close methods)
- `src/docker/` — do not modify
