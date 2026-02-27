# IMPROVEMENT_BACKLOG.md

> Last Audit: 2026-02-27T04:44:10Z
> Commit Audited: 488988e67aa876ca7cfac823be40982c702367ce
> Health Trend: degrading

## Summary

| Severity | Count | 
|----------|-------|
| Critical | 17 |
| High | 8 |
| Medium | 14 |
| Low | 8 |

## Active Findings

### FIND-001 — Build Failure: Bollard API Incompatibility
- **Category:** Structure
- **Severity:** Critical
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 22/22
- **Files:** [`src/docker/mod.rs:140`](src/docker/mod.rs:140)
- **Description:** Function `connect_with_named_pipe_defaults` does not exist in bollard 0.18.1. The Windows-specific Docker connection API has changed, causing compilation failure.
- **Evidence:** `src/docker/mod.rs:140` - function call to non-existent bollard API
- **Suggested Fix:** Update the Docker client initialization to use the new bollard API for Windows named pipe connections, or use cross-platform connection method
- **Status:** SCHEDULED - Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO1.md

### FIND-002 — Merge Conflicts in Test Files
- **Category:** Structure
- **Severity:** Critical
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 22/22
- **Files:** [`tests/mod.rs:8-12`](tests/mod.rs:8), [`tests/performance_common.rs:1-2457`](tests/performance_common.rs:1), [`tests/skills_install_performance.rs:39-65`](tests/skills_install_performance.rs:39), [`tests/skills_install_time_metrics.rs:10`](tests/skills_install_time_metrics.rs:10), [`tests/skills_list_performance.rs:46-80`](tests/skills_list_performance.rs:46)
- **Description:** Unresolved merge conflicts in test files block the build and prevent tests from running.
- **Evidence:** Merge conflict markers (<<<<<<<, =======, >>>>>>>) present in test files
- **Suggested Fix:** Resolve merge conflicts in all affected test files - keep incoming changes for performance tests or remove conflicts if tests are no longer needed
- **Status:** SCHEDULED - Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO1.md

### FIND-003 — panic!() in Production Code
- **Category:** Error Handling
- **Severity:** Critical
- **Effort:** S
- **Risk:** Medium
- **Priority Score:** 20/22
- **Skill:** rust-best-practices
- **Files:** [`src/docker/skills.rs:111`](src/docker/skills.rs:111), [`src/docker/skills.rs:119`](src/docker/skills.rs:119)
- **Description:** Direct `panic!()` calls in production code can cause application crashes on invalid input.
- **Evidence:** `panic!("Invalid skill format: {}", skill)`
- **Suggested Fix:** Replace `panic!()` with proper error return using `Result<(), SkillsError>` or similar error type
- **Status:** SCHEDULED - Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO2.md

### FIND-004 — .expect() in Production Code
- **Category:** Error Handling
- **Severity:** Critical
- **Effort:** M
- **Risk:** Medium
- **Priority Score:** 18/22
- **Skill:** rust-best-practices
- **Files:** [`src/docker/mod.rs:901-1162`](src/docker/mod.rs:901), [`src/cli/mod.rs:348`](src/cli/mod.rs:348), [`src/scheduler/mod.rs:1257`](src/scheduler/mod.rs:1257), [`src/logging.rs:44`](src/logging.rs:44), [`src/logging.rs:59`](src/logging.rs:59), [`src/config/mod.rs:41`](src/config/mod.rs:41)
- **Description:** 17 instances of `.expect()` in production code can panic if unwrapped value is None/Err.
- **Evidence:** Multiple `.expect("Docker client not available")` calls in docker/mod.rs
- **Suggested Fix:** Replace `.expect()` calls with proper error handling using `Result<T, Error>` and the `?` operator
- **Status:** OPEN

### FIND-005 — Swallowed Errors with .ok()
- **Category:** Error Handling
- **Severity:** High
- **Effort:** S
- **Risk:** Low
- **Priority Score:** 18/22
- **Skill:** rust-engineer
- **Files:** [`src/commands/list.rs:45`](src/commands/list.rs:45), [`src/commands/list.rs:78`](src/commands/list.rs:78), [`src/discord/llm.rs:457-460`](src/discord/llm.rs:457), [`src/discord/api.rs:272-305`](src/discord/api.rs:272)
- **Description:** Using `.ok()` silently converts errors to default values, hiding configuration errors and making debugging difficult.
- **Evidence:** `value_str.parse().ok()?` and `.and_then(|s| s.timezone.parse::<Tz>().ok())`
- **Suggested Fix:** Replace `.ok()` with proper error propagation to maintain error context
- **Status:** SCHEDULED - Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO2.md

### FIND-006 — God Module: docker/run/run.rs
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

### FIND-007 — God Module: config/mod.rs
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

### FIND-008 — God Module: skills/mod.rs
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

### FIND-009 — God Module: cli/mod.rs
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

### FIND-010 — God Module: commands/skills.rs
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

### FIND-011 — Non-thiserror Error Types
- **Category:** Skill Violation
- **Severity:** High
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 16/22
- **Skill:** rust-best-practices
- **Files:** [`src/skills/error.rs:34`](src/skills/error.rs:34), [`src/traits/mod.rs:686`](src/traits/mod.rs:686), [`src/config/mod.rs:70`](src/config/mod.rs:70), [`src/discord/tools.rs:23`](src/discord/tools.rs:23), [`src/discord/llm.rs:52`](src/discord/llm.rs:52), [`src/discord/api.rs:362`](src/discord/api.rs:362), [`src/logger/terminal.rs:16`](src/logger/terminal.rs:16), [`src/logger/file.rs:24`](src/logger/file.rs:24)
- **Description:** 8 custom error types not using `thiserror::Error` derive, causing inconsistent error handling and extra boilerplate.
- **Evidence:** `#[derive(Debug, Clone, PartialEq)]` instead of `#[derive(thiserror::Error)]`
- **Suggested Fix:** Migrate error types to use `thiserror::Error` derive macro
- **Status:** OPEN

### FIND-012 — Excessive .clone() Usage
- **Category:** Skill Violation
- **Severity:** High
- **Effort:** L
- **Risk:** Low
- **Priority Score:** 14/22
- **Skill:** rust-best-practices
- **Files:** [`src/scheduler/mod.rs`](src/scheduler/mod.rs), [`src/cli/mod.rs`](src/cli/mod.rs), [`src/docker/run/run.rs`](src/docker/run/run.rs), [`src/discord/mod.rs`](src/discord/mod.rs)
- **Description:** 159 instances of unnecessary `.clone()` calls cause unnecessary heap allocations.
- **Evidence:** `container_id.clone()`, `agent_name.clone()`, `config.prompt.clone()`
- **Suggested Fix:** Use references `&T` where ownership transfer is not required
- **Status:** OPEN

### FIND-012 — Unused Imports
- **Category:** Dead Code
- **Severity:** High
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 16/22
- **Files:** [`src/discord/gateway.rs:8`](src/discord/gateway.rs:8), [`src/discord/tools.rs:14`](src/discord/tools.rs:14), [`src/traits/mod.rs:17`](src/traits/mod.rs:17), [`src/docker/mod.rs:40`](src/docker/mod.rs:40)
- **Description:** 4 unused imports create noise and may indicate incomplete refactoring.
- **Evidence:** Unused imports: `error` in gateway.rs, `error`, `info` in tools.rs, `std::io::Write` in traits/mod.rs
- **Suggested Fix:** Remove unused imports or use them
- **Status:** SCHEDULED - Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO2.md

### FIND-014 — Broken Documentation Links
- **Category:** Documentation
- **Severity:** Medium
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 16/22
- **Files:** [`docs/README.md`](docs/README.md), [`docs/installation.md`](docs/installation.md)
- **Description:** 6 broken links in documentation reference files that don't exist.
- **Evidence:** Links to quickstart.md, cli.md, env-vars.md, CONTRIBUTING.md, ARCHITECTURE.md
- **Suggested Fix:** Create missing documentation files or remove broken links
- **Status:** SCHEDULED - Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO1.md

### FIND-015 — God Module: discord/tools.rs
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

### FIND-016 — God Module: discord/llm.rs
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

### FIND-017 — God Module: docker/skills.rs
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

### FIND-018 — God Module: commands/validate.rs
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

### FIND-019 — God Module: docker/mod.rs
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

### FIND-020 — Cron Format Documentation Inconsistency
- **Category:** Documentation
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 13/22
- **Files:** [`README.md:188-197`](README.md:188), [`docs/configuration.md:164`](docs/configuration.md:164)
- **Description:** README shows 6-field cron examples while configuration.md describes 5-field format.
- **Evidence:** Inconsistent documentation between README and configuration docs
- **Suggested Fix:** Clarify in docs/configuration.md that both 5-field and 6-field cron expressions are supported
- **Status:** SCHEDULED - Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO1.md

### FIND-021 — Missing API Documentation
- **Category:** Documentation
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 13/22
- **Files:** [`src/traits/mod.rs:778`](src/traits/mod.rs:778), [`src/scheduler/clock.rs:24`](src/scheduler/clock.rs:24), [`src/docker/mod.rs:542`](src/docker/mod.rs:542)
- **Description:** 3 public types lack complete documentation.
- **Evidence:** RealProcessExecutor, SystemClock, DockerClient lack full doc comments
- **Suggested Fix:** Add comprehensive doc comments to public types
- **Status:** SCHEDULED - Improvement Sprint 1, assigned to .switchboard/state/REFACTOR_TODO2.md

### FIND-022 — Unused Allow(dead_code) Annotations
- **Category:** Dead Code
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 13/22
- **Files:** [`src/config/mod.rs:846`](src/config/mod.rs:846), [`src/discord/gateway.rs:113`](src/discord/gateway.rs:113)
- **Description:** Dead code annotations on unused functions/structs indicate incomplete cleanup.
- **Evidence:** `#[allow(dead_code)]` on read_prompt_file and DiscordGateway
- **Suggested Fix:** Remove unused code or remove the allow annotation if code is intentionally kept
- **Status:** OPEN

### FIND-023 — God Module: scheduler/mod.rs
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

### FIND-024 — God Module: metrics/store.rs
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

### FIND-025 — God Module: discord/config.rs
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

### FIND-026 — Unsafe Code Without Safety Documentation
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

### FIND-027 — Redundant Dependencies
- **Category:** Dependency
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 10/22
- **Files:** [`Cargo.toml`](Cargo.toml)
- **Description:** Redundant dependencies detected - serde_derive re-exported by serde since 1.0.
- **Evidence:** Both `serde` and `serde_derive` listed in dependencies
- **Suggested Fix:** Remove serde_derive from dependencies (serde re-exports it)
- **Status:** OPEN

### FIND-028 — Inconsistent Error Type Implementations
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

### FIND-029 — Formatting Issues
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

---

## Priority Order Summary

| Priority | Finding | Score |
|----------|---------|-------|
| 1 | FIND-001: Build Failure (bollard) | 22 |
| 2 | FIND-002: Merge Conflicts | 22 |
| 3 | FIND-003: panic!() in Production | 20 |
| 4 | FIND-004: .expect() in Production | 18 |
| 5 | FIND-005: Swallowed Errors | 18 |
| 6 | FIND-006: God Module run.rs | 16 |
| 7 | FIND-007: God Module config/mod.rs | 16 |
| 8 | FIND-008: God Module skills/mod.rs | 16 |
| 9 | FIND-009: God Module cli/mod.rs | 16 |
| 10 | FIND-010: God Module commands/skills.rs | 16 |
| 11 | FIND-011: Non-thiserror Errors | 16 |
| 12 | FIND-012: Unused Imports | 16 |
| 13 | FIND-014: Broken Doc Links | 16 |
| 14 | FIND-015: God Module discord/tools.rs | 13 |
| 15 | FIND-016: God Module discord/llm.rs | 13 |
| 16 | FIND-017: God Module docker/skills.rs | 13 |
| 17 | FIND-018: God Module commands/validate.rs | 13 |
| 18 | FIND-019: God Module docker/mod.rs | 13 |
| 19 | FIND-012: Excessive .clone() | 14 |
| 20 | FIND-020: Cron Doc Inconsistency | 13 |
| 21 | FIND-021: Missing API Docs | 13 |
| 22 | FIND-022: Unused allow(dead_code) | 13 |
| 23 | FIND-023: God Module scheduler/mod.rs | 10 |
| 24 | FIND-024: God Module metrics/store.rs | 10 |
| 25 | FIND-025: God Module discord/config.rs | 10 |
| 26 | FIND-026: Unsafe Without Safety Docs | 10 |
| 27 | FIND-027: Redundant Dependencies | 10 |
| 28 | FIND-028: Inconsistent Error Types | 8 |
| 29 | FIND-029: Formatting Issues | 10 |

---

## Recommended Action Plan

### Immediate (This Sprint)
1. **FIX-001**: Resolve merge conflicts in test files (FIND-002) - Unblocks build
2. **FIX-002**: Fix bollard API compatibility (FIND-001) - Unblocks build
3. **FIX-003**: Replace panic!() calls (FIND-003) - Critical error handling

### Short-term (Next 2-4 Weeks)
4. **FIX-004**: Replace .expect() calls in production (FIND-004)
5. **FIX-005**: Fix swallowed errors (FIND-005)
6. **FIX-006**: Migrate to thiserror (FIND-011)
7. **FIX-007**: Clean up unused imports (FIND-012)
8. **FIX-008**: Fix broken doc links (FIND-014)

### Medium-term (1-2 Months)
9. **FIX-009**: Refactor god modules (FIND-006 through FIND-010) - Start with top 5
10. **FIX-010**: Reduce .clone() usage (FIND-012)

### Long-term (Backlog)
11. Refactor remaining god modules (FIND-015 through FIND-025)
12. Add safety documentation to unsafe blocks (FIND-026)
13. Clean up redundant dependencies (FIND-027)
14. Standardize error types (FIND-028)
15. Run formatting pass (FIND-029)
