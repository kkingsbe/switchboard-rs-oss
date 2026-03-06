# Story 006-01: Project Connection Management

> Epic: Epic 06 — Multi-Project Gateway
> Points: 3
> Sprint: 9
> Type: feature
> Risk: medium
> Created: 2026-03-03

## User Story

As a system, I want to track all connected projects, So that I can manage their state and route messages correctly.

## Acceptance Criteria

1. Track active connections with project_id, session_id, subscription info
   - **Test:** Connection list accurate - verify all registered projects appear in the connection list

2. Handle multiple simultaneous project connections
   - **Test:** Can connect 3+ projects - spawn 3+ concurrent connections and verify all are tracked

3. Detect and clean up stale connections
   - **Test:** Dead connections removed after timeout - simulate disconnection and verify cleanup

## Technical Context

### Architecture Reference
- Connection management will use `tokio::sync::mpsc` for async message passing
- Use `std::collections::HashMap` for O(1) connection lookups
- Connection state includes: project_id, session_id, subscription info, last heartbeat

### Project Conventions
- Build: `cargo build --features "discord gateway"`
- Test: `src/gateway/` for test examples

### Existing Code Context
```
src/gateway/
├── mod.rs
├── config.rs
├── protocol.rs
├── ratelimit.rs
├── registry.rs (project registration - already tracks connections)
└── routing.rs (message routing)
```

The existing [`ChannelRegistry`](src/gateway/registry.rs:87) in `registry.rs` already provides:
- Project registration/unregistration
- Channel subscriptions
- Session ID per connection

This story extends that functionality with explicit connection lifecycle management including heartbeat tracking and stale connection detection.

## Implementation Plan

1. **Create** `src/gateway/connections.rs` with:
   - `Connection` struct - tracks project_id, session_id, subscriptions, last_heartbeat
   - `ConnectionManager` - manages HashMap of active connections
   - `StaleConnectionDetector` - background task to detect and clean dead connections

2. **Modify** `src/gateway/mod.rs` - add `connections` module export

3. **Write tests** for:
   - Connection tracking and retrieval
   - Multiple simultaneous connections
   - Stale connection detection and cleanup

4. **Run** `cargo build --features "discord gateway"` and tests

### Skills to Read
- `skills/rust-best-practices/SKILL.md` — async patterns, error handling
- `skills/rust-engineer/SKILL.md` — concurrent data structures, tokio patterns

### Dependencies
- story-004-06 (Registration Protocol) — assumed complete
- story-005-03 (Route by Channel) — already supports fan-out

## Scope Boundaries

### This Story Includes
- Connection struct with project_id, session_id, subscriptions, heartbeat
- ConnectionManager with HashMap storage
- Stale connection detection with configurable timeout
- Cleanup of dead connections

### This Story Does NOT Include
- WebSocket connection handling (in server.rs)
- Discord event processing
- Rate limiting (story-006-06)

### Files in Scope
- `src/gateway/connections.rs` — create
- `src/gateway/mod.rs` — modify (add module)

### Files NOT in Scope
- `src/gateway/server.rs` — don't modify
- `src/discord/` — don't modify
- `src/cli/` — don't modify
