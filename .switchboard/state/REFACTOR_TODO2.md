# REFACTOR_TODO2 - Refactor Agent 2

> Sprint: Improvement Sprint 2
> Focus Area: Error Handling and Metrics
> Last Updated: 2026-02-28T12:41:28Z
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:

- Cargo.toml (build manifest)
- src/lib.rs (module root)
- src/metrics/store.rs (for metrics review task)
- Various files with error handling patterns (for error handling task)

## Tasks

[Tasks in the format below, ordered safe → risky]

- [ ] [FIND-LOW-002] Review metrics/store.rs for extractable functions
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: The file at 1107 lines should be reviewed for extractable functions or modules to reduce its complexity.
  - 📂 Files: `src/metrics/store.rs`
  - 🧭 Context: metrics/store.rs is 1107 lines - a large file that could benefit from extraction of smaller functions or sub-modules.
  - ⚡ Pre-check: Build and tests pass before starting
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (same inputs produce same outputs)
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (independent of other tasks)

- [ ] [FIND-CONV-001] Standardize Error Handling
  - 📚 SKILLS: `./skills/rust-engineer/references/error-handling.md`
  - 🎯 Goal: Use consistent error handling patterns. Prefer `thiserror` for application errors over mixed `Box<dyn Error>`, `String`, and custom types.
  - 📂 Files: Multiple files with inconsistent error handling
  - 🧭 Context: The codebase uses mixed patterns: `Box<dyn Error>`, `String` errors, custom `thiserror` types. Need to standardize on `thiserror` for application-specific errors.
  - ⚡ Pre-check: Build and tests pass before starting
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (same inputs produce same outputs)
  - 🔒 Risk: Medium (error handling changes can be subtle)
  - ↩️ Revert: `git revert` safe (but changes should be reviewed)

- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_2' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
