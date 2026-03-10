# Story 4.5: Define message protocol types

> Epic: epic-04 — Discord Gateway Phase 1: Basic Gateway
> Points: 2
> Sprint: 2
> Type: feature
> Risk: low
> Created: 2026-03-02

## User Story

**As a** developer,
**I want** clear message protocol types,
**So that** the gateway and projects can communicate reliably.

## Acceptance Criteria

1. Define `GatewayMessage` enum with variants: Register, RegisterAck, Message, Heartbeat, HeartbeatAck
   - **Test:** Types compile correctly

2. Implement serde serialization/deserialization
   - **Test:** JSON round-trip tests pass (serialize then deserialize produces equivalent data)

3. Document protocol in code comments
   - **Test:** Doc tests pass (`cargo test --doc`)

## Technical Context

### Architecture Reference

From architecture.md:
- GatewayMessage enum with variants: Register, RegisterAck, Message, Heartbeat, HeartbeatAck
- Protocol types for WebSocket communication
- Serialization with serde/serde_json

### Project Conventions

From project-context.md:
- **Build:** `cargo build --features "discord gateway"`
- **Test:** `cargo test --lib`
- **Lint:** `cargo clippy -- -D warnings`
- **Error Handling:** Use thiserror - never anyhow in library code
- **No unwrap()** in production - use `?` or `.expect()`
- **Logging:** Use tracing - never println!
- **Tests:** Inline in module files

### Existing Code Context

Example enum pattern from codebase:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GatewayMessage {
    Register { project_id: String, channels: Vec<String> },
    RegisterAck { success: bool, project_id: String },
    Message { channel_id: String, content: String, author: String },
    Heartbeat,
    HeartbeatAck,
}
```

## Implementation Plan

1. Create `src/gateway/protocol.rs`
2. Define GatewayMessage enum with all variants
3. Add Serialize, Deserialize derives
4. Add Debug, Clone derives
5. Implement helper methods if needed (e.g., message_type())
6. Add comprehensive doc comments explaining each variant
7. Write unit tests for serialization/deserialization
8. Write doc tests with examples
9. Run tests: `cargo test --lib`
10. Run doc tests: `cargo test --doc`
11. Run lint: `cargo clippy -- -D warnings`

### Skills to Read

- `./skills/rust-best-practices/SKILL.md` — Serde patterns
- `./skills/rust-engineer/references/testing.md` — Testing standards with serde

### Dependencies

- **Story 4.1**: Must complete first (module structure must exist)

## Scope Boundaries

### This Story Includes
- Creating protocol types
- Serde serialization/deserialization
- Documentation with doc comments
- Unit tests and doc tests

### This Story Does NOT Include
- WebSocket server implementation (story 4.4)
- Protocol handling logic (story 4.6)
- Message routing (epic 05)

### Files in Scope
- `src/gateway/protocol.rs` — create

### Files NOT in Scope
- `src/gateway/server.rs` — story 4.3
- WebSocket handling
- Message routing
