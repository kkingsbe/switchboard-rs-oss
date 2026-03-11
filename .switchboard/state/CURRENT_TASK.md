# Current Task

**Milestone:** 2 — Scheduler Events Integration
**Milestone ID:** M2
**Task type:** code
**Attempt:** 1
**Date:** 2026-03-11

## Objective

Implement scheduler lifecycle event emission for the switchboard observability system. Using STRICT test-driven development, implement the emission of `scheduler.started` and `scheduler.stopped` events that track the scheduler's uptime and lifecycle state.

## Success Criteria

- [ ] `scheduler.started` event emitted on switchboard up
- [ ] `scheduler.stopped` event emitted on graceful shutdown
- [ ] Uptime calculation tracked correctly
- [ ] Integration tests for scheduler lifecycle events pass

## Context

### Workspace
- **Project Type:** Existing Rust project (switchboard)
- **Tech Stack:** tokio 1.40, tokio-cron-scheduler 0.15, serde/serde_json
- **Key Files:**
  - `src/scheduler/` - Existing cron-based scheduling engine
  - `src/main.rs` - Entry point where `switchboard up` is invoked
  - `observability_design_spec.md` - Event schema definitions

### Event Schema (from observability_design_spec.md)

**scheduler.started:**
```json
{
  "event": "scheduler.started",
  "agent": null,
  "run_id": null,
  "data": {
    "agents": ["goal-planner", "goal-executor", ...],
    "agent_count": 4,
    "version": "0.5.0",
    "config_file": "switchboard.toml"
  }
}
```

**scheduler.stopped:**
```json
{
  "event": "scheduler.stopped",
  "agent": null,
  "run_id": null,
  "data": {
    "reason": "sigint",
    "uptime_seconds": 86400
  }
}
```

### Known Patterns

From `skills/rust-engineer/references/async.md`:
- **Lines 342-377:** Graceful shutdown patterns with `tokio::signal::ctrl_c()` - ESSENTIAL for `scheduler.stopped`
- **Lines 78-117:** `select!` patterns for handling shutdown signals alongside other async operations
- Use `Arc<Instant>` or similar to track scheduler start time for uptime calculation

## Scope Boundaries

**DO:**
- Write tests FIRST (TDD) - define expected event JSON structure
- Implement EventEmitter integration in scheduler startup
- Add graceful shutdown handler that emits `scheduler.stopped` with uptime_seconds
- Track scheduler start time using `std::time::Instant` or `tokio::time::Instant`

**DO NOT:**
- Do NOT modify M1 (Event Core Infrastructure) - already complete
- Do NOT work on M3-M7 - future milestones
- Do NOT refactor unrelated code in src/scheduler/
- Do NOT add features not in the success criteria

## Evidence Requirements

Before writing EXECUTION_REPORT.md, you MUST:
1. Run `git diff --stat` and include the output
2. Run `cargo build` and paste the output
3. Run `cargo test` and paste the output (all tests must pass)
4. Verify the events are correctly written to `.switchboard/events/events.jsonl`

## Relevant Skills

- **rust-engineer** skill - Specifically `references/async.md` for graceful shutdown patterns
- **rust-best-practices** skill - For testing best practices
