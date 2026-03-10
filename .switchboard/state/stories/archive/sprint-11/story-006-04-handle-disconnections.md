# Story 006-04: Handle Disconnections

> Epic: epic-06 — Multi-Project
> Points: 2
> Sprint: 10
> Type: feature
> Risk: low
> Created: 2026-03-04T01:20:05Z

## User Story

As a system, I want to handle project WebSocket disconnections cleanly, so that the gateway doesn't crash or leak resources.

## Acceptance Criteria

1. Detect WebSocket close events (Verification: Disconnection logged)
2. Remove project from routing (Verification: Messages not sent to disconnected project)
3. Allow project to re-register (Verification: Same project can reconnect)

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

## Implementation Plan

1. Modify `src/gateway/server.rs` — Add WebSocket close event handler to detect disconnections
2. Modify `src/gateway/connections.rs` — Add method to remove project from routing table on disconnect
3. Ensure registration allows reconnection — Modify registration logic to handle reconnecting projects
4. Write tests — Add unit tests for disconnection handling and reconnection
5. Run build + tests — Verify compilation and tests pass

### Skills to Read
- `skills/rust-engineer/SKILL.md` — Core Rust patterns
- `skills/rust-best-practices/SKILL.md` — Best practices from Apollo

### Dependencies
Stories that must be complete:
- story-005-01 (complete), story-005-02 (complete), story-005-03 (complete)
- story-006-01 (complete), story-006-02 (complete - heartbeat protocol for timeout detection)
- story-004-08 (in-progress - for story-007-02)

## Scope Boundaries

### This Story Includes
- WebSocket close event detection
- Project removal from routing
- Clean resource cleanup
- Support for project re-registration

### This Story Does NOT Include
- Automatic reconnection from client side
- Reconnection retry logic in gateway
- Persistence of project state across disconnections

### Files in Scope
- `src/gateway/protocol.rs` — modify (if needed)
- `src/gateway/server.rs` — modify
- `src/gateway/connections.rs` — modify

### Files NOT in Scope
- `src/discord/` — do not modify
- `src/docker/` — do not modify
