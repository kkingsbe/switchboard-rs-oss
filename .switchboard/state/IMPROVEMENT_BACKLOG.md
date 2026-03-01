# IMPROVEMENT_BACKLOG.md

> Last Audit: 2026-03-01T10:10:00Z
> Commit Audited: $(git rev-parse HEAD 2>/dev/null || echo "unknown")
> Health Trend: Degrading (critical issues persist)

## Summary
| Severity | Count | Change |
|----------|-------|--------|
| Critical | 2     | 0     |
| High     | 3     | 0     |
| Medium   | 3     | +1    |
| Low      | 0     | 0     |

## Recent Changes Since Last Audit
- No commits since last audit (f155c04)

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
  - discord::config::tests::test_env_config_missing_openrouter_api_key
  - discord::config::tests::test_env_config_success
  - discord::config::tests::test_load_switchboard_toml_discord_section
  - docker::run::run::tests::test_entrypoint_script_generation_all_scenarios
  - And 16 more...
  ```
- **Suggested Fix:** Analyze each failing test to identify the root cause. Many appear related to changes in skill installation logic and environment variable handling. Run individual tests with `cargo test <test_name>` to debug.
- **Status:** OPEN
- **Note:** STILL PRESENT - No fix applied since last audit

---

### FIND-002 [🔴 CRITICAL] — Clippy Lints Fail Build with -D Warnings

- **Category:** Code Quality / Skill Violation
- **Severity:** Critical
- **Effort:** S
- **Risk:** Medium
- **Priority Score:** 27/22
- **Skill:** skills/rust-best-practices/references/chapter_02.md §2.2
- **Files:** src/cli/mod.rs (lines 21-46), src/cli/commands/up.rs (lines 10-24)
- **Description:** Running `cargo clippy --all-targets -- -D warnings` fails due to unused imports and other clippy errors. This violates the skill requirement to run clippy and fix all warnings.
- **Evidence:**
  ```
  error: unused import: `std::env`
    --> src/cli/mod.rs:28:5
  error: unused import: `crate::config::env::resolve_config_value`
    --> src/cli/mod.rs:35:5
  error: unused import: `crate::discord::config::LlmConfig`
    --> src/cli/mod.rs:37:5
  error: unused imports: `DockerClientTrait`, `ProcessExecutorTrait`, and `RealProcessExecutor`
    --> src/cli/commands/up.rs:10:21
  error: empty line after doc comment
    --> src/cli/commands/up.rs:209:1
  error: cannot test inner items
    --> src/config/mod.rs:3453:9
  ```
- **Suggested Fix:** Remove all unused imports from cli/mod.rs and cli/commands/up.rs. Fix the empty line after doc comment in up.rs.
- **Status:** SCHEDULED
- **Scheduled:** Improvement Sprint 5, assigned to .switchboard/state/REFACTOR_TODO1.md
- **Note:** Scheduled for Sprint 5

---

### FIND-003 [🟡 MEDIUM] — Formatting Inconsistency

- **Category:** Code Quality
- **Severity:** Medium
- **Effort:** S
- **Risk:** Low
- **Priority Score:** 20/22
- **Skill:** N/A
- **Files:** src/commands/skills/install.rs, src/docker/client.rs, src/scheduler/mod.rs
- **Description:** `cargo fmt --check` fails due to formatting inconsistency.
- **Evidence:**
  ```
  Diff in /workspace/src/commands/skills/install.rs:1:
  Diff in /workspace/src/docker/client.rs:111:
  Diff in /workspace/src/scheduler/mod.rs:1282:
  ```
- **Suggested Fix:** Run `cargo fmt` to auto-fix formatting issues.
- **Status:** SCHEDULED
- **Scheduled:** Improvement Sprint 5, assigned to .switchboard/state/REFACTOR_TODO2.md

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
  1285 src/scheduler/mod.rs
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
- **Files:** src/scheduler/mod.rs, src/docker/run/streams.rs (line 91)
- **Description:** Using unwrap()/expect() in production code violates the skill requirement to avoid panic in production. Found unwrap() usage in production code:
- **Evidence:**
  ```rust
  // src/docker/run/streams.rs:91
  if let Err(e) = logger.lock().unwrap().write_agent_log(agent_name, &message) {
      eprintln!("Failed to write agent log: {}", e);
  }
  ```
- **Suggested Fix:** Replace with proper error handling using ? or match with appropriate error recovery.
- **Status:** PARTIALLY FIXED
- **Note:** New finding - unwrap() in streams.rs

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
- **Note:** Improved from 2082 lines in earlier audits

---

### FIND-008 [🟡 MEDIUM] — Test Assertion Failures in Config Tests

- **Category:** Testing
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 14/22
- **Skill:** N/A
- **Files:** src/discord/config.rs, src/commands/validate.rs
- **Description:** Multiple config-related tests failing due to assertion mismatches and environment variable handling.
- **Evidence:**
  ```
  // discord/config.rs:609
  assertion failed: err.contains(OPENROUTER_API_KEY_ENV)
  
  // discord/config.rs:635
  assertion failed: result.is_ok()
  
  // discord/config.rs:1057
  assertion `left == right` failed: Expected channel_id = 1472443428569874533
    left: "${DISCORD_CHANNEL_ID}"
   right: "1472443428569874533"
  ```
- **Suggested Fix:** Fix environment variable resolution in config tests.
- **Status:** NEW

---

## Recommendations Roadmap

### Immediate (This Sprint)
- [ ] Fix clippy unused imports (FIND-002) - 30 min
- [ ] Fix formatting issue (FIND-003) - 5 min
- [ ] Analyze and fix test failures (FIND-001) - 4+ hours

### Short-term (Next 2-4 weeks)
- [ ] Fix config test failures (FIND-008) - 2 hours
- [ ] Remove unwrap() from streams.rs (FIND-005) - 30 min
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

Positive trends:
- CLI module reduced from 2082 to 1254 lines
- Some unwrap/expect issues have been addressed

Scoring:
- 9-10: Excellent - minimal issues, well-maintained
- 7-8: Good - some tech debt but manageable
- 5-6: Fair - significant refactoring needed
- 3-4: Poor - major issues affecting development velocity
- 1-2: Critical - urgent intervention required

---

## Appendix

### Automated Health Check Results (2026-03-01T10:10:00Z)

| Check | Status | Details |
|-------|--------|---------|
| `cargo build` | ⚠️ WARNINGS | 12 unused import warnings |
| `cargo test` | ❌ FAIL | 523 passed, 24 failed |
| `cargo clippy -D warnings` | ❌ FAIL | 12 clippy errors |
| `cargo fmt --check` | ❌ FAIL | 3 files need formatting |

### Files Analyzed
- Total Rust source files: ~60
- Total lines of code: ~30,000+
- Largest modules: docker/run/run.rs (5115), config/mod.rs (3512)
