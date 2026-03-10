# Story 006-05: Fan-out Message Delivery

> Epic: Epic 06 — Multi-Project Support
> Points: 2
> Sprint: 15
> Type: feature
> Risk: low
> Created: 2026-03-04

## User Story

**As a** user,  
**I want** multiple projects to receive messages from the same channel,  
**So that** different projects can process the same Discord messages.

## Acceptance Criteria

1. When a message arrives on a channel with multiple subscribers
   - **Test:** All subscribed projects receive the message

2. Failure to one project doesn't affect others
   - **Test:** Other projects still receive message

3. Messages delivered in Discord event order
   - **Test:** Order preserved per project

## Technical Context

### Architecture Reference
- Module: `src/gateway/routing.rs`
- Uses ChannelRegistry from Epic 05

### Project Conventions
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`
- Use thiserror for errors
- Use tokio for async
- Use tracing for logging

## Implementation Plan

1. Modify message routing in `src/gateway/routing.rs`
2. Implement fan-out logic to all subscribed connections
3. Add error handling to not break other deliveries
4. Ensure ordering is preserved
5. Write tests for fan-out behavior
6. Run build + tests

### Skills to Read
- `./skills/rust-engineer/SKILL.md` — async Rust patterns

### Dependencies
- story-005-03 (Route messages by channel_id) — COMPLETE

## Scope Boundaries

### This Story Includes
- Fan-out delivery to multiple projects
- Error isolation between projects

### This Story Does NOT Include
- Rate limiting (story-006-06)
- Reconnection logic
- Message filtering
