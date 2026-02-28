# REFACTOR_TODO1 - Refactor Agent 1

> Sprint: Improvement Sprint 3
> Focus Area: CLI Module Refactoring
> Last Updated: 2026-02-28T18:33:00Z
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:

- Cargo.toml (build manifest)
- src/lib.rs (module root)
- src/cli/mod.rs (2146 lines - the file to refactor)
- src/commands/ (to understand command structure)

## Tasks

[Tasks in the format below, ordered safe → risky]

- [ ] [FIND-MED-003] Analyze and split CLI Module - Part 1: Extract subcommand handlers
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`, `./skills/rust-engineer/SKILL.md`
  - 🎯 Goal: Reduce src/cli/mod.rs from 2146 lines by extracting command handler implementations into separate modules in src/cli/commands/
  - 📂 Files: `src/cli/mod.rs` (2146 lines)
  - 🧭 Context: The CLI module contains all CLI commands and handlers in a single 2146-line file. This is a god module that should be split. Evidence:
    ```bash
    $ wc -l src/cli/mod.rs
    2146 src/cli/mod.rs
    ```
    The file contains:
    - Command argument parsing using clap derive macros
    - Command dispatch to scheduler, docker, and other modules
    - Individual command implementations (up, run, build, list, logs, down, validate)
    
    First, analyze the file to identify natural boundaries for extraction. Common patterns:
    - Each clap command struct + handler could become its own file
    - Discord-specific code could be extracted to cli/discord.rs (already has feature flag)
    - Unix signal handling could be extracted to cli/signals.rs
    
    Start by extracting ONE logical sub-module (e.g., discord handling or signals).
  - ⚡ Pre-check: Build and tests pass before starting (`cargo build && cargo test`)
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (CLI behaves identically)
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (independent of other tasks)

- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_1' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
