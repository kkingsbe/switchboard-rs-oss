# Sprint 14 - DEV_TODO2.md

## Header

- **Sprint**: 14
- **Focus Area**: Gateway Connection Management & CLI Commands
- **Total Points**: 8
- **Developer**: Dev-2

---

## Orientation

### Key Files to Read

| File | Description |
|------|-------------|
| [`src/gateway/mod.rs`](src/gateway/mod.rs) | Gateway module declarations |
| [`src/cli/commands/gateway.rs`](src/cli/commands/gateway.rs) | Gateway CLI commands |
| [`src/gateway/pid.rs`](src/gateway/pid.rs) | PID file management |
| [`src/gateway/reconnection.rs`](src/gateway/reconnection.rs) | Reconnection logic |
| [`src/logging.rs`](src/logging.rs) | Logging initialization |
| [`src/discord/gateway.rs`](src/discord/gateway.rs) | Discord gateway |
| [`skills/rust-engineer/SKILL.md`](skills/rust-engineer/SKILL.md) | Rust engineer skills |
| [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md) | Rust best practices |
| [`skills/rust-engineer/references/async.md`](skills/rust-engineer/references/async.md) | Async reference |
| [`skills/rust-engineer/references/ownership.md`](skills/rust-engineer/references/ownership.md) | Ownership reference |

---

## Stories

### [ ] story-006-03: Reconnection Logic
- **Points**: 3
- **Story File**: [`.switchboard/state/stories/story-006-03-reconnection-logic.md`](.switchboard/state/stories/story-006-03-reconnection-logic.md)
- **Risk**: Medium
- **Skills to Read**:
  - [`skills/rust-engineer/SKILL.md`](skills/rust-engineer/SKILL.md)
  - [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md)
  - [`skills/rust-engineer/references/async.md`](skills/rust-engineer/references/async.md)
  - [`skills/rust-engineer/references/ownership.md`](skills/rust-engineer/references/ownership.md)
- **Pre-Check**: ReconnectionManager struct does not exist
- **Post-Check**:
  - Backoff delays follow exponential progression (1s, 2s, 4s, 8s...)
  - Backoff caps at max_delay
  - Manager transitions through correct states
  - Max retries causes MaxRetriesExceeded error
  - Cancellation works correctly
- **Commit Message**: `feat(gateway): implement reconnection logic with exponential backoff`

### [ ] story-007-02: Gateway Down CLI
- **Points**: 2
- **Story File**: [`.switchboard/state/stories/story-007-02-gateway-down-cli.md`](.switchboard/state/stories/story-007-02-gateway-down-cli.md)
- **Risk**: Low
- **Skills to Read**:
  - [`skills/rust-engineer/SKILL.md`](skills/rust-engineer/SKILL.md)
  - [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md)
  - [`skills/rust-engineer/references/async.md`](skills/rust-engineer/references/async.md)
- **Pre-Check**: `gateway down` command does not exist
- **Post-Check**:
  - CLI command `gateway down` is available
  - Command reads PID file to find running gateway process
  - Command sends SIGTERM to gracefully stop the gateway
  - Command supports `--timeout` and `--force` arguments
  - Command returns appropriate error if gateway is not running
  - Command cleans up PID file after successful shutdown
- **Commit Message**: `feat(cli): add gateway down CLI command`

### [ ] story-007-03: PID File Management
- **Points**: 1
- **Story File**: [`.switchboard/state/stories/story-007-03-pid-file-management.md`](.switchboard/state/stories/story-007-03-pid-file-management.md)
- **Risk**: Low
- **Skills to Read**:
  - [`skills/rust-engineer/SKILL.md`](skills/rust-engineer/SKILL.md)
  - [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md)
- **Pre-Check**: PidFile struct does not exist in src/gateway/pid.rs
- **Post-Check**:
  - `PidFile::write_pid()` creates file with correct PID
  - `PidFile::check_existing()` returns correct states
  - `PidFile::cleanup()` removes existing file
  - Stale PID file handling works correctly
- **Commit Message**: `feat(gateway): implement PID file management`

### [ ] story-007-04: Gateway Logging
- **Points**: 2
- **Story File**: [`.switchboard/state/stories/story-007-04-gateway-logging.md`](.switchboard/state/stories/story-007-04-gateway-logging.md)
- **Risk**: Low
- **Skills to Read**:
  - [`skills/rust-engineer/SKILL.md`](skills/rust-engineer/SKILL.md)
  - [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md)
  - [`skills/rust-engineer/references/async.md`](skills/rust-engineer/references/async.md)
- **Pre-Check**: Gateway logging to files not initialized in CLI
- **Post-Check**:
  - Gateway logs to stdout with appropriate log levels
  - Gateway logs to files in `.switchboard/logs/` directory
  - Log files use daily rotation
  - Logging is initialized before gateway server starts
  - Log level is configurable via configuration file
- **Commit Message**: `feat(gateway): implement gateway logging with file rotation`

---

## AGENT QA

### Story Completion Checklist

- [ ] All acceptance criteria met for each story
- [ ] Code compiles without warnings (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] No unwrap() calls in production code
- [ ] Proper error handling implemented
- [ ] Documentation updated as needed

### Notes

- Ensure reconnection logic handles edge cases gracefully
- Follow Unix signal handling best practices for gateway down
- Use proper file locking for PID file operations
- Ensure logging doesn't block the main async runtime

---

*Generated: Sprint 14 - Dev-2*
