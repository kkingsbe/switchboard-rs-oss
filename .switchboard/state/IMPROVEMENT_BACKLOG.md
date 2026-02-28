# Codebase Scan Report

**Project**: switchboard  
**Scanned**: 2026-02-28T18:00:04Z  
**Commit Audited**: 1faeff7 (HEAD)
**Scope**: Full codebase (src/)
**Files Analyzed**: ~80 Rust source files  
**Audit Type**: Incremental audit (2 commits since last audit d197bee)

---

## Executive Summary

| Severity | Count | Change vs Last Audit |
|----------|-------|----------------------|
| 🔴 Critical | 1 | - |
| 🟠 High | 0 | - |
| 🟡 Medium | 4 | - |
| 🔵 Low | 1 | -1 |
| ⚪ Convention | 3 | - |

**Overall Health Score**: 5/10 (stable)

**Top 3 Priorities**:
1. Fix 24 failing tests (CRITICAL - down from 25)
2. Resolve inconsistent error handling patterns (CONV-001)
3. Continue splitting god modules (MED-001 to MED-004)

---

## Tech Stack Summary

- **Languages**: Rust (2021 edition)
- **Frameworks**: tokio, bollard (Docker), twilight (Discord)
- **Build Tools**: cargo
- **Testing**: assert_cmd, tempfile, predicates, serial_test
- **Linting**: cargo clippy
- **Coverage**: cargo-llvm-cov

---

## Phase 2: Automated Health Check Results

| Check | Status | Notes |
|-------|--------|-------|
| `cargo build --release` | ✅ PASS | No warnings |
| `cargo test` | ❌ FAIL | 24 tests failed (-1 from last audit's 25) |
| `cargo clippy` | ✅ PASS | Minor warnings only |
| `cargo fmt --check` | ✅ PASS | Formatting correct |

---

## Changes Since Last Audit (d197bee)

- **1faeff7**: refactor(refactor2): [FIND-LOW-002] extract helper functions in metrics/store.rs
- **49da791**: Improvements

---

## Findings by Category

### 🔴 Critical Issues

#### [CRIT-001] 🔄 RECURRING (×5) - 24 Test Failures

- **Category:** Test Suite
- **Severity:** Critical
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 16/22
- **Files:** Multiple test files in src/docker/run/run.rs, src/discord/config.rs, src/commands/validate.rs, src/skills/mod.rs
- **Description:** Test suite has 24 failing tests, indicating regression or broken functionality. This issue has persisted across multiple audits and decreased by 1 since last audit.
- **Evidence:**
```
test result: FAILED. 523 passed; 24 failed; 0 ignored
Failures include:
- docker::run::run::tests::test_entrypoint_script_generation_all_scenarios
- docker::run::run::tests::test_integration_complete_flow_single_skill
- docker::run::run::tests::test_script_injection_wrapper_executes_script
- docker::run::run::tests::test_skills_single_generates_custom_entrypoint
- docker::run::run::tests::test_integration_complete_flow_multiple_skills
- docker::run::run::tests::test_skills_multiple_generates_custom_entrypoint
- docker::skills::tests::test_generate_entrypoint_script_skill_not_in_preexisting_list
- skills::tests::test_check_npx_available_with_mock_error
- skills::tests::test_check_npx_available_with_mock_failure_exit_code
... (24 total)
```
- **Suggested Fix:** Investigate root cause of test failures - likely related to skill script generation logic changes and environment variable handling in tests
- **Status:** 🔄 RECURRING

---

### 🟡 Medium Priority (Refactoring)

#### [MED-001] 🔄 RECURRING - God Module - docker/run/run.rs at 5115 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 10/22
- **Files:** `src/docker/run/run.rs` (lines 1-5115)
- **Description:** Single file contains over 5000 lines with all Docker run logic. This is a major maintainability issue.
- **Evidence:**
```bash
$ wc -l src/docker/run/run.rs
5115 src/docker/run/run.rs
```
- **Suggested Fix:** Split into: docker/run/container.rs, docker/run/network.rs, docker/run/volumes.rs
- **Status:** 🔄 RECURRING

---

#### [MED-002] 🔄 RECURRING - God Module - config/mod.rs at 3512 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 10/22
- **Files:** `src/config/mod.rs` (lines 1-3512)
- **Description:** Single file contains Config, Agent, Settings structs + all validation + tests inline.
- **Evidence:**
```bash
$ wc -l src/config/mod.rs
3512 src/config/mod.rs
```
- **Suggested Fix:** Split into: config/agent.rs, config/settings.rs, config/validation.rs
- **Status:** 🔄 RECURRING

---

#### [MED-003] 🔄 RECURRING - CLI Module - 2146 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/cli/mod.rs` (2146 lines)
- **Description:** Contains all CLI commands and handlers in single file.
- **Status:** ✅ SCHEDULED
- **Scheduled:** Improvement Sprint 3, assigned to .switchboard/state/REFACTOR_TODO1.md
- **Note:** This is a multi-sprint effort - focus on first extraction this sprint

---

#### [MED-004] 🔄 RECURRING - Commands Module - commands/skills.rs at 2074 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/commands/skills.rs` (2074 lines)
- **Description:** Single file for all skills subcommands.
- **Status:** ✅ SCHEDULED
- **Scheduled:** Improvement Sprint 3, assigned to .switchboard/state/REFACTOR_TODO2.md
- **Note:** This is a multi-sprint effort - focus on first extraction this sprint

---

### 🔵 Low Priority

#### [LOW-001] 🔄 RECURRING - scheduler/mod.rs - 1293 Lines

- **Category:** Structure
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** `src/scheduler/mod.rs` (1293 lines)
- **Status:** ✅ SCHEDULED
- **Scheduled:** Improvement Sprint 3, assigned to .switchboard/state/REFACTOR_TODO2.md
- **Note:** This is a multi-sprint effort - focus on analysis and first extraction this sprint

---

### ⚪ Convention Issues

#### [CONV-001] ✅ SCHEDULED - Inconsistent Error Handling

- **Category:** Convention
- **Severity:** Low
- **Effort:** M
- **Risk:** Medium
- **Priority Score:** 7/22
- **Files:** Mixed Result<T,E> vs Box<dyn Error> vs thiserror
- **Description:** The codebase uses multiple error handling patterns inconsistently:
  - `thiserror::Error` in skills/error.rs, config/error.rs, metrics/error.rs
  - `Box<dyn std::error::Error>` in CLI entry points
  - Custom Result types with String errors scattered throughout
- **Evidence:**
```rust
// Pattern 1: Box<dyn Error> in CLI
pub fn run_list(_config: Option<String>) -> Result<(), Box<dyn std::error::Error>>

// Pattern 2: String errors
pub fn list_agents(config: &Config) -> Result<(), String>

// Pattern 3: Custom error types
pub fn load(&self) -> Result<AllMetrics, MetricsError>
```
- **Status:** ✅ SCHEDULED
- **Scheduled:** Improvement Sprint 2, assigned to .switchboard/state/REFACTOR_TODO2.md

---

#### [CONV-002] 🔄 RECURRING - Module Organization

- **Category:** Convention
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** src/docker/, src/discord/
- **Description:** Some modules have inconsistent organization patterns. docker/run/ has split submodules while docker/build.rs is still monolithic.
- **Status:** 🔄 RECURRING

---

#### [CONV-003] ✅ ANALYZED - Test Organization

- **Category:** Convention
- **Severity:** Low
- **Effort:** S
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** Throughout codebase
- **Description:** Tests are mixed between inline (#[cfg(test)] modules) and external files in tests/ directory. No consistent pattern.
- **Status:** ✅ ANALYZED
- **Analysis Result (commit 846206f):** Tests already well-organized - inline tests are appropriate for unit tests and integration tests within modules

---

## Recommendations Roadmap

### Immediate (This Sprint)
- [ ] Investigate and fix 24 failing tests (CRIT-001) - 4+ hours
- [ ] Continue god module refactoring (MED-001 to MED-004) - ongoing

### Short-term (Next 2-4 weeks)
- [ ] Standardize error handling approach (CONV-001) - 2-4 hours
- [ ] Address test organization inconsistencies (CONV-003) - 1-2 hours

### Long-term (Backlog)
- [ ] Split docker/run/run.rs into smaller modules (MED-001) - 8+ hours
- [ ] Split config/mod.rs into smaller modules (MED-002) - 8+ hours

---

## Appendix

### Files Scanned
- src/lib.rs, src/main.rs, src/logging.rs
- src/cli/mod.rs (2146 lines)
- src/commands/*.rs (build.rs, list.rs, logs.rs, metrics.rs, skills.rs, validate.rs)
- src/config/*.rs (mod.rs, env.rs, env_ext.rs)
- src/discord/*.rs (api.rs, config.rs, conversation.rs, gateway.rs, listener.rs, mod.rs, out.rs, security.rs)
- src/docker/*.rs (build.rs, client.rs, mod.rs, run.rs, skills.rs)
- src/docker/run/*.rs (run.rs, streams.rs, types.rs, wait.rs)
- src/logger/*.rs (file.rs, mod.rs, terminal.rs)
- src/metrics/*.rs (collector.rs, mod.rs, store.rs)
- src/scheduler/mod.rs (1293 lines)
- src/skills/*.rs (error.rs, lockfile.rs, metadata.rs, mod.rs)

### Skipped Files
- None - full scan completed

### Health History Trend
```
Date            | Total | Crit | High | Med | Low | Conv
----------------|-------|------|------|-----|-----|-----
2026-02-28T14:02| 10    | 1    | 0    | 4   | 2   | 3
2026-02-28T16:01| 10    | 1    | 0    | 4   | 1   | 3
2026-02-28T18:00| 9     | 1    | 0    | 4   | 1   | 3  ← Current
```
Health score stable at 5/10. Test failures decreased from 25 to 24.
