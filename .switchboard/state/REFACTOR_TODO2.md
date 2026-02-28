# REFACTOR_TODO2 - Refactor Agent 2

> Sprint: Improvement Sprint 3
> Focus Area: Error handling improvements and code refactoring
> Last Updated: 2026-02-28
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:
- Cargo.toml
- src/cli/mod.rs
- src/scheduler/mod.rs
- src/logging.rs
- src/docker/run/streams.rs

## Tasks (Ordered Safe → Riskier)

- [x] [HIGH-001] Replace unwrap()/expect() with Proper Error Handling
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md` §4.2, `./skills/rust-engineer/SKILL.md`
  - 🎯 Goal: Replace unsafe unwrap()/expect() calls with proper Result handling or meaningful error messages
  - 📂 Files: 
    - `src/cli/mod.rs:348` - `client.docker().expect("Docker client should be available")`
    - `src/scheduler/mod.rs:1164,1173,1182,1257` - mutex lock `.unwrap()`
    - `src/logging.rs:102,113,137,145,150` - mutex lock `.unwrap()`
    - `src/docker/run/streams.rs:41` - docker client expect
  - 🧭 Context: According to rust-best-practices skill: "Never use unwrap()/expect() outside tests". Evidence:
    ```rust
    // src/cli/mod.rs:348
    let docker = client.docker().expect("Docker client should be available");

    // src/scheduler/mod.rs:1164
    *self.queue_wait_time_seconds.lock().unwrap()

    // src/logging.rs:102
    *INIT_ERROR.lock().unwrap() = Some(err);
    ```
  - ⚡ Pre-check: Run `cargo build` and verify current state compiles
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (same error outcomes, just handled explicitly)
  - 🔒 Risk: Medium (requires careful refactoring to maintain behavior)
  - ↩️ Revert: `git revert` safe if tests pass after revert

- [x] [MED-001] Split discord/llm.rs into Submodules
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Extract client and response handling into separate modules
  - 📂 Files: `src/discord/llm.rs` → split into `src/discord/llm/client.rs`, `src/discord/llm/response.rs`
  - 🧭 Context: File has 1539 lines - a "god module" that should be split by responsibility (client logic vs response handling)
  - ⚡ Pre-check: Run `cargo build` and verify it passes
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (same API externally)
  - 🔒 Risk: Low (structural refactor, no logic changes)
  - ↩️ Revert: `git revert` safe (file reorganization)

- [ ] [HIGH-003] Consider Splitting CLI Module
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Extract CLI commands into separate modules if beneficial for maintainability
  - 📂 Files: `src/cli/mod.rs` (2144 lines)
  - 🧭 Context: Contains all CLI commands and handlers in single file. Consider extracting subcommands to src/commands/
  - ⚡ Pre-check: Run `cargo build` and verify it passes
  - ✅ ANALYSIS (2026-02-28):
    - The CLI module has ALREADY been partially split!
    - Extracted to src/commands/: build, list, logs, metrics, skills, validate (6 commands)
    - Remaining in cli/mod.rs: up, run, down (3 commands)
    - **Decision: DO NOT SPLIT further** - Reasons:
      1. The remaining commands (up, run, down) share significant setup logic (Docker client initialization, scheduler setup)
      2. Splitting would require extracting shared utilities, adding complexity
      3. The current architecture is already reasonable - 2144 lines is not excessively large
      4. Would require significant testing effort to verify no behavioral changes
    - ⚠️ BLOCKED BY: Pre-existing test failures (24 tests failing)
  - ✅ Acceptance:
    - [x] Analysis complete - documented decision not to split
    - [ ] Build passes (cargo build) - ✅ PASSED
    - [ ] All tests pass (cargo test) - ❌ 24 FAILED (pre-existing)

> AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_2' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
