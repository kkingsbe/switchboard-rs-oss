# REFACTOR_TODO1.md - Agent 1: Discord Focus

Create this file with the following tasks ordered from safest to riskiest:

**Task 1 - FIND-018 (Safe): Remove unused imports in discord module**
- [ ] [FIND-018] Remove unused imports in discord/gateway.rs and discord/tools.rs
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Remove `#[allow(unused_imports)]` wrapper and unused `Local` import from discord/tools.rs:16-17. Remove unused `from_u8` function from discord/gateway.rs:68. Remove unused `executor` parameter from docker/mod.rs:40 (this can be a separate task or combined if same agent handles docker mod).
  - 📂 Files: `src/discord/gateway.rs`, `src/discord/tools.rs`
  - 🧭 Context: Evidence - `#[allow(unused_imports)] use chrono::Local;` at line 16-17 in tools.rs should have the unused import removed. Function `fn from_u8(val: u8) -> Option<Self>` at gateway.rs:68 is unused. This is safe dead code removal.
  - ⚡ Pre-check: Build passes (`cargo build`) before starting
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (independent of other tasks)

**Task 2 - FIND-005 (Low): Fix swallowed errors in discord/security.rs**
- [ ] [FIND-005] Replace `let _ =` with proper error handling in discord/security.rs
  - 📚 SKILLS: `./skills/rust-engineer/SKILL.md`, `./skills/rust-engineer/references/error-handling.md`
  - 🎯 Goal: At discord/security.rs:362, replace `let _ = fs::remove_file(&target_file);` with proper error handling that logs or returns the error instead of silently swallowing it.
  - 📂 Files: `src/discord/security.rs`
  - 🧭 Context: Evidence - Line 362 has `let _ = fs::remove_file(&target_file);` which silently ignores file removal errors. This should log the error or propagate it.
  - ⚡ Pre-check: Build passes before starting
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] Tests pass (`cargo test`)
    - [ ] No behavioral change
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe

**Task 3 - FIND-006 (Low): Fix .ok() silent defaults in discord/llm.rs and discord/api.rs**
- [ ] [FIND-006] Replace .ok() with proper error handling for rate limit parsing
  - 📚 SKILLS: `./skills/rust-engineer/references/error-handling.md`
  - 🎯 Goal: In discord/llm.rs lines 456-466, replace `.and_then(|v| v.to_str().ok())` with proper error handling that doesn't silently default to 60. Use `and_then` with `ok()` then `ok_or_else` or map with proper error.
  - 📂 Files: `src/discord/llm.rs`, `src/discord/api.rs`
  - 🧭 Context: Evidence - Code uses `.ok()` which silently converts errors to None then falls back to 60. This masks parsing errors. Should use `and_then(|v| v.to_str().ok())` then properly handle the None case with meaningful error.
  - ⚡ Pre-check: Build passes before starting
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Tests pass
    - [ ] No behavioral change
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe

**Task 4 - FIND-005 (Low): Fix swallowed errors in cli/mod.rs**
- [ ] [FIND-005] Replace `let _ =` with proper error handling in cli/mod.rs
  - 📚 SKILLS: `./skills/rust-engineer/references/error-handling.md`
  - 🎯 Goal: At cli/mod.rs lines 901 and 927, replace `let _ = tokio::signal::ctrl_c().await;` with proper handling. For ctrl_c signals, it's acceptable to ignore the result but should use `let _ =` with a comment explaining why, or use `std::panic::take_hook` pattern.
  - 📂 Files: `src/cli/mod.rs`
  - 🧭 Context: Evidence - Lines 901 and 927 have `let _ = tokio::signal::ctrl_c().await;`. While ignoring ctrl_c result is often intentional, add a clarifying comment or use a more explicit pattern.
  - ⚡ Pre-check: Build passes before starting
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Tests pass
    - [ ] No behavioral change
  - 🔒 Risk: Low
  - ↩️ Revert: `git revert` safe

**ORIENTATION SECTION:**
Before starting any tasks, read these files to understand the current state:
- Cargo.toml
- src/lib.rs
- src/discord/mod.rs
- src/discord/security.rs
- src/discord/llm.rs

**QA TASK:**
- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_1' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
