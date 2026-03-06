# Story 4.5: Define message protocol types

> Epic: epic-04-discord-gateway-phase1 — Discord Gateway - Basic Gateway with Single Project
> Points: 2
> Sprint: 18
> Type: feature
> Risk: low
> Created: 2026-03-05T07:11:29Z

## User Story

**As a** developer,
**I want** clear message protocol types,
**So that** the gateway and projects can communicate reliably.

## Acceptance Criteria

1. Define `GatewayMessage` enum with variants: Register, RegisterAck, Message, Heartbeat, HeartbeatAck
   - Verification: Types serialize/deserialize correctly
2. Implement serde serialization/deserialization
   - Verification: JSON round-trip tests pass
3. Document protocol in code comments
   - Verification: Doc tests pass

## Technical Context

### Architecture Reference

From `.switchboard/planning/architecture.md` §5.4:

```
### 5.4 gateway::protocol

- **Purpose:** Define message types for gateway<->project communication
- **Public API:** Enums and structs for register, message, heartbeat
- **Dependencies:** serde, serde_json
```

From §6 Data Model:

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

From `.switchboard/planning/project-context.md`:

- **Serialization:** Use `serde` and `serde_json` for JSON. Use `toml` for config files.
- **Testing:** Place unit tests in the same file as the code (module tests). Use descriptive test names: `test_name_should_do_something()`.

### Existing Code Context

This is a new module, no existing code to reference. Follow standard Rust patterns for serde enums.

## Implementation Plan

1. Create `src/gateway/protocol.rs` with `GatewayMessage` enum
2. Implement serde Serialize/Deserialize for all variants
3. Add comprehensive unit tests for serialization/deserialization
4. Add documentation comments for the protocol
5. Verify with `cargo test --lib`

### Skills to Read
- `./skills/rust-best-practices/SKILL.md`
- `./skills/rust-engineer/SKILL.md`
- `./skills/rust-engineer/references/testing.md` (if applicable)

### Dependencies
- Story 4.1: Create gateway module structure — must complete first

## Scope Boundaries

### This Story Includes
- GatewayMessage enum with all required variants
- Serde serialization/deserialization
- Unit tests
- Protocol documentation

### This Story Does NOT Include
- WebSocket message handling
- Registration protocol implementation
- Discord connection

### Files in Scope
- `src/gateway/protocol.rs` — create

### Files NOT in Scope
- `src/gateway/server.rs` — not yet
- `src/gateway/registry.rs` — not yet
- `src/gateway/connections.rs` — not yet
