# Story 4.5: Define Message Protocol Types

> Epic: Epic 04 — Discord Gateway - Basic Gateway with Single Project
> Points: 2
> Sprint: 6
> Type: feature
> Risk: Low
> Status: not-started
> Created: 2026-03-03
> Depends on: 4.1

## User Story

As a developer,
I want clear message protocol types,
So that the gateway and projects can communicate reliably.

## Acceptance Criteria

1. Define `GatewayMessage` enum with variants: Register, RegisterAck, Message, Heartbeat, HeartbeatAck
   - **Verification:** Types serialize/deserialize correctly

2. Implement serde serialization/deserialization
   - **Verification:** JSON round-trip tests pass

3. Document protocol in code comments
   - **Verification:** Doc tests pass

## Technical Context

### Architecture Reference

Per architecture.md §5.4 - gateway::protocol:
- **Purpose:** Define message types for gateway<->project communication
- **Public API:** Enums and structs for register, message, heartbeat
- **Dependencies:** serde, serde_json
- **Data flow:** Messages serialized as JSON over WebSocket connection

Per architecture.md §5.18 - GatewayMessage (protocol):
```rust
enum GatewayMessage {
    Register { project_name: String, channels: Vec<String> },
    RegisterAck { status: String, session_id: Uuid },
    Message { channel_id: String, content: String, ... },
    Heartbeat { session_id: Uuid },
    HeartbeatAck { server_time: i64 },
}
```

### Project Conventions

From project-context.md:
- **Build:** `cargo build --features "discord gateway"`
- **Async:** Use tokio for async. Follow patterns in `src/discord/gateway.rs`
- **Error Handling:** Use `thiserror` for error types. Never use `unwrap()` in production
- **Logging:** Use `tracing` for logging
- **Serialization:** Use `serde` and `serde_json` for JSON

### Existing Code Context

Current gateway module structure:
```
src/gateway/
├── mod.rs        # Already exists - module exports
├── config.rs     # Already exists - GatewayConfig
├── protocol.rs   # Already exists - GatewayMessage (THIS STORY)
├── registry.rs   # Already exists - ChannelRegistry
└── server.rs     # Already exists - HTTP/WS server
```

Existing protocol.rs contains:
- `GatewayMessage` enum with all required variants
- serde Serialize/Deserialize derive macros
- Comprehensive unit tests for JSON round-trip
- Full documentation comments

Key existing code patterns:
- Use `#[derive(Serialize, Deserialize)]` for enums
- Use `#[serde(rename_all = "lowercase")]` for variant naming
- Place tests in `#[cfg(test)] mod tests` within same file
- Use descriptive test names: `test_name_should_do_something()`

```rust
// Example from existing src/gateway/protocol.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GatewayMessage {
    Register { project_id: String },
    RegisterAck { project_id: String, assigned_channel: u64 },
    Message { payload: String, channel_id: u64 },
    Heartbeat { timestamp: u64 },
    HeartbeatAck { timestamp: u64 },
}
```

## Implementation Plan

1. **Review existing `src/gateway/protocol.rs` implementation**
   - Verify GatewayMessage enum has all required variants
   - Confirm serde derives are present
   - Check documentation comments exist

2. **Run existing tests** — Verify all JSON round-trip tests pass
   ```bash
   cargo test --lib gateway::protocol
   ```

 doc tests** —3. **Run Verify documentation examples work
   ```bash
   cargo test --doc
   ```

4. **Run clippy** — Ensure no lint warnings
   ```bash
   cargo clippy -- -D warnings
   ```

5. **Verify build** — Confirm everything compiles
   ```bash
   cargo build --features "discord gateway"
   ```

### Skills to Read

- [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md) — Rust best practices (serialization, testing, error handling)
- [`skills/rust-engineer/SKILL.md`](skills/rust-engineer/SKILL.md) — Async patterns with tokio, error handling

### Dependencies

- Story 4.1 (Gateway configuration) — Complete ✓
- Story 4.4 (WebSocket server) — In Progress

## Scope Boundaries

### This Story Includes
- Verification of existing GatewayMessage enum
- Running serialization/deserialization tests
- Running doc tests
- Verifying clippy passes

### This Story Does NOT Include
- Implementing registration protocol (Story 4.6)
- Implementing Discord gateway connection (Story 4.7)
- Message routing logic (Story 5.3)

### Files in Scope
- `src/gateway/protocol.rs` — verify (already implemented)

### Files NOT in Scope
- `src/gateway/server.rs` — Story 4.4
- `src/gateway/registry.rs` — Already exists
- `src/gateway/config.rs` — Story 4.1
- `src/cli/commands/gateway.rs` — Story 4.8
