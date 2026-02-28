# REFACTOR_TODO1 - Refactor Agent 1

> Sprint: Improvement Sprint 3
> Focus Area: Test compilation and code quality fixes
> Last Updated: 2026-02-28
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:
- Cargo.toml
- src/skills/mod.rs
- Run `cargo test --no-run` to see compilation errors

## Tasks (Ordered Safe → Riskier)

- [ ] [CRIT-001] Fix Test Compilation - Add missing `use std::fs;` import
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Add `use std::fs;` to the test module in src/skills/mod.rs so tests compile
  - 📂 Files: `src/skills/mod.rs` (lines 428, 436, 448, 456, 472, 479, 483, 490, 519, 526, 530, 534, 645, 650, 671, 676, 699, 702, 706, 709, 958)
  - 🧭 Context: Tests don't compile because `fs` module is used but not imported. Evidence:
    ```
    error[E0433]: failed to resolve: use of unresolved module or unlinked crate `fs`
       --> src/skills/mod.rs:428:9
        |
    428 |         fs::create_dir_all(&skill_dir).unwrap();
        |         ^^ use of unresolved module or unlinked crate `fs`
    ```
  - ⚡ Pre-check: Run `cargo test --no-run` - should fail with fs errors before fix
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] Tests compile (`cargo test --no-run` succeeds)
    - [ ] No behavioral change (same test logic, just imports added)
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (single file, well-isolated change)

- [ ] [HIGH-002] Fix Formatting Inconsistencies
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Run `cargo fmt` to fix import ordering and re-export formatting across 8+ files
  - 📂 Files: 
    - `src/commands/validate.rs`
    - `src/discord/llm/mod.rs`
    - `src/discord/tools/mod.rs`
    - `src/docker/mod.rs`
    - `src/skills/manager.rs`
    - `src/skills/mod.rs`
  - 🧭 Context: Multiple files fail `cargo fmt --check`. Example diff:
    ```
    Diff in /workspace/src/discord/tools/mod.rs:13:
    -pub use definitions::{Tool, ToolError, MAX_FILE_SIZE, tools_schema};
    +pub use definitions::{tools_schema, Tool, ToolError, MAX_FILE_SIZE};
    ```
  - ⚡ Pre-check: Run `cargo fmt --check` - should show formatting issues
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] Formatting check passes (`cargo fmt --check`)
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (auto-formatting, deterministic)

- [ ] [MED-004] Remove Unused Imports (Clippy Warnings)
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove or allow unused imports identified by clippy
  - 📂 Files: 
    - `src/discord/llm/client.rs:14` - unused `ResponseMessage`, `ResponseToolCall`
    - `src/discord/llm/response.rs:6` - unused `ToolFunction`
    - `src/discord/tools/execution.rs:10` - unused `serde_json::Value`
    - `src/skills/manager.rs:5` - unused `Path`
    - `src/skills/lockfile.rs:369` - unused `SkillMetadata`
  - 🧭 Context: Clippy reports 5 unused imports. Evidence:
    ```
    warning: unused import: `serde_json::Value`
      --> src/discord/tools/execution.rs:10:5
       |
    10 | use serde_json::Value;
       |     ^^^^^^^^^^^^^^^^^^
    ```
  - ⚡ Pre-check: Run `cargo clippy` - should show unused import warnings
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] Clippy passes (`cargo clippy` with no warnings for these items)
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (simple import removal)

> AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_1' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
