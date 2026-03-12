# Current Task

**Milestone:** 7 — Derived Metrics (Consumer Layer)
**Milestone ID:** M7
**Task type:** code
**Attempt:** 1
**Date:** 2026-03-11T17:42:00Z

## Objective

Implement the consumer layer for derived metrics computation. Create a metrics aggregator that reads events from the event log and computes throughput and reliability metrics per observability_design_spec.md lines 238-270.

## Success Criteria

- [ ] Throughput metrics computed: agent runs, productive runs, commits, lines inserted/deleted
- [ ] Reliability metrics computed: failure rate, timeouts, skipped runs, empty runs
- [ ] Per-agent breakdown grouping works
- [ ] Metrics computation tests pass

## Context

### Workspace

- **Project Type:** Existing Rust project (v0.1.0)
- **Tech Stack:** Rust (edition 2021), tokio, serde_json
- **Key Files:**
  - `src/observability/emitter.rs` - EventEmitter implementation
  - `src/main.rs` - Main application entry point
  - `observability_design_spec.md` - Design specification (lines 238-270)

### Previous Attempts

This is the first attempt for M7. Previous milestones M1-M6 all completed successfully with PASS verdicts. M6 (Configuration Integration) verification was completed with PASS verdict.

### Known Patterns from Reflexion Memory

1. **TDD is mandatory** - All completed milestones used test-driven development with comprehensive test suites (35+ tests for M1, 21 for M3, 14 for M6)
2. **Pre-existing implementation pattern** - M4-M6 had existing code to verify. Check if metrics code already exists.
3. **Scope discipline** - Verifiers check that only observability-related files are modified. Keep M7 work focused on consumer layer only.
4. **Milestone reference accuracy** - Ensure any implementation for M7 correctly references [M7] in all commits/reports.

## Scope Boundaries

**DO:**
- Implement MetricsAggregator struct to read event logs
- Compute throughput metrics: agent runs, productive runs, productive run rate, commits, lines inserted/deleted, files changed, avg run duration, avg commits per run
- Compute reliability metrics: container failures, failure rate, timeouts, skipped runs, empty runs, scheduler uptime
- Implement per-agent breakdown grouping
- Write unit tests for metrics computation

**DO NOT:**
- Do NOT work on any milestone other than M7
- Do NOT modify files outside src/observability/ (consumer layer)
- Do NOT add features from M8 or other milestones
- Do NOT change the event JSON format or schema

## Evidence Requirements

Before writing EXECUTION_REPORT.md, you MUST:
1. Run `git diff --stat` and include the output
2. Run `cargo build` and paste the output
3. Run `cargo test` and paste the output (tests should pass)
4. If claiming any metrics, show tool output

The verifier cross-checks ALL claims. Be explicit about what gets modified.

## Relevant Skills

- skills/rust-best-practices/ - For Rust implementation patterns
- skills/rust-engineer/ - For async and testing patterns
- skills/rust-engineer/references/testing.md - For testing patterns

---

**Note:** The derived metrics specification is in observability_design_spec.md lines 238-270:

### Throughput & Velocity
- Agent runs: Count of container.started events in window
- Productive runs: Count of container.exited events where git.diff has commit_count > 0
- Productive run rate: Productive runs / agent runs
- Commits: Sum of git.diff.data.commit_count
- Lines inserted/deleted: Sum of git.diff.data.total_insertions/deletions
- Files changed: Sum of git.diff.data.total_files_changed
- Avg run duration: Mean of container.exited.data.duration_seconds
- Avg commits per run: Commits / productive runs

### Reliability
- Container failures: Count of container.exited where exit_code != 0
- Failure rate: Container failures / total container.exited
- Timeouts: Count of container.exited where timeout_hit == true
- Skipped runs: Count of container.skipped events
- Empty runs: Count of runs where commit_count == 0 AND exit_code == 0
- Scheduler uptime: Time from first scheduler.started to last event in window

### Per-Agent Breakdown
All metrics also computed per agent (grouped by agent field).
