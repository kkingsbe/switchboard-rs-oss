# Story 4.5: Define message protocol types

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

### Architecture
- Protocol types define the wire format between gateway and connected projects
- JSON serialization for simplicity (as per architecture decision)
- Message types cover: registration, message forwarding, heartbeat

### Project Conventions
- Use `#[derive(Serialize, Deserialize)]` for protocol types
- Use `thiserror` for protocol errors
- Follow existing patterns in `src/discord/` for type definitions

### Existing Code to Reference
- `src/discord/tools/definitions.rs` - similar serde enum patterns
- Architecture doc: `.switchboard/planning/architecture.md` §5.4

## Implementation Plan

1. Create `src/gateway/protocol.rs`:
   - Define `GatewayMessage` enum with all variants
   - Define associated data structures (RegisterPayload, MessagePayload, etc.)
   - Add serde derives
   - Add thiserror for protocol errors

2. Implement serialization:
   - Ensure all variants serialize to JSON correctly
   - Write unit tests for JSON round-trip

3. Document the protocol:
   - Add doc comments explaining each message type
   - Include JSON examples in documentation

4. Verify compilation:
   - `cargo build --features gateway` succeeds
   - `cargo test --lib` passes

## Skills

- [Rust Best Practices](../../skills/rust-best-practices/SKILL.md)
- [Rust Engineer](../../skills/rust-engineer/SKILL.md)

## Dependencies

- Depends on: Story 4.1 (gateway module structure)

## Scope Boundaries

**In Scope:**
- Protocol type definitions
- JSON serialization
- Basic error types
- Unit tests

**Out of Scope:**
- Protocol implementation (encoding/decoding logic)
- WebSocket transport
- Server integration

## Risk Assessment

- **Risk Level:** Low
- **Rationale:** Pure type definitions, no I/O or async code
