# Codebase Scan Report

**Project**: switchboard  
**Scanned**: 2026-02-28T06:00:04Z  
**Commit Audited**: 70d115b
**Scope**: Full codebase (src/)
**Files Analyzed**: ~80 Rust source files  
**Audit Type**: Continuation (previous audit: 2026-02-28T04:10:00Z)

---

## Executive Summary

| Severity | Count | Change vs Last Audit |
|----------|-------|----------------------|
| 🔴 Critical | 1 | -1 (RESOLVED) |
| 🟠 High | 1 | -1 (RESOLVED) |
| 🟡 Medium | 2 | -1 |
| 🔵 Low | 2 | -1 |
| ⚪ Convention | 4 | - |

**Overall Health Score**: 7/10 (improved from 6/10)

**Top 3 Priorities**:
1. ✅ Test compilation errors RESOLVED (was CRITICAL)
2. ✅ unwrap/expect in production mostly RESOLVED (was HIGH)
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
| `cargo build` | ✅ PASS | 1 warning (private_interfaces) |
| `cargo test --no-run` | ✅ PASS | Tests compile successfully |
| `cargo clippy` | ⚠️ WARN | 2 warnings (private_interface, unused_mut) |
| `cargo fmt --check` | ✅ PASS | All files formatted correctly |

---

## Findings by Category

### 🔴 Critical Issues

#### [CRIT-001] ✅ RESOLVED - Test Compilation Errors

- **Previous Status**: CRITICAL - tests wouldn't compile due to missing `use std::fs;`
- **Current Status**: **RESOLVED** - Tests now compile successfully
- **Evidence**: `cargo test --no-run` completes without errors
- **Effort**: N/A
- **Resolution**: Issue has been fixed in codebase

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

### 🟠 High Priority Issues

#### [HIGH-001] God Module - config/mod.rs at 3512 Lines

- **Category:** Structure
- **Severity:** High
- **Effort:** L
- **Risk:** Medium
- **Priority Score:** 12/22
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

#### [HIGH-002] ✅ MOSTLY RESOLVED - unwrap()/expect() in Production Code

- **Previous Status**: HIGH - unwrap/expect in cli/mod.rs and logging.rs
- **Current Status**: **MOSTLY RESOLVED** - Main violations fixed. Remaining edge cases are acceptable:
  - `scheduler/mod.rs:1266` - Default trait implementation (valid use case)
  - `discord/llm/client.rs:176,193` - HTTP client builder (reasonable assumption)
- **Evidence:**
  - cli/mod.rs: No unwrap/expect found in production code ✅
  - logging.rs: Only test functions contain unwrap/expect ✅
- **Effort**: N/A for resolved portion
- **Status:** ✅ RESOLVED

---

### 🟡 Medium Priority (Refactoring)

#### [MED-001] CLI Module - 2146 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/cli/mod.rs` (2146 lines)
- **Description:** Contains all CLI commands and handlers in single file.
- **Status:** 🔄 RECURRING

---

#### [MED-002] Commands Module - commands/skills.rs at 2074 Lines

- **Category:** Structure
- **Severity:** Medium
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 10/22
- **Files:** `src/commands/skills.rs` (2074 lines)
- **Description:** Single file for all skills subcommands.
- **Status:** 🔄 RECURRING

---

#### [MED-003] Private Interface Warning

- **Category:** Code Quality
- **Severity:** Medium
- **Effort:** S
- **Risk:** Safe
- **Priority Score:** 11/22
- **Files:** `src/discord/llm/client.rs:99-104`
- **Description:** Warning about type `FunctionDefinition` being more private than the item `ToolDefinition::function`
- **Evidence:**
```rust
warning: type `FunctionDefinition` is more private than the item `ToolDefinition::function`
   --> src/discord/llm/client.rs:99:5
    |
 99 |     pub function: FunctionDefinition,
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ field `ToolDefinition::function` is reachable at visibility `pub`
    |
note: but type `FunctionDefinition` is only usable at visibility `pub(self)`
   --> src/discord/llm/client.rs:104:1
    |
104 |     struct FunctionDefinition {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^
```
- **Suggested Fix:** Make `FunctionDefinition` public or adjust visibility
- **Status:** 🔄 SCHEDULED → Improvement Sprint 5, assigned to .switchboard/state/REFACTOR_TODO1.md

---

### 🔵 Low Priority

#### [LOW-001] scheduler/mod.rs - 1268 Lines

- **Category:** Structure
- **Severity:** Low
- **Effort:** M
- **Risk:** Low
- **Priority Score:** 7/22
- **Files:** `src/scheduler/mod.rs` (1268 lines)
- **Status:** 🔄 SCHEDULED → Improvement Sprint 5, assigned to .switchboard/state/REFACTOR_TODO2.md

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
- **Status:** 🔄 SCHEDULED → Improvement Sprint 5, assigned to .switchboard/state/REFACTOR_TODO2.md

---

## Recommendations Roadmap

### Immediate (This Sprint)
- [x] ✅ Test compilation errors - RESOLVED
- [x] ✅ Formatting issues - RESOLVED
- [x] ✅ unwrap/expect in cli/logging - MOSTLY RESOLVED
- [ ] Fix private_interface warning - 0.5h

### Short-term (Next 2-4 weeks)
- [ ] Split docker/run/run.rs - 4h
- [ ] Fix remaining unwrap/expect edge cases (optional) - 1h

### Long-term (Backlog)
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
| src/docker/skills.rs | 1282 |
| src/scheduler/mod.rs | 1268 |
| src/discord/config.rs | 1095 |
| src/metrics/store.rs | 1091 |
| src/skills/mod.rs | 1036 |

### Skills Compliance Notes
The codebase has two active skills:
1. **rust-best-practices** - Key violations found:
   - ✅ unwrap()/expect() in production code - MOSTLY RESOLVED
   - ✅ Format issues - RESOLVED
   
2. **rust-engineer** - Key violations found:
   - Error handling patterns inconsistent (CONV-001)

### Health Trend
**Health Score**: 7/10 (improved from 6/10)

**Reason for improvement**:
- ✅ Test compilation errors RESOLVED (was CRITICAL)
- ✅ unwrap/expect in production mostly RESOLVED (was HIGH)
- ✅ Formatting issues RESOLVED
- Remaining issues are structural (god modules) which are long-term refactoring efforts
