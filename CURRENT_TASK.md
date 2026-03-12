# Current Task

**Milestone:** 7 — Derived Metrics (Consumer Layer)
**Milestone ID:** M7
**Task type:** code
**Attempt:** 1
**Date:** 2026-03-11

## Objective

Implement the consumer layer for derived metrics computation. This involves creating a metrics aggregator that reads events from the event log and computes throughput and reliability metrics.

## Success Criteria

- [ ] Throughput metrics computed: agent runs, productive runs, commits, lines inserted/deleted
- [ ] Reliability metrics computed: failure rate, timeouts, skipped runs, empty runs
- [ ] Per-agent breakdown grouping works
- [ ] Metrics computation tests pass

## Context

### Workspace
- Project type: Rust application with observability features
- Tech stack: Rust, tokio, serde_json
- Key files: src/observability/, switchboard.toml

### Previous Attempts
- M6 (Configuration Integration) just completed with PASS verdict
- All prior milestones (M1-M6) are COMPLETE

### Known Patterns
- Pre-existing implementations may already exist in codebase
- STRICT TDD approach required per goals.md

## Scope Boundaries

**DO:**
- Implement MetricsAggregator struct to read event logs
- Compute throughput metrics: agent runs, productive runs, productive run rate, commits, lines inserted/deleted, files changed, avg run duration, avg commits per run
- Compute reliability metrics: container failures, failure rate, timeouts, skipped runs, empty runs, scheduler uptime
- Implement per-agent breakdown grouping
- Write comprehensive tests for metrics computation

**DO NOT:**
- Do NOT work on any milestone other than M7
- Do NOT modify files outside src/observability/ (consumer layer)
- Do NOT add features from other milestones

## Evidence Requirements

Before writing EXECUTION_REPORT.md, you MUST:
1. Run `git diff --stat` and include the output
2. Run `cargo build` and paste the output
3. Run `cargo test` and paste the output
4. Show any new tests created

## Relevant Skills

- rust-engineer/SKILL.md
- rust-best-practices/SKILL.md
