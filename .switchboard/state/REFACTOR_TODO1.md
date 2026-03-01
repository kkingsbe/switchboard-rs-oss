# REFACTOR_TODO1 - Refactor Agent 1

> Sprint: Improvement Sprint 3
> Focus Area: Code Quality (Clippy fixes)
> Last Updated: 2026-03-01
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:
- Cargo.toml (project manifest)
- src/cli/mod.rs (line 16 - unused import)
- src/commands/skills/mod.rs (line 14 - unused imports)

## Tasks

- [ ] [FIND-002] Remove unused import `list_agents` from src/cli/mod.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove the unused `list_agents` import from line 16 of src/cli/mod.rs so clippy no longer reports this error
  - 📂 Files: src/cli/mod.rs
  - 🧭 Context: Clippy is failing the build with -D warnings. The import `list_agents` is imported but not used. Evidence: `error: unused import: 'list_agents' --> src/cli/mod.rs:16:23`
  - ⚡ Pre-check: Run `cargo clippy -- -D warnings` and confirm this specific error exists
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] Clippy passes (`cargo clippy -- -D warnings`)
    - [ ] No behavioral change (same inputs produce same outputs)
  - 🔒 Risk: Medium
  - ↩️ Revert: `git revert` safe (independent task)

- [ ] [FIND-002] Remove unused imports from src/commands/skills/mod.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove unused imports (`LockfileStruct`, `SkillLockEntry`, `find_skill_directory`, etc.) from src/commands/skills/mod.rs line 14
  - 📂 Files: src/commands/skills/mod.rs
  - 🧭 Context: Clippy reports unused imports in this file. Evidence: `error: unused imports: 'LockfileStruct', 'SkillLockEntry', 'find_skill_directory', ... --> src/commands/skills/mod.rs:14:48`
  - ⚡ Pre-check: Run `cargo clippy -- -D warnings` and confirm these specific errors exist
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] Clippy passes (`cargo clippy -- -D warnings`)
    - [ ] No behavioral change
  - 🔒 Risk: Medium
  - ↩️ Revert: `git revert` safe

- [ ] [FIND-006] Run cargo fmt to fix formatting inconsistencies
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Run `cargo fmt` to fix formatting inconsistencies in the codebase
  - 📂 Files: src/commands/skills/installed.rs, src/cli/process.rs, and any others
  - 🧭 Context: Format check is failing. Evidence shows inconsistencies in import ordering and line wrapping in skills/installed.rs and cli/process.rs
  - ⚡ Pre-check: Run `cargo fmt -- --check` and confirm formatting issues exist
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] Format passes (`cargo fmt -- --check`)
    - [ ] No behavioral change
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (formatting only)

## AGENT QA

Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_1' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
