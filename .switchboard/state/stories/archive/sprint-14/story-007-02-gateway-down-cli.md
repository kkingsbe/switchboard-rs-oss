# Story: story-007-02 - Gateway Down CLI

## Metadata

- **Story ID**: story-007-02
- **Title**: Gateway Down CLI Command
- **Epic**: Epic 007 - Gateway CLI Commands
- **Points**: 2
- **Type**: feature
- **Risk Level**: Low
- **Status**: Implemented

---

## User Story

As an operator, I want to stop the gateway server using a CLI command so that I can cleanly shut down the gateway service.

---

## Acceptance Criteria

1. CLI command `gateway down` is available
2. Command reads PID file to find running gateway process
3. Command sends SIGTERM to gracefully stop the gateway
4. Command supports `--timeout` argument (default: 30 seconds)
5. Command supports `--force` flag to send SIGKILL if graceful shutdown fails
6. Command returns appropriate error if gateway is not running
7. Command cleans up PID file after successful shutdown
8. Command handles stale PID files correctly

**Test Methods**:
- `cargo test gateway::tests` passes
- CLI accepts valid arguments
- Error handling works for not-running scenario

---

## Technical Context

### Architecture References

The gateway down command is part of the CLI commands module, following the pattern established by `gateway up` and `gateway status`.

### Existing Code

- Gateway CLI: `src/cli/commands/gateway.rs`
- PID management: `src/gateway/pid.rs`
- Process signaling: Unix signals via `kill` command

---

## Implementation Plan

1. Add `Down(GatewayDownArgs)` variant to `GatewaySubcommand` enum
2. Create `GatewayDownArgs` struct with:
   - `timeout: u64` (default: 30)
   - `force: bool` (default: false)
3. Implement `run_gateway_down()` function:
   - Check if PID file exists
   - Read PID from file
   - Verify process is running using `kill -0`
   - Send SIGTERM signal
   - Wait for process to exit (with timeout)
   - If timeout and force=true, send SIGKILL
   - Clean up PID file
4. Add error handling for:
   - Gateway not running
   - Signal sending failure
   - Timeout waiting for shutdown
5. Add unit tests

---

## Skills to Read

- [Rust Engineer](../../skills/rust-engineer/SKILL.md)
- [Rust Best Practices](../../skills/rust-best-practices/SKILL.md)
- [Async Reference](../../skills/rust-engineer/references/async.md)

---

## Dependencies

- `src/gateway/pid.rs` for PID file operations
- `tokio` for async operations
- `clap` for CLI argument parsing

---

## Scope Boundaries

### In Scope
- Graceful shutdown via SIGTERM
- Force kill via SIGKILL with `--force` flag
- Timeout handling
- PID file cleanup

### Out of Scope
- Windows support (Unix-only signals)
- Restart functionality
- Multiple gateway instances

---

## Files in Scope

| File | Description |
|------|-------------|
| `src/cli/commands/gateway.rs` | CLI command implementation |
| `src/gateway/pid.rs` | PID file management |
| `src/cli/commands/mod.rs` | Module exports |

---

## CLI Usage

```bash
# Graceful shutdown (waits up to 30 seconds)
switchboard gateway down

# Custom timeout
switchboard gateway down --timeout 60

# Force kill if graceful shutdown fails
switchboard gateway down --force

# Force kill with custom timeout
switchboard gateway down --timeout 10 --force
```

---

## Error Handling

| Scenario | Behavior |
|----------|----------|
| No PID file | Return error "Gateway is not running" |
| Process doesn't exist | Clean up stale PID file, return success |
| SIGTERM fails | Return error with details |
| Timeout (no --force) | Return "Gateway did not exit in time" |
| Timeout (with --force) | Send SIGKILL, return success |
