# Story 007-04: Proper Logging

> Epic: epic-07 — Discord Gateway Phase 4
> Points: 2
> Sprint: 8
> Type: feature
> Risk: low
> Created: 2026-03-03

## User Story

As a operator, I want detailed logs from the gateway, So that I can troubleshoot issues.

## Acceptance Criteria

1. Log gateway startup with configuration - Startup info logged
   - **Test:** Start gateway and verify startup logs appear

2. Log project connections/disconnections - Connection events logged
   - **Test:** Connect/disconnect a project and verify events are logged

3. Log Discord events (connection, reconnection, errors) - Discord events visible in logs
   - **Test:** Trigger Discord events and verify they appear in logs

4. Log to file in addition to stdout - Log file created
   - **Test:** Verify `.switchboard/gateway.log` is created and contains logs

## Technical Context

### Architecture Reference
- Uses tracing for structured logging
- Existing logging infrastructure in src/logger/
- Log file location: `.switchboard/gateway.log`

### Project Conventions
- Use tracing macros (info!, warn!, error!)
- Include context (channel_id, project_id) in log fields

### Existing Code Context
```
src/
├── logging.rs (main logging setup)
└── logger/
    └── file.rs (file logging)
```

## Implementation Plan

1. **Identify** all gateway modules that need logging
2. **Add** tracing instrumentation to:
   - `src/gateway/server.rs` - startup, connections
   - `src/gateway/registry.rs` - project events
   - Discord event handlers
3. **Configure** file output to `.switchboard/gateway.log`
4. **Run** gateway and verify logs appear in both stdout and file

### Skills to Read
- `skills/rust-engineer/SKILL.md`
- `skills/rust-best-practices/SKILL.md` — Ch1 (inspect_err pattern)

### Dependencies
- story-004-01 (Logging infrastructure) — complete

## Scope Boundaries

### This Story Includes
- Gateway-specific logging
- Startup, connection, error logging
- File log output

### This Story Does NOT Include
- Changing logging library
- Log rotation
- Structured logging format changes

### Files in Scope
- `src/gateway/server.rs` — modify
- `src/gateway/registry.rs` — modify
- `src/gateway/config.rs` — modify (if needed for log config)

### Files NOT in Scope
- `src/logging.rs` — don't modify (use existing)
- `src/logger/` — don't modify
