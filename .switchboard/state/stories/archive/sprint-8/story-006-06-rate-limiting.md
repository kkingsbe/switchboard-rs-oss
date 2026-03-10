# Story 006-06: Rate Limiting

> Epic: epic-06 — Discord Gateway Phase 3
> Points: 2
> Sprint: 8
> Type: feature
> Risk: medium
> Created: 2026-03-03

## User Story

As a system, I want to respect Discord's rate limits, So that I don't get the bot suspended.

## Acceptance Criteria

1. Track requests per channel - Rate limit tracked correctly
   - **Test:** Verify rate limit counter increments and resets appropriately

2. Handle 429 responses with Retry-After header - Wait time respected
   - **Test:** Mock 429 response and verify wait time is respected

3. Implement exponential backoff on repeated rate limits - Backoff increases on continued 429s
   - **Test:** Simulate multiple 429s and verify backoff increases

## Technical Context

### Architecture Reference
- Discord has channel-based rate limits (usually 5 messages per 5 seconds)
- 429 response includes Retry-After header with seconds to wait
- Need to track per-channel state

### Existing Code Context
```
src/gateway/
├── server.rs (message sending)
└── protocol.rs
```

Uses tokio for async operations.

## Implementation Plan

1. **Create** `src/gateway/ratelimit.rs` - Rate limiting logic
2. **Modify** `src/gateway/server.rs` - Integrate rate limiting into message sending
3. **Write tests** for rate limit behavior
4. **Run** build and tests

### Skills to Read
- `skills/rust-engineer/SKILL.md`
- `skills/rust-engineer/references/async.md` — tokio time, rate limiting

### Dependencies
- story-004-07 (Discord Gateway) — complete

## Scope Boundaries

### This Story Includes
- Per-channel rate limit tracking
- 429 response handling
- Exponential backoff

### This Story Does NOT Include
- Global rate limiting across all channels
- Caching of rate limit state
- Priority queue for messages

### Files in Scope
- `src/gateway/ratelimit.rs` — create
- `src/gateway/server.rs` — modify

### Files NOT in Scope
- `src/discord/` — don't modify
