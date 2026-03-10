# Story: story-007-04 - Gateway Logging

## Metadata

- **Story ID**: story-007-04
- **Title**: Gateway Logging
- **Epic**: Epic 007 - Gateway CLI Commands
- **Points**: 2
- **Type**: feature
- **Risk Level**: Low
- **Status**: Implemented

---

## User Story

As an operator, I want the gateway to log to both stdout and files so that I can debug issues and monitor gateway behavior.

---

## Acceptance Criteria

1. Gateway logs to stdout with appropriate log levels
2. Gateway logs to files in `.switchboard/logs/` directory
3. Log files use daily rotation
4. Logging is initialized before gateway server starts
5. Log level is configurable via configuration file
6. Both main application and gateway-specific logs are captured
7. Log files include timestamps and log levels

**Test Methods**:
- `init_gateway_logging()` creates log directory
- Log files are created in `.switchboard/logs/`
- Multiple runs create dated log files
- Log output includes correct level indicators

---

## Technical Context

### Architecture References

The logging module uses the `tracing` crate for structured logging, with file output via `tracing_appender`.

### Existing Code

- Logging module: `src/logging.rs`
- Gateway CLI: `src/cli/commands/gateway.rs`
- Configuration: `src/gateway/config.rs` (LoggingConfig)

---

## Implementation Plan

1. Use existing `tracing` and `tracing_appender` crates
2. Implement `init_gateway_logging(log_dir)` function:
   - Create non-blocking file appender
   - Set up daily rolling file policy
   - Configure stdout subscriber
   - Return WorkerGuard to maintain logging state
3. Support log level configuration from GatewayConfig
4. Integrate with gateway CLI startup:
   - Load config
   - Initialize logging
   - Start server
5. Ensure log directory `.switchboard/logs/` is created if not exists

---

## Skills to Read

- [Rust Engineer](../../skills/rust-engineer/SKILL.md)
- [Rust Best Practices](../../skills/rust-best-practices/SKILL.md)
- [Async Reference](../../skills/rust-engineer/references/async.md)

---

## Dependencies

- `tracing` for structured logging
- `tracing_subscriber` for subscriber setup
- `tracing_appender` for file logging with rotation

---

## Scope Boundaries

### In Scope
- Stdout logging
- File logging with daily rotation
- Log level configuration
- Non-blocking operation

### Out of Scope
- Log aggregation systems
- Custom log formats
- Log retention policies (manual cleanup)

---

## Files in Scope

| File | Description |
|------|-------------|
| `src/logging.rs` | Logging initialization and configuration |
| `src/cli/commands/gateway.rs` | Gateway CLI with logging init |
| `src/gateway/config.rs` | LoggingConfig struct |

---

## Log Directory Structure

```
.switchboard/
└── logs/
    ├── gateway-2026-03-04.log
    ├── gateway-2026-03-05.log
    └── ...
```

---

## Configuration

```toml
[logging]
level = "debug"  # Options: trace, debug, info, warn, error
file = "/var/log/gateway.log"  # Optional custom path
```

---

## Implementation Notes

- Use `tracing_appender::non_blocking::WorkerGuard` to prevent data loss
- Keep guard alive for duration of program
- Default to `.switchboard/logs/` directory
- Daily rotation prevents unbounded disk growth
