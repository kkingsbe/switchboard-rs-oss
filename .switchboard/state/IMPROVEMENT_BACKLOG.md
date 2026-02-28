# Codebase Scan Report

**Project**: switchboard  
**Scanned**: 2026-02-28T10:15:00Z  
**Commit Audited**: 46c085c
**Scope**: Full codebase (src/)
**Files Analyzed**: ~80 Rust source files  
**Audit Type**: Incremental audit (1 commit since last audit)

---

## Executive Summary

| Severity | Count | Change vs Last Audit |
|----------|-------|----------------------|
| 🔴 Critical | 1 | - |
| 🟠 High | 0 | - |
| 🟡 Medium | 4 | - |
| 🔵 Low | 2 | - |
| ⚪ Convention | 4 | - |

**Overall Health Score**: 5/10 (stable)

**Top 3 Priorities**:
1. Fix 24 failing tests (CRITICAL - ongoing)
2. Resolve remaining unwrap/expect in production code
3. Split god modules (docker/run/run.rs - 5115 lines)

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
| `cargo test` | ❌ FAIL | 24 tests failed (same as last audit) |
| `cargo clippy` | ✅ PASS | Minor warnings only |
| `cargo fmt --check` | ✅ PASS | All files formatted correctly |

---

## Findings by Category

### 🔴 Critical Issues

#### [CRIT-001] 🔄 RECURRING (×2) - 24 Test Failures

- **Category:** Test Suite
- **Severity:** Critical
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 16/22
- **Files:** Multiple test files in src/docker/run/run.rs, src/discord/config.rs, src/commands/validate.rs
- **Description:** Test suite has 24 failing tests, indicating regression or broken functionality. This issue has persisted across multiple audits.
- **Evidence:**
```
test result: FAILED. 523 passed; 24 failed; 0 ignored
Failures include:
- docker::run::run::tests::test_entrypoint_script_generation_all_scenarios
- docker::run::run::tests::test_integration_complete_flow_single_skill
- docker::run::run::tests::test_script_injection_wrapper_executes_script
- discord::config::tests::test_env_config_missing_openrouter_api_key
- discord::config::tests::test_env_config_success
- commands::validate::tests::test_validate_lockfile_consistency_warns_orphaned_skills
... (24 total)
```
- **Suggested Fix:** Investigate root cause of test failures - likely related to skill script generation logic changes
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
- **Status:** 🔄 RECURRING

---

#### [MED-004] 🔄 RECURRING - Commands Module - commands/skills.rs at 2074 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/commands/skills.rs` (2074 lines)
- **Description:** Single file for all skills subcommands.
- **Status:** 🔄 RECURRING

---

### 🔵 Low Priority

#### [LOW-001] 🔄 RECURRING - scheduler/mod.rs - 1307 Lines

- **Category:** Structure
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** `src/scheduler/mod.rs` (1307 lines)
- **Status:** 🔄 RECURRING

#### [LOW-002] 🔄 RECURRING - metrics/store.rs RECURRING

- 1107 Lines

- **Category:** Structure
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** `src/metrics/store.rs` (1107 lines)
- **Status:** 🔄 RECURRING

---

### ⚪ Convention Issues

#### [CONV-001] 🔄 RECURRING - Inconsistent Error Handling

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
- **Status:** 🔄 RECURRING

---

#### [CONV-002] 🔄 RECURRING - Module Organization - Discord

- **Category:** Convention
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Status:** 🔄 RECURRING

---

#### [CONV-003] 🔄 RECURRING - Test Organization

- **Category:** Convention
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 8/22
- **Status:** 🔄 RECURRING

---

#### [CONV-004] ✅ RESOLVED - Missing Documentation

- **Previous Status**: Missing doc comments on public functions
- **Current Status**: **RESOLVED** - Most public functions now have doc comments
- **Effort**: N/A
- **Status:** ✅ RESOLVED

---

## Recommendations Roadmap

### Immediate (This Sprint)
- [ ] Fix 24 failing tests - 8h (CRITICAL)
- [ ] Review recent commits that may have caused test failures

### Short-term (Next 2-4 weeks)
- [ ] Fix remaining unwrap/expect in production code - 1h
- [ ] Resolve error handling inconsistencies - 4h

### Long-term (Backlog)
- [ ] Split docker/run/run.rs (5115 lines) - 4h
- [ ] Split config/mod.rs (3512 lines) - 4h
- [ ] Split cli/mod.rs (2146 lines) - 3h
- [ ] Split commands/skills.rs (2074 lines) - 3h

---

## Appendix

### Files Scanned (Top 20 by Line Count)
| File | Lines |
|------|-------|
| src/docker/run/run.rs | 5115 |
| src/config/mod.rs | 3512 |
| src/cli/mod.rs | 2146 |
| src/commands/skills.rs | 2074 |
| src/commands/validate.rs | 1445 |
| src/scheduler/mod.rs | 1307 |
| src/docker/skills.rs | 1282 |
| src/metrics/store.rs | 1107 |
| src/discord/config.rs | 1095 |
| src/skills/mod.rs | 1036 |

### Skills Compliance Notes
The codebase has two active skills:
1. **rust-best-practices** - Key violations:
   - unwrap()/expect() in production code - Mostly resolved, remaining in scheduler/mod.rs, docker/client.rs, commands/skills.rs, commands/logs.rs
   
2. **rust-engineer** - Key violations:
   - Error handling patterns inconsistent (CONV-001)

### Health Trend
**Health Score**: 5/10 (stable)

**Changes since last audit**:
- 1 commit audited: refactor(refactor1) - extract acquire_lock helper in scheduler module
- No new issues introduced
- Test failures remain unchanged at 24
