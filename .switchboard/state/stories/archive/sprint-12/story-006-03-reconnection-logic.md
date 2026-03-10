# Story 006-03: Reconnection Logic

> Epic: epic-06 — Discord Gateway - Multi-Project Support & Reconnection
> Points: 3
> Sprint: 12
> Type: feature
> Risk: medium
> Created: 2026-03-04
> Status: ready
> Assigned To: dev-2

## User Story

As a user, I want projects to automatically reconnect if they drop, So that message delivery resumes without manual intervention.

## Acceptance Criteria

1. Project can reconnect with same session_id
   - **Test:** Reconnection preserves subscription - Verify project can reconnect and still receive messages on subscribed channels

2. Implement exponential backoff (1s, 2s, 4s... max 60s)
   - **Test:** Backoff increases correctly - Verify retry intervals follow exponential pattern

3. After max retries, mark project as failed
   - **Test:** Failure status reported - Verify failed status is communicated to user

## Technical Context

### Architecture Reference

From architecture.md:
- Gateway maintains one twilight-gateway connection for Discord
- Connection resilience: Reconnection with exponential backoff
- ChannelRegistry maps channel_id → projects
- ProjectConnection tracks: project_id, session_id, subscribed_channels

From §5.6 gateway::client:
- `GatewayClient::connect(gateway_url: &str, project_name: String, channels: Vec<String>) -> Result<Self>`
- `GatewayClient::heartbeat().await`

### Project Conventions

From project-context.md:
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Use `thiserror` for error types
- Use `tracing` for logging
- Never use `unwrap()` in production

### Existing Code Context

```
src/gateway/
├── mod.rs (gateway module entry)
├── config.rs (gateway configuration)
├── protocol.rs (registration protocol)
├── connections.rs (connection management - EXISTS)
├── heartbeat.rs (heartbeat protocol - COMPLETED in story-006-02)
├── ratelimit.rs (Discord rate limiting)
├── routing.rs (message routing)
├── registry.rs (channel registry)
├── server.rs (gateway server - EXISTS)
└── client.rs (client library for projects)
```

**Existing implementations:**
- `src/gateway/connections.rs` - manages active connections with project_id, session_id, subscription info
- `src/gateway/heartbeat.rs` - handles heartbeat protocol (COMPLETED)

## Implementation Plan

1. **Examine** existing `src/gateway/connections.rs` - Understand current connection management
2. **Examine** `src/gateway/heartbeat.rs` - Understand heartbeat implementation (story-006-02)
3. **Design** reconnection module:
   - Create `src/gateway/reconnection.rs` with retry logic
   - Implement exponential backoff using `tokio::time`
   - Track retry attempts per project

4. **Implement** reconnection logic:
   - Add reconnection handler to connections.rs
   - Implement `tokio::sync::watch` for state updates
   - Handle reconnection with same session_id preservation

5. **Configure** retry parameters:
   - Initial delay: 1 second
   - Max delay: 60 seconds
   - Max retries: 10 (configurable)
   - Multiplier: 2x

6. **Integrate** with gateway server:
   - Add reconnection handling to `src/gateway/server.rs`
   - Handle WebSocket close events triggering reconnection

7. **Test** the implementation:
   - Test exponential backoff timing
   - Test session preservation on reconnect
   - Test failure after max retries
   - Test concurrent reconnection attempts

### Skills to Read

- `./skills/rust-engineer/SKILL.md` — Core Rust patterns
- `./skills/rust-engineer/references/async.md` — Async/await with tokio
- `./skills/rust-engineer/references/error-handling.md` — Error handling patterns

### Dependencies

- story-006-01 (Project Connections) — Must be complete first
- story-006-02 (Heartbeat Protocol) — Must be complete first (provides heartbeat infrastructure)

## Scope Boundaries

### This Story Includes
- Exponential backoff reconnection logic
- Session preservation on reconnect
- Max retry handling with failure status
- Integration with existing connection management

### This Story Does NOT Include
- Reconnection UI or status commands (defer to future)
- Automatic subscription restore after prolonged disconnect
- Project authentication during reconnection

### Files in Scope

- `src/gateway/reconnection.rs` — create (new module)
- `src/gateway/connections.rs` — modify (add reconnection support)
- `src/gateway/server.rs` — modify (integrate reconnection)
- `src/gateway/mod.rs` — modify (export new module)

### Files NOT in Scope

- `src/cli/commands/gateway.rs` — don't modify
- `src/discord/gateway.rs` — don't modify
- `src/discord/listener.rs` — don't modify
