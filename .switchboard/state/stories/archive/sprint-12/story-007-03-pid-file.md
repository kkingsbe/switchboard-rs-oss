# Story 007-03: PID Epic: epic- File Management

>07 — Discord Gateway - CLI Integration & Monitoring
> Points: 1
> Sprint: 12
> Type: feature
> Risk: low
> Created: 2026-03-04
> Status: ready
> Assigned To: dev-1

## User Story

As a user, I want the gateway to track its PID, So that I can manage the process externally.

## Acceptance Criteria

1. Write PID to file on start (.switchboard/gateway.pid)
   - **Test:** File created with correct PID - Verify file exists and contains the correct process ID

2. Check for existing PID on startup
   - **Test:** Error if gateway already running - Attempt to start second gateway and verify error

3. Clean up PID file on shutdown
   - **Test:** File removed on clean exit - Start gateway, stop it gracefully, verify file is removed

## Technical Context

### Architecture Reference

From architecture.md:
- Gateway runs as a background process
- PID file stored in .switchboard/ directory
- Used for process management and monitoring

### Project Conventions

From project-context.md:
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Use `thiserror` for error types
- Use `tracing` for logging
- Never use `unwrap()` in production
- Use std::fs for file operations

### Existing Code Context

```
src/gateway/
├── mod.rs (gateway module entry)
├── config.rs (gateway configuration)
├── protocol.rs (registration protocol)
├── connections.rs (connection management)
├── pid.rs (TO BE COMPLETED - this story)
└── server.rs (gateway server - EXISTS)
```

**Current state:**
- `src/gateway/pid.rs` exists with partial implementation
- Needs: error handling, startup check, cleanup on shutdown

## Implementation Plan

1. **Examine** existing `src/gateway/pid.rs` - Understand what's already implemented
2. **Enhance** PID file management:
   - Add proper error handling for file operations
   - Check for existing PID file on startup
   - Implement graceful cleanup on shutdown (SIGINT/SIGTERM)
   - Use tokio signal handling for async shutdown

3. **Integrate** with gateway server:
   - Modify `src/gateway/server.rs` to call PID management on startup/shutdown

4. **Test** the implementation:
   - Test file creation with correct PID
   - Test error when gateway already running
   - Test file cleanup on shutdown

### Skills to Read

- `./skills/rust-best-practices/SKILL.md` — Rust best practices
- `./skills/rust-engineer/references/error-handling.md` — Error handling patterns

### Dependencies

- story-004-08 (gateway up CLI command) — Should be complete first

## Scope Boundaries

### This Story Includes
- PID file creation on gateway start
- PID file check on startup (error if exists)
- PID file cleanup on graceful shutdown
- Integration with gateway server

### This Story Does NOT Include
- Gateway status command (story-007-01)
- Gateway down command (story-007-02)
- Gateway restart functionality
- Process monitoring/heartbeat

### Files in Scope

- `src/gateway/pid.rs` — modify (complete implementation)
- `src/gateway/server.rs` — modify (integrate PID management)
- `src/gateway/mod.rs` — modify (export pid module if needed)

### Files NOT in Scope

- `src/cli/commands/gateway.rs` — don't modify
- `src/discord/gateway.rs` — don't modify
- `src/gateway/protocol.rs` — don't modify
- `src/gateway/connections.rs` — don't modify
