# REFACTOR_TODO1 - Refactor Agent 1

> Sprint: Improvement Sprint 4
> Focus Area: Clippy Lint Fixes - CLI Module
> Last Updated: 2026-03-01
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md

## Orientation

Before starting any tasks, read these files to understand the current state:
- Cargo.toml
- src/cli/mod.rs (lines 20-30 for unused imports)
- src/cli/commands/up.rs (line 213 for unused function)

## Tasks

- [ ] [FIND-002A] Remove unused import `init_logging` from src/cli/mod.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove `use crate::logging::init_logging;` from line 21 in src/cli/mod.rs
  - 📂 Files: src/cli/mod.rs
  - 🧭 Context: This import is unused - the function is called elsewhere. Evidence: `use crate::logging::init_logging;` at line 21
  - ⚡ Pre-check: cargo build --lib && cargo clippy --all-targets -- -D warnings
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build --lib`)
    - [ ] Clippy passes (`cargo clippy --all-targets -- -D warnings`)
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (independent)

- [ ] [FIND-002B] Remove unused import `Scheduler` from src/cli/mod.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove `use crate::scheduler::Scheduler;` from line 23 in src/cli/mod.rs
  - 📂 Files: src/cli/mod.rs
  - 🧭 Context: Evidence: `use crate::scheduler::Scheduler` at line 23
  - ⚡ Pre-check: cargo build --lib && cargo clippy --all-targets -- -D warnings
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Clippy passes
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe

- [ ] [FIND-002C] Remove unused import `RealProcessExecutor` from src/cli/mod.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove `RealProcessExecutor` from the use statement at line 25 in src/cli/mod.rs
  - 📂 Files: src/cli/mod.rs
  - 🧭 Context: Evidence: `RealProcessExecutor` partial import unused at line 25
  - ⚡ Pre-check: cargo build --lib && cargo clippy --all-targets -- -D warnings
  - ✅ Acceptance:
    - complete
    - [ ] Change is [ ] Build passes
    - [ ] Clippy passes
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe

- [ ] [FIND-002D] Remove unused function `default_executor` from src/cli/commands/up.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove the unused `default_executor` function at lines 213-215 in src/cli/commands/up.rs
  - 📂 Files: src/cli/commands/up.rs
  - 🧭 Context: Evidence: `fn default_executor() -> Arc<dyn ProcessExecutorTrait> { Arc::new(RealProcessExecutor::new()) }` at lines 213-215 is never used
  - ⚡ Pre-check: cargo build --lib && cargo clippy --all-targets -- -D warnings
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Clippy passes
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe

## AGENT QA

Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_1' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
