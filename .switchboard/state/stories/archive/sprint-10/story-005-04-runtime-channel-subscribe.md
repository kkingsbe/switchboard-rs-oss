# Story 005-04: Runtime Channel Subscribe

> Epic: epic-05 — Channel Routing
> Points: 2
> Sprint: 10
> Type: feature
> Risk: low
> Created: 2026-03-04T01:20:05Z

## User Story

As a project developer, I want to change my channel subscriptions at runtime, so that I can add/remove channels without restarting the gateway.

## Acceptance Criteria

1. Project can send `channel_subscribe` message (Verification: New channels added)
2. Project can send `channel_unsubscribe` message (Verification: Channels removed)
3. Changes take effect immediately (Verification: Next message uses updated subscription)

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

1. Modify `src/gateway/protocol.rs` — Add `channel_subscribe` and `channel_unsubscribe` message types to GatewayMessage enum
2. Modify `src/gateway/server.rs` — Add handler for subscribe/unsubscribe messages to update project subscriptions in runtime
3. Write tests — Add unit tests for subscribe/unsubscribe functionality
4. Run build + tests — Verify compilation and tests pass

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
- Adding runtime channel subscribe/unsubscribe message types
- Implementing handler to update project subscriptions
- Ensuring changes take effect immediately for message routing

### This Story Does NOT Include
- Persisting subscriptions to disk
- Project reconnection logic (story-006-04)
- CLI commands for managing subscriptions

### Files in Scope
- `src/gateway/protocol.rs` — modify
- `src/gateway/server.rs` — modify
- `src/gateway/connections.rs` — modify (if applicable)

### Files NOT in Scope
- `src/discord/` — do not modify
- `src/docker/` — do not modify
