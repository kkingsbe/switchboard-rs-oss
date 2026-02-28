# REFACTOR_TODO1 - Refactor Agent 1

> ⚠️ Rebalanced from REFACTOR_TODO2.md by Planner on 2026-02-28

> Sprint: Improvement Sprint 5
> Focus Area: Discord LLM Client - Code Quality
> Last Updated: 2026-02-28T06:30:03Z
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:

- Cargo.toml - Project dependencies and structure
- src/discord/llm/client.rs - The file containing the issue (lines 99-170)
- src/discord/mod.rs - Discord module structure
- Run `cargo clippy` to see current warnings

## Tasks

- [x] [FIND-MED-003] Fix Private Interface Warning in Discord LLM Client
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Make `FunctionDefinition` struct public or adjust visibility to resolve clippy warning about private type
  - 📂 Files: `src/discord/llm/client.rs`
  - 🧭 Context: The clippy warning indicates that `FunctionDefinition` is more private than the item `ToolDefinition::function` which is `pub`. This is a code quality issue from the audit.
    
    Evidence from finding:
    ```rust
    warning: type `FunctionDefinition` is more private than the item `ToolDefinition::function`
       --> src/discord/llm/client.rs:99:5
        |
     99 |     pub function: FunctionDefinition,
        |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ field `ToolDefinition::function` is reachable at visibility `pub`
        |
    note: but type `FunctionDefinition` is only usable at visibility `pub(self)`
       --> src/discord/llm/client.rs:104:1
        |
    104 |     struct FunctionDefinition {
        |     ^^^^^^^^^^^^^^^^^^^^^^^^^
    ```
    
    The fix should either:
    1. Make `FunctionDefinition` a `pub struct`, OR
    2. Adjust the visibility of `ToolDefinition::function` to match
    
    Choose option 1 if `FunctionDefinition` needs to be used externally, or option 2 if it's only used internally.
  - ⚡ Pre-check: Run `cargo clippy 2>&1 | grep -A5 "FunctionDefinition"` to see current warning
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] Clippy warning is resolved (`cargo clippy 2>&1 | grep -c "FunctionDefinition"` should return 0)
    - [ ] No behavioral change (same inputs produce same outputs)
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (single file change)

- [ ] [FIND-CONV-004] Add Missing Documentation to Public APIs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`, `./skills/rust-engineer/SKILL.md`
  - 🎯 Goal: Add doc comments to public functions and types that are missing them. Focus on the most critical public APIs in the scheduler and metrics modules.
  - 📂 Files: `src/scheduler/mod.rs`, `src/metrics/store.rs`, `src/lib.rs`
  - 🧭 Context: The audit found missing documentation as a convention issue. Adding documentation improves maintainability and helps other developers understand the API.
    
    Evidence from finding:
    - Finding: [CONV-004] Missing Documentation
    - Category: Documentation
    - Severity: Low
    - Effort: S
    - Risk: Safe
    - Priority Score: 8/22
    
    Look for:
    - Public functions without `///` doc comments
    - Public structs without doc comments
    - Important error types without documentation
  - ⚡ Pre-check: Run `cargo doc --no-deps 2>&1 | grep -c "warning: missing"` to count current warnings
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] Documentation builds without errors (`cargo doc --no-deps`)
    - [ ] No behavioral change (same inputs produce same outputs)
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (documentation only)

- [ ] [FIND-LOW-001] Review scheduler module for extractable functions
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Identify opportunities to extract helper functions from scheduler/mod.rs to improve readability. Focus on finding repetitive code patterns that could be extracted.
  - 📂 Files: `src/scheduler/mod.rs`
  - 🧭 Context: The scheduler module is 1268 lines, which is large but not yet a critical "god module" issue. The goal is to improve it proactively.
    
    Evidence from finding:
    - Finding: [LOW-001] scheduler/mod.rs - 1268 Lines
    - Category: Structure
    - Severity: Low
    - Effort: M
    - Risk: Low
    - Priority Score: 7/22
    
    This is a RECURRING finding - previous sprints have noted this but not fully addressed it. Look for:
    - Functions over 50 lines that could be split
    - Repetitive error handling patterns
    - Helper functions that could be extracted to a separate module
  - ⚡ Pre-check: Run `wc -l src/scheduler/mod.rs` to confirm file size
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (same inputs produce same outputs)
    - [ ] Code is more readable/maintainable
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (refactoring only, no behavior change)

- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_1' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
