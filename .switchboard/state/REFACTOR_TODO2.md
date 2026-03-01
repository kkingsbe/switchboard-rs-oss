# REFACTOR_TODO2 - Refactor Agent 2

> Sprint: Improvement Sprint 5
> Focus Area: Formatting Fixes
> Last Updated: 2026-03-01T10:33:00Z
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md (FIND-003)

## Orientation

Before starting any tasks, read these files to understand the current state:
- Cargo.toml
- src/commands/skills/install.rs
- src/docker/client.rs
- src/scheduler/mod.rs

## Tasks

- [ ] [FIND-003A] Fix formatting in src/commands/skills/install.rs
  - 📚 SKILLS: N/A
  - 🎯 Goal: Run `cargo fmt` to fix formatting at line 1 of src/commands/skills/install.rs
  - 📂 Files: src/commands/skills/install.rs
  - 🧭 Context: Evidence from audit: `Diff in /workspace/src/commands/skills/install.rs:1`
  - ⚡ Pre-check: `cargo build --lib && cargo fmt --check`
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Formatting passes (`cargo fmt --check`)
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe

- [ ] [FIND-003B] Fix formatting in src/docker/client.rs
  - 📚 SKILLS: N/A
  - 🎯 Goal: Run `cargo fmt` to fix formatting at line 111 of src/docker/client.rs
  - 📂 Files: src/docker/client.rs
  - 🧭 Context: Evidence from audit: `Diff in /workspace/src/docker/client.rs:111`
  - ⚡ Pre-check: `cargo build --lib && cargo fmt --check`
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Formatting passes
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe

- [ ] [FIND-003C] Fix formatting in src/scheduler/mod.rs
  - 📚 SKILLS: N/A
  - 🎯 Goal: Run `cargo fmt` to fix formatting at line 1282 of src/scheduler/mod.rs
  - 📂 Files: src/scheduler/mod.rs
  - 🧭 Context: Evidence from audit: `Diff in /workspace/src/scheduler/mod.rs:1282`
  - ⚡ Pre-check: `cargo build --lib && cargo fmt --check`
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Formatting passes
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe

## AGENT QA

Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_2' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
