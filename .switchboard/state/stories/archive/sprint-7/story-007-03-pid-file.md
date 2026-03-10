# Story 007-03: PID File Management

> Epic: Epic 07 — CLI Integration
> Points: 1
> Sprint: 9
> Type: infrastructure
> Risk: low
> Created: 2026-03-03

## User Story

As a user, I want the gateway to track its PID, So that I can manage the process externally.

## Acceptance Criteria

1. Write PID to file on start (default: `.switchboard/gateway.pid`)
   - **Test:** File created with correct PID - verify the file exists and contains the correct PID

2. Check for existing PID on startup
   - **Test:** Error if gateway already running - verify error when PID file exists

3. Clean up PID file on shutdown
   - **Test:** File removed on clean exit - verify file is deleted after shutdown

## Technical Context

### Architecture Reference
- Use `std::fs` for PID file operations
- PID file location: `.switchboard/gateway.pid`
- Check for existing PID on startup - fail if file exists
- Remove file on graceful shutdown via signal handler

### Project Conventions
- Build: `cargo build --features "discord gateway"`
- Test: Use temp directories for testing

### Existing Code Context
```
src/gateway/
└── server.rs (main server with signal handling)
```

Current server.rs already has:
- Signal handling via `tokio::signal::ctrl_c()` at line handling
- Graceful shutdown support
- Server startup in `run()` method

The CLI gateway up command in [`src/cli/commands/gateway.rs`](src/cli/commands/gateway.rs:171) calls the server startup.

## Implementation Plan

1. **Create** `src/gateway/pid.rs` with:
   - `PidFile` struct - manages PID file lifecycle
   - `write_pid()` - create PID file on startup
   - `check_existing()` - check if gateway already running
   - `cleanup()` - remove PID file on shutdown

2. **Modify** `src/gateway/server.rs`:
   - Add PID file creation on server start
   - Add PID file cleanup on graceful shutdown

3. **Write tests** for:
   - PID file creation with correct content
   - Error on existing PID file
   - Cleanup on shutdown

4. **Run** `cargo build --features "discord gateway"` and tests

### Skills to Read
- `skills/rust-best-practices/SKILL.md` — file operations, error handling

### Dependencies
- story-004-08 (Gateway up CLI) — must be complete first

## Scope Boundaries

### This Story Includes
- PID file creation at startup
- PID file existence check
- PID file cleanup on shutdown

### This Story Does NOT Include
- Process management (killing old processes)
- PID file location configuration
- Lock file semantics

### Files in Scope
- `src/gateway/pid.rs` — create
- `src/gateway/server.rs` — modify (add PID file handling)

### Files NOT in Scope
- `src/cli/` — don't modify
- `src/discord/` — don't modify
