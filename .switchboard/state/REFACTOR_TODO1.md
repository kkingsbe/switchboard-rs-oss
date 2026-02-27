# REFACTOR_TODO1 - Refactor Agent 1

> Sprint: Improvement Sprint 2
> Focus Area: Discord Module & Safe Cleanup Tasks
> Last Updated: 2026-02-27
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:
- Cargo.toml (project structure)
- src/docker/mod.rs (Docker client - primary focus)
- src/config/mod.rs (config module)
- tests/performance_common.rs (test file to clean)

## Tasks

### Task 1: [CONV-005] Remove backup file
- [ ] [FIND-CONV-005] Delete backup file src/config/mod.rs.bak
  - 📚 SKILLS: None required
  - 🎯 Goal: Remove src/config/mod.rs.bak from the repository
  - 📂 Files: src/config/mod.rs.bak
  - 🧭 Context: A backup file exists in the source tree. Evidence: File src/config/mod.rs.bak contains old config module code. This pollutes the source tree.
  - ⚡ Pre-check: Build passes
  - ✅ Acceptance:
    - [ ] File deleted
    - [ ] Build passes
  - 🔒 Risk: Safe
  - ↩️ Revert: git checkout safe

### Task 2: [LOW-002] Remove unused Timer struct
- [ ] [FIND-LOW-002] Remove unused Timer struct from tests
  - 📚 SKILLS: None required
  - 🎯 Goal: Remove unused Timer struct from tests/performance_common.rs
  - 📂 Files: tests/performance_common.rs (lines 397-415)
  - 🧭 Context: Dead code identified by Clippy. Evidence: 
    ```
    warning: struct 'Timer' is never constructed
    #[derive(Clone)]
    pub struct Timer {
        name: String,
        start: std::time::Instant,
        laps: Vec<(String, std::time::Duration)>,
    }
    ```
  - ⚡ Pre-check: Build passes
  - ✅ Acceptance:
    - [ ] Timer struct removed
    - [ ] Build passes
  - 🔒 Risk: Safe
  - ↩️ Revert: git checkout safe

### Task 3: [LOW-003] Remove unused format_duration function  
- [ ] [FIND-LOW-003] Remove unused format_duration function
  - 📚 SKILLS: None required
  - 🎯 Goal: Remove unused format_duration function from tests
  - 📂 Files: tests/skills_install_performance.rs (lines 45-54)
  - 🧭 Context: Dead code identified by Clippy. Evidence:
    ```
    warning: function 'format_duration' is never used
    fn format_duration(duration: std::time::Duration) -> String { ... }
    ```
  - ⚡ Pre-check: Build passes
  - ✅ Acceptance:
    - [ ] format_duration function removed
    - [ ] Build passes
  - 🔒 Risk: Safe
  - ↩️ Revert: git checkout safe

### Task 4: [LOW-001] Fix Clippy warnings in test files
- [ ] [FIND-LOW-001] Fix Clippy warnings in test files
  - 📚 SKILLS: None required
  - 🎯 Goal: Fix all Clippy warnings in tests/ directory
  - 📂 Files: tests/*.rs (multiple files)
  - 🧭 Context: 24 Clippy warnings in test files. Evidence includes:
    - unused imports (ApiError, Message, ToolCall, ToolFunction, std::sync::Arc, std::time::Instant)
    - unused variables (manager, global_skills_dir, result, duration)
    - mutable variables that don't need to be mutable
    - useless vec! macros
  - ⚡ Pre-check: cargo clippy --tests passes
  - ✅ Acceptance:
    - [ ] All test file warnings fixed
    - [ ] cargo clippy --tests passes
  - 🔒 Risk: Safe
  - ↩️ Revert: git checkout safe

- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_1' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
