# Story 5.4: Support Runtime Channel Subscribe/Unsubscribe

> Epic: Epic 05 — Discord Gateway - Channel Routing with Config File
> Points: 2
> Sprint: 7
> Type: feature
> Risk: Low
> Created: 2026-03-03
> Status: not-started

## User Story

As a project developer,
I want to change my channel subscriptions at runtime,
So that I can add/remove channels without restarting the gateway.

## Acceptance Criteria

1. Project can send `channel_subscribe` message
   - **Test:** New channels added to subscription
   - Verify: Channels are added to project's subscription list in registry

2. Project can send `channel_unsubscribe` message
   - **Test:** Channels removed from subscription
   - Verify: Channels are removed from project's subscription list in registry

3. Changes take effect immediately
   - **Test:** Next message uses updated subscription
   - Verify: Messages are routed to/from newly subscribed channels

## Technical Context

### Architecture Reference

From planning/epics/epic-05-discord-gateway-phase2.md:

- **§5.4 Runtime subscribe/unsubscribe:**
  - Allow projects to modify channel subscriptions after initial registration
  - Files to modify: `src/gateway/protocol.rs`, `src/gateway/server.rs`

### Project Conventions

From project-context.md:
- **Build:** `cargo build --features "discord gateway"`
- **Async:** Use tokio for async. Follow patterns in `src/discord/gateway.rs`
- **Error Handling:** Use `thiserror` for error types. Never use `unwrap()` in production
- **Logging:** Use `tracing` for logging

### Existing Code Context

**ChannelRegistry in registry.rs** - Already exists with subscription methods:
```rust
// src/gateway/registry.rs lines 218-289

/// Add a channel subscription for a project
pub async fn add_channel_subscription(
    &self,
    project_id: &ProjectId,
    channel_id: &str,
) -> RegistryResult<()> {
    // Implementation exists
}

/// Remove a channel subscription for a project
pub async fn remove_channel_subscription(
    &self,
    project_id: &ProjectId,
    channel_id: &str,
) -> RegistryResult<()> {
    // Implementation exists
}
```

**GatewayMessage in protocol.rs** - Current message types:
```rust
// src/gateway/protocol.rs lines 14-69

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayMessage {
    Register { project_name: String, channels: Vec<String> },
    RegisterAck { status: String, session_id: String },
    RegisterError { error: String },
    Message { payload: String, channel_id: u64 },
    Heartbeat { timestamp: u64 },
    HeartbeatAck { timestamp: u64 },
}
```

**Missing:** `channel_subscribe` and `channel_unsubscribe` message variants

### Key Existing Code Patterns

- Protocol uses externally-tagged JSON format: `{"type": "register", "project_name": "...", "channels": [...]}`
- Registry methods return `RegistryResult<T>` with `RegistryError` enum
- Use `tracing::info!`, `tracing::debug!` for logging

## Implementation Plan

1. **Add message types to `src/gateway/protocol.rs`**
   - Add `ChannelSubscribe { channels: Vec<String> }` variant
   - Add `ChannelUnsubscribe { channels: Vec<String> }` variant
   - Add `ChannelSubscribeAck { status: String }` variant
   - Add `ChannelUnsubscribeAck { status: String }` variant
   - Ensure proper serialization/deserialization

2. **Add handlers in `src/gateway/server.rs`**
   - Parse incoming `channel_subscribe` messages
   - Call `registry.add_channel_subscription()` for each channel
   - Send `ChannelSubscribeAck` response
   - Parse incoming `channel_unsubscribe` messages
   - Call `registry.remove_channel_subscription()` for each channel
   - Send `ChannelUnsubscribeAck` response
   - Handle errors gracefully (project not found, etc.)

3. **Write tests** — Unit tests for:
   - ChannelSubscribe/Unsubscribe message serialization/deserialization
   - Handler logic for adding/removing subscriptions
   - Error handling for unregistered projects

4. **Run build + tests** — Verify everything compiles

### Skills to Read

- [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md) — Rust best practices
- [`skills/rust-engineer/SKILL.md`](skills/rust-engineer/SKILL.md) — Async patterns with tokio

### Dependencies

- Story 5.1 (ChannelRegistry) — Complete ✓
  - Provides: `add_channel_subscription()` and `remove_channel_subscription()` methods

## Scope Boundaries

### This Story Includes
- New protocol message types for subscribe/unsubscribe
- WebSocket message handlers in server.rs
- Runtime updates to channel subscriptions
- Immediate effect on message routing

### This Story Does NOT Include
- Discord gateway integration (Story 4.7)
- Message routing by channel (Story 5.3)
- Authentication or authorization
- Persistence of subscriptions (in-memory only)

### Files in Scope
- `src/gateway/protocol.rs` — modify (add subscribe/unsubscribe message types)
- `src/gateway/server.rs` — modify (add message handlers)

### Files NOT in Scope
- `src/gateway/registry.rs` — Already exists with subscription methods
- `src/gateway/config.rs` — Already exists
- `src/discord/gateway.rs` — Different module
