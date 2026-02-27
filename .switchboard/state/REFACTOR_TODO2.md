# REFACTOR_TODO2 - Refactor Agent 2

> Sprint: Improvement Sprint 2  
> Focus Area: Core Code Quality & Code Patterns
> Last Updated: 2026-02-27
> Source: .switchboard/state/IMPROVEMENT_BACKLOG.md findings

## Orientation

Before starting any tasks, read these files to understand the current state:
- Cargo.toml (project structure)
- src/docker/mod.rs (Docker client - primary focus for patterns)
- src/cli/mod.rs (CLI module)

## Tasks

### Task 1: [HIGH-002] Fix formatting issues
- [ ] [FIND-HIGH-002] Run cargo fmt to fix formatting
  - 📚 SKILLS: None required (automated tool)
  - 🎯 Goal: Fix all formatting violations across 180+ files
  - 📂 Files: src/*.rs, src/**/*.rs, tests/*.rs
  - 🧭 Context: cargo fmt --check returns exit code 1 with 2321 lines of diff. Evidence includes:
    - src/cli/mod.rs:973 - Long lines need wrapping
    - src/cli/mod.rs:1079 - Single statement blocks need braces
  - ⚡ Pre-check: Build passes
  - ✅ Acceptance:
    - [ ] cargo fmt --check passes (exit code 0)
    - [ ] Build passes
  - 🔒 Risk: Safe (automated formatter)
  - ↩️ Revert: git checkout safe

### Task 2: [CONV-004] Add documentation to private functions
- [ ] [FIND-CONV-004] Add doc comments to public API entry points
  - 📚 SKILLS: None required
  - 🎯 Goal: Add missing doc comments to module interfaces
  - 📂 Files: Multiple modules
  - 🧭 Context: Multiple private functions lack doc comments. Evidence: Lines 243-247 in backlog state "Some private functions lack doc comments"
  - ⚡ Pre-check: Build passes
  - ✅ Acceptance:
    - [ ] Key public APIs documented
    - [ ] Build passes
  - 🔒 Risk: Safe
  - ↩️ Revert: git checkout safe

### Task 3: [MED-001] Extract Docker client helper method
- [ ] [FIND-MED-001] Create get_docker() helper to eliminate expect pattern
  - 📚 SKILLS: ./skills/rust-best-practices/SKILL.md, ./skills/rust-engineer/SKILL.md
  - 🎯 Goal: Replace 10+ occurrences of .expect("Docker client not available") with a helper method
  - 📂 Files: src/docker/mod.rs (lines 909, 922, 956, 1021, 1054, 1091, 1119, 1138, 1157, 1174)
  - 🧭 Context: Duplicate code pattern violates DRY. Evidence:
    ```rust
    let docker = self.docker.as_ref().expect("Docker client not available");
    // Repeated 10+ times
    ```
    Create helper:
    ```rust
    fn get_docker(&self) -> Result<&Docker, DockerError> {
        self.docker.as_ref().ok_or(DockerError::ConnectionError(
            "Docker client not available".to_string()
        ))
    }
    ```
  - ⚡ Pre-check: Build passes, tests pass
  - ✅ Acceptance:
    - [ ] Helper method created
    - [ ] All 10+ occurrences replaced
    - [ ] Build passes
    - [ ] Tests pass
    - [ ] No behavioral change
  - 🔒 Risk: Low
  - ↩️ Revert: git checkout safe

### Task 4: [MED-002] Extract magic strings to constants
- [ ] [FIND-MED-002] Create constants for repeated error messages
  - 📚 SKILLS: None required
  - 🎯 Goal: Replace hardcoded error strings with constants
  - 📂 Files: src/docker/mod.rs
  - 🧭 Context: Error message strings repeated across codebase. Evidence:
    - "Docker client not available" - appears 10+ times
    - "Failed to create temp dir" - appears in tests
  - ⚡ Pre-check: Build passes, tests pass
  - ✅ Acceptance:
    - [ ] Constants defined
    - [ ] All occurrences replaced
    - [ ] Build passes
    - [ ] Tests pass
  - 🔒 Risk: Low
  - ↩️ Revert: git checkout safe

- [ ] AGENT QA: Run full build and test suite. Verify ALL changes maintain behavioral equivalence. If green, create '.switchboard/state/.refactor_done_2' with the current date. If ALL '.switchboard/state/.refactor_done_*' files exist, also create '.switchboard/state/.refactor_sprint_complete'.
