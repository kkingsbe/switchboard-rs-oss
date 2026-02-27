# REFACTOR_TODO2 - Refactor Agent 2

> Sprint: Improvement Sprint 1
> Focus Area: Error Handling and Code Quality
> Last Updated: 2026-02-27T04:46:35Z
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:

- [Build manifest — Cargo.toml]
- [Skills error module — src/skills/error.rs]
- [Docker skills — src/docker/skills.rs]
- [Commands list — src/commands/list.rs]

## Tasks

### Task 1: FIX-012 - Remove Unused Imports (FIND-012)

- [ ] [FIND-012] Remove unused imports
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove unused imports from source files to clean up noise
  - 📂 Files: 
    - `src/discord/gateway.rs:8` (unused `error`)
    - `src/discord/tools.rs:14` (unused `error`, `info`)
    - `src/traits/mod.rs:17` (unused `std::io::Write`)
    - `src/docker/mod.rs:40`
  - 🧭 Context: 4 unused imports create noise and may indicate incomplete refactoring.
    - Evidence: Unused imports: `error` in gateway.rs, `error`, `info` in tools.rs, `std::io::Write` in traits/mod.rs
  - ⚡ Pre-check: Build passes before starting
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (independent of other tasks)

### Task 2: FIX-003 - Replace panic!() Calls with Proper Error Handling (FIND-003)

- [ ] [FIND-003] Replace panic!() calls with proper error returns
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Replace `panic!()` calls with `Result<(), SkillsError>` returns
  - 📂 Files: 
    - `src/docker/skills.rs:111`
    - `src/docker/skills.rs:119`
  - 🧭 Context: Direct `panic!()` calls in production code can cause application crashes on invalid input.
    - Evidence: `panic!("Invalid skill format: {}", skill)`
    - Skill Violation: rust-best-practices (no panics in production)
  - ⚡ Pre-check: Build passes before starting
  - ⚠️ NOTE: This is a Medium-risk task - ensure error propagation is correct
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change (same error conditions return same errors)
  - 🔒 Risk: Medium
  - ↩️ Revert: `git revert` safe (but changes error handling - verify tests)

### Task 3: FIX-005 - Fix Swallowed Errors with .ok() (FIND-005)

- [ ] [FIND-005] Replace .ok() with proper error propagation
  - 📚 SKILLS: `./skills/rust-engineer/SKILL.md`
  - 🎯 Goal: Replace `.ok()` with proper error propagation to maintain error context
  - 📂 Files: 
    - `src/commands/list.rs:45`
    - `src/commands/list.rs:78`
    - `src/discord/llm.rs:457-460`
    - `src/discord/api.rs:272-305`
  - 🧭 Context: Using `.ok()` silently converts errors to default values, hiding configuration errors and making debugging difficult.
    - Evidence: `value_str.parse().ok()?` and `.and_then(|s| s.timezone.parse::<Tz>().ok())`
    - Skill Violation: rust-engineer (proper error handling)
  - ⚡ Pre-check: Build passes before starting
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] All tests pass (`cargo test`)
    - [ ] No behavioral change
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe (improves error handling)

### Task 4: FIX-021 - Add Missing API Documentation (FIND-021)

- [ ] [FIND-021] Add comprehensive doc comments to public types
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Add comprehensive doc comments to public types lacking documentation
  - 📂 Files: 
    - `src/traits/mod.rs:778` (RealProcessExecutor)
    - `src/scheduler/clock.rs:24` (SystemClock)
    - `src/docker/mod.rs:542` (DockerClient)
  - 🧭 Context: 3 public types lack complete documentation.
    - Evidence: RealProcessExecutor, SystemClock, DockerClient lack full doc comments
  - ⚡ Pre-check: Build passes before starting
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (documentation only)

---

- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_2' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
