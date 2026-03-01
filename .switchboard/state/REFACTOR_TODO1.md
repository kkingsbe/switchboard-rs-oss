# REFACTOR_TODO1 - Refactor Agent 1

> Sprint: Improvement Sprint 5
> Focus Area: Clippy Lint Fixes
> Last Updated: 2026-03-01T10:33:00Z
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md (FIND-002)

## Orientation

Before starting any tasks, read these files to understand the current state:
- Cargo.toml
- src/cli/mod.rs (lines 21-46)
- src/cli/commands/up.rs (lines 10-24, 209)
- src/config/mod.rs (line 3453)

## Tasks

- [ ] [FIND-002A] Remove unused import `std::env` from src/cli/mod.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove `use std::env;` from line 28 in src/cli/mod.rs
  - 📂 Files: src/cli/mod.rs
  - 🧭 Context: Evidence from audit: `unused import: std::env --> src/cli/mod.rs:28:5`
  - ⚡ Pre-check: `cargo build --lib && cargo clippy --all-targets -- -D warnings`
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build --lib`)
    - [ ] Clippy passes (`cargo clippy --all-targets -- -D warnings`)
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (independent)

- [ ] [FIND-002B] Remove unused import `resolve_config_value` from src/cli/mod.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove `use crate::config::env::resolve_config_value;` from line 35 in src/cli/mod.rs
  - 📂 Files: src/cli/mod.rs
  - 🧭 Context: Evidence from audit: `unused import: crate::config::env::resolve_config_value --> src/cli/mod.rs:35:5`
  - ⚡ Pre-check: `cargo build --lib && cargo clippy --all-targets -- -D warnings`
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Clippy passes
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe

- [ ] [FIND-002C] Remove unused import `LlmConfig` from src/cli/mod.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove `use crate::discord::config::LlmConfig;` from line 37 in src/cli/mod.rs
  - 📂 Files: src/cli/mod.rs
  - 🧭 Context: Evidence from audit: `unused import: crate::discord::config::LlmConfig --> src/cli/mod.rs:37:5`
  - ⚡ Pre-check: `cargo build --lib && cargo clippy --all-targets -- -D warnings`
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Clippy passes
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe

- [ ] [FIND-002D] Fix empty line after doc comment in src/cli/commands/up.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Fix formatting at line 209 in src/cli/commands/up.rs - remove empty line after doc comment
  - 📂 Files: src/cli/commands/up.rs
  - 🧭 Context: Evidence from audit: `empty line after doc comment --> src/cli/commands/up.rs:209:1`
  - ⚡ Pre-check: `cargo build --lib && cargo fmt --check`
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Formatting passes (`cargo fmt --check`)
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe

- [ ] [FIND-002E] Address "cannot test inner items" in src/config/mod.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Investigate and fix the "cannot test inner items" clippy warning at line 3453 in src/config/mod.rs
  - 📂 Files: src/config/mod.rs
  - 🧭 Context: Evidence from audit: `cannot test inner items --> src/config/mod.rs:3453:9` - This usually indicates a test module with `#[cfg(test)]` that tests private internal items
  - ⚡ Pre-check: `cargo build --lib && cargo clippy --all-targets -- -D warnings`
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Clippy passes
    - [ ] No behavioral change
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe

## AGENT QA

Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_1' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
