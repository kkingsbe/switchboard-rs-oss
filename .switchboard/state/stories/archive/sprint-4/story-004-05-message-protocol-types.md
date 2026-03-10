# Story 004-05: Define Message Protocol Types

> Epic: epic-04-discord-gateway-phase1 — Discord Gateway - Basic Gateway with Single Project
> Points: 2
> Sprint: 3
> Type: feature
> Risk: Low
> Created: 2026-03-02T22:13:56.718Z

## User Story

**As a** developer,
**I want** clear message protocol types,
**So that** the gateway and projects can communicate reliably.

## Acceptance Criteria

1. Define `GatewayMessage` enum with variants: Register, RegisterAck, Message, Heartbeat, HeartbeatAck
   - **Test:** Types serialize/deserialize correctly
2. Implement serde serialization/deserialization
   - **Test:** JSON round-trip tests pass
3. Document protocol in code comments
   - **Test:** Doc tests pass

## Technical Context

### Architecture Reference
- See `.switchboard/planning/architecture.md` Section 5.4 (gateway::protocol)
- Protocol types for gateway<->project communication

### Project Conventions
From `.switchboard/planning/project-context.md`:
- **Build:** `cargo build --features "discord gateway"`
- **Test:** `cargo test --lib`
- **Serialization:** Use `serde` and `serde_json`

### Existing Code Context
See `src/discord/` for similar protocol/enum patterns using serde

## Implementation Plan

1. Create `src/gateway/protocol.rs`
2. Define `GatewayMessage` enum with all variants
3. Implement Serialize/Deserialize for the enum
4. Add doc comments explaining each variant
5. Write unit tests for JSON serialization/deserialization
6. Run `cargo test --lib` to verify

### Skills to Read
- `./skills/rust-best-practices/SKILL.md` — Serialization patterns
- `./skills/rust-engineer/references/traits.md` — Trait implementations

### Dependencies
None - no dependencies on other stories

## Scope Boundaries

### This Story Includes
- Protocol type definitions
- Serde serialization implementation
- Unit tests

### This Story Does NOT Include
- WebSocket handling
- Connection management
- Message routing logic

### Files in Scope
- `src/gateway/protocol.rs` — CREATE

### Files NOT in Scope
- `src/gateway/server.rs` — belongs to Story 4.3/4.4
- `src/gateway/registry.rs` — belongs to Epic 05
