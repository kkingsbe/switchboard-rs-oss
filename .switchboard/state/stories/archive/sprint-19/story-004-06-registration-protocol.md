# Story 4.6: Implement Basic Registration Protocol

> **Sprint**: 19
> **Epic**: Epic 04 - Discord Gateway Phase 1
> **Points**: 3
> **Risk**: Medium
> **Type**: feature

## User Story

**As a** project developer,
**I want** to register my project with the gateway,
**So that** I can receive Discord messages.

## Dependencies

- Story 4.4: WebSocket server (COMPLETE)
- Story 4.5: Message protocol types (COMPLETE)

## Acceptance Criteria

1. Project sends `{"type": "register", "project_name": "...", "channels": [...]}`
   - Verification: Message parsed correctly
2. Gateway responds with `{"type": "register_ack", "status": "ok", "session_id": "..."}`
   - Verification: Registration completes successfully
3. Invalid registration returns `{"type": "register_error", "error": "..."}`
   - Verification: Error case handled gracefully

## Technical Notes

- Files to modify: `src/gateway/server.rs`, `src/gateway/protocol.rs`
- Pattern: Follow existing async patterns in `src/discord/gateway.rs`
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

`feat(gateway): [story-004-06] implement basic registration protocol`
