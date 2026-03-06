# Story 5.4: Support Runtime Channel Subscribe/Unsubscribe

> Epic: Epic 05 — Discord Gateway Phase 2
> Points: 2
> Sprint: 20
> Type: feature
> Risk: Low
> Created: 2026-03-05

## User Story

**As a** project developer,
**I want** to change my channel subscriptions at runtime,
**So that** I can add/remove channels without restarting the gateway.

## Acceptance Criteria

1. Project can send `channel_subscribe` message
   - **Test:** New channels added to subscription. Send subscribe message via WebSocket and verify registry updated.

2. Project can send `channel_unsubscribe` message
   - **Test:** Channels removed from subscription. Send unsubscribe message and verify removal.

3. Changes take effect immediately
   - **Test:** Next message uses updated subscription. Subscribe, then send message to new channel - should be received.

## Technical Context

### Architecture Reference

From `architecture.md` §5.3 (gateway::registry):
- **Purpose:** Track channel-to-project mappings
- **Public API:**
  - `ChannelRegistry::register(project: ProjectConnection, channels: Vec<String>)`
  - `ChannelRegistry::unregister(project_id: &ProjectId)`
  - `ChannelRegistry::projects_for_channel(channel_id: &str) -> &[ProjectId]`

The registry already has register/unregister. Need to add methods for adding/removing channels to existing registrations.

### Project Conventions

From `project-context.md`:
- **Build:** `cargo build --features "discord gateway"`
- **Protocol:** JSON messages via WebSocket
- **Async:** Use tokio for async operations

### Existing Code Context

**Current protocol.rs** already has GatewayMessage enum. Need to add:
- `ChannelSubscribe { channels: Vec<String> }`
- `ChannelUnsubscribe { channels: Vec<String> }`

**Current registry.rs** - already has registration functions. Need to add:
- `add_channels(project_id, channels)` 
- `remove_channels(project_id, channels)`

**Files to modify:**
- `src/gateway/protocol.rs` — Add new message types
- `src/gateway/registry.rs` — Add subscribe/unsubscribe methods
- `src/gateway/server.rs` — Handle new message types

### Files in Scope

- `src/gateway/protocol.rs` — MODIFY: Add subscribe/unsubscribe message types
- `src/gateway/registry.rs` — MODIFY: Add runtime subscription methods
- `src/gateway/server.rs` — MODIFY: Handle new message types

### Files NOT in Scope

- `src/gateway/config.rs` — Already done (story 5.2)
- `src/gateway/routing.rs` — Not yet (story 5.3)
- `src/cli/commands/gateway.rs` — Not yet

## Implementation Plan

1. **Add message types to protocol.rs** — Add ChannelSubscribe and ChannelUnsubscribe variants to GatewayMessage enum
   - Implement Serialize/Deserialize

2. **Add registry methods** — Add `subscribe_channels()` and `unsubscribe_channels()` to ChannelRegistry
   - Update existing subscriptions without removing project

3. **Handle messages in server.rs** — Add handlers for the new message types
   - Parse message → update registry → send ack

4. **Run build + tests** — Verify everything compiles

### Skills to Read

- `./skills/rust-engineer/SKILL.md` — For async patterns
- `./skills/rust-best-practices/SKILL.md` — For Rust idioms

### Dependencies

- Story 5.1: ChannelRegistry — COMPLETE

## Scope Boundaries

### This Story Includes
- Protocol message types for subscribe/unsubscribe
- Registry updates for runtime channel changes
- WebSocket message handling

### This Story Does NOT Include
- Config-based channel mapping (story 5.2)
- Message routing (story 5.3)
- CLI integration

## Verification

```bash
# Build verification
cargo build --features "discord gateway"

# Test verification
cargo test --lib gateway::registry

# Test subscribe/unsubscribe manually:
# 1. Connect project via WebSocket
# 2. Send {"type": "channel_subscribe", "channels": ["123456789"]}
# 3. Verify ack received
# 4. Send {"type": "channel_unsubscribe", "channels": ["123456789"]}
# 5. Verify ack received and channels removed
```
