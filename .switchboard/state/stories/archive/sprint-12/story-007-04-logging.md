# Story 007-04: Gateway Logging

> Epic: epic-07 — Discord Gateway - CLI Integration & Monitoring
> Points: 2
> Sprint: 12
> Type: feature
> Risk: low
> Created: 2026-03-04
> Status: ready
> Assigned To: dev-1

## User Story

As an operator, I want detailed logs from the gateway, So that I can troubleshoot issues.

## Acceptance Criteria

1. Log gateway startup with configuration
   - **Test:** Startup info logged - Verify gateway startup logs show configuration details

2. Log project connections/disconnections
   - **Test:** Connection events logged - Verify connection and disconnection events appear in logs

3. Log Discord events (connection, reconnection, errors)
   - **Test:** Discord events visible in logs - Verify Discord connection, reconnection, and error events are logged

4. Log to file in addition to stdout
   - **Test:** Log file created - Verify log file exists in .switchboard/logs/ directory

## Technical Context

### Architecture Reference

From architecture.md:
- Gateway uses tracing for structured logging
- Logs should be both stdout (for container) and file (for debugging)
- Log levels: ERROR, WARN, INFO, DEBUG, TRACE

### Project Conventions

From project-context.md:
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Use `thiserror` for error types
- Use `tracing` for logging (already in use)
- Never use `unwrap()` in production

### Existing Code Context

```
src/gateway/
├── mod.rs (gateway module entry)
├── config.rs (gateway configuration)
├── protocol.rs (registration protocol)
├── connections.rs (connection management)
├── pid.rs (PID file management)
└── server.rs (gateway server - EXISTS)
```

**Existing logging:**
- `src/logging.rs` exists with logging setup
- Gateway should integrate with existing logging infrastructure

## Implementation Plan

1. **Examine** existing `src/logging.rs` - Understand current logging setup
2. **Enhance** gateway logging:
   - Add structured logging with tracing spans
   - Log startup with configuration (log level, port, etc.)
   - Log project connection/disconnection events
   - Log Discord events (connect, reconnect, error)
   - Add file logging in addition to stdout

3. **Integrate** logging into gateway components:
   - Add logging to `src/gateway/server.rs`
   - Add logging to `src/gateway/connections.rs`
   - Add logging to `src/gateway/protocol.rs`

4. **Configure** log file output:
   - Create log directory: .switchboard/logs/
   - Log file format: gateway-{date}.log
   - Configure rotation if needed

5. **Test** the implementation:
   - Run gateway and verify startup logs
   - Test connection logging
   - Verify log file creation

### Skills to Read

- `./skills/rust-engineer/SKILL.md` — Core Rust patterns
- `./skills/rust-engineer/references/async.md` — Async/await with tokio

### Dependencies

- story-004-01 (gateway module structure) — Must be complete first
- story-006-01 (project connections) — Should be complete first (for connection logging)

## Scope Boundaries

### This Story Includes
- Gateway startup logging
- Project connection/disconnection logging
- Discord event logging
- File logging setup
- Integration with existing logging infrastructure

### This Story Does NOT Include
- Log rotation (defer to future)
- Log parsing/monitoring tools
- Metrics/observability integration
- Debug command for log viewing

### Files in Scope

- `src/gateway/server.rs` — modify (add logging)
- `src/gateway/connections.rs` — modify (add logging)
- `src/gateway/protocol.rs` — modify (add logging)
- `src/gateway/mod.rs` — modify (configure logging)
- `src/logging.rs` — modify (if needed for file setup)

### Files NOT in Scope

- `src/cli/commands/gateway.rs` — don't modify
- `src/discord/gateway.rs` — don't modify
- `src/discord/listener.rs` — don't modify
- Log viewing commands — defer to future
