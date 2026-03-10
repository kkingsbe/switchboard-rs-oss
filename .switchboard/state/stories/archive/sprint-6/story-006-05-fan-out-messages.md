# Story 006-05: Fan-out Message Delivery

> Epic: Epic 06 — Multi-Project Gateway
> Points: 2
> Sprint: 9
> Type: feature
> Risk: low
> Created: 2026-03-03

## User Story

As a user, I want multiple projects to receive messages from the same channel, So that different projects can process the same Discord messages.

## Acceptance Criteria

1. When a message arrives on a channel with multiple subscribers
   - **Test:** All subscribed projects receive the message - verify each project gets the message

2. Failure to one project doesn't affect others
   - **Test:** Other projects still receive message - simulate one failure and verify others succeed

3. Messages delivered in Discord event order
   - **Test:** Order preserved per project - send multiple messages and verify order per subscriber

## Technical Context

### Architecture Reference
- Existing [`ChannelRegistry`](src/gateway/registry.rs:87) already supports multiple projects per channel
- Existing [`Router`](src/gateway/routing.rs:51) already has basic fan-out logic
- Need to add error isolation so one failed send doesn't stop others

### Project Conventions
- Build: `cargo build --features "discord gateway"`
- Test: `src/gateway/routing.rs` for test examples

### Existing Code Context
```
src/gateway/
current implementation├── routing.rs ()
└── registry.rs (ChannelRegistry already supports fan-out)
```

Current implementation in [`routing.rs`](src/gateway/routing.rs:83):
- Uses `projects_for_channel()` to get all subscribed projects
- Iterates and sends to each project
- Logs warnings on failure but continues

The current implementation already handles error isolation at line 119-124 - it logs warnings but continues to the next project. This story verifies and strengthens that behavior.

## Implementation Plan

1. **Review** current `src/gateway/routing.rs` implementation
   - Verify error isolation is working correctly
   - Add explicit error handling per-project

2. **Strengthen error isolation**:
   - Wrap each send in its own error handler
   - Track per-project send results independently
   - Ensure one failure doesn't cascade

3. **Write tests** for:
   - All projects receive message when multiple subscribed
   - Failure isolation - one failure doesn't stop others
   - Message ordering preserved

4. **Run** `cargo build --features "discord gateway"` and tests

### Skills to Read
- `skills/rust-best-practices/SKILL.md` — error handling, async patterns

### Dependencies
- story-005-03 (Route by Channel) — must be complete first
- story-005-01 (Channel Registry) — already supports fan-out

## Scope Boundaries

### This Story Includes
- Error isolation per-project in routing
- Test verification of fan-out behavior
- Order preservation per subscriber

### This Story Does NOT Include
- Message transformation/filtering
- Priority routing
- Bulk message handling

### Files in Scope
- `src/gateway/routing.rs` — modify (strengthen error isolation)

### Files NOT in Scope
- `src/gateway/registry.rs` — don't modify (already complete)
- `src/gateway/server.rs` — don't modify
- `src/discord/` — don't modify
