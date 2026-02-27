# IMPROVEMENT_BACKLOG.md

> Last Audit: 2026-02-27T16:08:00Z
> Commit Audited: 6eb7e957
> Health Trend: degrading (11 total findings)

## Summary

| Severity | Count | Change |
|----------|-------|--------|
| Critical | 4     | -14    |
| High     | 3     | -6     |
| Medium   | 3     | -8     |
| Low      | 1     | -7     |

## Active Findings

[Priority ordered - highest score first]

### FIND-001 — Clippy Lint Errors (10 violations)

- **Category:** Skill Violation
- **Severity:** High
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 11/22
- **Skill:** `./skills/rust-best-practices/SKILL.md` §performance
- **Files:** 
  - `src/docker/mod.rs:38` (unused variable)
  - `src/discord/gateway.rs:68` (dead code)
  - `src/config/env_ext.rs:15` (needless_borrow)
  - `src/discord/gateway.rs:195` (redundant_pattern_matching)
  - `src/discord/tools.rs:750` (redundant_closure)
  - `src/docker/skills.rs:110,121` (needless_return)
  - `src/docker/mod.rs:975,996` (io_other_error)
  - `src/ui/colors.rs:27` (should_implement_trait)
- **Description:** 10 clippy lint errors that violate the skill requirement to run clippy and fix warnings.
- **Evidence:**
```
cargo clippy -- -D warnings
error: unused variable `executor`
   --> src/docker/mod.rs:38
error: dead code `from_u8`
   --> src/discord/gateway.rs:68
[... 8 more errors ...]
```
- **Suggested Fix:** Run `cargo clippy` and fix each warning individually.
- **Status:** OPEN

### FIND-002 — God Module: docker/run/run.rs (5,115 lines)

- **Category:** Structure
- **Severity:** Critical
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 9/22
- **Files:** `src/docker/run/run.rs`
- **Description:** Single file with 5,115 lines handling container execution, streaming, and wait logic. Violates single responsibility principle.
- **Evidence:**
```
src/docker/run/run.rs: 5115 lines
- Contains Docker execution logic
- Contains streaming logic  
- Contains wait/timeout logic
- Contains entrypoint generation
```
- **Suggested Fix:** Split into focused modules: executor.rs, streamer.rs, wait.rs, entrypoint.rs
- **Status:** OPEN (RECURRING)

### FIND-003 — Test Failures (25 failing tests)

- **Category:** Structure
- **Severity:** Critical
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 14/22
- **Files:** `src/docker/run/run.rs`, `src/skills/mod.rs`, `src/discord/config.rs`
- **Description:** 25 tests failing (4.1% failure rate). Root cause appears to be skills-related fixture issues.
- **Evidence:**
```
FAILED docker::run::run::tests::test_entrypoint_script_generation_all_scenarios
FAILED docker::run::run::tests::test_integration_complete_container_config_building_with_skills
[... 23 more failures ...]
```
- **Suggested Fix:** Investigate test fixtures and mock setup for skills-related tests.
- **Status:** OPEN

### FIND-004 — Unwrap in Production Code (Critical)

- **Category:** Error Handling
- **Severity:** Critical
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 14/22
- **Skill:** `./skills/rust-best-practices/SKILL.md` §error-handling
- **Files:** 
  - `src/docker/mod.rs:130` (.strip_prefix().unwrap())
  - `src/docker/mod.rs:139` (.strip_prefix().unwrap())
  - `src/config/mod.rs:1404` (timeout.unwrap())
- **Description:** Multiple .unwrap() calls in production code that can panic on malformed input.
- **Evidence:**
```rust
// src/docker/mod.rs:130
let path = socket_path.strip_prefix("unix://").unwrap();

// src/docker/mod.rs:139  
let path = socket_path.strip_prefix("npipe://").unwrap();
```
- **Suggested Fix:** Replace with proper error handling using ? operator or expect() with descriptive message.
- **Status:** OPEN

### FIND-005 — Unsafe Code Without Safety Documentation

- **Category:** Skill Violation
- **Severity:** Critical
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 16/22
- **Skill:** `./skills/rust-engineer/SKILL.md` §unsafe
- **Files:** 
  - `src/logging.rs:81-110`
  - `src/cli/mod.rs:383-389`
- **Description:** Unsafe blocks without safety documentation violate skill requirement.
- **Evidence:**
```rust
// src/logging.rs:81
unsafe {
    INIT.call_once(|| {
        GLOBAL_LOG_DIR.as_ref().unwrap().as_path()
    });
};

// src/cli/mod.rs:383
unsafe {
    match libc::kill(pid as libc::pid_t, 0) {
        ...
    }
}
```
- **Suggested Fix:** Add SAFETY comments explaining invariants.
- **Status:** OPEN

### FIND-006 — Formatting Issues (15+ files)

- **Category:** Structure
- **Severity:** Medium
- **Effort:** S
-
- **Priority Score:** 10/22
- **Files:** Multiple files
- **Description:** cargo fmt found formatting issues in **Risk:** Safe 15+ files.
- **Evidence:**
```
src/cli/mod.rs
src/commands/build.rs
src/commands/list.rs
src/commands/metrics.rs
[... 11 more ...]
```
- **Suggested Fix:** Run `cargo fmt` to auto-fix.
- **Status:** OPEN

### FIND-007 — God Module: config/mod.rs (3,503 lines)

- **Category:** Structure
- **Severity:** High
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 7/22
- **Files:** `src/config/mod.rs`
- **Description:** Large config module handling env, toml parsing, validation.
- **Suggested Fix:** Split into env.rs, toml.rs, validation.rs
- **Status:** OPEN (RECURRING)

### FIND-008 — Orphan File: architect/git_executor.rs

- **Category:** Dead Code
- **Severity:** Medium
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 10/22
- **Files:** `src/architect/git_executor.rs`
- **Description:** File exists but module not declared in lib.rs - appears unused.
- **Suggested Fix:** Either integrate into lib.rs or remove.
- **Status:** OPEN

### FIND-009 — Manual Error Implementation (thiserror not used)

- **Category:** Skill Violation
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Skill:** `./skills/rust-best-practices/SKILL.md` §error-handling
- **Files:** 
  - `src/skills/error.rs`
  - `src/config/mod.rs:70`
- **Description:** SkillsError and ConfigError manually implement Display instead of using thiserror.
- **Evidence:**
```rust
// src/skills/error.rs
#[derive(Debug, Clone, PartialEq)]
pub enum SkillsError {
    NpxNotFound,
    // ... manually implemented Display
}
```
- **Suggested Fix:** Refactor to use thiserror derive macro.
- **Status:** OPEN

### FIND-010 — Unnecessary .clone() Calls

- **Category:** Duplication
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Skill:** `./skills/rust-best-practices/SKILL.md` §borrowing
- **Files:** 
  - `src/logger/terminal.rs:277`
  - `src/scheduler/mod.rs:1173`
- **Description:** Several unnecessary .clone() calls that could use references.
- **Evidence:**
```rust
// src/logger/terminal.rs:277
String::from_utf8(buffer.clone()).unwrap()

// src/scheduler/mod.rs:1173
self.queue_wait_times.lock().unwrap().clone()
```
- **Suggested Fix:** Use references instead of cloning.
- **Status:** OPEN

### FIND-011 — Missing API Documentation

- **Category:** Documentation
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 8/22
- **Files:** 
  - `src/skills/mod.rs:155` (create_npx_command)
  - `src/skills/mod.rs:460` (SkillsSearchResponse)
- **Description:** Two public items lack doc comments.
- **Suggested Fix:** Add doc comments.
- **Status:** OPEN

## Recently Resolved

[Include findings from previous backlog that have been addressed]

### FIND-PREV-002 — Panic in docker/skills.rs
- **Resolved:** 2026-02-27 (via commit 0a92b35b)
- **Resolution:** Panic calls replaced with proper error handling

### FIND-PREV-005 — Swallowed Errors
- **Resolved:** 2026-02-27 (via commit 215fef8b)
- **Resolution:** Fixed with proper warning logs

### FIND-PREV-012 — Unused Imports
- **Resolved:** 2026-02-27 (via commits cc7d3627, c8207928, f046985e)
- **Resolution:** Removed unused imports and variables
