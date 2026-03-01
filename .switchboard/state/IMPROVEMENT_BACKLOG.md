# IMPROVEMENT_BACKLOG.md

> Last Audit: 2026-03-01T04:07:00Z
> Commit Audited: 6cbb824
> Health Trend: Stable (7 findings, -1 vs last audit due to formatting fix)

## Summary
| Severity | Count | Change |
|----------|-------|--------|
| Critical | 2     | 0     |
| High     | 2     | -1    |
| Medium   | 2     | 0     |
| Low      | 1     | 0     |

## Active Findings

### FIND-001 [🔄 RECURRING (×2)] — Test Failures in Test Suite

- **Category:** Testing
- **Severity:** Critical
- **Effort:** L
- **Risk:** High
- **Priority Score:** 15/22
- **Skill:** N/A
- **Files:** Multiple test modules
- **Description:** 24 tests are failing across the test suite. These failures affect validation, config parsing, and docker run integration tests. This is a significant regression that blocks production deployment.
- **Evidence:**
  ```
  test result: FAILED. 523 passed; 24 failed; 0 ignored; 0 measured; 0 filtered out

  Failed tests include:
  - commands::validate::tests::test_validate_lockfile_consistency_warns_orphaned_skills
  - discord::config::tests::test_env_config_missing_openrouter_api_key
  - docker::run::run::tests::test_integration_complete_flow_single_skill
  - docker::run::run::tests::test_skill_install_success_log_has_prefix
  - docker::skills::tests::test_generate_entrypoint_script_skill_not_in_preexisting_list
  - skills::tests::test_check_npx_available_with_mock_error
  - And 19 more...
  ```
- **Suggested Fix:** Analyze each failing test to identify the root cause. Many appear related to changes in skill installation logic and environment variable handling. Run individual tests with `cargo test <test_name>` to debug.
- **Status:** OPEN

---

### FIND-002 [🔄 RECURRING (×2)] — Clippy Lints Fail Build with -D Warnings

- **Category:** Code Quality / Skill Violation
- **Severity:** Critical
- **Effort:** S
- **Risk:** Medium
- **Priority Score:** 14/22
- **Skill:** skills/rust-best-practices/references/chapter_02.md §2.2
- **Files:** src/cli/mod.rs (lines 21-46), src/cli/commands/up.rs (lines 10-24), src/commands/skills/install.rs (lines 4-11), src/commands/skills/mod.rs (line 16)
- **Description:** Running `cargo clippy --all-targets -- -D warnings` fails due to unused imports. This violates the skill requirement to run clippy and fix all warnings. Partial fix was applied in recent commits but more unused imports remain.
- **Evidence:**
  ```
  error: unused import: `crate::logging::init_logging`
    --> src/cli/mod.rs:21:5
  error: unused import: `crate::scheduler::Scheduler`
    --> src/cli/mod.rs:23:5
  error: unused import: `RealProcessExecutor`
    --> src/cli/mod.rs:25:62
  error: unused imports: `LockfileStruct`, `SkillLockEntry`, `SkillMetadata`
    --> src/commands/skills/install.rs:4:48
  error: unused import: `crate::skills::SkillMetadata`
    --> src/commands/skills/mod.rs:16:5
  error: function `default_executor` is never used
    --> src/cli/commands/up.rs:213:4
  ```
- **Suggested Fix:** Remove all unused imports from cli/mod.rs, cli/commands/up.rs, commands/skills/install.rs, and commands/skills/mod.rs. Also remove the unused `default_executor` function or mark it with `#[allow(dead_code)]`.
- **Status:** OPEN

---

### FIND-003 [🔄 RECURRING (×2)] — God Module: docker/run/run.rs

- **Category:** Complexity
- **Severity:** High
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 12/22
- **Skill:** skills/rust-best-practices/SKILL.md
- **Files:** src/docker/run/run.rs (5115 lines)
- **Description:** The run.rs file has 5115 lines, making it extremely large and difficult to maintain. This violates Single Responsibility Principle - it handles container execution, skill installation, metrics, and error handling all in one module.
- **Evidence:**
  ```
  5115 src/docker/run/run.rs
  ```
- **Suggested Fix:** Extract into separate modules: container_execution.rs, skill_installation.rs, metrics_collection.rs, error_handling.rs. Consider creating a RunContext struct to encapsulate the execution state.
- **Status:** OPEN

---

### FIND-004 [🔄 RECURRING (×2)] — God Module: config/mod.rs

- **Category:** Complexity
- **Severity:** High
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 12/22
- **Skill:** skills/rust-best-practices/SKILL.md
- **Files:** src/config/mod.rs (3512 lines)
- **Description:** The config/mod.rs file has 3512 lines, containing Config, Agent, Settings structs with extensive validation logic. Should be split into multiple files.
- **Evidence:**
  ```
  3512 src/config/mod.rs
  ```
- **Suggested Fix:** Split into: config/structs.rs (types), config/validation.rs (validation functions), config/parsing.rs (TOML parsing), config/mod.rs (re-exports and thin orchestration layer).
- **Status:** OPEN

---

### FIND-005 [🔄 RECURRING (×2)] — Unwrap/Expect Usage in Production Code

- **Category:** Error Handling / Skill Violation
- **Severity:** High
- **Effort:** M
- **Risk:** Medium
- **Priority Score:** 11/22
- **Skill:** skills/rust-best-practices/references/chapter_04.md §4.2, skills/rust-engineer/SKILL.md
- **Files:** src/scheduler/mod.rs (line 1291), src/docker/client.rs (lines 115-127), src/commands/logs.rs (line 306)
- **Description:** Multiple .unwrap() and .expect() calls in production code outside of tests. While some are in initialization paths where failure is catastrophic, others could use proper error handling. This violates the skill requirement to avoid unwrap/expect in production.
- **Evidence:**
  ```rust
  // src/scheduler/mod.rs:1291
  Self::new_sync(None, None, None).expect("Failed to create default Scheduler")
  
  // src/docker/client.rs:115-116
  .strip_prefix("unix://")
      .expect("socket_path starts with 'unix://' so strip_prefix should succeed");
  
  // src/commands/logs.rs:306
  let sigterm = signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler");
  ```
- **Suggested Fix:** For scheduler initialization, consider returning Result from new() or using unwrap_or_else with proper error logging. For parse operations, use proper error mapping. For signal handler, use Result propagation.
- **Status:** OPEN

---

### FIND-006 — Formatting Inconsistencies (RESOLVED)

- **Category:** Convention
- **Severity:** Medium
- **Effort:** S
- **Risk:** Low
- **Priority Score:** 8/22
- **Skill:** skills/rust-engineer/SKILL.md
- **Description:** `cargo fmt --check` showed diffs in multiple files. This has been FIXED in recent commits.
- **Evidence:**
  ```
  cargo fmt --check passes without diffs (2026-03-01)
  ```
- **Suggested Fix:** N/A - Already resolved
- **Status:** RESOLVED
  - **Resolved:** 2026-03-01 (commit 6cbb824)
  - **Resolution:** Formatting issues fixed by running `cargo fmt`

---

### FIND-007 [🔄 RECURRING (×2)] — Large CLI Module

- **Category:** Complexity
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Skill:** N/A
- **Files:** src/cli/mod.rs (1256 lines, down from 2082)
- **Description:** The CLI module at 1256 lines is large but improved from 2082 lines. Partial refactoring was done to remove duplicate code. Still large but has improved.
- **Evidence:**
  ```
  1256 src/cli/mod.rs (was 2082 in previous audit)
  ```
- **Suggested Fix:** Consider extracting command handlers to separate modules (src/cli/handlers/) if the file grows further.
- **Status:** OPEN (improved)

---

### FIND-008 [🔄 RECURRING (×2)] — Large Scheduler Module

- **Category:** Complexity
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 5/22
- **Skill:** N/A
- **Files:** src/scheduler/mod.rs (1293 lines)
- **Description:** The scheduler module at 1293 lines contains cron parsing, scheduling logic, and agent management. Already reasonably organized.
- **Evidence:**
  ```
  1293 src/scheduler/mod.rs
  ```
- **Suggested Fix:** Monitor for growth. Currently acceptable.
- **Status:** OPEN

---

## Recommendations Roadmap

### Immediate (This Sprint)
- [ ] Fix clippy unused imports - Remove 19 unused imports (S)
- [ ] Investigate and fix 24 failing tests (L)

### Short-term (Next 2-4 weeks)
- [ ] Refactor docker/run/run.rs - Extract into multiple modules (L)
- [ ] Refactor config/mod.rs - Split into validation/parsing/structs modules (L)
- [ ] Audit .unwrap()/.expect() usage in production code - Convert to proper error handling (M)

### Long-term (Backlog)
- [ ] Add #[derive(thiserror::Error)] to more error types for consistent error handling
- [ ] Consider adding integration tests for critical paths
- [ ] Set up CI to fail on clippy warnings

---

## Appendix

### Files Scanned (Top 20 by line count)
| File | Lines |
|------|-------|
| src/docker/run/run.rs | 5115 |
| src/config/mod.rs | 3512 |
| src/commands/validate.rs | 1453 |
| src/scheduler/mod.rs | 1293 |
| src/docker/skills.rs | 1282 |
| src/cli/mod.rs | 1256 |
| src/metrics/store.rs | 1132 |
| src/discord/config.rs | 1101 |
| src/skills/mod.rs | 1036 |
| src/docker/client.rs | 1025 |

### Health Check Results
- **Build:** PASS (with 19 warnings - unused imports)
- **Tests:** FAIL (24 failed, 523 passed)
- **Clippy:** FAIL (unused imports block -D warnings build)
- **Format:** PASS ✓

### Skill Compliance Notes
- **rust-best-practices/SKILL.md:** Clippy not passing (Critical - FIND-002), unwrap/expect usage (High - FIND-005)
- **rust-engineer/SKILL.md:** Formatting fixed ✓

### Previous Audit Findings - Resolution Status
| Finding | Status | Change |
|---------|--------|--------|
| FIND-001 (Test failures) | OPEN | Same |
| FIND-002 (Clippy) | OPEN | Partial fix applied |
| FIND-003 (God module run.rs) | OPEN | Same |
| FIND-004 (God module config) | OPEN | Same |
| FIND-005 (Unwrap/expect) | OPEN | Same |
| FIND-006 (Formatting) | RESOLVED | Fixed |
| FIND-007 (CLI module) | OPEN | Improved (2082→1256) |
| FIND-008 (Scheduler) | OPEN | Same |
