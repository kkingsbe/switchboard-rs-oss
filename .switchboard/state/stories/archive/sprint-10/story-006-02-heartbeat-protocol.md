# Story 006-02: Heartbeat Protocol

> Epic: epic-06 — Multi-Project
> Points: 2
> Sprint: 10
> Type: feature
> Risk: low
> Created: 2026-03-04T01:20:05Z

## User Story

As a system, I want to detect when projects disconnect unexpectedly, so that I can stop routing messages to them.

## Acceptance Criteria

1. Projects send heartbeat every 30 seconds (Verification: Heartbeat received)
2. Gateway responds with `heartbeat_ack` (Verification: Ack sent with timestamp)
3. Mark project disconnected if no heartbeat for 90 seconds (Verification: Project removed)

## Technical Context

### Architecture Reference
- Gateway module: `src/gateway/` - handles project connections, routing, WebSocket
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

1. Create `src/gateway/heartbeat.rs` — New module for heartbeat tracking with timer-based disconnection detection
2. Modify `src/gateway/protocol.rs` — Add `heartbeat` and `heartbeat_ack` message types to GatewayMessage enum
3. Modify `src/gateway/server.rs` — Integrate heartbeat handling into project connection lifecycle
4. Write tests — Add unit tests for heartbeat timeout and acknowledgment
5. Run build + tests — Verify compilation and tests pass

### Skills to Read
- `skills/rust-engineer/SKILL.md` — Core Rust patterns
- `skills/rust-best-practices/SKILL.md` — Best practices from Apollo

### Dependencies
Stories that must be complete:
- story-005-01 (complete), story-005-02 (complete), story-005-03 (complete)
- story-006-01 (complete)
- story-004-08 (in-progress - for story-007-02)

## Scope Boundaries

### This Story Includes
- Heartbeat message types (send/receive)
- 30-second heartbeat interval from projects
- 90-second timeout detection
- Gateway acknowledgment response

### This Story Does NOT Include
- Handling disconnections (story-006-04)
- Reconnection logic
- Project re-registration

### Files in Scope
- `src/gateway/protocol.rs` — modify
- `src/gateway/server.rs` — modify
- `src/gateway/heartbeat.rs` — create
- `src/gateway/connections.rs` — modify (if applicable)

### Files NOT in Scope
- `src/discord/` — do not modify
- `src/docker/` — do not modify
