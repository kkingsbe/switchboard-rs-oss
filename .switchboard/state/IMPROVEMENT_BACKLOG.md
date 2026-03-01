# IMPROVEMENT_BACKLOG.md

> Last Audit: 2026-03-01T06:10:00Z
> Commit Audited: $(git rev-parse HEAD)
> Health Trend: Stable (7 findings, similar to last audit)

## Summary
| Severity | Count | Change |
|----------|-------|--------|
| Critical | 2     | 0     |
| High     | 3     | +1    |
| Medium   | 2     | 0     |
| Low      | 0     | -1    |

## Active Findings

### FIND-001 [🔴 CRITICAL] — Test Failures in Test Suite

- **Category:** Testing
- **Severity:** Critical
- **Effort:** L
- **Risk:** High
- **Priority Score:** 20/22
- **Skill:** N/A
- **Files:** Multiple test modules
- **Description:** 24 tests are failing across the test suite. These failures affect validation, config parsing, and docker run integration tests. This is a significant regression that blocks production deployment.
- **Evidence:**
  ```
  test result: FAILED. 523 passed; 24 failed; 0 ignored; 0 measured; 0 filtered out

  Failed tests include:
  - commands::validate::tests::test_validate_lockfile_consistency_warns_orphaned_skills
  - docker::run::run::tests::test_integration_complete_flow_single_skill
  - docker::skills::tests::test_generate_entrypoint_script_skill_not_in_preexisting_list
  - skills::tests::test_check_npx_available_with_mock_error
  - And 19 more...
  ```
- **Suggested Fix:** Analyze each failing test to identify the root cause. Many appear related to changes in skill installation logic and environment variable handling. Run individual tests with `cargo test <test_name>` to debug.
- **Status:** OPEN

---

### FIND-002 [🔴 CRITICAL] — Clippy Lints Fail Build with -D Warnings

- **Category:** Code Quality / Skill Violation
- **Severity:** Critical
- **Effort:** S
- **Risk:** Medium
- **Priority Score:** 27/22
- **Skill:** skills/rust-best-practices/references/chapter_02.md §2.2
- **Files:** src/cli/mod.rs (lines 21-46), src/cli/commands/up.rs (lines 10-24), src/commands/skills/install.rs (lines 4-11)
- **Description:** Running `cargo clippy --all-targets -- -D warnings` fails due to unused imports. This violates the skill requirement to run clippy and fix all warnings. The previous audit noted partial fix but more unused imports remain.
- **Evidence:**
  ```
  error: unused import: `std::env`
    --> src/cli/mod.rs:28:5
  error: unused import: `crate::config::env::resolve_config_value`
    --> src/cli/mod.rs:35:5
  error: unused import: `crate::discord::config::LlmConfig`
    --> src/cli/mod.rs:37:5
  error: unused import: `tokio::sync::broadcast`
    --> src/cli/mod.rs:41:5
  error: unused imports: `SignalKind` and `signal`
    --> src/cli/mod.rs:44:27
  error: unused imports: `DockerClientTrait`, `ProcessExecutorTrait`, and `RealProcessExecutor`
    --> src/cli/commands/up.rs:10:21
  ```
- **Suggested Fix:** Remove all unused imports from cli/mod.rs, cli/commands/up.rs, and commands/skills/install.rs.
- **Status:** OPEN

---

### FIND-003 [🟡 MEDIUM] — Formatting Inconsistency

- **Category:** Code Quality
- **Severity:** Medium
- **Effort:** S
- **Risk:** Low
- **Priority Score:** 20/22
- **Skill:** N/A
- **Files:** src/commands/skills/install.rs (line 2-5)
- **Description:** `cargo fmt --check` fails due to formatting inconsistency in import statements.
- **Evidence:**
  ```
  Diff in /workspace/src/commands/skills/install.rs:2:
   use crate::skills::{
       add_skill_to_lockfile, create_npx_command, get_agents_using_skill, scan_global_skills,
  -    scan_project_skills, skills_sh_search,
  -    SkillsError, SkillsManager, NPX_NOT_FOUND_ERROR,
  +    scan_project_skills, skills_sh_search, SkillsError, SkillsManager, NPX_NOT_FOUND_ERROR,
   };
  ```
- **Suggested Fix:** Run `cargo fmt` to auto-fix formatting issues.
- **Status:** OPEN

---

### FIND-004 [🟠 HIGH] — God Module: docker/run/run.rs (5115 lines)

- **Category:** Code Structure
- **Severity:** High
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 16/22
- **Skill:** N/A
- **Files:** src/docker/run/run.rs
- **Description:** This module is excessively large (5115 lines) making it difficult to maintain, test, and understand. It violates the Single Responsibility Principle.
- **Evidence:**
  ```
  5115 src/docker/run/run.rs
  3512 src/config/mod.rs
  1293 src/scheduler/mod.rs
  1282 src/docker/skills.rs
  1254 src/cli/mod.rs
  ```
- **Suggested Fix:** Break down into smaller, focused modules. Extract distinct responsibilities like container configuration, script generation, and execution logic into separate modules.
- **Status:** ACKNOWLEDGED
- **Note:** This is a known issue from previous audits. Refactoring is a long-term effort.

---

### FIND-005 [🟠 HIGH] — unwrap()/expect() Usage in Production Code

- **Category:** Error Handling / Skill Violation
- **Severity:** High
- **Effort:** M
- **Risk:** Medium
- **Priority Score:** 20/22
- **Skill:** skills/rust-best-practices/references/chapter_04.md §4.2
- **Files:** src/scheduler/mod.rs (line 1291), src/docker/client.rs (lines 116, 127)
- **Description:** Using unwrap()/expect() in production code violates the skill requirement to avoid panic in production. These should return Result types instead.
- **Evidence:**
  ```rust
  // src/scheduler/mod.rs:1291
  Self::new_sync(None, None, None).expect("Failed to create default Scheduler")

  // src/docker/client.rs:116
  .strip_prefix("unix://")
      .expect("socket_path starts with 'unix://' so strip_prefix should succeed");

  // src/docker/client.rs:127
  .strip_prefix("npipe://")
      .expect("socket_path starts with 'npipe://' so strip_prefix should succeed");
  ```
- **Suggested Fix:** Refactor to return proper Result types with appropriate error variants.
- **Status:** OPEN

---

### FIND-006 [🟠 HIGH] — God Module: config/mod.rs (3512 lines)

- **Category:** Code Structure
- **Severity:** High
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 16/22
- **Skill:** N/A
- **Files:** src/config/mod.rs
- **Description:** This module is excessively large (3512 lines) and handles multiple responsibilities including config parsing, validation, and environment handling.
- **Evidence:**
  ```
  3512 src/config/mod.rs
  ```
- **Suggested Fix:** Split into submodules: config/parsing.rs, config/validation.rs, config/env.rs (already exists), etc.
- **Status:** ACKNOWLEDGED
- **Note:** Long-term refactoring opportunity.

---

### FIND-007 [🟡 MEDIUM] — Large CLI Module (1254 lines)

- **Category:** Code Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 14/22
- **Skill:** N/A
- **Files:** src/cli/mod.rs
- **Description:** CLI module is still large but has improved from previous audit (2082 -> 1254 lines).
- **Evidence:**
  ```
  1254 src/cli/mod.rs
  ```
- **Suggested Fix:** Continue extracting command handlers into separate modules.
- **Status:** IMPROVED

---

## Recommendations Roadmap

### Immediate (This Sprint)
- [ ] Fix clippy unused imports (FIND-002) - 30 min
- [ ] Fix formatting issue (FIND-003) - 5 min
- [ ] Analyze and fix test failures (FIND-001) - 4+ hours

### Short-term (Next 2-4 weeks)
- [ ] Refactor unwrap/expect in production code (FIND-005) - 2-4 hours
- [ ] Continue CLI module improvements (FIND-007) - 4 hours

### Long-term (Backlog)
- [ ] Break down docker/run/run.rs (FIND-004) - 20+ hours
- [ ] Break down config/mod.rs (FIND-006) - 16+ hours

---

## Health Score

**Overall Health Score: 5/10**

The project has significant issues blocking production deployment:
- Critical test failures (24 tests)
- Critical clippy failures (skill violation)
- Large god modules causing maintenance burden

Scoring:
- 9-10: Excellent - minimal issues, well-maintained
- 7-8: Good - some tech debt but manageable
- 5-6: Fair - significant refactoring needed
- 3-4: Poor - major issues affecting development velocity
- 1-2: Critical - urgent intervention required
