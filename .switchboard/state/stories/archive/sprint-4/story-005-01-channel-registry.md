# Story 005-01: Implement ChannelRegistry

> Epic: epic-05-discord-gateway-phase2 — Discord Gateway - Channel Routing with Config File
> Points: 3
> Sprint: 4
> Type: feature
> Risk: Medium
> Created: 2026-03-02T23:05:00Z

## User Story

**As a** system,
**I want** to track which projects are subscribed to which channels,
**So that** I can route messages correctly.

## Acceptance Criteria

1. Create `ChannelRegistry` struct with thread-safe interior
   - **Test:** Can be accessed from multiple tasks
2. Implement `register(project, channels)` method
   - **Test:** Project added to channel mapping
3. Implement `unregister(project_id)` method
   - **Test:** Project removed from all channels
4. Implement `projects_for_channel(channel_id)` method
   - **Test:** Returns correct list of projects

## Technical Context

### Architecture Reference

From `.switchboard/planning/architecture.md` Section 5.3 (gateway::registry):

- **Purpose:** Track channel-to-project mappings
- **Public API:**
  - `ChannelRegistry::register(project: ProjectConnection, channels: Vec<String>)`
  - `ChannelRegistry::unregister(project_id: &ProjectId)`
  - `ChannelRegistry::projects_for_channel(channel_id: &str) -> &[ProjectId]`
- **Dependencies:** tokio::sync::RwLock
- **Data flow:** Maintained in memory, updated on project connect/disconnect

**Data Model:**
```rust
struct ProjectConnection {
    project_id: ProjectId,
    project_name: String,
    ws_sender: mpsc::Sender<String>,
    session_id: Uuid,
    subscribed_channels: Vec<String>,
    registered_at: DateTime<Utc>,
}
```

### Project Conventions

From `.switchboard/planning/project-context.md`:
- **Build:** `cargo build --features "discord gateway"`
- **Test:** `cargo test --lib`
- **Lint:** `cargo clippy -- -D warnings`
- **Format:** `cargo fmt`
- **Language:** Rust 2021 edition
- **Error Handling:** Use `thiserror` for error types
- **No unwrap() in production:** Use `?` operator or `.expect()` with descriptive messages
- **Async conventions:** Use tokio for async
- **Module organization:** New gateway code goes in `src/gateway/`
- **Logging:** Use `tracing` for logging

### Existing Code Context

**Thread-safe patterns** in existing codebase:
- Use `Arc<RwLock<T>>` for shared state
- Use `tokio::sync::RwLock` for async contexts
- See `src/discord/gateway.rs` for similar patterns

**Gateway module** (from `src/gateway/mod.rs`):
```rust
//! Gateway module for Discord Gateway Service

pub mod config;
pub mod protocol;
// Add this line after implementation:
pub mod registry;
```

### Current Directory Structure

```
src/gateway/
├── mod.rs      # Module declarations (exists)
├── protocol.rs # Protocol types (exists, story 4.5)
├── config.rs   # Config loading (story 4.2 - may be in progress)
└── registry.rs # TO BE CREATED by this story
```

## Implementation Plan

1. Create `src/gateway/registry.rs` with:
   - `ChannelRegistry` struct using `Arc<RwLock<RegistryInner>>`
   - `RegistryInner` struct with channel-to-project mappings
   - `ProjectConnection` struct for tracking project state
   - Proper error types using `thiserror`

2. Implement thread-safe methods:
   - `register(project: ProjectConnection, channels: Vec<String>)`
   - `unregister(project_id: &ProjectId)` 
   - `projects_for_channel(channel_id: &str) -> Vec<ProjectId>`
   - `add_channel_subscription(project_id, channel_id)`
   - `remove_channel_subscription(project_id, channel_id)`

3. Handle edge cases:
   - Duplicate registrations (update existing)
   - Project subscribes to multiple channels
   - Multiple projects can subscribe to same channel (fan-out)

4. Add unit tests:
   - Test basic registration
   - Test unregistration
   - Test channel lookup
   - Test concurrent access

5. Verify:
   - Run `cargo build --features gateway`
   - Run `cargo test --lib`
   - Run `cargo clippy -- -D warnings`

### Skills to Read

- `./skills/rust-best-practices/SKILL.md` — Concurrency patterns, thread safety
- `./skills/rust-engineer/SKILL.md` — Rust engineering patterns
- `./skills/rust-engineer/references/async.md` — Async tokio patterns

### Dependencies

- Story 4.1 (gateway module structure) — **COMPLETE**
  - `src/gateway/mod.rs` exists
  - Feature flag `gateway` is in Cargo.toml

## Scope Boundaries

### This Story Includes
- ChannelRegistry struct with thread-safe interior
- Project registration and unregistration
- Channel subscription management
- Query methods for routing

### This Story Does NOT Include
- HTTP server implementation (belongs to Story 4.3)
- WebSocket handling (belongs to Story 4.4)
- Message routing logic (belongs to Story 5.3)
- Runtime subscribe/unsubscribe via WebSocket (belongs to Story 5.4)

### Files in Scope
- `src/gateway/registry.rs` — CREATE
- `src/gateway/mod.rs` — MODIFY (add `pub mod registry`)

### Files NOT in Scope
- `src/gateway/server.rs` — belongs to Story 4.3
- `src/gateway/routing.rs` — belongs to Story 5.3
