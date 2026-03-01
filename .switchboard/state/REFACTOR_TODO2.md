# REFACTOR_TODO2 - Refactor Agent 2

> Sprint: Improvement Sprint 4
> Focus Area: Error Handling & Config Refactoring
> Last Updated: 2026-03-01
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md

## Orientation

Before starting any tasks, read these files to understand the current state:
- Cargo.toml
- src/scheduler/mod.rs (line 1291)
- src/docker/client.rs (lines 115-127)
- src/commands/logs.rs (line 306)
- src/config/mod.rs (lines 1235-1680 for validation functions)

## Tasks

- [ ] [FIND-002E] Remove unused imports from src/commands/skills/install.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove unused `LockfileStruct`, `SkillLockEntry`, `SkillMetadata` imports from line 4-5
  - 📂 Files: src/commands/skills/install.rs
  - 🧭 Context: Evidence: `LockfileStruct, SkillLockEntry, SkillMetadata,` at line 4-5 are unused
  - ⚡ Pre-check: cargo build --lib && cargo clippy --all-targets -- -D warnings
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Clippy passes
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe

- [ ] [FIND-002F] Remove unused import from src/commands/skills/mod.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove unused `crate::skills::SkillMetadata` import from line 16
  - 📂 Files: src/commands/skills/mod.rs
  - 🧭 Context: Evidence: `use crate::skills::SkillMetadata;` at line 16 is unused
  - ⚡ Pre-check: cargo build --lib && cargo clippy --all-targets -- -D warnings
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Clippy passes
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe

- [ ] [FIND-005A] Replace .expect() in src/scheduler/mod.rs:1291
  - 📚 SKILLS: `./skills/rust-engineer/references/error-handling.md`
  - 🎯 Goal: Replace `.expect("Failed to create default Scheduler")` with proper error handling using `?` or match
  - 📂 Files: src/scheduler/mod.rs
  - 🧭 Context: Evidence at line 1291: `Self::new_sync(None, None, None).expect("Failed to create default Scheduler")` - this panics on error
  - ⚡ Pre-check: cargo build --lib && cargo test
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Tests pass
    - [ ] No behavioral change for valid inputs
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe

- [ ] [FIND-005B] Replace .expect() in src/docker/client.rs:115-116
  - 📚 SKILLS: `./skills/rust-engineer/references/error-handling.md`
  - 🎯 Goal: Replace `.expect("socket_path starts with 'unix://' so strip_prefix should succeed")` with proper error handling
  - 📂 Files: src/docker/client.rs
  - 🧭 Context: Evidence at lines 115-116: `.strip_prefix("unix://").expect("socket_path starts with 'unix://' so strip_prefix should succeed")` - panics on malformed input
  - ⚡ Pre-check: cargo build --lib && cargo test
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Tests pass
    - [ ] No behavioral change for valid inputs
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe

- [ ] [FIND-005C] Replace .expect() in src/commands/logs.rs:306
  - 📚 SKILLS: `./skills/rust-engineer/references/error-handling.md`
  - 🎯 Goal: Replace `.expect("Failed to setup SIGTERM handler")` with proper error handling
  - 📂 Files: src/commands/logs.rs
  - 🧭 Context: Evidence at line 306: `signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler")` - panics if signal setup fails
  - ⚡ Pre-check: cargo build --lib && cargo test
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Tests pass
    - [ ] No behavioral change for valid inputs
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe

## AGENT QA

Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_2' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
