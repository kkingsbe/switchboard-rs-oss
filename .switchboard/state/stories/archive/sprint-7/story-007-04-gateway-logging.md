# Story 007-04: Gateway Logging

> Epic: Epic 07 — CLI Integration
> Points: 2
> Sprint: 9
> Type: infrastructure
> Risk: low
> Created: 2026-03-03

## User Story

As an operator, I want detailed logs from the gateway, So that I can troubleshoot issues.

## Acceptance Criteria

1. Log gateway startup with configuration
   - **Test:** Startup info logged - verify configuration values appear in logs

2. Log project connections/disconnections
   - **Test:** Connection events logged - verify connect/disconnect events in logs

3. Log Discord events (connection, reconnection, errors)
   - **Test:** Discord events visible in logs - verify Discord gateway events logged

4. Log to file in addition to stdout
   - **Test:** Log file created - verify `.switchboard/gateway.log` is created

## Technical Context

### Architecture Reference
- Use existing `crate::logging` module which provides file logging to `.switchboard/logs/switchboard.log`
- Use `tracing` for structured logging throughout gateway modules
- Log file location: `.switchboard/gateway.log`

### Project Conventions
- Build: `cargo build --features "discord gateway"`
- Test: Verify log output contains expected messages

### Existing Code Context
```
src/
├── logging.rs (existing logging infrastructure)
├── gateway/
│   ├── server.rs (has tracing already)
│   ├── registry.rs (has tracing for register/unregister)
│   └── routing.rs (has tracing for message routing)
└── discord/
    └── gateway.rs (Discord gateway connection)
```

Existing tracing usage in gateway modules:
- [`server.rs`](src/gateway/server.rs:24) - imports `tracing::{debug, error, info, warn}`
- [`registry.rs`](src/gateway/registry.rs:11) - logs project registration
- [`routing.rs`](src/gateway/routing.rs:9) - logs message routing
- [`logging.rs`](src/logging.rs:42) - provides `init_logging()` for file output

## Implementation Plan

1. **Add** gateway-specific logging configuration:
   - Create/extend logging setup to write to `.switchboard/gateway.log`
   - Configure log level for gateway modules

2. **Add tracing** to gateway modules for:
   - Server startup/shutdown with configuration
   - Project connections (connect/disconnect events)
   - Discord gateway events (connect, reconnect, errors)

3. **Modify** `src/gateway/server.rs`:
   - Add startup logging with config values
   - Add graceful shutdown logging

4. **Modify** `src/gateway/registry.rs`:
   - Ensure connection events are logged

5. **Modify** `src/discord/gateway.rs`:
   - Add Discord event logging

6. **Write tests** for log output verification

7. **Run** `cargo build --features "discord gateway"` and tests

### Skills to Read
- `skills/rust-best-practices/SKILL.md` — logging patterns, tracing
- `skills/rust-engineer/SKILL.md` — structured logging

### Dependencies
- story-004-01 (Module Structure) — assumed complete

## Scope Boundaries

### This Story Includes
- Gateway startup/shutdown logging
- Project connection/disconnection logging
- Discord event logging
- File logging to gateway.log

### This Story Does NOT Include
- Log rotation
- Log parsing/analysis tools
- Metrics collection

### Files in Scope
- `src/gateway/server.rs` — modify (add startup logging)
- `src/gateway/registry.rs` — modify (ensure connection logging)
- `src/discord/gateway.rs` — modify (add Discord event logging)
- `src/logging.rs` — modify (add gateway log file)

### Files NOT in Scope
- `src/cli/` — don't modify
- `src/discord/listener.rs` — don't modify
