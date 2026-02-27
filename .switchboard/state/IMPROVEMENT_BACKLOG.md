# Codebase Scan Report

**Project**: switchboard  
**Scanned**: 2026-02-27T22:14:15Z  
**Commit Audited**: a58094e
**Scope**: Full codebase (src/, tests/)  
**Files Analyzed**: ~80 Rust source files  
**Audit Type**: Continuation (previous audit: 2026-02-27T20:09:00Z)

---

## Executive Summary

| Severity | Count | Change vs Last Audit |
|----------|-------|---------------------|
| 🔴 Critical | 3 | - |
| 🟠 High | 4 | -1 (resolved CONV-005) |
| 🟡 Medium | 6 | -1 (resolved MED-001, MED-002) |
| 🔵 Low | 3 | -1 |
| ⚪ Convention | 4 | -1 |

**Overall Health Score**: 6.5/10  

**Top 3 Priorities**:
1. Fix failing tests (25 test failures)
2. Address unwrap/expect in production code (skills compliance)
3. Split god modules for maintainability

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
| `cargo build` | ✅ PASS | 1 warning: unused config key |
| `cargo test` | ❌ FAIL | 25 test failures (same as last audit) |
| `cargo clippy` | ⚠️ WARN | Dead code in test files |
| `cargo fmt --check` | ✅ PASS | Formatting consistent |

---

## Findings by Category

### 🔴 Critical Issues

#### [CRIT-001] Test Failures - 25 Tests Failing

- **Category:** Testing
- **Severity:** Critical
- **Effort:** L
- **Risk:** High
- **Priority Score:** 17/22
- **Files:** Multiple test files in `tests/` and `src/`
- **Description:** 25 tests failing, including docker skill tests, discord config tests, and validate tests. This indicates regression detection is compromised and CI pipeline is likely broken.
- **Evidence:**
```
test docker::run::run::tests::test_skills_single_generates_custom_entrypoint ... FAILED
thread 'docker::run::run::tests::test_skills_single_generates_custom_entrypoint' panicked at src/docker/run/run.rs:1463:17:
Valid skill should generate script successfully

test skills::tests::test_check_npx_available_with_mock_error ... FAILED
thread 'skills::tests::test_check_npx_available_with_mock_error' panicked at src/skills/mod.rs:2539:9:
Expected error when mock returns error

test result: FAILED. 583 passed; 25 failed; 1 ignored
```
- **Suggested Fix:** Fix skill validation mocks in docker/run/run.rs tests. Update test setup for environment config tests. Ensure test isolation is properly configured.
- **Status:** 🔄 RECURRING (×2)

---

#### [CRIT-002] God Module - docker/run/run.rs at 5115 Lines

- **Category:** Structure
- **Severity:** Critical
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 14/22
- **Files:** `src/docker/run/run.rs` (lines 1-5115)
- **Description:** Single file contains over 5000 lines with all Docker run logic. This is a major maintainability issue - the file is too large to comprehend in one sitting and creates long compile times.
- **Evidence:**
```rust
// File line count confirms size
$ wc -l src/docker/run/run.rs
5115 src/docker/run/run.rs
```
- **Suggested Fix:** Split into: docker/run/container.rs (container creation), docker/run/network.rs (networking), docker/run/volumes.rs (volume management), docker/run/wait.rs (already exists but imported here)
- **Status:** 🆕 NEW - This file has grown significantly since last audit

---

#### [CRIT-003] God Module - config/mod.rs at 3512 Lines

- **Category:** Structure
- **Severity:** Critical
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 14/22
- **Files:** `src/config/mod.rs` (lines 1-3512)
- **Description:** Single file contains Config, Agent, Settings structs + all validation + tests inline. This is a maintainability nightmare.
- **Evidence:**
```rust
// From previous audit - file still exists at same size
$ wc -l src/config/mod.rs
3512 src/config/mod.rs
```
- **Suggested Fix:** Split into: config/agent.rs, config/settings.rs, config/validation.rs, config/tests/
- **Status:** 🔄 RECURRING

---

### 🟠 High Priority Issues

#### [HIGH-001] unwrap()/expect() in Production Code (Skills Violation)

- **Category:** Skill Violation
- **Severity:** High
- **Effort:** M
- **Risk:** Medium
- **Priority Score:** 13/22
- **Skill:** `skills/rust-best-practices/SKILL.md` §4.2, `skills/rust-engineer/SKILL.md` §MUST NOT DO
- **Files:** `src/cli/mod.rs` (line 348), `src/scheduler/mod.rs` (lines 1164, 1173, 1182, 1257), `src/logging.rs` (lines 102, 113, 137, 145, 150), `src/docker/run/streams.rs` (line 41), `src/discord/llm.rs` (lines 333, 350)
- **Description:** According to rust-best-practices skill: "Never use unwrap()/expect() outside tests". These patterns violate the project's skill conventions and can cause runtime panics.
- **Evidence:**
```rust
// src/cli/mod.rs:348
let docker = client.docker().expect("Docker client should be available");

// src/scheduler/mod.rs:1164
*self.queue_wait_time_seconds.lock().unwrap()

// src/logging.rs:102
*INIT_ERROR.lock().unwrap() = Some(err);

// src/docker/run/streams.rs:41
let docker = client.docker().expect("Docker client should be available");

// src/discord/llm.rs:333
.build().expect("Failed to build HTTP client");
```
- **Suggested Fix:** Replace with proper Result handling and ? operator, or use expect with meaningful error context for truly impossible failures
- **Status:** 🔄 RECURRING (×2)

---

#### [HIGH-002] Clippy Warnings - Dead Code in Test Files

- **Category:** Code Quality
- **Severity:** High
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 16/22
- **Files:** `tests/performance_common.rs` (multiple unused items)
- **Description:** Multiple unused functions, structs, and enums in test files trigger clippy warnings. While not critical, these reduce signal-to-noise ratio.
- **Evidence:**
```rust
// tests/performance_common.rs
warning: enum `RegressionStatus` is never used
warning: struct `BaselineTracker` is never constructed
warning: function `detect_regression` is never used
warning: function `log_regression_warning` is never used
```
- **Suggested Fix:** Remove unused test code or mark as #[allow(unused)]
- **Status:** 🔄 RECURRING

---

#### [HIGH-003] CLI Module - 2144 Lines

- **Category:** Structure
- **Severity:** High
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 13/22
- **Files:** `src/cli/mod.rs` (2144 lines)
- **Description:** Contains all CLI commands and handlers in single file, making it hard to navigate.
- **Suggested Fix:** Extract commands to individual files in commands/ directory
- **Status:** 🔄 RECURRING

---

#### [HIGH-004] Commands Module - 2074 Lines

- **Category:** Structure
- **Severity:** High
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 13/22
- **Files:** `src/commands/skills.rs` (2074 lines)
- **Description:** Single file for all skills subcommands, difficult to maintain.
- **Suggested Fix:** Extract to skills/list.rs, skills/install.rs, skills/update.rs
- **Status:** 🔄 RECURRING

---

### 🟡 Medium Priority (Refactoring)

#### [MED-001] discord/tools.rs - 1663 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/discord/tools.rs` (1663 lines)
- **Description:** Large file with tool definitions and executions.
- **Suggested Fix:** Split into tools/definitions.rs, tools/execution.rs
- **Status:** 🔄 RECURRING

---

#### [MED-002] discord/llm.rs - 1539 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/discord/llm.rs` (1539 lines)
- **Description:** Large LLM client module.
- **Suggested Fix:** Split into llm/client.rs, llm/response.rs
- **Status:** 🔄 RECURRING

---

#### [MED-003] docker/skills.rs - 1489 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/docker/skills.rs` (1489 lines)
- **Description:** Skill-related Docker integration module.
- **Suggested Fix:** Extract skill validation to separate module
- **Status:** 🔄 RECURRING

---

#### [MED-004] docker/mod.rs - 1394 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/docker/mod.rs` (1394 lines)
- **Description:** Docker client wrapper module.
- **Suggested Fix:** Split into docker/client.rs, docker/build.rs
- **Status:** 🔄 RECURRING

---

#### [MED-005] skills/mod.rs - 2709 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/skills/mod.rs` (2709 lines)
- **Description:** All skills management logic in single file.
- **Suggested Fix:** Split into skills/manager.rs, skills/lockfile.rs, skills/metadata.rs
- **Status:** 🔄 RECURRING

---

#### [MED-006] Unused Config Key Warning

- **Category:** Dependency
- **Severity:** Medium
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 12/22
- **Files:** `.cargo/config.toml`
- **Description:** Warning: unused config key `profile.test.features`
- **Evidence:**
```
warning: unused config key `profile.test.features` in `/workspace/.cargo/config.toml`
```
- **Suggested Fix:** Remove or fix the config key in .cargo/config.toml
- **Status:** 🔄 RECURRING

---

### 🔵 Low Priority

#### [LOW-001] scheduler/mod.rs - 1259 Lines

- **Category:** Structure
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** `src/scheduler/mod.rs` (1259 lines)
- **Description:** Large scheduler module - borderline acceptable size.
- **Suggested Fix:** Consider splitting into scheduler/clock.rs, scheduler/queue.rs
- **Status:** 🔄 RECURRING

---

#### [LOW-002] metrics/store.rs - 1091 Lines

- **Category:** Structure
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** `src/metrics/store.rs` (1091 lines)
- **Description:** Large metrics storage module.
- **Suggested Fix:** Consider splitting
- **Status:** 🔄 RECURRING

---

#### [LOW-003] discord/config.rs - 1095 Lines

- **Category:** Structure
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** `src/discord/config.rs` (1095 lines)
- **Description:** Large discord config module.
- **Suggested Fix:** Consider splitting
- **Status:** 🔄 RECURRING

---

### ⚪ Convention Issues

#### [CONV-001] Inconsistent Error Handling

- **Category:** Convention
- **Severity:** Low
- **Effort:** M
- **Risk:** Medium
- **Priority Score:** 7/22
- **Files:** Mixed Result<T,E> vs Box<dyn Error>
- **Description:** Some functions use Result<T,E>, others Box<dyn std::error::Error>. Standardize on Result<T,E> with custom error types per skill guidelines.
- **Suggested Fix:** Standardize on Result<T,E> with custom error types using thiserror
- **Status:** 🔄 RECURRING

---

#### [CONV-002] Module Organization - Discord

- **Category:** Convention
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** `src/discord/*.rs` (8 files)
- **Description:** Some modules could be reorganized into subdirectories.
- **Suggested Fix:** Consider discord/api/, discord/gateway/ subdirectories
- **Status:** 🔄 RECURRING

---

#### [CONV-003] Test Organization

- **Category:** Convention
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 8/22
- **Files:** `tests/` (25+ test files)
- **Description:** Mix of unit tests in src/ and integration tests in tests/. Follow standard Rust conventions strictly.
- **Suggested Fix:** Move inline tests to tests/ directory
- **Status:** 🔄 RECURRING

---

#### [CONV-004] Missing Documentation

- **Category:** Documentation
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 8/22
- **Files:** Multiple
- **Description:** Some private functions lack doc comments.
- **Suggested Fix:** Add docs to public API entry points
- **Status:** 🔄 RECURRING

---

## Recently Resolved

### FIND-MED-001 — Duplicate Docker Expect Pattern
- **Resolved:** 2026-02-27 (commit fe7aff0)
- **Resolution:** Extracted get_docker helper method in docker/mod.rs

### FIND-MED-002 — Magic Strings
- **Resolved:** 2026-02-27 (commit 10b4c76)
- **Resolution:** Extracted DOCKER_NOT_AVAILABLE constant in docker/mod.rs

### FIND-CONV-005 — Backup File Present
- **Resolved:** 2026-02-27 (commit 28ee418)
- **Resolution:** Deleted src/config/mod.rs.bak

---

## Systemic Patterns

### Pattern: Expect on Internal State
- **Occurrences:** 15+ files
- **Files Affected:** docker/run/streams.rs, scheduler/mod.rs, logging.rs, cli/mod.rs, discord/llm.rs
- **Description:** Using .expect() on internal state that should never fail (like Arc<Mutex> locks, Option fields that are set at construction)
- **Recommendation:** Create type-safe wrappers or use expect_with() for internal invariants

### Pattern: Giant Test Modules in Source
- **Occurrences:** config/mod.rs (2000+ lines tests), skills/mod.rs, discord/config.rs
- **Description:** Tests inline in source files make files huge
- **Recommendation:** Move tests to tests/ directory

---

## Recommendations Roadmap

### Immediate (This Sprint)
- [ ] Fix 25 failing tests - 8h
- [ ] Remove dead code in test files (clippy warnings) - 1h

### Short-term (Next 2-4 weeks)
- [ ] Replace unwrap/expect with proper error handling - 6h
- [ ] Split docker/run/run.rs (5115 lines) - 4h
- [ ] Split config/mod.rs (3512 lines) - 4h

### Long-term (Backlog)
- [ ] Split skills/mod.rs (2709 lines) - 3h
- [ ] Split cli/mod.rs (2144 lines) - 3h
- [ ] Split commands/skills.rs (2074 lines) - 3h

---

## Appendix

### Files Scanned (Top 20 by Line Count)
| File | Lines |
|------|-------|
| src/docker/run/run.rs | 5115 |
| src/config/mod.rs | 3512 |
| src/skills/mod.rs | 2709 |
| src/cli/mod.rs | 2144 |
| src/commands/skills.rs | 2074 |
| src/discord/tools.rs | 1663 |
| src/discord/llm.rs | 1539 |
| src/docker/skills.rs | 1489 |
| src/commands/validate.rs | 1445 |
| src/docker/mod.rs | 1394 |
| src/scheduler/mod.rs | 1259 |
| src/discord/config.rs | 1095 |
| src/metrics/store.rs | 1091 |
| src/discord/security.rs | 899 |
| src/discord/api.rs | 882 |
| src/traits/mod.rs | 879 |
| src/discord/mod.rs | 877 |
| src/discord/conversation.rs | 823 |
| src/skills/error.rs | 735 |
| src/commands/metrics.rs | 604 |

### Skipped Files
- None - full codebase scanned

### Skills Compliance Notes
The codebase has two active skills:
1. **rust-best-practices** - Key violations found:
   - unwrap()/expect() in production code (HIGH-001)
   
2. **rust-engineer** - Key violations found:
   - Error handling patterns inconsistent (CONV-001)

### Changes Since Last Audit
- **NEW**: docker/run/run.rs grew to 5115 lines (now a god module)
- **FIXED**: Duplicate docker expect pattern (MED-001 resolved)
- **FIXED**: Magic strings (MED-002 resolved)
- **FIXED**: Backup file removed (CONV-005 resolved)
- **SAME**: 25 test failures persist
- **SAME**: Other god modules unchanged
