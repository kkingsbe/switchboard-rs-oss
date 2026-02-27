# Refactor Agent 1 - Blockers

## Date: 2026-02-27

### Blocker 1: Build Failure - Missing bollard API (RESOLVED)

**Status:** RESOLVED

**Error:**
```
error[E0599]: no function or associated item named `connect_with_named_pipe_defaults` found for struct `Docker` in the current scope
```

**Resolution:** Wrapped Windows-specific code in `#[cfg(target_os = "windows")]`

---

### Blocker 2: Test Compilation Errors (ACTIVE)

**Status:** BLOCKED

**Errors:**
```
error[E0063]: missing field `sync` in initializer of `validate::ValidateCommand`
   --> src/commands/validate.rs:728:28
    |
728 |         let validate_cmd = ValidateCommand {};
    |                            ^^^^^^^^^^^^^^^ missing `sync`

error[E0609]: no field `workspace_path` on type `&config::Settings`
    --> src/config/mod.rs:1754:29
     |
1754 |         assert_eq!(settings.workspace_path, "/workspace");
     |                             ^^^^^^^^^^^^^^ unknown field

error[E0609]: no field `workspace_path` on type `config::Settings`
    --> src/config/mod.rs:1962:29
     |
1962 |         assert_eq!(settings.workspace_path, ".");
     |                             ^^^^^^^^^^^^^^ unknown field
```

**Impact:** Tests won't compile - cannot establish test baseline

**Git SHA at time of blocker:** e31c2babcc6daf7e71454e826c554825079fac81 (pre-build-fix)

---

## Refactor Agent 2 - Blocker

**Date:** 2026-02-27
**Agent:** Refactor Agent 2 (AGENT_ID=2)
**Phase:** IMPLEMENTATION

### Blocker Summary
The baseline test suite has 25 failing tests. Per the Safety Protocol, refactoring cannot proceed on a broken build.

### Git SHA at Baseline
`28ee4180a4c7471ef088c5c59235b3b4a1290544`

### Failing Tests (25 total)
The following tests are failing before any refactoring began:

**Configuration/Validation (3 tests):**
- commands::validate::tests::test_validate_lockfile_consistency_no_agents_with_skills
- commands::validate::tests::test_validate_lockfile_consistency_warns_orphaned_skills
- discord::config::tests::test_load_switchboard_toml_discord_section

**Discord Config (2 tests):**
- discord::config::tests::test_env_config_missing_openrouter_api_key
- discord::config::tests::test_env_config_success

**Docker/Run Integration (17 tests):**
- docker::run::run::tests::test_entrypoint_script_generation_all_scenarios
- docker::run::run::tests::test_integration_complete_flow_single_skill
- docker::run::run::tests::test_integration_complete_container_config_building_with_skills
- docker::run::run::tests::test_integration_complete_flow_multiple_skills
- docker::run::run::tests::test_integration_script_installs_skills_to_correct_location
- docker::run::run::tests::test_script_injection_container_config_integration
- docker::run::run::tests::test_script_injection_content_matches_expected_format
- docker::run::run::tests::test_script_injection_entrypoint_configuration
- docker::run::run::tests::test_script_injection_shebang_and_executable
- docker::run::run::tests::test_script_injection_wrapper_executes_script
- docker::run::run::tests::test_skill_install_stderr_has_distinct_prefix
- docker::run::run::tests::test_skill_install_logs_are_distinguishable_from_agent_logs
- docker::run::run::tests::test_skill_install_success_log_has_prefix
- docker::run::run::tests::test_skills_field_handling_non_empty_vec
- docker::run::run::tests::test_skills_match_statement_all_cases
- docker::run::run::tests::test_skills_multiple_generates_custom_entrypoint
- docker::run::run::tests::test_skills_single_generates_custom_entrypoint
- docker::run::run::tests::test_skills_field_handling_integration

**Skills (3 tests):**
- skills::tests::test_check_npx_available_with_mock_error
- skills::tests::test_check_npx_available_with_mock_failure_exit_code

### Root Cause Analysis
The failures appear to be pre-existing issues related to:
1. Mock/external dependency issues in tests
2. Skills not found in test fixtures (tests expecting skills in ./skills/ directory that don't exist)
3. Environment variable configuration issues in tests

### Tasks Blocked
All REFACTOR_TODO2.md tasks are blocked until test suite is green:
- [HIGH-002] Run cargo fmt across 180+ files
- [CONV-004] Add documentation to public API entry points  
- [MED-001] Extract Docker client helper get_docker()
- [MED-002] Extract magic strings to constants

### Resolution Required
The test suite must be fixed (or tests acknowledged as expected failures) before refactoring can proceed.
