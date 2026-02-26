# Bug Fix Tasks
> Generated from BUGS.md on 2026-02-20T16:00:00Z
> Total tasks: 4

## Task 1: Timeout Validation Improvements
- **Bugs:** BUG-005, BUG-006 (related — same module)
- **Files to modify:** `src/docker/run/wait/timeout.rs`
- **Estimate:** M (High + Medium = ~60 min)
- **Priority:** High
- **Notes:** Both bugs are in the same timeout.rs file and related to timeout parsing and handling logic. BUG-005 (zero timeout) should be fixed first since it's a simple check, then BUG-006 (overflow checking) which requires more careful handling. Both issues affect the reliability of timeout handling across the codebase.

---

## Task 2: File System Error Handling Improvements
- **Bugs:** BUG-007, BUG-008 (related — file system operations)
- **Files to modify:** `src/logger/file.rs`, `src/metrics/store.rs`
- **Estimate:** M (Medium + Medium = ~90 min)
- **Priority:** Medium
- **Notes:** BUG-007 addresses disk space exhaustion handling in logger writes. BUG-008 addresses concurrent write protection for metrics file. Both are related to file I/O operations and data integrity. Independent fixes that can be done in parallel or sequence without dependencies between them.

---

## Task 3: Code Cleanup and Validation Improvements
- **Bugs:** DEAD-001, BUG-010 (related — cleanup and validation)
- **Files to modify:** `src/scheduler/mod.rs`, `src/docker/run/run.rs`, `src/cli/mod.rs`, `src/config/mod.rs`
- **Estimate:** S (Small + Small = ~45 min)
- **Priority:** Medium
- **Notes:** DEAD-001 is a dead code cleanup task (remove unused function). BUG-010 is a validation improvement (add duplicate agent name checking). Independent fixes that can be done in parallel. BUG-010 requires modifying config validation to check for duplicates before creating containers.

---

## Task 4: Test Expectation Fixes
- **Bugs:** TEST-001 (Docker-dependent test expectations)
- **Files to modify:** `tests/cli_validate.rs`
- **Estimate:** S (Small = ~15 min)
- **Priority:** Low
- **Notes:** This is a test suite fix, not a functional bug. Update test assertions to expect "Docker connection error" instead of "Docker connection failed" and update tests expecting success to expect failure when Docker is unavailable.

---

## Task 5: Code Quality Improvements
- **Bugs:** POTENTIAL-001 (unwrap() in production code)
- **Files to modify:** `src/docker/mod.rs`
- **Estimate:** XS (< 15 min)
- **Priority:** Low
- **Notes:** Replace `unwrap()` with `.expect()` to provide better error context. This is a theoretical improvement since the code is currently safe given the prior string check.

---

## Notes

### Fix Dependencies
- **Task 1** can be done independently of other tasks
- **Task 2** can be done independently of other tasks
- **Task 3** has no dependencies on other tasks
- **Task 4** has no dependencies on other tasks
- **Task 5** has no dependencies on other tasks

### Recommended Fix Order
1. **Task 4 (Test fixes)** - Quick fix, unblocks tests for validation
2. **Task 1 (Timeout validation)** - High priority, fixes BUG-005 and BUG-006
3. **Task 3 (Cleanup and validation)** - Medium priority, clean up dead code and add validation
4. **Task 2 (File system handling)** - Medium priority, data integrity fixes
5. **Task 5 (Code quality)** - Low priority, polish existing code

### Risk Assessment
- **Low risk:** All fixes are localized, non-breaking changes
- **Test coverage:** Adding tests for duplicate agent name validation would improve coverage
- **Backward compatibility:** All changes maintain backward compatibility

### Estimated Total Time
- **Task 1:** ~60 min
- **Task 2:** ~90 min
- **Task 3:** ~45 min
- **Task 4:** ~15 min
- **Task 5:** ~15 min
- **Total:** ~225 minutes (~3.75 hours)
