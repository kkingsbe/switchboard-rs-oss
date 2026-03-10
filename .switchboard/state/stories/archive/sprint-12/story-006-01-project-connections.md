# Story 006-01: Project Connections Management

> Epic: epic-06 — Discord Gateway - Multi-Project Support & Reconnection
> Points: 3
> Sprint: 12
> Type: feature
> Risk: medium
> Created: 2026-03-04
> Status: ready
> Assigned To: dev-2

## User Story

As a system, I want to track all connected projects, So that I can manage their state and route messages correctly.

## Acceptance Criteria

1. Track active connections with project_id, session_id, subscription info
   - **Test:** Connection list accurate - Verify connection metadata is correctly stored and retrievable

2. Handle multiple simultaneous project connections (3+ projects)
   - **Test:** Can connect 3+ projects - Create 3+ mock projects and verify all can connect simultaneously

3. Detect and clean up stale connections
   - **Test:** Dead connections removed after timeout - Verify connections timeout and are cleaned up

## Technical Context

### Architecture Reference

From architecture.md:
- Gateway maintains WebSocket connections to multiple projects
- Each project has a session_id and subscription info (channels)
- Projects connect via HTTP/WebSocket to the gateway
- Connection state needs to be managed with cleanup for dead connections

### Project Conventions

From project-context.md:
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Use `thiserror` for error types
- Use `tracing` for logging
- Never use `unwrap()` in production
- Use tokio for async runtime

### Existing Code Context

```
src/gateway/
├── mod.rs (gateway module entry)
├── config.rs (gateway configuration)
├── protocol.rs (registration protocol - COMPLETE)
├── connections.rs (TO BE CREATED - this story)
├── pid.rs (PID file management - COMPLETE)
└── server.rs (gateway server - EXISTS)
```

**Dependencies:**
- `src/gateway/protocol.rs` — Already implements registration protocol (story-004-06)
- Gateway server at `src/gateway/server.rs` exists and provides the main entry point

## Implementation Plan

1. **Create** `src/gateway/connections.rs` with:
   - `Connection` struct with project_id, session_id, subscriptions, created_at, last_heartbeat
   - `ConnectionManager` struct with HashMap of connections
   - Methods: add_connection, remove_connection, get_connection, list_connections
   - Stale connection detection with configurable timeout
   - Cleanup task for dead connections

2. **Integrate** with existing gateway:
   - Update `src/gateway/mod.rs` to export connections module
   - Modify `src/gateway/server.rs` to use ConnectionManager

3. **Test** the implementation:
   - Write unit tests for ConnectionManager
   - Test multiple simultaneous connections
   - Test stale connection cleanup

### Skills to Read

- `./skills/rust-engineer/SKILL.md` — Core Rust patterns
- `./skills/rust-engineer/references/async.md` — Async/await with tokio
- `./skills/rust-engineer/references/ownership.md` — Rust ownership patterns

### Dependencies

- story-004-06 (registration protocol) — Must be complete first

## Scope Boundaries

### This Story Includes
- Connection struct with all metadata fields
- ConnectionManager with thread-safe storage
- Stale connection detection and cleanup
- Unit tests for connection management

### This Story Does NOT Include
- Message routing between projects
- Project authentication beyond registration
- Reconnection logic (story-006-02)
- Disconnection handling (story-006-04)

### Files in Scope

- `src/gateway/connections.rs` — create (new file)
- `src/gateway/mod.rs` — modify (add exports)
- `src/gateway/server.rs` — modify (integrate connection manager)

### Files NOT in Scope

- `src/discord/gateway.rs` — don't modify
- `src/discord/listener.rs` — don't modify
- `src/gateway/protocol.rs` — don't modify (already complete)
- `src/gateway/pid.rs` — don't modify (already complete)
