# Current Task

**Milestone:** 3 — Container Events Integration
**Milestone ID:** M3
**Task type:** code
**Attempt:** 1
**Date:** 2026-03-11

## Objective

Implement container lifecycle event emission for the switchboard observability system. Using STRICT test-driven development, implement the emission of `container.started`, `container.exited`, `container.skipped`, and `container.queued` events that track container lifecycle and execution results.

## Success Criteria

- [ ] `container.started` event emitted when launching containers
- [ ] `container.exited` event emitted on container completion
- [ ] Exit code, duration_seconds, timeout_hit captured
- [ ] `container.skipped` and `container.queued` events implemented
- [ ] Integration tests for container lifecycle pass

## Context

### Workspace
- **Project Type:** Existing Rust project (switchboard)
- **Tech Stack:** tokio 1.40, bollard 0.18 (Docker API), serde/serde_json
- **Key Files:**
  - `src/docker/mod.rs` - Container lifecycle management
  - `src/docker/run.rs` - Container execution logic
  - `src/scheduler/mod.rs` - Where scheduler events are implemented (reference for pattern)
  - `observability_design_spec.md` - Event schema definitions

### Event Schema (from observability_design_spec.md)

**container.started:**
```json
{
  "event": "container.started",
  "agent": "goal-executor",
  "run_id": "a1b2c3d4",
  "data": {
    "image": "kilosynth/prompter:latest",
    "trigger": "cron",
    "schedule": "*/5 * * * *",
    "container_id": "docker-container-hash"
  }
}
```

**container.exited:**
```json
{
  "event": "container.exited",
  "agent": "goal-executor",
  "run_id": "a1b2c3d4",
  "data": {
    "exit_code": 0,
    "duration_seconds": 847,
    "timeout_hit": false
  }
}
```

**container.skipped:**
```json
{
  "event": "container.skipped",
  "agent": "goal-executor",
  "run_id": null,
  "data": {
    "reason": "overlap_skip",
    "running_run_id": "a1b2c3d4"
  }
}
```

**container.queued:**
```json
{
  "event": "container.queued",
  "agent": "goal-executor",
  "run_id": "b2c3d4e5",
  "data": {
    "queue_position": 1,
    "running_run_id": "a1b2c3d4"
  }
}
```

### Run ID Generation
From the design spec, run_id should be an 8-character hex string:
```rust
fn generate_run_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:08x}", rng.gen::<u32>())
}
```

### Reference: M2 Implementation Pattern
M2 (Scheduler Events) was implemented in `src/scheduler/mod.rs`. The pattern:
1. EventEmitter is passed to the scheduler/container manager
2. Events are emitted at key lifecycle points
3. Tests verify event JSON structure and content
4. All tests use TDD approach - write tests first

### Known Patterns

From `skills/rust-engineer/references/async.md`:
- Async container operations with bollard
- Graceful shutdown handling

From custom skill `tdd-comprehensive-tests.md`:
- Write tests FIRST before implementation
- Each event type should have its own test
- Verify complete JSON structure including all fields

## Scope Boundaries

**DO:**
- Write tests FIRST (TDD) - define expected event JSON structure for each container event type
- Implement EventEmitter integration in container launch (src/docker/run.rs)
- Add event emission on container completion with exit code, duration, timeout
- Implement container.skipped for overlap_mode="skip"
- Implement container.queued for overlap_mode="queue"
- Track run_id to tie started/exited events together

**DO NOT:**
- Do NOT modify M1-M2 - already complete
- Do NOT work on M4-M7 - future milestones
- Do NOT refactor unrelated code in src/docker/
- Do NOT add features not in the success criteria
- Do NOT modify the event schema (must match observability_design_spec.md exactly)

## Evidence Requirements

Before writing EXECUTION_REPORT.md, you MUST:
1. Run `git diff --stat` and include the output
2. Run `cargo build` and paste the output
3. Run `cargo test` and paste the output (all tests must pass)
4. Verify the events are correctly written to `.switchboard/events/events.jsonl`

## Relevant Skills

- **rust-engineer** skill - Specifically `references/async.md` for async container operations
- **rust-best-practices** skill - For testing best practices
- Custom skill: **tdd-comprehensive-tests.md** - TDD approach guidance
