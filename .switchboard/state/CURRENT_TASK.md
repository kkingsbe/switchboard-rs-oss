# Current Task

**Milestone:** 4 — Git Diff Capture
**Milestone ID:** M4
**Task type:** code
**Attempt:** 1
**Date:** 2026-03-11

## Objective

Implement git diff capture functionality that records HEAD hash before container launch, captures HEAD hash after container exits, and emits a `git.diff` event with commit information when there are new commits.

## Success Criteria

- [ ] HEAD hash recorded before container launch
- [ ] HEAD hash captured after container exits
- [ ] git.log output parsed into structured commit data
- [ ] Edge case handled: no commits made
- [ ] Unit tests for git diff parsing pass

## Context

### Workspace
This is an existing Rust project (switchboard) with established event infrastructure from M1-M3. The project uses:
- Rust with async/Tokio runtime
- Serde for JSON serialization
- Existing event emission patterns in `src/scheduler/mod.rs`

### Previous Attempts
This is the first attempt for M4.

### Relevant Patterns from M1-M3
- Event types are defined in `src/lib.rs` (EventData enum)
- Events are emitted via `event_emitter.emit()` pattern
- Tests follow TDD approach with comprehensive unit and integration tests

## Scope Boundaries

**DO:**
- Implement GitDiff event type in the EventData enum (following pattern from M1-M3)
- Record HEAD hash before container launch in the scheduler
- Record HEAD hash after container exits
- Parse `git log {before}..{after} --format="%H|%s" --numstat --no-merges` output
- Emit `git.diff` event with structured commit data
- Handle edge case where no commits were made (commit_count: 0)
- Write unit tests for git diff parsing

**DO NOT:**
- Do NOT work on any milestone other than M4
- Do NOT modify files unrelated to git diff capture
- Do NOT add features from other milestones (M5-M7)
- Do NOT change existing event types from M1-M3 unless adding to them

## Evidence Requirements

Before writing EXECUTION_REPORT.md, you MUST:
1. Run `git diff --stat` and include the output
2. Run `cargo build` and paste the output  
3. Run `cargo test` and paste the output
4. Show the git diff event implementation with relevant code snippets

## Relevant Skills

- **TDD Comprehensive Tests** - Required approach per milestone specification. Write tests before implementing.
- **Rust Testing Best Practices** from `skills/rust-best-practices/references/chapter_05.md` - For testing patterns
- **Event Emission Patterns** from M1-M3 - Follow same patterns for consistency
