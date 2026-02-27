# REFACTOR_TODO2.md - Agent 2: Core/Clippy Focus

Create this file with the following tasks ordered from safest to riskiest:

**Task 1 - FIND-001 (Safe): Fix syntax error in traits/mod.rs**
- [ ] [FIND-001] Fix invalid Rust syntax in src/traits/mod.rs:17
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Change invalid `use std::io::self;` to valid `use std::io;` at line 17
  - 📂 Files: `src/traits/mod.rs`
  - 🧭 Context: Evidence - Line 17 has `use std::io::self;` which is invalid Rust syntax. Should be `use std::io;`. This is a blocker that prevents the build.
  - ⚡ Pre-check: Build fails before starting (this is the blocker!)
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes (`cargo build`)
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe

**Task 2 - FIND-002 (Medium): Replace panic!() with proper error handling**
- [ ] [FIND-002] Replace panic!() with proper error handling in logging.rs
  - 📚 SKILLS: `./skills/rust-engineer/references/error-handling.md`
  - 🎯 Goal: At logging.rs lines 86-88, replace the `panic!()` call with proper error propagation using `map_err` returning a SkillsError::Io variant.
  - 📂 Files: `src/logging.rs`
  - 🧭 Context: Evidence - Current code: `std::fs::create_dir_all(&log_dir).unwrap_or_else(|_| { panic!("Failed to create log directory: {}", log_dir.display()) });`. Should be: `std::fs::create_dir_all(&log_dir).map_err(|e| SkillsError::Io { path: log_dir.display().to_string(), source: e })?;`
  - ⚡ Pre-check: Build passes before starting
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Tests pass
    - [ ] No behavioral change
  - 🔒 Risk: Medium
  - ↩️ Revert: `git revert` safe

**Task 3 - FIND-020 (Safe/Medium): Fix clippy violations across codebase**
- [ ] [FIND-020] Fix clippy errors - unused_imports, unused_variables, dead_code
  - 📚 SKILLS: `./skills/rust-best-practices/SKILL.md`
  - 🎯 Goal: Run `cargo clippy --all-targets --all-features -- -D warnings` and fix all violations. Common fixes: remove unused imports, remove unused variables (prefix with `_` if intentional), remove dead code.
  - 📂 Files: Multiple (run clippy to find)
  - 🧭 Context: Evidence - Running clippy produces 31+ violations including unused_imports, unused_variables, dead_code, clippy::needless_borrow. This is blocking CI.
  - ⚡ Pre-check: Run `cargo clippy --all-targets --all-features -- -D warnings` to see current errors
  - ✅ Acceptance:
    - [ ] Change is complete
    - [ ] Build passes
    - [ ] Clippy passes with no warnings (`cargo clippy --all-targets --all-features -- -D warnings`)
    - [ ] Tests pass
    - [ ] No behavioral change
  - 🔒 Risk: Safe
  - ↩️ Revert: `git revert` safe (but may have many files)

**Task 4 - FIND-005 (Low): Fix swallowed errors in docker/mod.rs**
- [ ] [FIND-005] Replace `let _ =` with proper error handling in docker/mod.rs
  - 📚 SKILLS: `./skills/rust-engineer/references/error-handling.md`
  - 🎯 Goal: At docker/mod.rs line 890, replace `let _ = std::fs::remove_file(&dockerfile_path);` with proper error handling that logs or returns the error.
  - 📂 Files: `src/docker/mod.rs`
  - 🧭 Context: Evidence - Line 890 has `let _ = std::fs::remove_file(&dockerfile_path);` which silently ignores file removal errors.
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
- src/logging.rs
- src/traits/mod.rs

**QA TASK:**
- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_2' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
