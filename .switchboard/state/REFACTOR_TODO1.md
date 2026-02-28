# REFACTOR_TODO1 - Refactor Agent 1

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

- [ ] [FIND-MED-003] Fix Private Interface Warning in Discord LLM Client
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

- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_1' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
