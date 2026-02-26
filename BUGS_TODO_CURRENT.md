# Bug Fix Tasks
> Generated from BUGS_CURRENT.md on 2026-02-20T16:43:44.104Z
> Total tasks: 8

---

## Task 1: Fix Critical Cron and Docker Timeout Issues
- **Bugs:** BUG-001, BUG-002
- **Files to modify:** [`src/config/mod.rs`](src/config/mod.rs), [`src/commands/validate.rs`](src/commands/validate.rs), [`src/docker/run/wait/timeout.rs`](src/docker/run/wait/timeout.rs)
- **Estimate:** M (60-120 min)
- **Priority:** Critical
- **Notes:**
  - BUG-001: Update `validate_cron_expression()` to handle 5-field Unix cron expressions. Either auto-convert by prepending "0 " (seconds field) or return a clear error message explaining 6-field format requirement
  - BUG-002: Modify `wait_with_timeout()` to store the result of `wait_for_exit_with_docker()` before timeout expires. If timeout expired but `wait_for_exit_with_docker` returned `Err(e)`, propagate that error instead of treating as timeout
  - Sequencing: Both fixes are independent and can be done in parallel

---

## Task 2: Fix Docker Runtime Operations
- **Bugs:** BUG-006, BUG-008, BUG-009, BUG-016, BUG-018
- **Files to modify:** [`src/docker/run/run.rs`](src/docker/run/run.rs)
- **Estimate:** L (60+ min)
- **Priority:** High
- **Notes:**
  - BUG-006: Add explicit check for container existence using `inspect_container()` before attempting removal. Treat "not found" as success and log as informational message
  - BUG-008: Add error handling to the log streaming loop. On error, attempt to restart the stream with backoff. If restart fails multiple times, notify user and possibly fail the run
  - BUG-009: Implement proper signal handling that blocks on log flush before exit. Use `tokio::signal` with explicit cleanup and flush operations. Consider using buffered writers with flush on drop
  - BUG-016: In config validation, collect all agent names into a HashSet, check for duplicates, return `ConfigError::DuplicateAgentName` with clear message listing conflicting names (also requires src/config/mod.rs)
  - BUG-018: Use a synchronization primitive (e.g., tokio::sync::oneshot channel) to ensure log streaming task is ready before starting container
  - Dependencies: BUG-016 requires config module changes. Other bugs can be fixed independently

---

## Task 3: Standardize Error Handling Across Entry Points
- **Bugs:** BUG-004, BUG-005
- **Files to modify:** [`src/docker/mod.rs`](src/docker/mod.rs), [`src/config/mod.rs`](src/config/mod.rs), [`src/cli/mod.rs`](src/cli/mod.rs)
- **Estimate:** M (30-45 min)
- **Priority:** High
- **Notes:**
  - BUG-004: Create a centralized Docker client initialization function that returns a consistent error type. Audit all call sites to ensure consistent error handling
  - BUG-005: Centralize workspace validation logic into a single function and call it from all entry points (config parsing, CLI commands, runtime execution). Define clear validation criteria (existence, readability, writability) and apply consistently
  - These bugs are related (error handling consistency) and should be fixed together

---

## Task 4: Fix Logger Write and Disk I/O Error Handling
- **Bugs:** BUG-007, BUG-017
- **Files to modify:** [`src/logger/file.rs`](src/logger/file.rs), [`src/logger/terminal.rs`](src/logger/terminal.rs)
- **Estimate:** M (30-45 min)
- **Priority:** High
- **Notes:**
  - BUG-007: Return `Result` from log write operations. For critical logs, fail the run if write fails. Implement fallback logging to stderr if primary destination fails
  - BUG-017: Match on `std::io::ErrorKind::StorageFull` and return custom `FileWriteError::StorageFull` variant with actionable message: "Disk full. Cannot write to {path}. Free disk space and retry."
  - These bugs are both in the logger module and related to error handling, should be fixed together

---

## Task 5: Fix Metrics Data Integrity and Propagation
- **Bugs:** BUG-003, BUG-010, BUG-011, BUG-012, BUG-014
- **Files to modify:** [`src/scheduler/mod.rs`](src/scheduler/mod.rs), [`src/metrics/store.rs`](src/metrics/store.rs)
- **Estimate:** L (60+ min)
- **Priority:** High
- **Notes:**
  - BUG-003: Return `Result` from metrics collection functions and propagate errors to run execution context. Add a configurable "strict_metrics" mode if non-blocking behavior is desired
  - BUG-010: Add file locking using `fslock` crate or implement a mutex-protected shared metrics store for in-process operations. For cross-process safety, use `fslock` or `fcntl` file locks
  - BUG-011: Move queue_wait_time_seconds population before run execution, not just on success. Ensure it's saved even when the run fails or times out
  - BUG-012: Recalculate queue_wait_time_seconds before each save attempt, including retries. Use `chrono::Utc::now()` at save time to ensure accuracy
  - BUG-014: Log metrics errors at WARN level. Track consecutive metrics errors and alert if they persist. Add a metrics health check that reports status
  - Dependencies: BUG-010 must be done before BUG-003 to ensure safe concurrent access when propagating errors. BUG-011 and BUG-012 are related queue wait time fixes.

---

## Task 6: Fix Scheduler Queue Race Condition
- **Bugs:** BUG-013
- **Files to modify:** [`src/scheduler/mod.rs`](src/scheduler/mod.rs)
- **Estimate:** M (30-60 min)
- **Priority:** Medium
- **Notes:**
  - Implement proper queue operations using a lock-protected state machine. Mark runs as "in_progress" atomically before execution. Consider using a channel-based queue pattern
  - This is independent of other tasks and can be done in parallel

---

## Task 7: Make PID File Operations Atomic
- **Bugs:** BUG-015
- **Files to modify:** [`src/main.rs`](src/main.rs) or scheduler entry point
- **Estimate:** S (15-30 min)
- **Priority:** Medium
- **Notes:**
  - Use atomic write pattern: write PID to `switchboard.pid.tmp`, then `rename()` to `switchboard.pid`. Read operations will always see complete files
  - This is independent of other tasks and can be done in parallel

---

## Task 8: Clean Up Dead Code and Outdated Comments
- **Bugs:** BUG-019, BUG-020
- **Files to modify:** [`src/scheduler/mod.rs`](src/scheduler/mod.rs), multiple files for comment cleanup
- **Estimate:** S (15-30 min)
- **Priority:** Low
- **Notes:**
  - BUG-019: Delete the entire `suggest_cron_correction()` function including the `#[allow(dead_code)]` attribute since cron conversion is already implemented in `validate_cron_expression()`
  - BUG-020: Perform global search for TODO/FIXME/HACK comments. For each, either: complete the fix, remove if obsolete, or update with clear status and timeline
  - This should be done last after all functional fixes are complete

---

## Task Dependencies and Sequencing

### Critical Path (must do first):
1. **Task 1** (Critical: Cron & Docker Timeout) - Blocks scheduler functionality

### High Priority (parallel after Task 1):
2. **Task 2** (Docker Runtime) - Can be done in parallel with Tasks 3, 4, 5
3. **Task 3** (Error Handling) - Can be done in parallel
4. **Task 4** (Logger & Disk I/O) - Can be done in parallel
5. **Task 5** (Metrics) - Note: BUG-010 within this task must be done before BUG-003

### Medium Priority (can be done in parallel with High tasks):
6. **Task 6** (Queue Race Condition) - Independent
7. **Task 7** (PID File Atomic) - Independent

### Low Priority (do last):
8. **Task 8** (Code Cleanup) - Do after all functional fixes complete

---

## Summary

- **Total bugs to fix:** 20 (BUG-001 through BUG-020)
- **Total tasks:** 8
- **Critical bugs:** 2 (grouped in Task 1)
- **High bugs:** 10 (grouped in Tasks 2, 3, 4, 5)
- **Medium bugs:** 6 (grouped in Tasks 6, 7)
- **Low bugs:** 2 (grouped in Task 8)
- **Total estimated time:** ~7-8 hours for all tasks
