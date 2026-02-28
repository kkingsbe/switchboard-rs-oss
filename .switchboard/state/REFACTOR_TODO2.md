# REFACTOR_TODO2 - Refactor Agent 2

> Sprint: Improvement Sprint 3
> Focus Area: Commands Module and Scheduler Refactoring
> Last Updated: 2026-02-28T18:33:00Z
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:

- Cargo.toml (build manifest)
- src/lib.rs (module root)
- src/commands/skills.rs (2074 lines)
- src/scheduler/mod.rs (1293 lines)

## Tasks

[Tasks in the format below, ordered safe → risky]

- [ ] [FIND-MED-004] Analyze and split Commands Module - Part 1: Extract skills subcommands
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`, `./skills/rust-engineer/SKILL.md`
  - 🎯 Goal: Reduce src/commands/skills.rs from 2074 lines by extracting subcommand handlers into separate modules
  - 📂 Files: `src/commands/skills.rs` (2074 lines)
  - 🧭 Context: Single file contains all skills subcommands (list, install, uninstall, etc.). Evidence:
    ```bash
    $ wc -l src/commands/skills.rs
    2074 src/commands/skills.rs
    ```
    The file contains:
    - SkillsCommand struct with clap derive
    - SkillsSubcommand enum with all subcommands (List, Install, Uninstall, etc.)
    - Handler implementations for each subcommand
    
    First, analyze the file to identify natural boundaries. Consider extracting:
    - Each subcommand handler to its own file in commands/skills/
    - Common types/shared code to commands/skills/common.rs
    
    Start by extracting ONE logical piece (e.g., the List handler or shared types).
  - ⚡ Pre-check: Build and tests pass before starting (`cargo build && cargo test`)
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (`switchboard skills` commands work identically)
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (independent of other tasks)

- [ ] [FIND-LOW-001] Analyze and split Scheduler Module - Part 1: Identify extractable components
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`, `./skills/rust-engineer/SKILL.md`
  - 🎯 Goal: Reduce src/scheduler/mod.rs from 1293 lines by identifying extractable sub-modules
  - 📂 Files: `src/scheduler/mod.rs` (1293 lines)
  - 🧭 Context: The scheduler module is 1293 lines and may contain multiple responsibilities. Evidence:
    ```bash
    $ wc -l src/scheduler/mod.rs
    1293 src/scheduler/mod.rs
    ```
    The scheduler likely contains:
    - Scheduler struct and its methods
    - Scheduling logic
    - Clock/time handling (note: there's already a clock.rs submodule)
    
    First, analyze what already exists in src/scheduler/ (clock.rs), then identify what else could be split. Consider extracting:
    - Job execution logic to scheduler/execution.rs
    - Configuration handling to scheduler/config.rs
    
    Start by analyzing the file structure and identifying ONE natural extraction point.
  - ⚡ Pre-check: Build and tests pass before starting (`cargo build && cargo test`)
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (scheduler works identically)
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (independent of other tasks)

- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_2' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
