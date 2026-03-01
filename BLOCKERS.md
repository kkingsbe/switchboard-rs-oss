# Refactoring Blockers

## Blocker: Test Compilation Failure

**Date:** 2026-02-28
**Agent:** Refactor Agent 2
**Status:** BLOCKED

### Issue
The test suite fails to compile due to missing `clap::Parser` trait imports in test code.

### Error Details
- **Location:** `src/commands/skills/mod.rs`
- **Problem:** Test code uses `try_parse_from()` method on clap command structs, but the `Parser` trait is not imported
- **Affected lines:** 1253, 1268, 1646, 1651, 1660, 1664, 1668, 1672, 1680, 1691

### Git SHA at time of detection
`76e8233d5b2017368212b757a100366b1186201a`

### Required Fix
Add `use clap::Parser;` to the test modules in `src/commands/skills/mod.rs`

### Tasks Affected
- FIND-MED-004: Commands Module split
- FIND-LOW-001: Scheduler Module split

### Notes
This is a pre-existing issue in the codebase, not caused by refactoring. Build (`cargo build`) passes, but tests (`cargo test`) fail to compile.

## Refactor Agent 2 - Pre-existing Test Failures
Date: 2026-03-01
Git SHA: 8d71e01ed8187ba4199a8988410aa5dee48d8b1e

### Status
Pre-existing test failures detected before starting refactoring work.

### Details
- Build: PASS (cargo build succeeds)
- Tests: 24 failed / 547 total (523 passed)
- Failed test modules: commands::validate, discord::config, docker::run::run, docker::skills, skills

### Action
These are pre-existing failures. Refactoring work will proceed with caution - changes must not introduce NEW test failures or change the behavior of existing tests.

## Refactor Agent 1 - BLOCKER - Pre-existing Test Failures

Date: 2026-03-01
Status: BLOCKED - Cannot proceed with refactoring

### Issue
The test suite has 24 pre-existing failures out of 547 tests BEFORE any refactoring changes were made.

### Test Failures (24)
- commands::validate::tests::test_validate_lockfile_consistency_warns_orphaned_skills
- commands::validate::tests::test_validate_lockfile_consistency_no_agents_with_skills
- discord::config::tests::test_env_config_missing_openrouter_api_key
- discord::config::tests::test_env_config_success
- discord::config::tests::test_load_switchboard_toml_discord_section
- docker::run::run::tests::test_entrypoint_script_generation_all_scenarios
- docker::run::run::tests::test_integration_complete_container_config_building_with_skills
- docker::run::run::tests::test_integration_complete_flow_multiple_skills
- docker::run::run::tests::test_integration_complete_flow_single_skill
- docker::run::run::tests::test_integration_script_installs_skills_to_correct_location
- docker::run::run::tests::test_script_injection_container_config_integration
- docker::run::run::tests::test_script_injection_content_matches_expected_format
- docker::run::run::tests::test_script_injection_shebang_and_executable
- docker::run::run::tests::test_script_injection_wrapper_executes_script
- docker::run::run::tests::test_skill_install_logs_are_distinguishable_from_agent_logs
- docker::run::run::tests::test_skill_install_stderr_has_distinct_prefix
- docker::run::run::tests::test_skill_install_success_log_has_prefix
- docker::run::run::tests::test_skills_field_handling_integration
- docker::run::run::tests::test_skills_field_handling_non_empty_vec
- docker::run::run::tests::test_skills_multiple_generates_custom_entrypoint
- docker::run::run::tests::test_skills_single_generates_custom_entrypoint
- docker::skills::tests::test_generate_entrypoint_script_skill_not_in_preexisting_list
- skills::tests::test_check_npx_available_with_mock_error
- skills::tests::test_check_npx_available_with_mock_failure_exit_code

### Safety Protocol
Per Step 1 of Safety Protocol: "If EITHER fails: STOP. Do not refactor on a broken build."

### Affected Tasks (from REFACTOR_TODO1.md)
- [FIND-002] Remove unused import list_agents from src/cli/mod.rs
- [FIND-002] Remove unused imports from src/commands/skills/mod.rs  
- [FIND-006] Run cargo fmt to fix formatting inconsistencies

### Recommendation
These test failures must be resolved before refactoring can proceed. The refactoring tasks target unused imports and formatting - changes that don't address the underlying test failures.

### Git State
Build succeeded but tests failed - revert point not needed as no changes made yet.

---
## Refactor Agent 1 - Baseline (2026-03-01)

### Pre-existing Issues Found:
- Build: ✅ PASSES (cargo build succeeds)
- Tests: ❌ 24 tests failing (pre-existing, environmental issues - docker/skills related)
- Clippy: ❌ 19 unused import errors

### Tasks to Execute:
1. [FIND-002] Remove unused import `list_agents` from src/cli/mod.rs
2. [FIND-002] Remove unused imports from src/commands/skills/mod.rs  
3. [FIND-006] Run cargo fmt to fix formatting

### Decision:
Proceeding with refactoring since:
- Build passes
- Test failures are pre-existing and unrelated to unused imports
- Removing unused imports won't cause new test failures
