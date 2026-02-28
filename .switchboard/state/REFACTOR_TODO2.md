# REFACTOR_TODO2 - Refactor Agent 2

> Sprint: Improvement Sprint 2
> Focus Area: Docker and Skills modules refactoring
> Last Updated: 2026-02-28
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

> ⚠️ Rebalanced: 2 tasks moved to REFACTOR_TODO1.md by Planner on 2026-02-28

## Orientation

Before starting any tasks, read these files to understand the current state:
- Cargo.toml
- src/docker/mod.rs
- src/docker/skills.rs
- src/skills/mod.rs
- src/scheduler/mod.rs

## Tasks

- [ ] [MED-003] Extract skill validation from docker/skills.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Extract skill validation logic into a separate focused module
  - 📂 Files: `src/docker/skills.rs` - extract validation to separate module
  - 🧭 Context: File has 1489 lines - extract skill validation to its own module
  - ⚡ Pre-check: Run `cargo build` and verify it passes
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (independent)

- [ ] [MED-004] Split docker/mod.rs into submodules
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Split into client and build modules
  - 📂 Files: `src/docker/mod.rs` → split into `src/docker/client.rs`, `src/docker/build.rs`
  - 🧭 Context: File has 1394 lines - split by responsibility (client wrapper vs build logic)
  - ⚡ Pre-check: Run `cargo build` and verify it passes
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (same API externally)
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (independent)

- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_2' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
