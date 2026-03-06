# Story 5.1: Implement ChannelRegistry

> **Sprint**: 19
> **Epic**: Epic 05 - Discord Gateway Phase 2 (Channel Routing)
> **Points**: 3
> **Risk**: Medium
> **Type**: feature

## User Story

**As a** system,
**I want** to track which projects are subscribed to which channels,
**So that** I can route messages correctly.

## Dependencies

- Story 4.1: Gateway module structure (COMPLETE)

## Acceptance Criteria

1. Create `ChannelRegistry` struct with thread-safe interior
   - Verification: Can be accessed from multiple tasks
2. Implement `register(project, channels)` method
   - Verification: Project added to channel mapping
3. Implement `unregister(project_id)` method
   - Verification: Project removed from all channels
4. Implement `projects_for_channel(channel_id)` method
   - Verification: Returns correct list of projects

## Technical Notes

- Files to create: `src/gateway/registry.rs`
- Dependencies: tokio::sync::RwLock, std::collections::HashMap
- Pattern: Use Arc<RwLock<T>> for shared state
- Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-best-practices/SKILL.md`

## Verification

```bash
# Build verification
cargo build --features "discord gateway"

# Test verification
cargo test --lib

# Lint verification
cargo clippy -- -D warnings
```

## Commit

`feat(gateway): [story-005-01] implement channel registry for routing`
