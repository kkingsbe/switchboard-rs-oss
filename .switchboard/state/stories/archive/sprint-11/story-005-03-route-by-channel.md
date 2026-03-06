# Story 005-03: Route Messages by Channel

> Epic: epic-05 — Discord Gateway Phase 2
> Points: 3
> Sprint: 10
> Type: feature
> Risk: medium
> Created: 2026-03-03
> Status: in-progress
> Assigned To: dev-2

## User Story

As a user, I want messages from Discord to be routed to the correct project based on channel, So that each project only receives messages from its subscribed channels.

## Acceptance Criteria

1. When MessageCreate event arrives, extract channel_id
   - **Test:** Add logging to verify channel_id is extracted from events
   - **Test:** Run gateway with connected project, send message in Discord channel, verify channel_id logged

2. Look up projects subscribed to that channel
   - **Test:** Create test with mock project subscriptions and verify lookup works
   - **Test:** Register project with channel subscription, verify registry returns correct projects

3. Forward message to those projects via WebSocket
   - **Test:** Verify messages are forwarded to correct project WebSocket clients
   - **Test:** Two projects subscribed to same channel, verify both receive message

## Technical Context

### Architecture Reference

From architecture.md:
- Gateway server maintains project registry (which projects are connected)
- Each project can subscribe to channel IDs
- Messages forwarded via WebSocket to subscribed projects
- ChannelRegistry maps channel_id → projects
- MessageRouter routes Discord events to appropriate projects

Module specifications:
- `gateway::registry`: Track channel-to-project mappings
  - `ChannelRegistry::register(project, channels)`
  - `ChannelRegistry::projects_for_channel(channel_id) -> &[ProjectId]`
- `gateway::routing`: Route Discord events to projects
  - `MessageRouter::route(event: DiscordEvent) -> Vec<ProjectMessage>`

### Project Conventions

From project-context.md:
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`
- Lint: `cargo clippy -- -D warnings`
- Use `tracing` for logging
- Never use `unwrap()` in production
- Follow async patterns in `src/discord/gateway.rs`

### Existing Code Context

```
src/gateway/
├── mod.rs          # Module exports
├── config.rs       # Gateway configuration (EXISTS)
├── protocol.rs     # Message protocol types (EXISTS)
├── registry.rs     # Channel/project registry (EXISTS)
├── server.rs       # Main server with WebSocket handling (EXISTS)
├── routing.rs      # Channel-based routing (EXISTS - needs completion)
├── connections.rs  # Project connection management (EXISTS)
├── heartbeat.rs    # Heartbeat protocol (may exist)
└── ratelimit.rs    # Discord rate limiting (EXISTS)
```

**Existing routing.rs:** Contains partial implementation - needs message routing logic integrated with Discord events.

**Existing registry.rs:** Contains ChannelRegistry struct - tracks project subscriptions.

**Existing server.rs:** Handles WebSocket connections - needs to call routing logic on MessageCreate events.

## Implementation Plan

1. **Examine** `src/gateway/routing.rs` - Understand current state
2. **Examine** `src/gateway/registry.rs` - Understand subscription tracking
3. **Integrate** routing with Discord events in `src/gateway/server.rs`:
   - On MessageCreate event, extract channel_id
   - Call `registry.projects_for_channel(channel_id)`
   - Forward message to each connected project's WebSocket
4. **Handle** fan-out: If multiple projects subscribed, send to all
5. **Write tests** for routing logic in `src/gateway/routing.rs`
6. **Run** `cargo build --features "discord gateway"` and verify compilation
7. **Run** `cargo test --lib` and verify all tests pass

### Skills to Read
- `./skills/rust-engineer/SKILL.md` — Core Rust patterns
- `./skills/rust-engineer/references/async.md` — Broadcast channels for fan-out
- `./skills/rust-best-practices/SKILL.md` — Testing patterns

### Dependencies
- story-005-01 (ChannelRegistry) — complete (registry.rs exists)
- story-005-02 (Project subscriptions in config) — complete (config.rs supports channels)
- story-004-07 (Discord Gateway) — complete (server connects to Discord)

## Scope Boundaries

### This Story Includes
- Channel ID extraction from MessageCreate events
- Project-to-channel subscription mapping lookup
- Message forwarding to subscribed projects via WebSocket
- Fan-out to multiple projects (same channel)

### This Story Does NOT Include
- Rate limiting (story-006-06)
- Message filtering/transformation based on content
- Bulk message handling
- Runtime subscribe/unsubscribe (story-005-04)

### Files in Scope
- `src/gateway/routing.rs` — modify (complete routing logic)
- `src/gateway/server.rs` — modify (integrate routing with Discord events)
- `src/gateway/registry.rs` — use (don't modify, use existing API)

### Files NOT in Scope
- `src/discord/listener.rs` — don't modify
- `src/cli/` — don't modify
- `src/gateway/config.rs` — already done
