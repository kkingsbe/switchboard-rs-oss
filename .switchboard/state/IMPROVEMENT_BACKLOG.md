# Codebase Scan Report

**Project**: switchboard  
**Scanned**: 2026-02-28T02:25:00Z  
**Commit Audited**: 39b6682
**Scope**: Full codebase (src/, tests/)  
**Files Analyzed**: ~80 Rust source files  
**Audit Type**: Continuation (previous audit: 2026-02-28T00:12:00Z)

---

## Executive Summary

| Severity | Count | Change vs Last Audit |
|----------|-------|---------------------|
| 🔴 Critical | 3 | - |
| 🟠 High | 4 | - |
| 🟡 Medium | 4 | - |
| 🔵 Low | 3 | - |
| ⚪ Convention | 4 | - |

**Overall Health Score**: 5/10 (degraded from 5.5/10)

**Top 3 Priorities**:
1. Fix test compilation errors (CRITICAL - tests won't compile due to missing `use std::fs;`)
2. Fix formatting inconsistencies
3. Continue addressing god modules

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
| `cargo build` | ✅ PASS | 6 warnings (unused imports) |
| `cargo test --no-run` | ❌ FAIL | 21 compilation errors - tests won't compile |
| `cargo test` | ❌ FAIL | 527 passed, 20 failed (cannot run due to compilation errors) |
| `cargo clippy` | ⚠️ WARN | 5 unused imports |
| `cargo fmt --check` | ❌ FAIL | 8+ files with formatting issues |

---

## Findings by Category

### 🔴 Critical Issues

#### [CRIT-001] Test Compilation Errors - Missing `use std::fs;` Import

- **Category:** Testing
- **Severity:** Critical
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 19/22
- **Files:** `src/skills/mod.rs` (lines 428, 436, 448, 456, 472, 479, 483, 490, 519, 526, 530, 534, 645, 650, 671, 676, 699, 702, 706, 709, 958)
- **Description:** Tests don't compile because `fs` module is used but not imported. Multiple test functions call `fs::create_dir_all()`, `fs::write()`, and `fs::read_to_string()` without importing `std::fs`. This is a REGRESSION - the issue was present in the previous audit.
- **Evidence:**
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `fs`
   --> src/skills/mod.rs:428:9
    |
428 |         fs::create_dir_all(&skill_dir).unwrap();
    |         ^^ use of unresolved module or unlinked crate `fs`
```
- **Suggested Fix:** Add `use std::fs;` to the test module in `src/skills/mod.rs`
- **Status:** 🔄 RECURRING → SCHEDULED: Improvement Sprint 3, assigned to REFACTOR_TODO1.md

---

#### [CRIT-002] God Module - docker/run/run.rs at 5115 Lines

- **Category:** Structure
- **Severity:** Critical
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 14/22
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

#### [CRIT-003] God Module - config/mod.rs at 3512 Lines

- **Category:** Structure
- **Severity:** Critical
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 14/22
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

### 🟠 High Priority Issues

#### [HIGH-001] unwrap()/expect() in Production Code (Skills Violation)

- **Category:** Skill Violation
- **Severity:** High
- **Effort:** M
- **Risk:** Medium
- **Priority Score:** 13/22
- **Skill:** `skills/rust-best-practices/SKILL.md` §4.2, `skills/rust-engineer/SKILL.md` §MUST NOT DO
- **Files:** 
  - `src/cli/mod.rs:348` - `client.docker().expect("Docker client should be available")`
  - `src/scheduler/mod.rs:1164,1173,1182,1257` - mutex lock `.unwrap()`
  - `src/logging.rs:102,113,137,145,150` - mutex lock `.unwrap()`
  - `src/docker/run/streams.rs:41` - docker client expect
- **Description:** According to rust-best-practices skill: "Never use unwrap()/expect() outside tests". These patterns violate the project's skill conventions.
- **Evidence:**
```rust
// src/cli/mod.rs:348
let docker = client.docker().expect("Docker client should be available");

// src/scheduler/mod.rs:1164
*self.queue_wait_time_seconds.lock().unwrap()

// src/logging.rs:102
*INIT_ERROR.lock().unwrap() = Some(err);
```
- **Suggested Fix:** Replace with proper Result handling and ? operator, or use expect with meaningful error context
- **Status:** 🔄 RECURRING → SCHEDULED: Improvement Sprint 3, assigned to REFACTOR_TODO2.md

---

#### [HIGH-002] Formatting Inconsistencies - 8 Files

- **Category:** Code Quality
- **Severity:** High
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 16/22
- **Files:** 
  - `src/commands/validate.rs`
  - `src/discord/llm/mod.rs`
  - `src/discord/tools/mod.rs`
  - `src/docker/mod.rs`
  - `src/skills/manager.rs`
  - `src/skills/mod.rs`
- **Description:** Multiple files fail `cargo fmt --check` with import ordering and re-export formatting issues.
- **Evidence:**
```
Diff in /workspace/src/discord/tools/mod.rs:13:
-pub use definitions::{Tool, ToolError, MAX_FILE_SIZE, tools_schema};
+pub use definitions::{tools_schema, Tool, ToolError, MAX_FILE_SIZE};
```
- **Suggested Fix:** Run `cargo fmt` to fix formatting
- **Status:** 🔄 RECURRING → SCHEDULED: Improvement Sprint 3, assigned to REFACTOR_TODO1.md

---

#### [HIGH-003] CLI Module - 2144 Lines

- **Category:** Structure
- **Severity:** High
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 13/22
- **Files:** `src/cli/mod.rs` (2144 lines)
- **Description:** Contains all CLI commands and handlers in single file.
- **Status:** 🔄 RECURRING → SCHEDULED: Improvement Sprint 3, assigned to REFACTOR_TODO2.md

---

#### [HIGH-004] Commands Module - commands/skills.rs at 2074 Lines

- **Category:** Structure
- **Severity:** High
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 13/22
- **Files:** `src/commands/skills.rs` (2074 lines)
- **Description:** Single file for all skills subcommands.
- **Status:** 🔄 RECURRING

---

### 🟡 Medium Priority (Refactoring)

#### [MED-001] discord/llm.rs - 1539 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/discord/llm.rs` (1539 lines)
- **Status:** 🔄 RECURRING → SCHEDULED: Improvement Sprint 3, assigned to REFACTOR_TODO2.md

---

#### [MED-002] docker/skills.rs - 1282 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/docker/skills.rs` (1282 lines)
- **Status:** 🔄 RECURRING

---

#### [MED-003] docker/mod.rs - 1394 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/docker/mod.rs` (1394 lines)
- **Status:** 🔄 RECURRING

---

#### [MED-004] Unused Imports (Clippy Warnings)

- **Category:** Code Quality
- **Severity:** Medium
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 12/22
- **Files:** 
  - `src/discord/llm/client.rs:14` - unused `ResponseMessage`, `ResponseToolCall`
  - `src/discord/llm/response.rs:6` - unused `ToolFunction`
  - `src/discord/tools/execution.rs:10` - unused `serde_json::Value`
  - `src/skills/manager.rs:5` - unused `Path`
  - `src/skills/lockfile.rs:369` - unused `SkillMetadata`
- **Evidence:**
```rust
warning: unused import: `serde_json::Value`
  --> src/discord/tools/execution.rs:10:5
   |
10 | use serde_json::Value;
    |     ^^^^^^^^^^^^^^^^^
```
- **Status:** 🔄 RECURRING → SCHEDULED: Improvement Sprint 3, assigned to REFACTOR_TODO1.md

---

### 🔵 Low Priority

#### [LOW-001] scheduler/mod.rs - 1259 Lines

- **Category:** Structure
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** `src/scheduler/mod.rs` (1259 lines)
- **Status:** 🔄 RECURRING

---

#### [LOW-002] metrics/store.rs - 1091 Lines

- **Category:** Structure
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** `src/metrics/store.rs` (1091 lines)
- **Status:** 🔄 RECURRING

---

#### [LOW-003] discord/config.rs - 1095 Lines

- **Category:** Structure
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** `src/discord/config.rs` (1095 lines)
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
- **Status:** 🔄 RECURRING

---

#### [CONV-002] Module Organization - Discord

- **Category:** Convention
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Status:** 🔄 RECURRING

---

#### [CONV-003] Test Organization

- **Category:** Convention
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 8/22
- **Status:** 🔄 RECURRING

---

#### [CONV-004] Missing Documentation

- **Category:** Documentation
- **Severity:** Low
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 8/22
- **Status:** 🔄 RECURRING

---

## Recommendations Roadmap

### Immediate (This Sprint)
- [ ] Fix test compilation errors (add `use std::fs;` to test modules in src/skills/mod.rs) - 0.5h
- [ ] Run `cargo fmt` to fix formatting issues - 0.5h

### Short-term (Next 2-4 weeks)
- [ ] Replace unwrap/expect with proper error handling - 6h
- [ ] Remove unused imports (clippy warnings) - 1h

### Long-term (Backlog)
- [ ] Split docker/run/run.rs (5115 lines) - 4h
- [ ] Split config/mod.rs (3512 lines) - 4h

---

## Appendix

### Files Scanned (Top 20 by Line Count)
| File | Lines |
|------|-------|
| src/docker/run/run.rs | 5115 |
| src/config/mod.rs | 3512 |
| src/cli/mod.rs | 2144 |
| src/commands/skills.rs | 2074 |
| src/commands/validate.rs | 1445 |
| src/docker/skills.rs | 1282 |
| src/scheduler/mod.rs | 1259 |
| src/discord/config.rs | 1095 |
| src/metrics/store.rs | 1091 |
| src/skills/mod.rs | 1038 |
| src/docker/client.rs | 1025 |
| src/discord/security.rs | 899 |
| src/discord/api.rs | 882 |
| src/traits/mod.rs | 879 |
| src/discord/mod.rs | 877 |
| src/skills/metadata.rs | 873 |
| src/discord/tools/mod.rs | 826 |
| src/discord/conversation.rs | 823 |
| src/skills/error.rs | 735 |
| src/docker/tools/execution.rs | 650 |

### Skills Compliance Notes
The codebase has two active skills:
1. **rust-best-practices** - Key violations found:
   - unwrap()/expect() in production code (HIGH-001)
   
2. **rust-engineer** - Key violations found:
   - Error handling patterns inconsistent (CONV-001)

### Health Trend
**Health Score**: 5/10 (degraded from 5.5/10)

**Reason for degradation**:
- Test compilation errors persist (CRITICAL - was present in previous audit)
- 20 tests failing due to inability to compile test code
- Formatting issues persist across 8+ files
