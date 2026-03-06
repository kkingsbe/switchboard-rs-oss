# Story: story-007-03 - PID File Management

## Metadata

- **Story ID**: story-007-03
- **Title**: PID File Management
- **Epic**: Epic 007 - Gateway CLI Commands
- **Points**: 1
- **Type**: infrastructure
- **Risk Level**: Low
- **Status**: Implemented

---

## User Story

As an operator, I want the gateway to manage a PID file so that I can check if the gateway is running and avoid starting multiple instances.

---

## Acceptance Criteria

1. Gateway writes PID file on startup at `.switchboard/gateway.pid`
2. Gateway checks for existing PID file before starting
3. If PID file exists but process is not running (stale), allow startup
4. If PID file exists and process is running, return error
5. Gateway removes PID file on graceful shutdown
6. Default PID path is configurable
7. PID file contains only the process ID

**Test Methods**:
- `PidFile::write_pid()` creates file with correct PID
- `PidFile::check_existing()` returns Ok for non-existent file
- `PidFile::check_existing()` returns AlreadyRunning for active process
- `PidFile::check_existing()` returns Ok for stale PID file
- `PidFile::cleanup()` removes existing file

---

## Technical Context

### Architecture References

The PID file module provides process tracking for the gateway server, used by the CLI commands to manage gateway lifecycle.

### Existing Code

- PID module: `src/gateway/pid.rs`
- Gateway up: `src/cli/commands/gateway.rs` (run_gateway_up)
- Gateway down: `src/cli/commands/gateway.rs` (run_gateway_down)
- Gateway status: `src/cli/commands/gateway.rs` (run_gateway_status)

---

## Implementation Plan

1. Create `PidFile` struct with static methods
2. Implement `PidFile::write_pid(path)`:
   - Create parent directories if needed
   - Write current process ID to file
3. Implement `PidFile::check_existing(path)`:
   - Return Ok if file doesn't exist
   - Read PID from file
   - Check if process exists using `kill -0` (Unix)
   - Return AlreadyRunning if process exists
   - Return Ok if stale (process not running)
4. Implement `PidFile::cleanup(path)`:
   - Remove PID file if exists
   - Silently succeed if file doesn't exist
5. Implement `PidFile::default_path()` returning `.switchboard/gateway.pid`
6. Add comprehensive unit tests

---

## Skills to Read

- [Rust Engineer](../../skills/rust-engineer/SKILL.md)
- [Rust Best Practices](../../skills/rust-best-practices/SKILL.md)

---

## Dependencies

- `libc` for Unix process existence check (conditional compilation)

---

## Scope Boundaries

### In Scope
- Writing PID file on startup
- Checking for existing gateway process
- Cleaning up PID file on shutdown
- Stale PID file handling

### Out of Scope
- Windows process checking
- PID file locking mechanisms
- Automatic cleanup on abnormal termination (SIGKILL)

---

## Files in Scope

| File | Description |
|------|-------------|
| `src/gateway/pid.rs` | PID file management implementation |
| `src/gateway/mod.rs` | Module export |

---

## Default Path

```
.switchboard/gateway.pid
```

---

## Error Types

```rust
pub enum PidFileError {
    IoError(std::io::Error),
    ParseError,
    AlreadyRunning(u32), // Contains the PID of running process
}
```

---

## Usage Flow

```
Gateway Start:
1. Check if PID file exists
2. If exists, verify process is running
3. If running, return error
4. If not running (stale), proceed
5. Write new PID file
6. Start gateway server

Gateway Stop:
1. Read PID from file
2. Send SIGTERM to process
3. Wait for graceful shutdown
4. Remove PID file
```
