### BLOCKER: [Pre-existing Docker Test Failures]

- **Agent:** dev-2
- **Date:** 2026-03-03
- **Type:** test-failure
- **Description:** 6 tests in the Docker module are failing. These tests are unrelated to dev-2's stories (Gateway Registry and Config Validation). The failing tests are:
  - docker::run::run::tests::test_entrypoint_script_generation_all_scenarios
  - docker::build::tests::test_kilocode_included_in_build_context_tarball
  - docker::run::run::tests::test_skill_install_stderr_has_distinct_prefix
  - docker::run::run::tests::test_skill_install_logs_are_distinguishable_from_agent_logs
  - docker::run::run::tests::test_skill_install_success_log_has_prefix
  - docker::skills::tests::test_generate_entrypoint_script_skill_not_in_preexisting_list
  
- **Impact:** Tests failing in unrelated module prevent dev-2 from completing AGENT QA verification phase
- **Note:** Build passes. Gateway stories (ChannelRegistry, Config Validation) implemented by dev-2 do not touch Docker module.

---

### BLOCKER: [Sprint 10] Pre-existing Test Failures]

- **Agent:** dev-1
- **Date:** 2026-03-04
- **Type:** test-failure
- **Description:** 5 pre-existing test failures in docker/skills modules, unrelated to dev-1's gateway CLI stories (story-004-08, story-007-01). Failures are in:
  - docker::build::tests::test_kilocode_included_in_build_context_tarball
  - docker::run::run::tests::test_skill_install_logs_are_distinguishable_from_agent_logs
  - docker::run::run::tests::test_skill_install_stderr_has_distinct_prefix
  - docker::run::run::tests::test_skill_install_success_log_has_prefix
  - docker::skills::tests::test_generate_entrypoint_script_skill_not_in_preexisting_list

- **Impact:** Tests fail (693 pass, 5 fail), but build passes. These appear to be regressions from dev-2's stories (story-004-05, story-005-03).
- **Verification:** Build: `cargo build --features "discord gateway"` ✅ passes. Tests: `cargo test --lib --features "discord gateway"` shows 693 pass, 5 fail.

---

### BLOCKER: [story-003] Refactor docker/mod.rs

- **Agent:** dev-1
- **Date:** 2026-03-05T03:18:00Z
- **Type:** build-failure
- **Description:** Pre-existing build failures when running `cargo build --features "discord gateway"`:
  - 35 compilation errors in docker/client.rs (type inference with dyn DockerClientTrait, ? operator issues)
  - Missing `docker()` method on DockerClient in cli/commands/up.rs, cli/process.rs, docker/run/streams.rs
  - Gateway module not feature-gated in cli/commands/mod.rs
- **Attempted:** Baseline check - build fails before any changes
- **Impact:** Cannot verify "pre-check: Build + tests pass" requirement. However, story acceptance criteria includes "Verify code compiles without Docker feature" which suggests fixing build issues IS part of the story scope.
- **Decision:** Proceeding with implementation - the build errors appear to be exactly what this story's trait refactoring should address.

---

### BLOCKER: [story-005-02, story-005-04] Channel Configuration Stories

- **Agent:** dev-2
- **Date:** 2026-03-05
- **Type:** build-failure
- **Description:** The codebase has 3 compilation errors that prevent building with the `gateway` feature:

  1. `src/cli/commands/gateway.rs:236` - Cannot borrow `server` as mutable, as it is not declared as mutable
  2. `src/cli/commands/gateway.rs:352` - Cannot borrow `server` as mutable, as it is not declared as mutable
  3. `src/gateway/server.rs:567` - Clone method not found for oneshot::Sender

- **Attempted:** Ran baseline build with `cargo build --features "discord gateway"` - failed with exit code 101
- **Impact:** Cannot proceed with story-005-02 (Channel Mapping Config) or story-005-04 (Runtime Channel Subscribe) as the gateway module does not compile

- **Note:** The stories reference functionality that allegedly already exists in the codebase (ChannelMapping in config.rs, ChannelSubscribe/Unsubscribe in protocol.rs), but the build is broken so this cannot be verified.
