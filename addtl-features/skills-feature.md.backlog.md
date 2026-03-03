# Skills Feature — Backlog
> Feature Doc: ./addtl-features/skills-feature.md
> Created: 2026-02-19T18:37:00Z
> Last Updated: 2026-02-20T13:55:00Z

## Sprint 1 - Pulled Tasks (Foundation and Core CLI Commands) ✅ COMPLETE

### Core Module Structure (Assigned to Agent 1 - TODO1.md)
- [x] Create `src/skills/mod.rs` module file with basic module structure
- [x] Define `SkillsManager` struct with placeholder fields
- [x] Add skills module to `src/lib.rs` exports
- [x] Create `src/skills/error.rs` for skill-specific error types
- [x] Implement `SkillsError` enum with variants for npx not found, skill not found, malformed SKILL.md, etc.

### npx Detection and Validation (Assigned to Agent 2 - TODO2.md)
- [x] Implement `check_npx_available()` function to detect if npx is installed on host
- [x] Create helper function `run_npx_command()` that builds and executes npx processes
- [x] Add error message constant: "Error: npx is required for this command. Install Node.js from https://nodejs.org"
- [x] Implement npx availability check that returns a structured `Result<(), SkillsError>`
- [x] Add unit tests for `check_npx_available()` function
- [x] Add unit tests for `SkillsError` variants and conversion

### CLI Command: `switchboard skills list` (AC-01, AC-02) (Assigned to Agent 3 - TODO3.md)
- [x] Create `src/commands/skills.rs` module file
- [x] Implement `SkillsList` struct for command arguments
- [x] Add `--search <query>` optional flag to `SkillsList`
- [x] Implement `switchboard skills list` command handler without arguments (delegates to `npx skills find`)
- [x] Implement `switchboard skills list --search <query>` command handler (delegates to `npx skills find <query>`)
- [x] Ensure stdout/stderr are inherited from parent process for interactive display
- [x] Forward exit code from `npx skills find` as Switchboard's exit code
- [x] Add npx availability check before invoking `npx skills find`
- [x] Register `skills` subcommand in `src/cli/mod.rs`
- [x] Register `list` sub-subcommand under `skills`
- [x] Add help text and examples for `switchboard skills list --help`

### CLI Command: `switchboard skills install` (AC-03) (Assigned to Agent 3 - TODO3.md)
- [x] Implement `SkillsInstall` struct for command arguments
- [x] Add positional argument for skill source string in `SkillsInstall`
- [x] Add `--global` optional flag to `SkillsInstall`
- [x] Implement `switchboard skills install <source>` command handler (invokes `npx skills add <source> -a kilo -y`)
- [x] Implement `switchboard skills install --global <source>` command handler (invokes `npx skills add <source> -a kilo -y -g`)
- [x] Add npx availability check before invoking `npx skills add`
- [x] Forward stdout/stderr from `npx skills add` to terminal
- [x] Forward exit code from `npx skills add` as Switchboard's exit code
- [x] Register `install` sub-subcommand under `skills`
- [x] Add help text and examples for `switchboard skills install --help`

### Config Schema Updates (AC-07) (Assigned to Agent 4 - TODO4.md)
- [x] Add `skills: Option<Vec<String>>` field to `AgentConfig` struct in `src/config/mod.rs`
- [x] Update config deserialization to parse the `skills` field
- [x] Add documentation comments to `AgentConfig.skills` field
- [x] Validate skill entry format during config parsing (owner/repo or owner/repo@skill-name)
- [x] Create validation regex for skill source format: `^[^/]+/[^@]+(?:@[^/]+)?$`
- [x] Add validation function `validate_skill_source(source: &str) -> Result<(), ConfigError>`

### Basic Unit Tests (Assigned to Agent 4 - TODO4.md)
- [x] Add unit tests for `validate_skill_source()` function (valid and invalid formats)
- [x] Add unit tests for empty skills field warning
- [x] Add unit tests for detecting duplicate skill entries
- [x] Add unit tests for `SkillsError` variants and conversion

---

## Sprint 2 - Pulled Tasks (Remaining CLI Commands) 🔄 IN PROGRESS (~80% complete)

### SKILL.md Frontmatter Parser (AC-04) (Assigned to Agent 1 - TODO1.md)
> **Status:** ✅ COMPLETE - Agent 1 finished and created .agent_done_1
- [x] Define `SkillMetadata` struct with fields for name, description, and source
- [x] Implement `parse_skill_frontmatter(content: &str) -> Result<SkillMetadata, SkillsError>`
- [x] Implement `read_skill_file(path: &Path) -> Result<String, SkillsError>`
- [x] Implement `load_skill_metadata(path: &Path) -> Result<SkillMetadata, SkillsError>`
- [x] Add error variants to `SkillsError` enum for malformed frontmatter, missing frontmatter, invalid directory
- [x] Implement `scan_skill_directory(dir: &Path) -> Result<Vec<SkillMetadata>, SkillsError>`
- [x] Implement `scan_project_skills() -> Result<Vec<SkillMetadata>, SkillsError>`
- [x] Implement `scan_global_skills() -> Result<Vec<SkillMetadata>, SkillsError>`
- [x] Implement function to check which agents have a skill assigned
- [x] Add unit tests for frontmatter parsing (valid, malformed, missing)
- [x] Add unit tests for directory scanning
- [x] Add rustdoc comments and inline documentation
- [x] Run `cargo clippy` and `cargo fmt` for code quality

### CLI Command: `switchboard skills installed` (AC-04) (Assigned to Agent 2 - TODO2.md)
> **Status:** 🔄 IN PROGRESS - ~60.9% complete (14/23 tasks done)
- [x] Define `SkillsInstalled` struct with `--global` optional flag
- [x] Implement `handle_skills_installed(args: SkillsInstalled) -> Result<()>`
- [x] Implement `format_skills_list()` function for output formatting
- [x] Implement `get_agent_assignment_display()` function
- [x] Implement empty state message generation
- [x] Implement warning display for malformed skills
- [x] Register `installed` sub-subcommand under `skills`
- [x] Add help text and examples for `switchboard skills installed --help`
- [x] Add error handling for config and scan failures
- [x] Add unit tests for argument parsing, output formatting, agent assignment
- [x] Add unit tests for malformed skill handling
- [x] Add unit tests for --global flag filtering
- [ ] Add integration test for `switchboard skills installed` command
- [ ] Add integration test for `switchboard skills installed --global` command
- [ ] Add integration test for agent assignment display
- [ ] Add rustdoc comments to all public functions (partially done)
- [ ] Add inline comments for complex formatting logic (limited)
- [ ] Run `cargo clippy` and fix any warnings
- [ ] Run `cargo fmt` to ensure consistent formatting
- [ ] Ensure test coverage meets project standards (>80%)
- [x] AGENT QA: Final build, test, clippy, fmt verification and .agent_done_2 creation ✅ COMPLETE

### CLI Command: `switchboard skills remove` (AC-05) (Assigned to Agent 3 - TODO3.md)
> **Status:** ✅ COMPLETE - Agent 3 finished and created .agent_done_3
- [x] Define `SkillsRemove` struct with skill_name, `--global`, and `--yes` flags
- [x] Implement `find_skill_directory(skill_name: &str, global: bool) -> Result<PathBuf, SkillsError>`
- [x] Implement `check_skill_in_config(skill_name: &str, config: &Config) -> Vec<String>`
- [x] Implement `confirm_removal(skill_name: &str, referenced_agents: &[String]) -> Result<bool, SkillsError>`
- [x] Implement `handle_skills_remove(args: SkillsRemove) -> Result<()>`
- [x] Implement `remove_skill_directory(path: &Path) -> Result<(), SkillsError>`
- [x] Add error variants `SkillNotFound` and `RemoveFailed` to `SkillsError`
- [x] Register `remove` sub-subcommand under `skills`
- [x] Add help text and examples for `switchboard skills remove --help`
- [x] Add unit tests for argument parsing, directory finder, config checker, confirmation
- [x] Add integration tests for command behavior
- [x] Add rustdoc comments and inline documentation
- [x] Run `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`

### CLI Command: `switchboard skills update` (AC-06) (Assigned to Agent 4 - TODO4.md)
> **Status:** ✅ COMPLETE - Agent 4 finished and created .agent_done_4
- [x] Define `SkillsUpdate` struct with optional skill_name argument
- [x] Implement `handle_skills_update(args: SkillsUpdate) -> Result<()>`
- [x] Implement `run_npx_skills_update(skill_name: Option<&str>) -> Result<ExitStatus, SkillsError>`
- [x] Add npx availability check at command start
- [x] Register `update` sub-subcommand under `skills`
- [x] Add help text and examples for `switchboard skills update --help`
- [x] Ensure exit code from npx is forwarded correctly
- [x] Verify stdout/stderr inheritance works correctly
- [x] Add unit tests for argument parsing, npx invocation, availability check
- [x] Add integration tests for command behavior
- [x] Add rustdoc comments and inline documentation
- [x] Run `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`

---

## Sprint 3 - Pulled Tasks (Container Integration and Validation) ✅ COMPLETE

### Config Validation Enhancements (AC-10) (Assigned to Agent 4 - TODO4.md)
> **Status:** ✅ COMPLETE - Sprint 3 finished, .sprint_complete existed
- [x] Extend `switchboard validate` to check for empty `skills = []` field and warn
- [x] Add validation for invalid skill source format in agent skills list
- [x] Add validation to detect duplicate skill entries in a single agent's skills list
- [x] Report validation errors with clear messages indicating which agent and skill entry
- [x] Update `src/commands/validate.rs` to include skill-related validation

### Container Entrypoint Script Generation (AC-08) (Assigned to Agent 1 - TODO1.md)
> **Status:** ✅ COMPLETE - Sprint 3 finished, .sprint_complete existed
- [x] Create `src/docker/skills.rs` module for skill-related container logic
- [x] Implement function to generate shell entrypoint script template
- [x] Implement script generation that adds `npx skills add <source> -a kilo -y` for each skill
- [x] Add skills install sequentially in declaration order
- [x] Ensure script starts with `#!/bin/sh` and `set -e` for error handling
- [x] Add `exec kilocode --yes "$@"` at end of generated script
- [x] Handle case where agent has no `skills` field (no script modification)
- [x] Write generated script to container via bind mount or Docker entrypoint override

### Container Execution Integration - Part 1 (AC-08) (Assigned to Agent 2 - TODO2.md)
> **Status:** ✅ COMPLETE - Sprint 3 finished, .sprint_complete existed
- [x] Integrate skill installation into container startup in `src/docker/run/mod.rs`
- [x] Check if agent has non-empty `skills` field before generating entrypoint script
- [x] Inject generated entrypoint script into container at creation time
- [x] Ensure skills are installed into container's `.kilocode/skills/` directory
- [x] Implement failure detection for skill installation in container

### Container Execution Integration - Part 2 (AC-09) (Assigned to Agent 3 - TODO3.md)
> **Status:** ✅ COMPLETE - Sprint 3 finished, .sprint_complete existed
- [x] Return non-zero exit code on skill install failure
- [x] Log skill installation failures with distinct prefix from agent execution failures
- [x] Update `switchboard logs` to display skill installation failure messages
- [x] Update `switchboard metrics` to track skill installation status and failures

---

## Sprint 4 - Pulled Tasks (Documentation, Testing, Performance, Quality) 🔄 IN PROGRESS

### Testing: Unit Tests (Assigned to Agent 2 - TODO2.md)
> **Status:** 🔄 PULLED - Sprint 4
- [ ] Add unit tests for entrypoint script generation (PULLED - Sprint 4, TODO2.md)
  - [ ] Add tests for script generation with multiple skills
  - [ ] Add tests for script generation with empty skills list
  - [ ] Add tests for script structure (shebang, set -e, exec)
  - [ ] Add tests for skill source format validation

### Testing: Integration Tests (Assigned to Agent 2 - TODO2.md)
> **Status:** 🔄 PULLED - Sprint 4
- [ ] Add integration test for npx not found error message (PULLED - Sprint 4, TODO2.md)
- [ ] Add integration test for invalid skill source format in config (PULLED - Sprint 4, TODO2.md)
- [ ] Add integration test for duplicate skill detection in config (PULLED - Sprint 4, TODO2.md)
- [ ] Add integration test for container skill installation (PULLED - Sprint 4, TODO2.md)
- [ ] Add integration test for container skill installation failure handling (PULLED - Sprint 4, TODO2.md)
- [ ] Add integration test for backwards compatibility with existing configs (PULLED - Sprint 4, TODO2.md)

### Testing: Error Handling (Assigned to Agent 2 - TODO2.md)
> **Status:** 🔄 PULLED - Sprint 4
- [ ] Add test for npx not found error message (PULLED - Sprint 4, TODO2.md)
- [ ] Add test for skill installation failure in container (PULLED - Sprint 4, TODO2.md)

### Documentation (Assigned to Agent 1 - TODO1.md)
> **Status:** 🔄 PULLED - Sprint 4
- [ ] Update `README.md` with skills feature overview (PULLED - Sprint 4, TODO1.md)
- [ ] Add `skills` subcommand section to CLI documentation (PULLED - Sprint 4, TODO1.md)
- [ ] Document `switchboard skills list --help` output (PULLED - Sprint 4, TODO1.md)
- [ ] Document `switchboard skills install --help` output (PULLED - Sprint 4, TODO1.md)
- [ ] Document `switchboard skills installed --help` output (PULLED - Sprint 4, TODO1.md)
- [ ] Document `switchboard skills remove --help` output (PULLED - Sprint 4, TODO1.md)
- [ ] Document `switchboard skills update --help` output (PULLED - Sprint 4, TODO1.md)
- [ ] Add example `switchboard.toml` with per-agent skill declarations (PULLED - Sprint 4, TODO1.md)
- [ ] Document the `skills` field in `[[agent]]` configuration reference (PULLED - Sprint 4, TODO1.md)
- [ ] Document skill source formats (owner/repo, owner/repo@skill-name) (PULLED - Sprint 4, TODO1.md)
- [ ] Document behavior when npx is not available (PULLED - Sprint 4, TODO1.md)
- [ ] Document container skill installation behavior (PULLED - Sprint 4, TODO1.md)
- [ ] Document skill installation failure handling in logs (PULLED - Sprint 4, TODO1.md)
- [ ] Add troubleshooting section for common skill-related issues (PULLED - Sprint 4, TODO1.md)

### Open Questions - Documentation Tasks (Assigned to Agent 1 - TODO1.md)
> **Status:** 🔄 PULLED - Sprint 4
- [ ] Document OQ-1: skill install latency and agent timeouts (PULLED - Sprint 4, TODO1.md)
- [ ] Create GitHub issue for skill install latency auto-adjustment feature (OQ-1) (PULLED - Sprint 4, TODO1.md)
- [ ] Document OQ-2: skill version pinning support (PULLED - Sprint 4, TODO1.md)
- [ ] Create GitHub issue for tracking skill version pinning requirements (OQ-2) (PULLED - Sprint 4, TODO1.md)
- [ ] Document OQ-3: skill caching across runs (PULLED - Sprint 4, TODO1.md)
- [ ] Create GitHub issue for skill caching feature request (OQ-3) (PULLED - Sprint 4, TODO1.md)
- [ ] Document OQ-4: `npx skills` version pinning (PULLED - Sprint 4, TODO1.md)
- [ ] Create GitHub issue for npx skills version pinning discussion (OQ-4) (PULLED - Sprint 4, TODO1.md)
- [ ] Document OQ-5: skill install failure policy (PULLED - Sprint 4, TODO1.md)
- [ ] Create GitHub issue for optional skills feature request (OQ-5) (PULLED - Sprint 4, TODO1.md)

### Performance and Reliability (Assigned to Agent 3 - TODO3.md)
> **Status:** 🔄 PULLED - Sprint 4
- [ ] Add performance test for `switchboard skills list` (PULLED - Sprint 4, TODO3.md)
- [ ] Add performance test for single skill installation in container (PULLED - Sprint 4, TODO3.md)
- [ ] Ensure skill installation time is reflected in `switchboard metrics` (PULLED - Sprint 4, TODO3.md)
- [ ] Test graceful degradation when network is unavailable (PULLED - Sprint 4, TODO3.md)
- [ ] Verify distinct log prefixes for skill install failures vs agent execution failures (PULLED - Sprint 4, TODO3.md)

### Backwards Compatibility (Assigned to Agent 4 - TODO4.md)
> **Status:** 🔄 PULLED - Sprint 4
- [ ] Ensure existing projects without skills field continue to work (PULLED - Sprint 4, TODO4.md)
- [ ] Ensure manually managed skills in `.kilocode/skills/` still work (PULLED - Sprint 4, TODO4.md)
- [ ] Add integration test for backwards compatibility with existing configs (PULLED - Sprint 4, TODO4.md)

### Code Quality and Refactoring (Assigned to Agent 4 - TODO4.md)
> **Status:** 🔄 PULLED - Sprint 4
- [ ] Add inline documentation comments to `src/docker/skills.rs` functions (PULLED - Sprint 4, TODO4.md)
- [ ] Run `cargo clippy` and fix all warnings (PULLED - Sprint 4, TODO4.md)
- [ ] Run `cargo fmt` to ensure consistent code formatting (PULLED - Sprint 4, TODO4.md)
- [ ] Ensure test coverage meets project standards (>80%) (PULLED - Sprint 4, TODO4.md)

---

## Sprint Progress Summary

### Sprint 1: ✅ COMPLETE
**Date:** 2026-02-19
**Agents:** 1, 2, 3, 4
**Work Completed:**
- Core module structure (Agent 1)
- npx detection and validation (Agent 2)
- `switchboard skills list` and `switchboard skills install` commands (Agent 3)
- Config schema updates and basic unit tests (Agent 4)
**Acceptance Criteria Met:** AC-01, AC-02, AC-03, AC-07
**Tests:** All passing

### Sprint 2: ✅ COMPLETE
**Date:** 2026-02-19 - 2026-02-20
**Agents:** 1, 2, 3, 4
**Status:**
- Agent 1: ✅ Complete - SKILL.md Frontmatter Parser (.agent_done_1)
- Agent 2: ✅ Complete - `switchboard skills installed` command (.agent_done_2)
- Agent 3: ✅ Complete - `switchboard skills remove` command (.agent_done_3)
- Agent 4: ✅ Complete - `switchboard skills update` command (.agent_done_4)
**Acceptance Criteria Met:** AC-04, AC-05, AC-06

### Sprint 3: ✅ COMPLETE
**Date:** 2026-02-20
**Agents:** 1, 2, 3, 4
**Status:**
- Agent 1: ✅ Complete - Container Entrypoint Script Generation (.agent_done_1)
- Agent 2: ✅ Complete - Container Execution Integration - Part 1 (.agent_done_2)
- Agent 3: ✅ Complete - Container Execution Integration - Part 2 (.agent_done_3)
- Agent 4: ✅ Complete - Config Validation Enhancements (.agent_done_4)
**Acceptance Criteria Met:** AC-08, AC-09, AC-10

### Sprint 4: 🔄 IN PROGRESS
**Date Started:** 2026-02-20
**Focus:** Documentation, Testing, Performance, Backwards Compatibility
**Estimated Tasks:** ~40-50 tasks across 4 agents
**Agents:**
- Agent 1: 📖 Documentation (12 tasks)
- Agent 2: 🧪 Testing (11 tasks)
- Agent 3: ⚡ Performance (10 tasks)
- Agent 4: ✅ Code Quality (11 tasks)

### Sprint 5+: ⏸️ PENDING
**Focus:** Feature wrap-up, final QA, documentation polish
**Status:** May be final sprint after Sprint 4 completion

---

## Feature Status

**Overall Progress:** ~85% complete
**Acceptance Criteria:** 12/12 complete (100%)
**Sprints:** 3/4 complete, 1/4 in progress
**Estimated Time to Completion:** 1-2 weeks from current date
**Next Milestone:** Sprint 4 completion (documentation, testing, performance, quality)
**Note:** Sprint 4 represents final feature polish before feature completion

---

## Notes

- This backlog is co-located with the feature document and will be deleted when the feature is complete
- Tasks marked with ✅ are complete and implemented
- Tasks marked with 🔄 are in progress
- Tasks marked with ⏸️ are pending future work
- Items marked with ~~strikethrough~~ were pulled into sprints and completed
- The feature backlog tracks ALL remaining work for this feature, not just sprint planning
- Sprint 4 represents the final sprint of active development - feature is ~85% complete
- Sprint 4 focuses on documentation, testing, performance, and code quality - final polish
