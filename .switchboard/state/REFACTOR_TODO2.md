# REFACTOR_TODO2 - Refactor Agent 2

> Sprint: Improvement Sprint 3
> Focus Area: Complexity (Large CLI Module)
> Last Updated: 2026-03-01
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:
- Cargo.toml (project manifest)
- src/cli/mod.rs (2082 lines - the main CLI module)
- src/cli/commands/ (sub-commands directory)

## Tasks

- [ ] [FIND-007] Analyze and split large CLI module
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Reduce the size of src/cli/mod.rs (currently 2082 lines) by extracting logical units into separate modules. Focus on identifying cohesive groups of code that can be extracted.
  - 📂 Files: src/cli/mod.rs, potentially new files in src/cli/
  - 🧭 Context: The CLI module is too large (2082 lines). This makes the code harder to maintain and understand. Extract logical subunits like:
    - Signal handling (move to cli/signals.rs if not already separate)
    - Process management (move to cli/process.rs if not already separate)
    - Command registration/helpers
  - ⚡ Pre-check: Run `cargo build` and `cargo test` to establish baseline
  - ✅ Acceptance:
    - [ ] Change is complete - code is reorganized but behavior unchanged
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] CLI still works - test a few commands like `cargo run -- --help`
    - [ ] No behavioral change
  - 🔒 Risk: Low (extraction refactor, no behavior change)
  - ↩️ Revert: `git revert` safe (can revert entire change)

## AGENT QA

Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_2' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
