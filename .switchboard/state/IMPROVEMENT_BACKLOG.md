# Codebase Scan Report

**Project**: switchboard  
**Scanned**: 2026-02-27T20:09:00Z  
**Scope**: Full codebase (src/, tests/)  
**Files Analyzed**: ~80 Rust source files  
**Audit Type**: Continuation (previous audit: 2026-02-27T18:07:00Z)

---

## Executive Summary

| Severity | Count | Estimated Effort |
|----------|-------|------------------|
| 🔴 Critical | 3 | 8h |
| 🟠 High | 4 | 6h |
| 🟡 Medium | 7 | 10h |
| 🔵 Low | 4 | 3h |
| ⚪ Convention | 5 | 2h |

**Overall Health Score**: 6.5/10  

**Top 3 Priorities**:
1. Fix failing tests (25 test failures)
2. Address unwrap/expect in production code (skills compliance)
3. Address clippy failures in test files

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
| `cargo test` | ❌ FAIL | 25 test failures |
| `cargo clippy` | ❌ FAIL | Test file issues (unused imports, mut) |
| `cargo fmt --check` | ✅ PASS | Formatting fixed since last audit |

---

## Findings by Category

### 🔴 Critical Issues

#### [CRIT-001] Test Failures - 25 Tests Failing
- **File**: Multiple test files in `tests/` and `src/`
- **Issue**: 25 tests failing, including:
  - `docker::run::run::tests::test_skills_*` (16 failures) - skill validation errors
  - `discord::config::tests::test_env_config_*` (2 failures) - environment config
  - `commands::validate::tests::test_validate_lockfile_*` (2 failures)
  - `skills::tests::test_check_npx_available_*` (2 failures)
- **Risk**: CI pipeline likely broken, regression detection compromised
- **Recommendation**: Fix skill validation mock in tests, update environment handling
- **Effort**: L

```rust
// Example failure - docker/run/run.rs:3959
Failed to generate entrypoint script: ScriptGenerationFailed { 
    agent_name: "test-agent", 
    reason: "Skill 'repo' is not found in ./skills/ directory." 
}
```

#### [CRIT-002] God Module - config/mod.rs at 3512 Lines
- **File**: `src/config/mod.rs` (lines 1-3512)
- **Issue**: Single file contains 3512 lines with Config, Agent, Settings structs + all validation + tests inline
- **Risk**: Maintainability nightmare, impossible to understand in one sitting
- **Recommendation**: Split into: config/agent.rs, config/settings.rs, config/validation.rs, config/tests/
- **Effort**: L

#### [CRIT-003] Another God Module - skills/mod.rs at 2709 Lines  
- **File**: `src/skills/mod.rs` (lines 1-2709)
- **Issue**: Single file contains all skills management logic
- **Risk**: Hard to navigate, long compile times
- **Recommendation**: Split into: skills/manager.rs, skills/lockfile.rs, skills/metadata.rs
- **Effort**: L

---

### 🟠 High Priority Issues

#### [HIGH-001] unwrap()/expect() in Production Code (Skills Violation)
- **File**: Multiple files in `src/` (not tests)
- **Issue**: According to rust-best-practices SKILL: "Never use unwrap()/expect() outside tests"
- **Evidence**:
```rust
// src/cli/mod.rs:348
let docker = client.docker().expect("Docker client should be available");

// src/scheduler/mod.rs:1164
*self.queue_wait_time_seconds.lock().unwrap()

// src/logging.rs:150
Ok(GLOBAL_LOG_DIR.as_ref().unwrap().as_path())

// src/logging.rs:102, 113, 137, 145 (mutex locks)
*INIT_ERROR.lock().unwrap() = Some(err);
```
- **Risk**: Runtime panics, poor error messages
- **Recommendation**: Replace with proper Result handling and ? operator
- **Effort**: M
- **Status**: PREVIOUS - Recurring from last audit

#### [HIGH-002] Clippy Failures - Test Files
- **Files**: tests/*.rs
- **Issue**: Multiple clippy failures in test files:
  - `tests/discord_listener_integration.rs` - unused imports (ToolCall, ToolFunction)
  - `tests/discord_send_message.rs` - unused imports (ApiError, Message)
  - `tests/container_skill_install_failure.rs` - useless_vec
  - `tests/listener_conversation_tests.rs` - unused mut
  - `tests/docker_availability_check.rs` - unused imports
  - `src/config/mod.rs` - cannot test inner items
- **Recommendation**: Clean up test files, move inline tests out
- **Effort**: S
- **Status**: NEW

#### [HIGH-003] CLI Module - 2144 Lines
- **File**: `src/cli/mod.rs` (2144 lines)
- **Issue**: Contains all CLI commands and handlers in single file
- **Recommendation**: Extract commands to individual files in commands/ directory
- **Effort**: M

#### [HIGH-004] Commands Module Split - 2074 Lines  
- **File**: `src/commands/skills.rs` (2074 lines)
- **Issue**: Single file for all skills subcommands
- **Recommendation**: Extract to skills/list.rs, skills/install.rs, etc.
- **Effort**: M

---

### 🟡 Medium Priority (Refactoring)

#### [MED-001] Duplicate Code - Docker Client Expect Pattern
- **Files**: `src/docker/mod.rs`
- **Issue**: Same `.expect("Docker client not available")` pattern repeated
- **Recommendation**: Add helper method `fn get_docker(&self) -> Result<&Docker, DockerError>`
- **Effort**: S

#### [MED-002] Magic Strings - Error Messages
- **Files**: Multiple
- **Issue**: Error messages repeated across files
```rust
"Docker client not available" // appears 10+ times
```
- **Recommendation**: Create constants in respective error modules
- **Effort**: S

#### [MED-003] Unused Config Key Warning
- **File**: `.cargo/config.toml`
- **Issue**: Warning: unused config key `profile.test.features`
- **Recommendation**: Remove or fix the config key
- **Effort**: S

#### [MED-004] discord/tools.rs - 1663 Lines
- **File**: `src/discord/tools.rs` (1663 lines)
- **Issue**: Large file with tool definitions and executions
- **Recommendation**: Split into tools/definitions.rs, tools/execution.rs
- **Effort**: M

#### [MED-005] discord/llm.rs - 1539 Lines
- **File**: `src/discord/llm.rs` (1539 lines)
- **Issue**: Large LLM client module
- **Recommendation**: Split into llm/client.rs, llm/response.rs
- **Effort**: M

#### [MED-006] docker/skills.rs - 1489 Lines
- **File**: `src/docker/skills.rs` (1489 lines)
- **Issue**: Skill-related Docker integration
- **Recommendation**: Extract skill validation to separate module
- **Effort**: M

#### [MED-007] docker/mod.rs - 1394 Lines
- **File**: `src/docker/mod.rs` (1394 lines)
- **Issue**: Docker client wrapper module
- **Recommendation**: Split into docker/client.rs, docker/build.rs
- **Effort**: M

---

### 🔵 Low Priority

#### [LOW-001] Clippy Warnings in Test Files (Pre-existing)
- **Files**: `tests/*.rs`
- **Issue**: Unused imports, mutable variables
- **Recommendation**: Clean up test files with `cargo clippy --fix`
- **Effort**: S

#### [LOW-002] Dead Code - Timer Struct Never Used
- **File**: `tests/performance_common.rs` (line 402)
- **Issue**: `pub struct Timer` is never constructed
- **Recommendation**: Remove or implement
- **Effort**: S

#### [LOW-003] Dead Code - format_duration Function
- **File**: `tests/skills_install_performance.rs` (line 45)
- **Issue**: Function never used
- **Recommendation**: Remove
- **Effort**: S

#### [LOW-004] Unused Test Functions
- **Files**: Multiple test files
- **Issue**: Functions marked `#[test]` but unused
- **Effort**: S

---

### ⚪ Convention Issues

#### [CONV-001] Inconsistent Error Handling
- **Files**: Mixed Result<T,E> vs Box<dyn Error>
- **Issue**: Some functions use Result<T,E>, others Box<dyn std::error::Error>
- **Recommendation**: Standardize on Result<T,E> with custom error types
- **Effort**: M

#### [CONV-002] Module Organization - Discord
- **File**: `src/discord/*.rs` (8 files)
- **Issue**: Some modules could be reorganized (api, gateway, listener in same level)
- **Recommendation**: Consider discord/api/, discord/gateway/ subdirectories
- **Effort**: M

#### [CONV-003] Test Organization
- **File**: `tests/` (25+ test files)
- **Issue**: Mix of unit tests in src/ and integration tests in tests/
- **Recommendation**: Follow standard Rust conventions strictly
- **Effort**: S

#### [CONV-004] Missing Documentation - Private Functions
- **Files**: Multiple
- **Issue**: Some private functions lack doc comments
- **Recommendation**: Add docs to public API entry points
- **Effort**: S

#### [CONV-005] Backup File Present
- **File**: `src/config/mod.rs.bak`
- **Issue**: Backup file in source tree
- **Recommendation**: Remove from version control, add to .gitignore
- **Effort**: S

---

## Systemic Patterns

### Pattern: Expect on Internal State
- **Occurrences**: 15+ files
- **Files Affected**: docker/mod.rs, scheduler/mod.rs, logging.rs, cli/mod.rs
- **Description**: Using .expect() on internal state that should never fail (like Arc<Mutex> locks, Option fields that are set at construction)
- **Recommendation**: Create type-safe wrappers or use expect_with() for internal invariants

### Pattern: Giant Test Modules in Source
- **Occurrences**: config/mod.rs (2000+ lines tests), skills/mod.rs, discord/config.rs
- **Description**: Tests inline in source files make files huge
- **Recommendation**: Move tests to tests/ directory

### Pattern: Error Messages as Literals
- **Occurrences**: 50+ occurrences
- **Description**: Error strings repeated across codebase
- **Recommendation**: Centralize in error modules

---

## Recommendations Roadmap

### Immediate (This Sprint)
- [ ] Fix 25 failing tests - 8h
- [ ] Fix clippy test file issues - 2h

### Short-term (Next 2-4 weeks)
- [ ] Replace unwrap/expect with proper error handling - 6h
- [ ] Split config/mod.rs (3512 lines) - 4h
- [ ] Split skills/mod.rs (2709 lines) - 4h

### Long-term (Backlog)
- [ ] Split cli/mod.rs (2144 lines) - 3h
- [ ] Split commands/skills.rs (2074 lines) - 3h
- [ ] Extract Docker helper methods - 2h

---

## Appendix

### Files Scanned (Top 20 by Line Count)
| File | Lines |
|------|-------|
| src/config/mod.rs | 3512 |
| src/skills/mod.rs | 2709 |
| src/cli/mod.rs | 2144 |
| src/commands/skills.rs | 2074 |
| src/discord/tools.rs | 1663 |
| src/discord/llm.rs | 1539 |
| src/docker/skills.rs | 1489 |
| src/docker/mod.rs | 1394 |
| src/scheduler/mod.rs | 1259 |
| src/metrics/store.rs | 1091 |
| src/discord/config.rs | 1091 |
| src/discord/security.rs | 899 |
| src/traits/mod.rs | 882 |
| src/discord/api.rs | 876 |
| src/discord/mod.rs | 858 |
| src/discord/conversation.rs | 823 |
| src/skills/error.rs | 735 |
| src/commands/metrics.rs | 596 |
| src/logger/file.rs | 540 |
| src/logger/terminal.rs | 447 |

### Skipped Files
- None - full codebase scanned

### Skills Compliance Notes
The codebase has two active skills:
1. **rust-best-practices** - Key violations found:
   - unwrap()/expect() in production code (HIGH-001)
   
2. **rust-engineer** - Key violations found:
   - Error handling patterns inconsistent

### Changes Since Last Audit
- **FIXED**: Formatting issues (cargo fmt --check now passes)
- **SAME**: 25 test failures persist
- **SAME**: God modules unchanged (slightly grew)
- **NEW**: Clippy test file failures identified
