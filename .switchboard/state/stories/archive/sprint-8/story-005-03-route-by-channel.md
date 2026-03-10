# Story 005-03: Route Messages by Channel

> Epic: epic-05 — Discord Gateway Phase 2
> Points: 3
> Sprint: 8
> Type: feature
> Risk: medium
> Created: 2026-03-03

## User Story

As a user, I want messages from Discord to be routed to the correct project based on channel, So that each project only receives messages from its subscribed channels.

## Acceptance Criteria

1. When MessageCreate event arrives, extract channel_id
   - **Test:** Add logging to verify channel_id is extracted from events

2. Look up projects subscribed to that channel
   - **Test:** Create test with mock project subscriptions and verify lookup works

3. Forward message to those projects via WebSocket
   - **Test:** Verify messages are forwarded to correct project WebSocket clients

## Technical Context

### Architecture Reference
- Gateway server maintains project registry (which projects are connected)
- Each project can subscribe to channel IDs
- Messages forwarded via WebSocket to subscribed projects

### Project Conventions
- Build: `cargo build --features "discord gateway"`
- Test: `src/discord/tests/` for test examples

### Existing Code Context
```
src/gateway/
├── mod.rs
├── config.rs
├── protocol.rs
├── registry.rs (project registration)
└── server.rs (main server)
```

Current server.rs handles WebSocket connections. Need to add message routing.

## Implementation Plan

1. **Create** `src/gateway/routing.rs` - Channel-based routing logic
2. **Modify** `src/gateway/server.rs` - Add message routing on MessageCreate
3. **Modify** `src/gateway/registry.rs` - Add channel subscription tracking per project
4. **Write tests** for routing logic
5. **Run** build and tests

### Skills to Read
- `skills/rust-engineer/SKILL.md`
- `skills/rust-engineer/references/async.md` — broadcast channels
- `skills/DISCLI.md` — outbox pattern for message handling

### Dependencies
- story-005-01 (ChannelRegistry) — complete
- story-005-02 (Project subscriptions) — complete  
- story-004-07 (Discord Gateway) — complete

## Scope Boundaries

### This Story Includes
- Channel ID extraction from MessageCreate events
- Project-to-channel subscription mapping
- Message forwarding logic

### This Story Does NOT Include
- Rate limiting (story-006-06)
- Message filtering/transformation
- Bulk message handling

### Files in Scope
- `src/gateway/routing.rs` — create
- `src/gateway/server.rs` — modify
- `src/gateway/registry.rs` — modify

### Files NOT in Scope
- `src/discord/listener.rs` — don't modify
- `src/cli/` — don't modify
