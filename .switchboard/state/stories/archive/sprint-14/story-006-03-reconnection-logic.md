# Story: story-006-03 - Reconnection Logic

## Metadata

- **Story ID**: story-006-03
- **Title**: Reconnection Logic
- **Epic**: Epic 006 - Gateway Connection Management
- **Points**: 3
- **Type**: feature
- **Risk Level**: Medium
- **Status**: Implemented

---

## User Story

As a system, I want the gateway to automatically reconnect to projects when connections are lost so that service continuity is maintained without manual intervention.

---

## Acceptance Criteria

1. Reconnection uses exponential backoff strategy
2. Configuration supports: initial_delay, max_delay, max_retries, multiplier
3. ReconnectionManager tracks connection state (Idle, Reconnecting, Reconnected, Failed, Cancelled)
4. Backoff calculator computes delays correctly
5. Maximum retry limit is enforced
6. Reconnection can be cancelled
7. Callbacks are invoked for each reconnection attempt

**Test Methods**:
- Backoff delays follow exponential progression (1s, 2s, 4s, 8s...)
- Backoff caps at max_delay
- Manager transitions through correct states
- Max retries causes MaxRetriesExceeded error
- Cancellation works correctly

---

## Technical Context

### Architecture References

The reconnection module provides a generic retry mechanism that can be used by any connection type that needs reconnection logic.

### Existing Code

- Reconnection module: `src/gateway/reconnection.rs`
- Discord gateway: `src/discord/gateway.rs` (may use reconnection)
- Connection management: `src/gateway/connections.rs`

---

## Implementation Plan

1. Define `ReconnectionConfig` struct:
   - `initial_delay: Duration`
   - `max_delay: Duration`
   - `max_retries: u32`
   - `multiplier: f64`
2. Define `ReconnectionState` enum:
   - Idle, Reconnecting, Reconnected, Failed, Cancelled
3. Define `ReconnectionError` enum:
   - MaxRetriesExceeded, ReconnectionCancelled, Aborted, InvalidConfig
4. Implement `Backoff` struct:
   - Calculate delay using exponential formula
   - Track attempt number
   - Support reset
5. Implement `ReconnectionManager`:
   - Track state and retry count
   - `attempt_reconnection()` async method
   - Invoke callback for each attempt
   - Wait with backoff between attempts
6. Add comprehensive unit tests

---

## Skills to Read

- [Rust Engineer](../../skills/rust-engineer/SKILL.md)
- [Rust Best Practices](../../skills/rust-best-practices/SKILL.md)
- [Async Reference](../../skills/rust-engineer/references/async.md)
- [Ownership Reference](../../skills/rust-engineer/references/ownership.md)

---

## Dependencies

- `tokio` for async sleep
- `thiserror` for error types

---

## Scope Boundaries

### In Scope
- Exponential backoff calculation
- Retry limit enforcement
- State management
- Async callback support

### Out of Scope
- Specific connection implementations
- Rate limiting during reconnection
- Jitter addition to backoff
- Reconnection scheduling

---

## Files in Scope

| File | Description |
|------|-------------|
| `src/gateway/reconnection.rs` | Reconnection logic implementation |
| `src/gateway/mod.rs` | Module export |

---

## Configuration

```rust
let config = ReconnectionConfig {
    initial_delay: Duration::from_secs(1),
    max_delay: Duration::from_secs(60),
    max_retries: 10,
    multiplier: 2.0,
};
```

---

## Backoff Formula

```
delay = min(initial_delay * multiplier^attempt, max_delay)
```

Example with defaults (1s initial, 2.0 multiplier, 60s max):
- Attempt 0: min(1 * 2^0, 60) = 1s
- Attempt 1: min(1 * 2^1, 60) = 2s
- Attempt 2: min(1 * 2^2, 60) = 4s
- Attempt 3: min(1 * 2^3, 60) = 8s
- Attempt 4: min(1 * 2^4, 60) = 16s
- Attempt 5: min(1 * 2^5, 60) = 32s
- Attempt 6+: min(1 * 2^6, 60) = 60s (capped)

---

## Usage Example

```rust
let mut manager = ReconnectionManager::new(
    "project-123".to_string(),
    ReconnectionConfig::default(),
);

let success = manager.attempt_reconnection(|attempt| async {
    // Attempt to reconnect
    reconnect_to_project("project-123").await
}).await?;

if success {
    println!("Reconnected successfully!");
} else {
    println!("Failed to reconnect after max retries");
}
```
