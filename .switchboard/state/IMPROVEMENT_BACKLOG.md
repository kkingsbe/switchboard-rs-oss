# IMPROVEMENT_BACKLOG.md

> Last Audit: 2026-02-27T15:12:04Z
> Commit Audited: 488988e67aa876ca7cfac823be40982c702367ce
> Health Trend: degrading (Critical: 18, High: 9, Medium: 11, Low: 8)

## Summary

| Severity | Count |
|----------|-------|
| Critical | 18 |
| High | 9 |
| Medium | 11 |
| Low | 8 |

## Active Findings

### FIND-001 — Build Failure: Syntax Error in traits/mod.rs
- **Category:** Structure
- **Severity:** Critical
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 22/22
- **Files:** [`src/traits/mod.rs:17`](src/traits/mod.rs:17)
- **Description:** Invalid Rust syntax: `use std::io::self;` - self imports are only allowed within a { } list
- **Evidence:** `src/traits/mod.rs:17` - `use std::io::self;`
- **Suggested Fix:** Change to `use std::io;`
- **Status:** SCHEDULED
- **Scheduled:** Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO2.md

### FIND-002 — panic!() in Production Code
- **Category:** Error Handling
- **Severity:** Critical
- **Effort:** S
- **Risk:** Medium
- **Priority Score:** 20/22
- **Skill:** rust-best-practices
- **Files:** [`src/docker/skills.rs:111`](src/docker/skills.rs:111), [`src/docker/skills.rs:119`](src/docker/skills.rs:119), [`src/logging.rs:87`](src/logging.rs:87)
- **Description:** Direct `panic!()` calls in production code can cause application crashes on invalid input.
- **Evidence:** `panic!("Invalid skill format: {}", skill)` and `panic!("Failed to create log directory: {}", log_dir.display())`
- **Suggested Fix:** Replace `panic!()` with proper error return using `Result<(), SkillsError>` or similar error type
- **Status:** SCHEDULED
- **Scheduled:** Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO2.md

### FIND-003 — .expect()/.unwrap() in Production Code (Mutex Locks)
- **Category:** Error Handling
- **Severity:** Critical
- **Effort:** M
- **Risk:** Medium
- **Priority Score:** 18/22
- **Skill:** rust-engineer
- **Files:** [`src/scheduler/mod.rs:1164`](src/scheduler/mod.rs:1164), [`src/scheduler/mod.rs:1173`](src/scheduler/mod.rs:1173), [`src/scheduler/mod.rs:1182`](src/scheduler/mod.rs:1182), [`src/docker/run/streams.rs:86`](src/docker/run/streams.rs:86)
- **Description:** Using `.unwrap()` on mutex locks can panic if the lock is poisoned. 4 instances found.
- **Evidence:** `*self.queue_wait_time_seconds.lock().unwrap()` and `logger.lock().unwrap().write_agent_log(...)`
- **Suggested Fix:** Use `match` or `if let` to handle poisoned lock errors gracefully
- **Status:** OPEN

### FIND-004 — .expect() in Production Code (Docker Client)
- **Category:** Error Handling
- **Severity:** Critical
- **Effort:** M
- **Risk:** Medium
- **Priority Score:** 18/22
- **Skill:** rust-best-practices
- **Files:** [`src/docker/mod.rs:901-1168`](src/docker/mod.rs:901) (14 instances)
- **Description:** 14 instances of `.expect("Docker client not available")` in production code can panic if Docker is not initialized.
- **Evidence:** `self.docker.as_ref().expect("Docker client not available")` at multiple lines
- **Suggested Fix:** Replace `.expect()` with proper error handling - return `Result<T, DockerError>` or use `if let`/`match`
- **Status:** OPEN

### FIND-005 — Swallowed Errors with let _ =
- **Category:** Error Handling
- **Severity:** High
- **Effort:** S
- **Risk:** Low
- **Priority Score:** 18/22
- **Skill:** rust-engineer
- **Files:** [`src/docker/run/streams.rs:82`](src/docker/run/streams.rs:82), [`src/cli/mod.rs:901-928`](src/cli/mod.rs:901), [`src/skills/mod.rs:1477`](src/skills/mod.rs:1477), [`src/discord/security.rs:362`](src/discord/security.rs:362), [`src/docker/mod.rs:883`](src/docker/mod.rs:883)
- **Description:** Using `let _ = ...` silently discards errors, hiding failures from file operations, signal handling, and logging.
- **Evidence:** `let _ = logger_guard.write_terminal_output(&message);`, `let _ = tokio::signal::ctrl_c().await;`
- **Suggested Fix:** Propagate errors using `?` operator or log warnings for non-critical failures
- **Status:** SCHEDULED
- **Scheduled:** Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO1.md and .switchboard/state/REFACTOR_TODO2.md

### FIND-006 — .ok() Silently Defaulting Errors
- **Category:** Error Handling
- **Severity:** High
- **Effort:** S
- **Risk:** Low
- **Priority Score:** 18/22
- **Skill:** rust-best-practices
- **Files:** [`src/discord/llm.rs:457-460`](src/discord/llm.rs:457), [`src/discord/api.rs:272-305`](src/discord/api.rs:272)
- **Description:** Using `.ok()` silently converts errors to default values, hiding configuration errors and rate limit parsing failures.
- **Evidence:** `.and_then(|s| s.timezone.parse::<Tz>().ok())` and rate limit header parsing
- **Suggested Fix:** Replace `.ok()` with proper error propagation using `?` to maintain error context
- **Status:** SCHEDULED
- **Scheduled:** Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO1.md

### FIND-007 — God Module: docker/run/run.rs
- **Category:** Complexity
- **Severity:** Critical
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 16/22
- **Files:** [`src/docker/run/run.rs:5115`](src/docker/run/run.rs:1)
- **Description:** Single file with 5115 lines violates single responsibility principle and is extremely difficult to maintain.
- **Evidence:** File contains 5115 lines of code
- **Suggested Fix:** Split into focused submodules: container creation, stream handling, wait strategies, cleanup logic
- **Status:** OPEN

### FIND-008 — God Module: config/mod.rs
- **Category:** Complexity
- **Severity:** Critical
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 16/22
- **Files:** [`src/config/mod.rs:3503`](src/config/mod.rs:1)
- **Description:** Single file with 3503 lines handles configuration loading, validation, parsing, and environment handling.
- **Evidence:** File contains 3503 lines of code
- **Suggested Fix:** Extract validation logic, parsing logic, and environment handling into separate modules
- **Status:** OPEN

### FIND-009 — God Module: skills/mod.rs
- **Category:** Complexity
- **Severity:** Critical
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 16/22
- **Files:** [`src/skills/mod.rs:2717`](src/skills/mod.rs:1)
- **Description:** Single file with 2717 lines contains core skills management functionality.
- **Evidence:** File contains 2717 lines of code
- **Suggested Fix:** Split into skill discovery, installation, validation, and execution modules
- **Status:** OPEN

### FIND-010 — God Module: cli/mod.rs
- **Category:** Complexity
- **Severity:** Critical
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 16/22
- **Files:** [`src/cli/mod.rs:2123`](src/cli/mod.rs:1)
- **Description:** Single file with 2123 lines contains CLI argument parsing and command handling.
- **Evidence:** File contains 2123 lines of code
- **Suggested Fix:** Extract command handlers into separate modules in src/commands/
- **Status:** OPEN

### FIND-011 — God Module: commands/skills.rs
- **Category:** Complexity
- **Severity:** Critical
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 16/22
- **Files:** [`src/commands/skills.rs:2067`](src/commands/skills.rs:1)
- **Description:** Single file with 2067 lines handles skills CLI commands.
- **Evidence:** File contains 2067 lines of code
- **Suggested Fix:** Split into subcommands: install, list, remove, update
- **Status:** OPEN

### FIND-012 — God Module: discord/tools.rs
- **Category:** Complexity
- **Severity:** High
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 13/22
- **Files:** [`src/discord/tools.rs:1616`](src/discord/tools.rs:1)
- **Description:** Single file with 1616 lines contains Discord tool implementations.
- **Evidence:** File contains 1616 lines of code
- **Suggested Fix:** Split into separate tool implementations
- **Status:** OPEN

### FIND-013 — God Module: discord/llm.rs
- **Category:** Complexity
- **Severity:** High
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 13/22
- **Files:** [`src/discord/llm.rs:1530`](src/discord/llm.rs:1)
- **Description:** Single file with 1530 lines contains LLM integration logic.
- **Evidence:** File contains 1530 lines of code
- **Suggested Fix:** Extract LLM client, prompt handling, and response parsing
- **Status:** OPEN

### FIND-014 — God Module: docker/skills.rs
- **Category:** Complexity
- **Severity:** High
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 13/22
- **Files:** [`src/docker/skills.rs:1484`](src/docker/skills.rs:1)
- **Description:** Single file with 1484 lines contains Docker-specific skills operations.
- **Evidence:** File contains 1484 lines of code
- **Suggested Fix:** Consolidate with skills/mod.rs or extract Docker skill operations
- **Status:** OPEN

### FIND-015 — God Module: commands/validate.rs
- **Category:** Complexity
- **Severity:** High
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 13/22
- **Files:** [`src/commands/validate.rs:1420`](src/commands/validate.rs:1)
- **Description:** Single file with 1420 lines contains validation commands.
- **Evidence:** File contains 1420 lines of code
- **Suggested Fix:** Extract validation logic into shared module
- **Status:** OPEN

### FIND-016 — God Module: docker/mod.rs
- **Category:** Complexity
- **Severity:** High
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 13/22
- **Files:** [`src/docker/mod.rs:1377`](src/docker/mod.rs:1)
- **Description:** Single file with 1377 lines contains Docker client operations.
- **Evidence:** File contains 1377 lines of code
- **Suggested Fix:** Split into client, container, image, network modules
- **Status:** OPEN

### FIND-017 — Non-thiserror Error Types
- **Category:** Skill Violation
- **Severity:** High
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 16/22
- **Skill:** rust-best-practices
- **Files:** [`src/skills/error.rs:34`](src/skills/error.rs:34), [`src/traits/mod.rs:686`](src/traits/mod.rs:686), [`src/config/mod.rs:70`](src/config/mod.rs:70)
- **Description:** 3 custom error types not using `thiserror::Error` derive, causing inconsistent error handling.
- **Evidence:** `#[derive(Debug, Clone, PartialEq)]` instead of `#[derive(thiserror::Error)]`
- **Suggested Fix:** Migrate error types to use `thiserror::Error` derive macro
- **Status:** OPEN

### FIND-018 — Unused Imports/Variables (Clippy)
- **Category:** Dead Code
- **Severity:** High
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 16/22
- **Files:** [`src/skills/mod.rs:2409`](src/skills/mod.rs:2409), [`src/discord/gateway.rs:68`](src/discord/gateway.rs:68), [`src/docker/mod.rs:40`](src/docker/mod.rs:40), [`src/discord/tools.rs:878`](src/discord/tools.rs:878)
- **Description:** 4+ unused items identified by clippy: unused imports, unused variables, dead code.
- **Evidence:** `async_trait::async_trait` unused, `executor` unused variable, `backlog_path` unused (x2)
- **Suggested Fix:** Remove unused imports/variables or add `#[allow(dead_code)]` if intentionally kept
- **Status:** SCHEDULED
- **Scheduled:** Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO1.md

### FIND-019 — Test Failures: 26 tests failing
- **Category:** Structure
- **Severity:** Critical
- **Effort:** M
- **Risk:** Medium
- **Priority Score:** 16/22
- **Files:** [`tests/`](tests/)
- **Description:** 26 out of 608 tests failing (4.3% failure rate). Categories: Skills (17), Config (3), Docker (1), Validate (2), Mock (2).
- **Evidence:** Skills-related tests fail because test skills don't exist in ./skills/, config parsing issues
- **Suggested Fix:** Fix test setup/teardown, ensure test skills exist, fix environment variable handling in tests
- **Status:** OPEN

### FIND-020 — Clippy Linter Errors: 31+ violations
- **Category:** Structure
- **Severity:** Critical
- **Effort:** M
- **Risk:** Safe
- **Priority Score:** 16/22
- **Files:** Multiple files
- **Description:** 31+ clippy errors including: unused_imports, unused_variables, dead_code, clippy::needless_borrow, clippy::redundant_pattern_matching
- **Evidence:** Running `cargo clippy --all-targets --all-features -- -D warnings` fails
- **Suggested Fix:** Fix each clippy warning - remove unused imports, use proper borrowing, fix pattern matching
- **Status:** SCHEDULED
- **Scheduled:** Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO2.md

### FIND-021 — Debug Print in Production Code
- **Category:** Documentation
- **Severity:** Medium
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 14/22
- **Files:** [`src/docker/mod.rs:453`](src/docker/mod.rs:453)
- **Description:** DEBUG print statement in production code: `eprintln!("DEBUG: create_build_context_tarball about to return")`
- **Evidence:** `eprintln!("DEBUG: create_build_context_tarball about to return")`
- **Suggested Fix:** Remove debug print or replace with proper tracing/logging
- **Status:** OPEN

### FIND-022 — Broken Module Reference in lib.rs
- **Category:** Documentation
- **Severity:** Medium
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 14/22
- **Files:** [`src/lib.rs:19`](src/lib.rs:19), [`src/architect/git_executor.rs:8`](src/architect/git_executor.rs:8)
- **Description:** lib.rs references non-existent `architect` module, and git_executor.rs has broken imports.
- **Evidence:** `src/lib.rs:19` mentions `architect` module but file doesn't exist
- **Suggested Fix:** Either create the architect module or remove the reference from lib.rs
- **Status:** OPEN

### FIND-023 — String Instead of &str Parameters
- **Category:** Skill Violation
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 12/22
- **Skill:** rust-best-practices
- **Files:** [`src/docker/mod.rs:578`](src/docker/mod.rs:578), [`src/discord/api.rs:33`](src/discord/api.rs:33), [`src/docker/run/types.rs:139`](src/docker/run/types.rs:139), [`src/discord/gateway.rs:166`](src/discord/gateway.rs:166), [`src/commands/validate.rs:131`](src/commands/validate.rs:131)
- **Description:** 5 instances of `String` parameters where `&str` would be more idiomatic.
- **Evidence:** `pub async fn new(image_name: String, image_tag: String)` and similar
- **Suggested Fix:** Change parameters to `&str` to avoid unnecessary allocations
- **Status:** OPEN

### FIND-024 — Excessive .clone() Usage
- **Category:** Skill Violation
- **Severity:** Medium
- **Effort:** L
- **Risk:** Low
- **Priority Score:** 10/22
- **Skill:** rust-best-practices
- **Files:** [`src/scheduler/mod.rs`](src/scheduler/mod.rs), [`src/cli/mod.rs`](src/cli/mod.rs), [`src/docker/run/run.rs`](src/docker/run/run.rs), [`src/discord/mod.rs`](src/discord/mod.rs)
- **Description:** 159 instances of unnecessary `.clone()` calls cause unnecessary heap allocations.
- **Evidence:** `container_id.clone()`, `agent_name.clone()`, `config.prompt.clone()`
- **Suggested Fix:** Use references `&T` where ownership transfer is not required
- **Status:** OPEN

### FIND-025 — God Module: scheduler/mod.rs
- **Category:** Complexity
- **Severity:** Medium
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 10/22
- **Files:** [`src/scheduler/mod.rs:1259`](src/scheduler/mod.rs:1)
- **Description:** Single file with 1259 lines contains scheduling logic.
- **Evidence:** File contains 1259 lines of code
- **Suggested Fix:** Extract clock, scheduling algorithms, and job management
- **Status:** OPEN

### FIND-026 — God Module: metrics/store.rs
- **Category:** Complexity
- **Severity:** Medium
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 10/22
- **Files:** [`src/metrics/store.rs:1091`](src/metrics/store.rs:1)
- **Description:** Single file with 1091 lines contains metrics storage logic.
- **Evidence:** File contains 1091 lines of code
- **Suggested Fix:** Extract storage backends and query logic
- **Status:** OPEN

### FIND-027 — God Module: discord/config.rs
- **Category:** Complexity
- **Severity:** Medium
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 10/22
- **Files:** [`src/discord/config.rs:1091`](src/discord/config.rs:1)
- **Description:** Single file with 1091 lines contains Discord configuration handling.
- **Evidence:** File contains 1091 lines of code
- **Suggested Fix:** Split config loading, validation, and management
- **Status:** OPEN

### FIND-028 — Unsafe Code Without Safety Documentation
- **Category:** Skill Violation
- **Severity:** Medium
- **Effort:** S
- **Risk:** Medium
- **Priority Score:** 10/22
- **Skill:** rust-engineer
- **Files:** [`src/logging.rs:81`](src/logging.rs:81), [`src/cli/mod.rs:383`](src/cli/mod.rs:383)
- **Description:** Unsafe code blocks lack safety documentation comments.
- **Evidence:** `unsafe { INIT.call_once(...) }` and `unsafe { libc::kill(pid, 0) }`
- **Suggested Fix:** Add safety comments explaining invariants for each unsafe block
- **Status:** OPEN

### FIND-029 — Redundant Dependencies
- **Category:** Dependency
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 10/22
- **Files:** [`Cargo.toml`](Cargo.toml)
- **Description:** serde_derive is redundant - serde already re-exports it. Twilight crates outdated (v0.17 vs v0.20+).
- **Evidence:** Both `serde` and `serde_derive` listed in dependencies
- **Suggested Fix:** Remove serde_derive from dependencies, consider updating Twilight crates
- **Status:** OPEN

### FIND-030 — Code Duplication Patterns
- **Category:** Complexity
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 8/22
- **Files:** [`src/config/env.rs`](src/config/env.rs), [`src/skills/mod.rs`](src/skills/mod.rs), [`src/commands/list.rs`](src/commands/list.rs), [`src/commands/metrics.rs`](src/commands/metrics.rs)
- **Description:** Table preset loading, file reading patterns, status icon logic duplicated across files.
- **Evidence:** `.load_preset(comfy_table::presets::UTF8_FULL)` repeated in list.rs, metrics.rs, skills.rs
- **Suggested Fix:** Extract common patterns into shared utility functions
- **Status:** OPEN

### FIND-031 — Inconsistent Error Type Implementations
- **Category:** Error Handling
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 8/22
- **Files:** Multiple error types across codebase
- **Description:** 13 custom error types with inconsistent derive implementations.
- **Evidence:** Mix of thiserror::Error, std::error::Error, and manual implementations
- **Suggested Fix:** Standardize all error types to use thiserror::Error
- **Status:** OPEN

### FIND-032 — Formatting Issues
- **Category:** Structure
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 10/22
- **Files:** 50+ files need formatting
- **Description:** Multiple files need formatting per cargo fmt rules.
- **Evidence:** Files like src/cli/mod.rs, src/commands/build.rs, src/discord/config.rs need formatting
- **Suggested Fix:** Run `cargo fmt` to fix formatting issues
- **Status:** OPEN

### FIND-033 — Unused Allow(dead_code) Annotations
- **Category:** Dead Code
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 10/22
- **Files:** [`src/config/mod.rs:846`](src/config/mod.rs:846), [`src/discord/gateway.rs:113`](src/discord/gateway.rs:113)
- **Description:** Dead code annotations on unused functions/structs indicate incomplete cleanup.
- **Evidence:** `#[allow(dead_code)]` on read_prompt_file and DiscordGateway
- **Suggested Fix:** Remove unused code or remove the allow annotation if code is intentionally kept
- **Status:** OPEN

### FIND-034 — Cron Format Documentation Inconsistency
- **Category:** Documentation
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 10/22
- **Files:** [`README.md:188-197`](README.md:188), [`docs/configuration.md:164`](docs/configuration.md:164)
- **Description:** README shows 6-field cron examples while configuration.md describes 5-field format.
- **Evidence:** Inconsistent documentation between README and configuration docs
- **Suggested Fix:** Clarify in docs/configuration.md that both 5-field and 6-field cron expressions are supported
- **Status:** OPEN

---

## Recently Resolved

The following findings from the previous audit (2026-02-27T04:44:10Z) have been resolved:

| Finding | Previous Status | Resolution |
|---------|-----------------|------------|
| FIND-001 (old): Bollard API Incompatibility | SCHEDULED | RESOLVED - Windows named pipe API issue appears to be resolved |
| FIND-002 (old): Merge Conflicts in Test Files | SCHEDULED | RESOLVED - Merge conflict markers no longer present |
| FIND-014 (old): Broken Documentation Links | SCHEDULED | RESOLVED - All documentation links are now valid |

---

## Priority Order Summary

| Priority | Finding | Score |
|----------|---------|-------|
| 1 | FIND-001: Build Failure (syntax error) | 22 |
| 2 | FIND-002: panic!() in Production | 20 |
| 3 | FIND-003: Mutex .unwrap() | 18 |
| 4 | FIND-004: Docker Client .expect() | 18 |
| 5 | FIND-005: Swallowed Errors (let _ =) | 18 |
| 6 | FIND-006: .ok() Silent Defaults | 18 |
| 7 | FIND-007: God Module run.rs | 16 |
| 8 | FIND-008: God Module config/mod.rs | 16 |
| 9 | FIND-009: God Module skills/mod.rs | 16 |
| 10 | FIND-010: God Module cli/mod.rs | 16 |
| 11 | FIND-011: God Module commands/skills.rs | 16 |
| 12 | FIND-017: Non-thiserror Errors | 16 |
| 13 | FIND-018: Unused Imports/Variables | 16 |
| 14 | FIND-019: Test Failures (26) | 16 |
| 15 | FIND-020: Clippy Errors (31+) | 16 |
| 16 | FIND-012: God Module discord/tools.rs | 13 |
| 17 | FIND-013: God Module discord/llm.rs | 13 |
| 18 | FIND-014: God Module docker/skills.rs | 13 |
| 19 | FIND-015: God Module commands/validate.rs | 13 |
| 20 | FIND-016: God Module docker/mod.rs | 13 |
| 21 | FIND-021: Debug Print in Production | 14 |
| 22 | FIND-022: Broken Module Reference | 14 |
| 23 | FIND-023: String vs &str Parameters | 12 |
| 24 | FIND-024: Excessive .clone() | 10 |
| 25 | FIND-025: God Module scheduler/mod.rs | 10 |
| 26 | FIND-026: God Module metrics/store.rs | 10 |
| 27 | FIND-027: God Module discord/config.rs | 10 |
| 28 | FIND-028: Unsafe Without Safety Docs | 10 |
| 29 | FIND-029: Redundant Dependencies | 10 |
| 30 | FIND-032: Formatting Issues | 10 |
| 31 | FIND-033: Unused allow(dead_code) | 10 |
| 32 | FIND-030: Code Duplication | 8 |
| 33 | FIND-031: Inconsistent Error Types | 8 |
| 34 | FIND-034: Cron Doc Inconsistency | 10 |

---

## Recommended Action Plan

### Immediate (This Sprint)
1. **FIX-001**: Fix syntax error in src/traits/mod.rs:17 - Unblocks build
2. **FIX-002**: Fix 26 test failures - Unblocks CI
3. **FIX-003**: Fix 31+ clippy errors - Unblocks CI
4. **FIX-004**: Replace panic!() calls in production (FIND-002)
5. **FIX-005**: Fix .expect()/.unwrap() on mutex locks (FIND-003)

### Short-term (Next 2-4 Weeks)
6. **FIX-006**: Replace .expect() on Docker client (FIND-004)
7. **FIX-007**: Fix swallowed errors with let _ = (FIND-005)
8. **FIX-008**: Fix .ok() silent defaults (FIND-006)
9. **FIX-009**: Clean up unused imports/variables (FIND-018)
10. **FIX-010**: Remove debug print (FIND-021)
11. **FIX-011**: Fix broken module reference (FIND-022)

### Medium-term (1-2 Months)
12. **FIX-012**: Refactor god modules (FIND-007 through FIND-011) - Start with top 5
13. **FIX-013**: Migrate remaining error types to thiserror (FIND-017)
14. **FIX-014**: Change String to &str parameters (FIND-023)
15. **FIX-015**: Reduce .clone() usage (FIND-024)

### Long-term (Backlog)
16. Refactor remaining god modules (FIND-012 through FIND-027)
17. Add safety documentation to unsafe blocks (FIND-028)
18. Clean up redundant dependencies (FIND-029)
19. Remove code duplication (FIND-030)
20. Standardize error types (FIND-031)
21. Run formatting pass (FIND-032)
