# Story 4.7: Wire up Discord Gateway Connection

> Epic: Epic 04 — Discord Gateway - Phase 1: Basic Gateway
> Points: 5
> Sprint: 7
> Type: feature
> Risk: HIGH
> Created: 2026-03-03
> Status: not-started

## User Story

As a user,
I want the gateway to connect to Discord,
So that it can receive messages from my channels.

## Acceptance Criteria

1. Gateway connects to Discord using twilight-gateway
   - **Test:** Connection established, Ready event received
   - Verification: Check logs for "Connected to Discord" message

2. Gateway listens for MessageCreate events
   - **Test:** Messages logged/forwarded
   - Verification: Incoming Discord messages are processed

3. Handle reconnection on disconnect
   - **Test:** Auto-reconnect after Discord disconnect
   - Verification: Connection resumes after simulated disconnect

## Technical Context

### Architecture Reference

From `.switchboard/planning/architecture.md`:

- **§5.2 gateway::server:** HTTP and WebSocket server for gateway
- **§5.6 gateway::client:** Client library for projects to connect to gateway
- **§7 Error Handling Strategy:** Use `thiserror` for error types

### Project Conventions

From `.switchboard/planning/project-context.md`:
- **Build:** `cargo build --features "discord gateway"`
- **Async:** Use tokio for async. Follow patterns in `src/discord/gateway.rs`
- **Error Handling:** Use `thiserror` for error types. Never use `unwrap()` in production
- **Logging:** Use `tracing` for logging

### Existing Code Context

**Discord Gateway in `src/discord/gateway.rs`:**
```rust
// Lines 1-80 - Discord gateway types already exist
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};

pub enum DiscordEvent {
    MessageCreate {
        channel_id: String,
        content: String,
        author_id: String,
        message_id: String,
        guild_id: Option<String>,
    },
    Ready { user_id: String, session_id: String },
    // ...
}
```

**Gateway Server in `src/gateway/server.rs`:**
```rust
// Lines 1-100 - HTTP server with WebSocket support already exists
use crate::discord::gateway::DiscordEvent;  // Need to add this import
```

**Current server.rs has:**
- Health check endpoint at `/health`
- WebSocket endpoint at `/ws`
- AppState with config and registry

### Files in src/gateway/
```
src/gateway/
├── mod.rs        # Module exports (exists)
├── config.rs     # Config loading (exists)
├── protocol.rs   # Message protocol types (exists)
├── registry.rs   # Channel registry (exists)
└── server.rs     # HTTP/WS server (exists)
```

## Implementation Plan

1. **Import Discord Gateway in `src/gateway/server.rs`**
   - Add: `use crate::discord::gateway::{DiscordGateway, DiscordEvent, GatewayError};`
   - Add DiscordGateway to AppState

2. **Add Discord connection to GatewayServer**
   - Create DiscordGateway instance in server startup
   - Start Discord event listener in background task
   - Handle MessageCreate events and forward to registered projects

3. **Wire up event routing**
   - Extract channel_id and content from DiscordEvent::MessageCreate
   - Look up subscribed projects in ChannelRegistry
   - Forward message to each subscribed project's WebSocket

4. **Add reconnection handling**
   - Implement auto-reconnect on disconnect
   - Log connection status changes

5. **Write tests**
   - Test DiscordGateway initialization
   - Test message routing to projects
   - Test reconnection logic

6. **Run build + tests**
   - `cargo build --features "discord gateway"`
   - `cargo test --lib`

### Skills to Read

- [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md) — Rust best practices
- [`skills/rust-engineer/SKILL.md`](skills/rust-engineer/SKILL.md) — Async patterns with tokio
- [`skills/rust-engineer/references/async.md`](skills/rust-engineer/references/async.md) — Async/tokio specifics

### Dependencies

- Story 4.2 (Gateway config loading) — Complete ✓
- Story 4.6 (Registration protocol) — Complete ✓

## Scope Boundaries

### This Story Includes
- Discord Gateway connection using twilight-gateway
- MessageCreate event handling
- Reconnection on disconnect
- Event routing to registered projects

### This Story Does NOT Include
- CLI command to start gateway (Story 4.8)
- Message routing by channel (Story 5.3)
- Fan-out to multiple projects (Story 6.5)
- Rate limiting (Story 6.6)

### Files in Scope
- `src/gateway/server.rs` — modify (add Discord connection)
- `src/gateway/mod.rs` — modify (export new types if needed)

### Files NOT in Scope
- `src/cli/commands/gateway.rs` — Different module (Story 4.8)
- `src/gateway/routing.rs` — Not created yet (Story 5.3)
- `src/discord/gateway.rs` — Already exists, don't modify internal logic
