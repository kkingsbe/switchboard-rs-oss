# Story 5.1: Implement ChannelRegistry

> Epic: Epic 05 — Discord Gateway - Phase 2: Channel Routing
> Points: 3
> Sprint: 7
> Type: feature
> Risk: medium
> Created: 2026-03-03
> Status: not-started

## User Story

As a system,
I want to track which projects are subscribed to which channels,
So that I can route messages correctly.

## Acceptance Criteria

1. Create `ChannelRegistry` struct with thread-safe interior
   - **Test:** Can be accessed from multiple tasks concurrently
   - Verification: Unit tests with concurrent access pass

2. Implement `register(project, channels)` method
   - **Test:** Project added to channel mapping
   - Verification: After registration, `projects_for_channel()` returns the project

3. Implement `unregister(project_id)` method
   - **Test:** Project removed from all channels
   - Verification: After unregister, project not in any channel's project list

4. Implement `projects_for_channel(channel_id)` method
   - **Test:** Returns correct list of projects
   - Verification: Returns all projects subscribed to a given channel

## Technical Context

### Architecture Reference

From `.switchboard/planning/architecture.md`:

- **§5.3 gateway::registry:** Track channel-to-project mappings
- **Public API:**
  - `ChannelRegistry::register(project: ProjectConnection, channels: Vec<String>)`
  - `ChannelRegistry::unregister(project_id: &ProjectId)`
  - `ChannelRegistry::projects_for_channel(channel_id: &str) -> &[ProjectId]`
- **Dependencies:** tokio::sync::RwLock

### Project Conventions

From `.switchboard/planning/project-context.md`:
- **Build:** `cargo build --features "discord gateway"`
- **Async:** Use tokio for async. Follow patterns in `src/discord/gateway.rs`
- **Error Handling:** Use `thiserror` for error types. Never use `unwrap()` in production
- **Testing:** Place unit tests in the same file as the code

### Existing Code Context

**ChannelRegistry in `src/gateway/registry.rs`:**
```rust
// Lines 82-160 - ChannelRegistry already exists with basic structure
pub struct ChannelRegistry {
    inner: Arc<RwLock<RegistryInner>>,
}

impl ChannelRegistry {
    pub fn new() -> Self { ... }
    pub async fn register(&self, project: ProjectConnection, channels: Vec<String>) -> RegistryResult<()> { ... }
    // Missing: unregister(), projects_for_channel()
}
```

**ProjectConnection struct:**
```rust
// src/gateway/registry.rs lines 47-62
pub struct ProjectConnection {
    pub project_id: ProjectId,
    pub project_name: String,
    pub ws_sender: mpsc::Sender<String>,
    pub session_id: Uuid,
    pub subscribed_channels: Vec<String>,
    pub registered_at: DateTime<Utc>,
}
```

**RegistryError enum:**
```rust
// src/gateway/registry.rs lines 18-31
pub enum RegistryError {
    #[error("Project not found: {0}")]
    ProjectNotFound(ProjectId),
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),
    #[error("Project already registered: {0}")]
    ProjectAlreadyRegistered(ProjectId),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type RegistryResult<T> = Result<T, RegistryError>;
```

### Files in src/gateway/
```
src/gateway/
├── mod.rs        # Module exports (exists)
├── config.rs     # Config loading (exists)
├── protocol.rs   # Message protocol types (exists)
├── registry.rs   # Channel registry - NEEDS NEW METHODS
└── server.rs     # HTTP/WS server (exists)
```

## Implementation Plan

1. **Add `unregister` method to `src/gateway/registry.rs`**
   - Remove project from projects HashMap
   - Remove project from all channel_to_projects mappings
   - Return error if project not found

2. **Add `projects_for_channel` method**
   - Look up channel in channel_to_projects HashMap
   - Return list of project IDs (clone to avoid borrow issues)
   - Return empty vec if channel not found

3. **Add additional helper methods (optional)**
   - `get_project(project_id)` - Get project details
   - `list_all_projects()` - List all registered projects
   - `list_project_channels(project_id)` - Get channels for a project

4. **Write unit tests**
   - Test register, unregister, projects_for_channel flow
   - Test concurrent access from multiple tasks
   - Test error cases (project not found, etc.)

5. **Run build + tests**
   - `cargo build --features "discord gateway"`
   - `cargo test --lib`

### Skills to Read

- [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md) — Rust best practices
- [`skills/rust-engineer/SKILL.md`](skills/rust-engineer/SKILL.md) — Async patterns with tokio
- [`skills/rust-engineer/references/ownership.md`](skills/rust-engineer/references/ownership.md) — Ownership/borrowing patterns

### Dependencies

- Story 4.1 (Gateway module structure) — Complete ✓

## Scope Boundaries

### This Story Includes
- ChannelRegistry struct methods (register, unregister, projects_for_channel)
- Thread-safe access using RwLock
- Unit tests for registry operations

### This Story Does NOT Include
- Message routing logic (Story 5.3)
- Runtime subscribe/unsubscribe (Story 5.4)
- Connection management (Story 6.1)
- Fan-out delivery (Story 6.5)

### Files in Scope
- `src/gateway/registry.rs` — modify (add new methods)
- `src/gateway/mod.rs` — no changes needed

### Files NOT in Scope
- `src/gateway/server.rs` — Don't modify routing logic here
- `src/gateway/routing.rs` — Not created yet (Story 5.3)
- `src/discord/gateway.rs` — Different module
