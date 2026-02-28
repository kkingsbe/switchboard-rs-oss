# REFACTOR_TODO1 - Refactor Agent 1

> Sprint: Improvement Sprint 2
> Focus Area: Discord modules, config, and test cleanup
> Last Updated: 2026-02-27
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:
- Cargo.toml
- src/discord/mod.rs
- src/discord/tools.rs
- src/discord/llm.rs

## Tasks

- [ ] [HIGH-002] Remove Clippy dead code warnings
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove or allow all unused code warnings in tests/performance_common.rs
  - 📂 Files: `tests/performance_common.rs`
  - 🧭 Context: Clippy reports 4 unused items:
    ```
    warning: enum `RegressionStatus` is never used
    warning: struct `BaselineTracker` is never constructed
    warning: function `detect_regression` is never used
    warning: function `log_regression_warning` is never used
    ```
    Suggested fix: Remove unused test code or mark as `#[allow(unused)]`
  - ⚡ Pre-check: Run `cargo clippy --tests` and verify it passes
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (independent)

- [ ] [MED-001] Split discord/tools.rs into submodules
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Extract tool definitions and execution logic into separate modules
  - 📂 Files: `src/discord/tools.rs` → split into `src/discord/tools/definitions.rs`, `src/discord/tools/execution.rs`
  - 🧭 Context: File has 1663 lines - a "god module" that should be split. Current structure should be reorganized into focused submodules by responsibility.
  - ⚡ Pre-check: Run `cargo build` and verify it passes
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (same API externally)
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (independent)

- [ ] [MED-002] Split discord/llm.rs into submodules
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Extract client and response handling into separate modules
  - 📂 Files: `src/discord/llm.rs` → split into `src/discord/llm/client.rs`, `src/discord/llm/response.rs`
  - 🧭 Context: File has 1539 lines - a "god module" that should be split by responsibility (client logic vs response handling)
  - ⚡ Pre-check: Run `cargo build` and verify it passes
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (same API externally)
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (independent)

- [ ] [MED-006] Fix unused config key warning
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove or fix the unused config key in .cargo/config.toml
  - 📂 Files: `.cargo/config.toml`
  - 🧭 Context: Warning: `unused config key 'profile.test.features'` - remove this key or fix it
  - ⚡ Pre-check: Run `cargo build` and verify no warnings
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] No config warnings
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (independent)

> ⚠️ Rebalanced from REFACTOR_TODO2.md by Planner on 2026-02-28

- [ ] [MED-005] Split skills/mod.rs into submodules
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Split into manager, lockfile, and metadata modules
  - 📂 Files: `src/skills/mod.rs` → split into `src/skills/manager.rs`, `src/skills/lockfile.rs`, `src/skills/metadata.rs`
  - 🧭 Context: File has 2709 lines - largest god module. Split by functional responsibility: skills management, lockfile handling, metadata
  - ⚡ Pre-check: Run `cargo build` and verify it passes
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (same API externally)
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (independent)

- [ ] [LOW-001] Consider splitting scheduler/mod.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Split into clock and queue modules if beneficial
  - 📂 Files: `src/scheduler/mod.rs` → split into `src/scheduler/clock.rs`, `src/scheduler/queue.rs`
  - 🧭 Context: File has 1259 lines - borderline acceptable size. Consider splitting if it improves maintainability.
  - ⚡ Pre-check: Run `cargo build` and verify it passes
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (independent)

- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_1' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
