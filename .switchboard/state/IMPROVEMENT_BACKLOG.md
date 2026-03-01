# IMPROVEMENT_BACKLOG.md

> Last Audit: 2026-03-01T00:15:00Z
> Commit Audited: 8d71e01

## Summary
| Severity | Count |
|----------|-------|
| Critical | 2 |
| High     | 3 |
| Medium   | 2 |
| Low      | 1 |

## Active Findings

### FIND-001 🆕 NEW — Test Failures in Test Suite
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
  - And 20 more...
  ```
- **Suggested Fix:** Analyze each failing test to identify the root cause. Many appear related to changes in skill installation logic and environment variable handling. Run individual tests with `cargo test <test_name>` to debug.
- **Status:** OPEN

---

### FIND-002 🆕 NEW — Clippy Lints Fail Build with -D Warnings
- **Category:** Code Quality
- **Severity:** Critical
- **Effort:** S
- **Risk:** Medium
- **Priority Score:** 14/22
- **Skill:** skills/rust-best-practices/references/chapter_02.md
- **Files:** src/cli/mod.rs (line 16), src/commands/skills/mod.rs (lines 14-21), src/config/mod.rs (line 3453)
- **Description:** Running `cargo clippy --all-targets -- -D warnings` fails due to unused imports and cannot test inner items. This violates the skill requirement to run clippy and fix warnings.
- **Evidence:**
  ```
  error: unused import: `list_agents`
    --> src/cli/mod.rs:16:23
     |
  16 | use crate::commands::{list_agents, metrics, BuildCommand, SkillsCommand, ValidateCommand};
     |                       ^^^^^^^^^^^

  error: unused imports: `LockfileStruct`, `SkillLockEntry`, `find_skill_directory`, ...
    --> src/commands/skills/mod.rs:14:48
  ```
- **Suggested Fix:** Remove unused imports from src/cli/mod.rs and src/commands/skills/mod.rs. Add #[allow(unnameable_test_items)] or move inner tests to proper test module in src/config/mod.rs.
- **Status:** OPEN

---

### FIND-003 🆕 NEW — God Module: docker/run/run.rs
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

### FIND-004 🆕 NEW — God Module: config/mod.rs
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

### FIND-005 🆕 NEW — Unwrap/Expect Usage in Production Code
- **Category:** Error Handling
- **Severity:** High
- **Effort:** M
- **Risk:** Medium
- **Priority Score:** 11/22
- **Skill:** skills/rust-best-practices/references/chapter_04.md, skills/rust-engineer/references/error-handling.md
- **Files:** src/scheduler/mod.rs (line 1291), src/docker/client.rs (lines 115-127), src/commands/list.rs (line 45), src/discord/api.rs (lines 273-319)
- **Description:** Multiple .unwrap() and .expect() calls in production code outside of tests. While some are in initialization paths where failure is catastrophic, others could use proper error handling.
- **Evidence:**
  ```rust
  // src/scheduler/mod.rs:1291
  Self::new_sync(None, None, None).expect("Failed to create default Scheduler")
  
  // src/docker/client.rs:115-116
  .strip_prefix("unix://")
      .expect("socket_path starts with 'unix://' so strip_prefix should succeed");
  
  // src/commands/list.rs:45
  let value: u64 = value_str.parse().ok()?;
  ```
- **Suggested Fix:** For scheduler initialization, consider returning Result from new() or using unwrap_or_else with proper error logging. For parse operations, use proper error mapping.
- **Status:** OPEN

---

### FIND-006 🆕 NEW — Formatting Inconsistencies
- **Category:** Convention
- **Severity:** Medium
- **Effort:** S
- **Risk:** Low
- **Priority Score:** 8/22
- **Skill:** skills/rust-engineer/SKILL.md
- **Files:** src/commands/skills/installed.rs, src/commands/skills/list.rs, src/commands/skills/mod.rs
- **Description:** `cargo fmt --check` shows diffs in multiple files, indicating inconsistent formatting that doesn't match project standards.
- **Evidence:**
  ```
  Diff in /workspace/src/commands/skills/installed.rs:6:
  -    get_agents_using_skill, read_lockfile, scan_global_skills, scan_project_skills,
  +    get_agents_using_skill, read_lockfile, scan_global_skills, scan_project_skills, LockfileStruct,
  ```
- **Suggested Fix:** Run `cargo fmt` to auto-fix formatting issues across the codebase.
- **Status:** OPEN

---

### FIND-007 🆕 NEW — Large CLI Module
- **Category:** Complexity
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Skill:** N/A
- **Files:** src/cli/mod.rs (2082 lines)
- **Description:** The CLI module at 2082 lines is large but acceptable for a CLI application. Contains command parsing and handler functions.
- **Evidence:**
  ```
  2082 src/cli/mod.rs
  ```
- **Suggested Fix:** Consider extracting command handlers to separate modules (src/cli/handlers/) if the file grows further.
- **Status:** OPEN

---

### FIND-008 🆕 NEW — Large Scheduler Module
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
- [ ] Fix 24 failing tests - Run individual tests to debug root causes (L)
- [ ] Fix clippy unused imports in src/cli/mod.rs and src/commands/skills/mod.rs (S)
- [ ] Run `cargo fmt` to fix formatting issues (S)

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

### Files Scanned (Top 30 by line count)
| File | Lines |
|------|-------|
| src/docker/run/run.rs | 5115 |
| src/config/mod.rs | 3512 |
| src/cli/mod.rs | 2082 |
| src/commands/validate.rs | 1453 |
| src/commands/skills/mod.rs | 1365 |
| src/scheduler/mod.rs | 1293 |
| src/docker/skills.rs | 1282 |
| src/metrics/store.rs | 1132 |
| src/discord/config.rs | 1101 |
| src/skills/mod.rs | 1036 |

### Skipped Files
- None - all source files scanned

### Health Check Results
- **Build:** PASS (with warnings)
- **Tests:** FAIL (24 failed, 523 passed)
- **Clippy:** FAIL (unused imports block -D warnings build)
- **Format:** FAIL (inconsistent formatting in skills commands)
