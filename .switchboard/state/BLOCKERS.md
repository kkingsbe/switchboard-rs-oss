# Refactor Agent Blockers

---

### Blockers for Refactor Agent 1 - Updated: 2026-02-28

#### BLOCKER-1: Test Compilation Failure in skills/mod.rs
**Status:** BLOCKING all refactoring work
**Issue:** `cargo test` fails with 21 compilation errors in src/skills/mod.rs
**Root Cause:** The `fs` module is being shadowed by `crate::metrics::store::tests::fs`. Test code uses `fs::create_dir_all`, `fs::write`, `fs::read_to_string` without explicitly importing `std::fs`.
**Impact:** Cannot run tests to verify refactoring safety
**Resolution:** This appears to be related to MED-005 (split skills/mod.rs). When that refactoring is done, it should fix this issue.

#### Remaining Tasks for Agent 1:
- [ ] MED-005: Split skills/mod.rs into submodules (manager.rs, lockfile.rs, metadata.rs)
- [ ] LOW-001: Consider splitting scheduler/mod.rs

#### BLOCKER-2: Verification Failure - Test Suite Not Establishable
**Status:** BLOCKING verification
**Issue:** Unable to establish a reliable test baseline due to test compilation failures.
**Root Cause:** BLOCKER-1 must be resolved first before any verification can occur.
**Impact:** Cannot verify that refactoring changes maintain behavioral equivalence.
**Resolution:** Fix BLOCKER-1 (test compilation errors in src/skills/mod.rs) to enable test baseline establishment.

---

### Blockers for Refactor Agent 2 - Updated: 2026-02-28

#### BLOCKER-1: Pre-existing Test Failures

**Status:** BLOCKED

**Issue:** Tests compile but 24 tests fail with assertion failures. These appear to be pre-existing issues from previous refactoring sprints.

**Details:**
- `cargo build`: ✅ PASSED
- `cargo test`: ❌ 24 FAILED, 523 passed
- Test failures include:
  - `discord::config::tests::test_env_config_*` (3 failures)
  - `docker::run::run::tests::*` (16 failures related to skill installation scripts)
  - `commands::validate::tests::test_validate_lockfile_*` (2 failures)
  - `skills::tests::test_check_npx_available_*` (2 failures)
  - `docker::skills::tests::test_generate_entrypoint_script_skill_not_in_preexisting_list` (1 failure)

**Root Cause:** These failures appear to be side effects from previous refactoring work:
- MED-001: Split discord/llm.rs into submodules
- HIGH-001: Replace unwrap()/expect() with proper error handling

**Impact:** Cannot establish reliable test baseline for refactoring verification.

**Resolution Required:** These test failures need to be investigated and fixed by the appropriate refactor agent before further refactoring can proceed safely.

---

## Tasks Affected:
- [HIGH-003] Consider Splitting CLI Module - Cannot safely refactor without passing test baseline

**Decision:** Stopped. Cannot verify behavioral equivalence with failing test suite.
