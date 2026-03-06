# Story 007-02: Gateway Down CLI

> Epic: epic-07 — CLI Integration
> Points: 2
> Sprint: 10
> Type: feature
> Risk: low
> Created: 2026-03-04T01:20:05Z

## User Story

As a user, I want to stop the gateway, so that I can shut down the service.

## Acceptance Criteria

1. Gateway stops gracefully (Verification: Clean shutdown with no errors)
2. Connected projects notified of shutdown (Verification: Projects receive shutdown message)
3. Discord connection closed properly (Verification: No stale connections)

## Technical Context

### Architecture Reference
- Gateway module: `src/gateway/` — handles project connections, routing, WebSocket
- Protocol: `src/gateway/protocol.rs` - GatewayMessage enum for communication
- CLI: `src/cli/commands/gateway.rs` - gateway up/down commands

### Project Conventions
- Use `thiserror` for error types
- Never use `unwrap()` in production
- Use tracing for logging (never println!)
- Use tokio for async
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`

### Existing Code Context
See current implementations in:
- `src/gateway/protocol.rs` - existing message types
- `src/gateway/server.rs` - existing server setup
- `src/gateway/connections.rs` - connection management
- `src/cli/commands/gateway.rs` - existing gateway up command

## Implementation Plan

1. Modify `src/cli/commands/gateway.rs` — Add `gateway down` command implementation
2. Modify `src/gateway/server.rs` — Add graceful shutdown handler with project notification
3. Ensure Discord connection closure — Verify Discord WebSocket and API clients close properly
4. Write tests — Add integration tests for graceful shutdown
5. Run build + tests — Verify compilation and tests pass

### Skills to Read
- `skills/rust-engineer/SKILL.md` — Core Rust patterns
- `skills/rust-best-practices/SKILL.md` — Best practices from Apollo

### Dependencies
Stories that must be complete:
- story-005-01 (complete), story-005-02 (complete), story-005-03 (complete)
- story-006-01 (complete), story-006-02 (complete - heartbeat), story-006-04 (complete - disconnections)
- story-004-08 (complete - gateway up CLI)

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

### Files in Scope
- `src/gateway/protocol.rs` — modify (add shutdown message if needed)
- `src/gateway/server.rs` — modify
- `src/gateway/connections.rs` — modify (if needed)
- `src/cli/commands/gateway.rs` — modify

### Files NOT in Scope
- `src/discord/` — do not modify (but may need to call existing close methods)
- `src/docker/` — do not modify
