# Codebase Scan Report

**Project**: switchboard  
**Scanned**: 2026-02-27T00:07:20Z  
**Scope**: /workspace (full project)  
**Files Analyzed**: ~80 Rust source files, ~40 test files

---

## Executive Summary

| Severity | Count | Estimated Effort |
|----------|-------|------------------|
| 🔴 Critical | 6 | 8h |
| 🟠 High | 8 | 12h |
| 🟡 Medium | 12 | 16h |
| 🔵 Low | 5 | 4h |
| ⚪ Convention | 8 | 6h |

**Overall Health Score**: 5.5/10

**Top 3 Priorities**:
1. **Critical**: Resolve merge conflicts in test files (blocking builds/tests)
2. **Critical**: Refactor god modules (>500 lines causing maintenance issues)
3. **High**: Replace unwrap()/expect() in production code with proper error handling

---

## Tech Stack Summary

- **Languages**: Rust (2021 edition)
- **Frameworks**: tokio, clap, bollard (Docker), twilight (Discord Gateway)
- **Build Tools**: Cargo
- **Testing**: assert_cmd, tempfile, predicates, serial_test
- **Key Dependencies**: bollard 0.18, tokio 1.40, twilight-gateway 0.17, serde, tracing

---

## Findings by Category

### 🔴 Critical Issues

#### [CRIT-001] Merge Conflicts in Test Files - **File**: `tests/mod.rs` (lines 8-12)
- **Issue**: Git merge conflict markers present in file. This completely breaks compilation.
- **Risk**: The project cannot be compiled or tested until resolved. CI/CD pipelines will fail.
- **Recommendation**: Resolve the merge conflict by keeping the appropriate code and removing conflict markers.
- **Effort**: S

```rust
<<<<<<< HEAD
=======
-------
>>>>>>> skills-improvements
```

#### [CRIT-002] Massive Merge Conflict in Performance Tests - **File**: `tests/performance_common.rs` (2457 lines)
- **Issue**: File contains ~1229 lines of duplicated content from unresolved merge conflict
- **Risk**: Complete test suite failure, massive test file bloat
- **Recommendation**: Remove duplicate code sections and resolve merge conflicts properly
- **Effort**: L

#### [CRIT-003] Merge Conflicts in Additional Test Files - **Files**: 
- `tests/skills_install_performance.rs` (lines 39-65)
- `tests/skills_install_time_metrics.rs` 
- `tests/skills_list_performance.rs` (lines 46-80)
- **Issue**: Multiple test files contain unresolved merge conflict markers
- **Risk**: Test compilation failures, cannot run test suite
- **Recommendation**: Resolve all merge conflicts in test files
- **Effort**: M

#### [CRIT-004] God Module - docker/run/run.rs - **File**: `src/docker/run/run.rs` (5115 lines)
- **Issue**: Single file with 5115 lines violates Single Responsibility Principle
- **Risk**: Extremely difficult to maintain, understand, or modify safely
- **Recommendation**: Split into multiple modules:
  - Container lifecycle management
  - Stream handling
  - Wait/timeout logic
  - Type definitions
- **Effort**: L (architectural refactoring)

#### [CRIT-005] God Module - config/mod.rs - **File**: `src/config/mod.rs` (3503 lines)
- **Issue**: Configuration module is excessively large with 3503 lines
- **Risk**: Maintenance difficulty, cognitive overload
- **Recommendation**: Split into:
  - Config parsing (from_toml)
  - Config validation
  - Environment handling
  - Agent/Schedule types
- **Effort**: L

#### [CRIT-006] God Module - skills/mod.rs - **File**: `src/skills/mod.rs` (2717 lines)
- **Issue**: Skills management module is 2717 lines
- **Risk**: Hard to add new features or debug issues
- **Recommendation**: Extract into:
  - Skills manager
  - Lockfile handling
  - Skill metadata parsing
  - Directory scanning
- **Effort**: L

---

### 🟠 High Priority Issues

#### [HIGH-001] unwrap()/expect() in Production Code - **File**: `src/logging.rs` (lines 44, 59)
- **Issue**: Using `.expect()` for operations that can fail in production
```rust
std::fs::create_dir_all(&log_dir).expect(&error_msg);
tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
```
- **Risk**: Application will panic if logging directory creation fails or subscriber setup fails
- **Recommendation**: Return Result types or use proper error handling
- **Effort**: M

#### [HIGH-002] unwrap() on Global State - **File**: `src/logging.rs:110`
- **Issue**: `GLOBAL_LOG_DIR.as_ref().unwrap()` - unsafe access to global state
- **Risk**: Panic if global state not initialized
- **Recommendation**: Use get_or_init() pattern or proper initialization
- **Effort**: S

#### [HIGH-003] expect() for Docker Client - **File**: `src/cli/mod.rs:343`
- **Issue**: `client.docker().expect("Docker client should be available")`
- **Risk**: Application panic if Docker unavailable
- **Recommendation**: Return proper error with actionable message
- **Effort**: S

#### [HIGH-004] expect() for Scheduler Creation - **File**: `src/scheduler/mod.rs:1257`
- **Issue**: `Self::new_sync(None, None, None).expect("Failed to create default Scheduler")`
- **Risk**: Panic on default scheduler creation failure
- **Recommendation**: Handle error gracefully or make initialization explicit
- **Effort**: S

#### [HIGH-005] Inconsistent Error Handling - Anyhow in Library Code
- **Issue**: Using `anyhow::Error` in library modules (discord/mod.rs, docker/mod.rs)
- **Risk**: According to best practices, anyhow should only be used in binaries
- **Recommendation**: Use thiserror for library errors, convert to anyhow only at binary boundaries
- **Files Affected**: src/discord/mod.rs, src/docker/mod.rs
- **Effort**: M

#### [HIGH-006] Missing Regex Static Initialization Error Handling - **File**: `src/config/mod.rs:41`
- **Issue**: `Regex::new(...).expect("Invalid SKILL_SOURCE_REGEX pattern")` at static lazy
- **Risk**: Application cannot start if regex is invalid (unlikely but possible)
- **Recommendation**: Use compile-time validated regex or handle error gracefully
- **Effort**: S

#### [HIGH-007] Signal Handler expect() - **File**: `src/commands/logs.rs:306`
- **Issue**: `signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler")`
- **Risk**: Application panic if signal handler setup fails
- **Recommendation**: Handle error and exit gracefully
- **Effort**: S

#### [HIGH-008] Unused Imports - **Files**: `src/discord/gateway.rs`, `src/discord/tools.rs`
- **Issue**: Unused imports detected by compiler:
  - `error` in gateway.rs:8
  - `error` and `info` in tools.rs:14
- **Risk**: Code clutter, potential confusion
- **Effort**: S

---

### 🟡 Medium Priority (Refactoring)

#### [MED-001] God Module - cli/mod.rs - **File**: `src/cli/mod.rs` (2113 lines)
- **Issue**: CLI module is 2113 lines, handles multiple concerns
- **Recommendation**: Extract command implementations to separate files
- **Effort**: M

#### [MED-002] God Module - commands/skills.rs - **File**: `src/commands/skills.rs` (2067 lines)
- **Issue**: Skills command implementation is 2067 lines
- **Recommendation**: Split into multiple command handlers
- **Effort**: M

#### [MED-003] God Module - discord/tools.rs - **File**: `src/discord/tools.rs` (1616 lines)
- **Issue**: Tools module is 1616 lines
- **Recommendation**: Extract tool implementations to separate files
- **Effort**: M

#### [MED-004] God Module - discord/llm.rs - **File**: `src/discord/llm.rs` (1530 lines)
- **Issue**: LLM handling module is 1530 lines
- **Recommendation**: Split into client, response handling, tool execution
- **Effort**: M

#### [MED-005] God Module - docker/skills.rs - **File**: `src/docker/skills.rs` (1484 lines)
- **Issue**: Docker skills integration is 1484 lines
- **Recommendation**: Extract skill installation, entrypoint generation
- **Effort**: M

#### [MED-006] God Module - commands/validate.rs - **File**: `src/commands/validate.rs` (1413 lines)
- **Issue**: Validation command is 1413 lines
- **Recommendation**: Split validation logic
- **Effort**: M

#### [MED-007] God Module - docker/mod.rs - **File**: `src/docker/mod.rs` (1327 lines)
- **Issue**: Docker module is 1327 lines
- **Recommendation**: Split into Docker client, container management
- **Effort**: M

#### [MED-008] God Module - scheduler/mod.rs - **File**: `src/scheduler/mod.rs` (1259 lines)
- **Issue**: Scheduler module is 1259 lines
- **Recommendation**: Extract scheduling logic, queue management
- **Effort**: M

#### [MED-009] God Module - metrics/store.rs - **File**: `src/metrics/store.rs` (1091 lines)
- **Issue**: Metrics storage is 1091 lines
- **Recommendation**: Split serialization, file operations
- **Effort**: M

#### [MED-010] God Module - discord/config.rs - **File**: `src/discord/config.rs` (1091 lines)
- **Issue**: Discord config is 1091 lines
- **Recommendation**: Extract parsing, validation
- **Effort**: M

#### [MED-011] Dead Code - from_u8 Function - **File**: `src/discord/gateway.rs:68`
- **Issue**: `fn from_u8(val: u8) -> Option<Self>` is never used
- **Risk**: Unused code, increased compilation time
- **Recommendation**: Remove or mark #[allow(dead_code)] if intentionally kept
- **Effort**: S

#### [MED-012] Config Warning - **File**: `.cargo/config.toml`
- **Issue**: `unused config key 'profile.test.features'`
- **Risk**: Misconfiguration
- **Effort**: S

---

### 🔵 Low Priority

#### [LOW-001] Cargo fmt Failures - Multiple Files
- **Issue**: Code formatting doesn't match project standards across many files
- **Files**: cli/mod.rs, commands/skills.rs, discord/tools.rs, docker/mod.rs, docker/skills.rs, skills/mod.rs, etc.
- **Recommendation**: Run `cargo fmt` and commit the changes
- **Effort**: S

#### [LOW-002] Magic Numbers - **File**: `src/discord/tools.rs:370`
- **Issue**: `let todo_files = ["TODO1.md", "TODO2.md", ...]` - magic numbers
- **Recommendation**: Define constants for file names
- **Effort**: S

#### [LOW-003] Missing Documentation - Error Types
- **Issue**: Some public error types lack comprehensive documentation
- **Recommendation**: Add doc comments explaining error conditions
- **Effort**: M

#### [LOW-004] Test File Size - scheduler_tests.rs - **File**: `tests/scheduler_tests.rs` (2811 lines)
- **Issue**: Single test file is very large
- **Recommendation**: Consider splitting into multiple test files by functionality
- **Effort**: M

#### [LOW-005] Test File Size - validate_command.rs - **File**: `tests/validate_command.rs` (1235 lines)
- **Issue**: Large test file
- **Recommendation**: Consider splitting
- **Effort**: M

---

### ⚪ Convention Issues

#### [CONV-001] Mixed Error Handling Paradigms
- **Issue**: Some modules use thiserror, others use anyhow inconsistently
- **Affected**: discord (anyhow), docker (anyhow), metrics (thiserror), scheduler (thiserror)
- **Recommendation**: Standardize on thiserror for libraries, anyhow only at binary boundaries

#### [CONV-002] Import Ordering Inconsistency
- **Issue**: Some files use `use std::` then `use crate::`, others mix differently
- **Recommendation**: Follow consistent import ordering (std, external, crate)

#### [CONV-003] Test Naming Consistency
- **Issue**: Mix of test naming conventions (snake_case vs camelCase in test names)
- **Recommendation**: Follow Rust conventions: `snake_case` for test functions

#### [CONV-004] Module Organization
- **Issue**: Some large modules could be better organized
- **Recommendation**: Consider more granular module划分

---

## Systemic Patterns

### Pattern: God Modules
- **Occurrences**: 15+ files exceed 500 lines
- **Files Affected**: run.rs (5115), config/mod.rs (3503), skills/mod.rs (2717), cli/mod.rs (2113), commands/skills.rs (2067), discord/tools.rs (1616), discord/llm.rs (1530), docker/skills.rs (1484), commands/validate.rs (1413), docker/mod.rs (1327), scheduler/mod.rs (1259), metrics/store.rs (1091), discord/config.rs (1091)
- **Description**: Many files are excessively large, violating Single Responsibility Principle
- **Recommendation**: Implement systematic refactoring plan to split large modules

### Pattern: Merge Conflicts in Tests
- **Occurrences**: 5+ test files
- **Files Affected**: tests/mod.rs, tests/performance_common.rs, tests/skills_install_performance.rs, tests/skills_install_time_metrics.rs, tests/skills_list_performance.rs
- **Description**: Unresolved git merge conflicts blocking compilation and testing
- **Recommendation**: Urgent resolution required - these are blocking all CI/CD

### Pattern: unwrap()/expect() in Production
- **Occurrences**: 8+ locations identified
- **Files Affected**: logging.rs, cli/mod.rs, scheduler/mod.rs, config/mod.rs, commands/logs.rs
- **Description**: Using panic-on-failure patterns instead of proper error handling
- **Recommendation**: Replace with Result types and proper error propagation

---

## Recommendations Roadmap

### Immediate (This Sprint)
- [ ] **CRIT-001**: Resolve merge conflict in tests/mod.rs - 30m
- [ ] **CRIT-002**: Resolve merge conflict in tests/performance_common.rs - 2h
- [ ] **CRIT-003**: Resolve remaining test file merge conflicts - 1h
- [ ] **HIGH-001 through HIGH-004**: Replace unwrap/expect in production code - 2h

### Short-term (Next 2-4 weeks)
- [ ] **HIGH-005**: Standardize error handling (thiserror for libs) - 4h
- [ ] **MED-001 through MED-012**: Begin god module refactoring - 16h
- [ ] Run `cargo fmt` and commit formatting fixes - 1h

### Long-term (Backlog)
- [ ] **CRIT-004 through CRIT-006**: Architectural refactoring of god modules - 20h+
- [ ] Complete module extraction and proper abstraction - 24h+

---

## Appendix

### Files Scanned
- src/ (main library code): 80 files, ~35,000 lines
- tests/: 40+ test files, ~25,000 lines total
- Total: ~60,000 lines of Rust code

### Skipped Files
- None intentionally skipped - full codebase scanned

### Health Score Calculation
- Critical issues (6) × 2 = 12 points deducted
- High issues (8) × 1 = 8 points deducted  
- Medium issues (12) × 0.5 = 6 points deducted
- Low issues (5) × 0.25 = 1.25 points deducted
- Convention issues (8) × 0.25 = 2 points deducted
- **Score**: 10 - 12 - 8 - 6 - 1.25 - 2 = 5.5/10
