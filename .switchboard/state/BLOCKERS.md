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

---

### Blockers for Refactor Agent 2 - Updated: 2026-02-28

#### BLOCKER-1: Test Compilation Failure (Pre-existing)

**Status:** BLOCKED

**Issue:** Tests fail to compile due to missing `std::fs` imports in test module.

**Details:**
- Location: `src/skills/mod.rs` - test code uses `fs::create_dir_all`, `fs::write`, `fs::read_to_string` without importing the `std::fs` module
- Error count: 21 compilation errors at lines: 428, 436, 448, 456, 472, 479, 483, 490, 519, 526, 530, 534, 645, 650, 671, 676, 699, 702, 706, 709, 958

**Build Status:** 
- `cargo build`: ✅ PASSED
- `cargo test`: ❌ FAILED (compilation error)

**Impact:** Cannot establish test baseline. Refactoring cannot proceed until tests compile.

**Resolution Required:** Add `use std::fs;` to the test module in `src/skills/mod.rs`.

---

## Tasks Affected:
- [MED-003] Extract skill validation from docker/skills.rs
- [MED-004] Split docker/mod.rs into client.rs and build.rs submodules

**Decision:** Stopped. Cannot refactor with broken test suite.
